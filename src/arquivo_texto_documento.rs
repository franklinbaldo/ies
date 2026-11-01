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

/// Cada componente do pai é aberto em relação ao descritor do anterior.
#[cfg(target_os = "linux")]
fn diretorio_ancorado(pai: &Path) -> Result<std::fs::File, ErroArquivoTexto> {
    use std::fs::{File, OpenOptions};
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::OpenOptionsExt;
    use std::path::{Component, PathBuf};

    let mut atual = File::open(if pai.is_absolute() { "/" } else { "." })
        .map_err(|_| ErroArquivoTexto::DiretorioAusente)?;
    for componente in pai.components() {
        let nome = match componente {
            Component::RootDir | Component::CurDir => continue,
            Component::ParentDir => std::ffi::OsStr::new(".."),
            Component::Normal(nome) => nome,
            Component::Prefix(_) => return Err(ErroArquivoTexto::DiretorioAusente),
        };
        let proximo = PathBuf::from(format!("/proc/self/fd/{}", atual.as_raw_fd())).join(nome);
        let mut opcoes = OpenOptions::new();
        // Linux: O_DIRECTORY | O_NOFOLLOW. Falhar fechado se /proc não estiver disponível.
        opcoes.read(true).custom_flags(0o600_000);
        let aberto = opcoes.open(&proximo).map_err(|erro| {
            if erro.raw_os_error() == Some(40) {
                ErroArquivoTexto::DiretorioLinkSimbolico
            } else {
                ErroArquivoTexto::DiretorioAusente
            }
        })?;
        if !aberto
            .metadata()
            .map_err(|_| ErroArquivoTexto::DiretorioAusente)?
            .is_dir()
        {
            return Err(ErroArquivoTexto::DiretorioAusente);
        }
        atual = aberto;
    }
    Ok(atual)
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

fn salvar_com_ponto_de_corrida(
    destino: &Path,
    texto: &str,
    entre_inspecao_e_abertura: impl FnOnce(),
) -> Result<usize, ErroArquivoTexto> {
    use std::fs::OpenOptions;
    use std::io::{ErrorKind, Write};
    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;

    conferir_destino(destino)?;
    #[cfg(target_os = "linux")]
    let pai_ancorado = diretorio_ancorado(
        destino.parent().ok_or(ErroArquivoTexto::DiretorioAusente)?,
    )?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    entre_inspecao_e_abertura();
    #[cfg(target_os = "linux")]
    let caminho_ancorado = {
        use std::os::fd::AsRawFd;
        std::path::PathBuf::from(format!("/proc/self/fd/{}", pai_ancorado.as_raw_fd()))
            .join(destino.file_name().ok_or(ErroArquivoTexto::NomeAusente)?)
    };
    #[cfg(not(target_os = "linux"))]
    let caminho_ancorado = destino.to_path_buf();
    let mut arquivo = options.open(&caminho_ancorado).map_err(|erro| {
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
        // O caminho original pode ter sido substituído: não remover arquivo
        // alheio por pathname. O parcial continua privado (0600).
        return Err(ErroArquivoTexto::EscritaFalhou);
    }
    Ok(texto.len())
}

pub fn salvar(destino: &Path, texto: &str) -> Result<usize, ErroArquivoTexto> {
    salvar_com_ponto_de_corrida(destino, texto, || {})
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


    #[cfg(target_os = "linux")]
    #[test]
    fn troca_de_ancestral_apos_inspecao_nao_redireciona_texto_sigiloso() {
        use std::os::unix::fs::symlink;
        let raiz = pasta();
        let saida = raiz.join("saida");
        let antiga = raiz.join("saida-original");
        let invasor = raiz.join("invasor");
        fs::create_dir(&saida).unwrap();
        fs::create_dir(&invasor).unwrap();
        let destino = saida.join("0007.txt");
        let resultado = salvar_com_ponto_de_corrida(&destino, "SEGREDO", || {
            fs::rename(&saida, &antiga).unwrap();
            symlink(&invasor, &saida).unwrap();
        });
        let texto_ancorado = fs::read_to_string(antiga.join("0007.txt")).ok();
        let texto_desviado = invasor.join("0007.txt").exists();
        fs::remove_dir_all(&raiz).unwrap();
        assert_eq!(resultado, Ok("SEGREDO".len()));
        assert_eq!(texto_ancorado.as_deref(), Some("SEGREDO"));
        assert!(!texto_desviado, "texto nao pode seguir link concorrente");
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
