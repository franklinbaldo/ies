//! Persistência privada de texto de documento já verificado offline.
//! Nunca sobrescreve destino existente, não publica conteúdo em diagnóstico.
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErroArquivoTexto {
    NomeAusente,
    DiretorioAusente,
    DiretorioLinkSimbolico,
    DestinoJaExiste,
    EscritaFalhou,
}
impl ErroArquivoTexto {
    pub fn subcodigo(self) -> &'static str {
        match self {
            Self::NomeAusente => "destino_texto_documento_sem_nome",
            Self::DiretorioAusente => "diretorio_texto_documento_ausente",
            Self::DiretorioLinkSimbolico => "diretorio_texto_documento_link_simbolico",
            Self::DestinoJaExiste => "arquivo_texto_documento_ja_existe",
            Self::EscritaFalhou => "escrita_texto_documento_falhou",
        }
    }
}
/// Rejeita atalhos de diretório em qualquer ancestral conhecido do destino.
/// Não estabelece exclusividade contra alterações concorrentes do caminho.
fn conferir_diretorios(destino: &Path) -> Result<(), ErroArquivoTexto> {
    let pai = destino.parent().ok_or(ErroArquivoTexto::DiretorioAusente)?;
    for ancestral in pai.ancestors() {
        if ancestral.as_os_str().is_empty() {
            continue;
        }
        let metadados =
            std::fs::symlink_metadata(ancestral).map_err(|_| ErroArquivoTexto::DiretorioAusente)?;
        if metadados.file_type().is_symlink() {
            return Err(ErroArquivoTexto::DiretorioLinkSimbolico);
        }
        if !metadados.is_dir() {
            return Err(ErroArquivoTexto::DiretorioAusente);
        }
    }
    Ok(())
}

/// Prévia sem rede nem escrita: recusa um destino sabidamente inutilizável
/// antes que o GET potencialmente registre recebimento no SEI.
/// Não substitui `O_EXCL/create_new` nem elimina corridas concorrentes.
pub(crate) fn conferir_destino(destino: &Path) -> Result<(), ErroArquivoTexto> {
    if destino.file_name().is_none() {
        return Err(ErroArquivoTexto::NomeAusente);
    }
    if !destino.parent().is_some_and(Path::is_dir) {
        return Err(ErroArquivoTexto::DiretorioAusente);
    }
    conferir_diretorios(destino)?;
    match std::fs::symlink_metadata(destino) {
        Ok(_) => Err(ErroArquivoTexto::DestinoJaExiste),
        Err(erro) if erro.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(ErroArquivoTexto::EscritaFalhou),
    }
}

pub fn salvar(destino: &Path, texto: &str) -> Result<usize, ErroArquivoTexto> {
    use std::fs::{self, OpenOptions};
    use std::io::{ErrorKind, Write};
    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;

    conferir_destino(destino)?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut arquivo = options.open(destino).map_err(|erro| {
        if erro.kind() == ErrorKind::AlreadyExists {
            ErroArquivoTexto::DestinoJaExiste
        } else {
            ErroArquivoTexto::EscritaFalhou
        }
    })?;
    if arquivo
        .write_all(texto.as_bytes())
        .and_then(|()| arquivo.sync_all())
        .is_err()
    {
        drop(arquivo);
        let _ = fs::remove_file(destino);
        return Err(ErroArquivoTexto::EscritaFalhou);
    }
    Ok(texto.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn pasta() -> std::path::PathBuf {
        let pasta = std::env::temp_dir().join(format!(
            "ies-texto-documento-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&pasta).unwrap();
        pasta
    }

    #[test]
    fn salva_texto_exato_em_arquivo_privado_novo_sem_sobrescrever() {
        let pasta = pasta();
        let destino = pasta.join("0007.txt");
        assert_eq!(
            salvar(&destino, "Decisão\nAssinada"),
            Ok("Decisão\nAssinada".len())
        );
        assert_eq!(fs::read_to_string(&destino).unwrap(), "Decisão\nAssinada");
        assert_eq!(
            salvar(&destino, "OUTRO"),
            Err(ErroArquivoTexto::DestinoJaExiste)
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&destino).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        fs::remove_dir_all(pasta).unwrap();
    }

    #[test]
    fn ausencia_de_diretorio_tem_erro_tipado_e_nao_cria_destino() {
        let pasta = pasta();
        let destino = pasta.join("inexistente/0007.txt");
        assert_eq!(
            salvar(&destino, "conteúdo"),
            Err(ErroArquivoTexto::DiretorioAusente)
        );
        assert!(!destino.exists());
        fs::remove_dir_all(pasta).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn recusa_link_simbolico_como_diretorio_de_saida() {
        use std::os::unix::fs::symlink;
        let raiz = pasta();
        let real = raiz.join("real");
        fs::create_dir(&real).unwrap();
        let link = raiz.join("atalho");
        symlink(&real, &link).unwrap();
        let destino = link.join("0007.txt");
        assert_eq!(
            salvar(&destino, "DADO CONFIDENCIAL"),
            Err(ErroArquivoTexto::DiretorioLinkSimbolico)
        );
        assert!(!real.join("0007.txt").exists());
        fs::remove_dir_all(raiz).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn recusa_link_em_ancestral_do_diretorio_de_saida() {
        use std::os::unix::fs::symlink;
        let raiz = pasta();
        let real = raiz.join("real");
        let subpasta = real.join("subpasta");
        fs::create_dir_all(&subpasta).unwrap();
        let link = raiz.join("atalho");
        symlink(&real, &link).unwrap();
        let destino = link.join("subpasta").join("0007.txt");
        assert_eq!(
            salvar(&destino, "DADO CONFIDENCIAL"),
            Err(ErroArquivoTexto::DiretorioLinkSimbolico)
        );
        assert!(!subpasta.join("0007.txt").exists());
        assert_eq!(salvar(&subpasta.join("0007.txt"), "OK"), Ok(2));
        fs::remove_dir_all(raiz).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn destino_simbolico_nao_pode_alterar_arquivo_existente() {
        use std::os::unix::fs::symlink;
        let pasta = pasta();
        let original = pasta.join("original.txt");
        fs::write(&original, "NAO SUBSTITUIR").unwrap();
        let destino = pasta.join("texto.txt");
        symlink(&original, &destino).unwrap();
        assert_eq!(
            salvar(&destino, "INTRUSO"),
            Err(ErroArquivoTexto::DestinoJaExiste)
        );
        assert_eq!(fs::read_to_string(&original).unwrap(), "NAO SUBSTITUIR");
        fs::remove_dir_all(pasta).unwrap();
    }
}
