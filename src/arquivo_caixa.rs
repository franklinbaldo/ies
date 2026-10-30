//! Publicação local exclusiva de caixa validada; nunca grava uma leitura parcial.
use serde_json::Value;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErroArquivoCaixa {
    DiretorioPaiAusente,
    DestinoJaExistente,
    CaminhoDestinoSemNome,
    DiretorioPaiInacessivel,
    TemporarioNaoCriado,
    ConteudoNaoSerializado,
    FalhaAoGravarTemporario,
    FalhaAoSincronizarTemporario,
    FalhaAoPublicarDestino,
}
impl ErroArquivoCaixa {
    pub fn subcodigo(self) -> &'static str {
        match self {
            Self::DiretorioPaiAusente => "diretorio_da_caixa_ausente",
            Self::DestinoJaExistente => "arquivo_da_caixa_ja_existe",
            Self::CaminhoDestinoSemNome => "caminho_da_caixa_sem_nome",
            Self::DiretorioPaiInacessivel => "diretorio_da_caixa_inacessivel",
            Self::TemporarioNaoCriado => "temporario_da_caixa_nao_criado",
            Self::ConteudoNaoSerializado => "json_da_caixa_nao_serializado",
            Self::FalhaAoGravarTemporario => "temporario_da_caixa_nao_gravado",
            Self::FalhaAoSincronizarTemporario => "temporario_da_caixa_nao_sincronizado",
            Self::FalhaAoPublicarDestino => "arquivo_da_caixa_nao_publicado",
        }
    }
}
fn diretorio_pai(destino: &Path) -> Result<&Path, ErroArquivoCaixa> {
    destino.file_name().ok_or(ErroArquivoCaixa::CaminhoDestinoSemNome)?;
    let pai = destino.parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    match fs::metadata(pai) {
        Ok(info) if info.is_dir() => Ok(pai),
        Ok(_) => Err(ErroArquivoCaixa::DiretorioPaiInacessivel),
        Err(erro) if erro.kind() == io::ErrorKind::NotFound => {
            Err(ErroArquivoCaixa::DiretorioPaiAusente)
        }
        Err(_) => Err(ErroArquivoCaixa::DiretorioPaiInacessivel),
    }
}

pub fn validar_destino(destino: &Path) -> Result<(), ErroArquivoCaixa> {
    let _ = diretorio_pai(destino)?;
    match fs::symlink_metadata(destino) {
        Ok(_) => Err(ErroArquivoCaixa::DestinoJaExistente),
        Err(erro) if erro.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(ErroArquivoCaixa::DiretorioPaiInacessivel),
    }
}

/// Publica o arquivo completo com hard link exclusivo: rename poderia
/// sobrescrever silenciosamente uma criação concorrente de outro processo.
pub fn salvar_exclusivo(destino: &Path, valor: &Value) -> Result<(), ErroArquivoCaixa> {
    validar_destino(destino)?;
    let pai = diretorio_pai(destino)?;
    let mut bytes = serde_json::to_vec_pretty(valor)
        .map_err(|_| ErroArquivoCaixa::ConteudoNaoSerializado)?;
    bytes.push(b'\n');
    let hora = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duracao| duracao.as_nanos());
    let nome = destino
        .file_name()
        .ok_or(ErroArquivoCaixa::CaminhoDestinoSemNome)?
        .to_string_lossy();
    let temporario = pai.join(format!(".ies-caixa-{}-{hora}-{nome}.tmp", std::process::id()));
    let mut opcoes = OpenOptions::new();
    opcoes.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opcoes.mode(0o600);
    }
    let mut arquivo = opcoes
        .open(&temporario)
        .map_err(|_| ErroArquivoCaixa::TemporarioNaoCriado)?;
    let gravacao = arquivo.write_all(&bytes);
    let sincronizacao = if gravacao.is_ok() {
        arquivo.sync_all()
    } else {
        Ok(())
    };
    drop(arquivo);
    if gravacao.is_err() {
        let _ = fs::remove_file(&temporario);
        return Err(ErroArquivoCaixa::FalhaAoGravarTemporario);
    }
    if sincronizacao.is_err() {
        let _ = fs::remove_file(&temporario);
        return Err(ErroArquivoCaixa::FalhaAoSincronizarTemporario);
    }
    let publicacao = fs::hard_link(&temporario, destino);
    let _ = fs::remove_file(&temporario);
    match publicacao {
        Ok(()) => Ok(()),
        Err(erro) if erro.kind() == io::ErrorKind::AlreadyExists => {
            Err(ErroArquivoCaixa::DestinoJaExistente)
        }
        Err(_) => Err(ErroArquivoCaixa::FalhaAoPublicarDestino),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn teste_pasta() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ies-exportacao-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir(&dir).unwrap();
        dir
    }

    #[test]
    fn exportacao_cria_json_privado_e_nao_sobrescreve_destino() {
        let pasta = teste_pasta();
        let destino = pasta.join("processos.json");
        let valor = serde_json::json!({"recebidos":{"processos":[]},"gerados":{"processos":[]}});
        assert_eq!(validar_destino(&destino), Ok(()));
        assert_eq!(salvar_exclusivo(&destino, &valor), Ok(()));
        let lido: Value = serde_json::from_slice(&fs::read(&destino).unwrap()).unwrap();
        assert_eq!(lido, valor);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(&destino).unwrap().permissions().mode() & 0o777, 0o600);
        }
        assert_eq!(validar_destino(&destino), Err(ErroArquivoCaixa::DestinoJaExistente));
        assert_eq!(salvar_exclusivo(&destino, &Value::Null), Err(ErroArquivoCaixa::DestinoJaExistente));
        assert_eq!(fs::read_to_string(&destino).unwrap().contains("recebidos"), true);
        fs::remove_dir_all(pasta).unwrap();
    }

    #[test]
    fn diretorio_ausente_falha_com_erro_especifico_sem_arquivo_parcial() {
        let pasta = teste_pasta();
        let destino = pasta.join("nao-existe").join("processos.json");
        assert_eq!(validar_destino(&destino), Err(ErroArquivoCaixa::DiretorioPaiAusente));
        assert_eq!(salvar_exclusivo(&destino, &serde_json::json!({})), Err(ErroArquivoCaixa::DiretorioPaiAusente));
        assert!(!destino.exists());
        fs::remove_dir_all(pasta).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn destino_symlink_ja_existente_nunca_e_substituido() {
        use std::os::unix::fs::symlink;
        let pasta = teste_pasta();
        let alvo = pasta.join("alvo.txt");
        let destino = pasta.join("processos.json");
        fs::write(&alvo, b"preservar").unwrap();
        symlink(&alvo, &destino).unwrap();
        assert_eq!(salvar_exclusivo(&destino, &Value::Null), Err(ErroArquivoCaixa::DestinoJaExistente));
        assert_eq!(fs::read(&alvo).unwrap(), b"preservar");
        fs::remove_dir_all(pasta).unwrap();
    }
}
