//! Exportação local de caixa verificada, nunca sobrepõe arquivo preexistente.
use serde_json::Value;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErroArquivoCaixa {
    DestinoSemNomeDeArquivo,
    DiretorioDeDestinoAusente,
    DestinoJaExiste,
    EscritaDeArquivoFalhou,
}
impl ErroArquivoCaixa {
    pub fn subcodigo(self) -> &'static str {
        match self {
            Self::DestinoSemNomeDeArquivo => "destino_da_caixa_sem_nome_de_arquivo",
            Self::DiretorioDeDestinoAusente => "diretorio_do_arquivo_da_caixa_ausente",
            Self::DestinoJaExiste => "arquivo_da_caixa_ja_existe",
            Self::EscritaDeArquivoFalhou => "escrita_de_arquivo_da_caixa_falhou",
        }
    }
    pub fn mensagem(self) -> &'static str {
        match self {
            Self::DestinoSemNomeDeArquivo => "destino da caixa não tem nome de arquivo",
            Self::DiretorioDeDestinoAusente => "diretório de destino não existe",
            Self::DestinoJaExiste => "arquivo de destino já existe e não será substituído",
            Self::EscritaDeArquivoFalhou => "não foi possível gravar a caixa em arquivo privado",
        }
    }
}
/// Grava JSON apenas em arquivo novo; em Unix a permissão inicial é 0600.
pub fn gravar(destino: &Path, valor: &Value) -> Result<usize, ErroArquivoCaixa> {
    use std::fs::{self, OpenOptions};
    use std::io::{ErrorKind, Write};
    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;

    if destino.file_name().is_none() {
        return Err(ErroArquivoCaixa::DestinoSemNomeDeArquivo);
    }
    if !destino.parent().is_some_and(Path::is_dir) {
        return Err(ErroArquivoCaixa::DiretorioDeDestinoAusente);
    }
    let bytes =
        serde_json::to_vec_pretty(valor).map_err(|_| ErroArquivoCaixa::EscritaDeArquivoFalhou)?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut arquivo = options.open(destino).map_err(|erro| {
        if erro.kind() == ErrorKind::AlreadyExists {
            ErroArquivoCaixa::DestinoJaExiste
        } else {
            ErroArquivoCaixa::EscritaDeArquivoFalhou
        }
    })?;
    if arquivo
        .write_all(&bytes)
        .and_then(|()| arquivo.sync_all())
        .is_err()
    {
        drop(arquivo);
        let _ = fs::remove_file(destino);
        return Err(ErroArquivoCaixa::EscritaDeArquivoFalhou);
    }
    Ok(bytes.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    fn pasta() -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!(
            "ies-exporta-caixa-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&p).unwrap();
        p
    }
    #[test]
    fn exporta_json_legivel_sem_sobrescrever() {
        let dir = pasta();
        let nome = dir.join("caixa.json");
        let dado =
            serde_json::json!({"unidade":"ABC","recebidos":{"total_declarado":0,"processos":[]}});
        let tamanho = gravar(&nome, &dado).unwrap();
        assert!(tamanho > 10);
        assert_eq!(
            serde_json::from_slice::<Value>(&fs::read(&nome).unwrap()).unwrap(),
            dado
        );
        assert_eq!(gravar(&nome, &dado), Err(ErroArquivoCaixa::DestinoJaExiste));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&nome).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn nao_cria_diretorio_implicitamente() {
        let dir = pasta();
        assert_eq!(
            gravar(&dir.join("ausente/caixa.json"), &serde_json::json!({})),
            Err(ErroArquivoCaixa::DiretorioDeDestinoAusente)
        );
        fs::remove_dir_all(dir).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn nao_segue_link_simbolico_no_destino() {
        use std::os::unix::fs::symlink;
        let dir = pasta();
        let alvo = dir.join("original.txt");
        fs::write(&alvo, "NAO ALTERAR").unwrap();
        let link = dir.join("caixa.json");
        symlink(&alvo, &link).unwrap();
        assert_eq!(
            gravar(&link, &serde_json::json!({})),
            Err(ErroArquivoCaixa::DestinoJaExiste)
        );
        assert_eq!(fs::read_to_string(&alvo).unwrap(), "NAO ALTERAR");
        fs::remove_dir_all(dir).unwrap();
    }
}
