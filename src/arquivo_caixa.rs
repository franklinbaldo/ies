//! Exportação local de caixa verificada, nunca sobrepõe arquivo preexistente.
use serde_json::Value;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErroArquivoCaixa {
    DestinoSemNomeDeArquivo,
    DiretorioDeDestinoAusente,
    DiretorioDeDestinoGravavelPorOutros,
    DestinoJaExiste,
    EscritaDeArquivoFalhou,
}
impl ErroArquivoCaixa {
    pub fn subcodigo(self) -> &'static str {
        match self {
            Self::DestinoSemNomeDeArquivo => "destino_da_caixa_sem_nome_de_arquivo",
            Self::DiretorioDeDestinoAusente => "diretorio_do_arquivo_da_caixa_ausente",
            Self::DiretorioDeDestinoGravavelPorOutros => {
                "diretorio_do_arquivo_da_caixa_gravavel_por_outros"
            }
            Self::DestinoJaExiste => "arquivo_da_caixa_ja_existe",
            Self::EscritaDeArquivoFalhou => "escrita_de_arquivo_da_caixa_falhou",
        }
    }
    pub fn mensagem(self) -> &'static str {
        match self {
            Self::DestinoSemNomeDeArquivo => "destino da caixa não tem nome de arquivo",
            Self::DiretorioDeDestinoAusente => "diretório de destino não existe",
            Self::DiretorioDeDestinoGravavelPorOutros => {
                "o diretório de destino é gravável por grupo ou outros (sem sticky bit)"
            }
            Self::DestinoJaExiste => "arquivo de destino já existe e não será substituído",
            Self::EscritaDeArquivoFalhou => "não foi possível gravar a caixa em arquivo privado",
        }
    }
}
/// Grava JSON apenas em arquivo novo pela gravação privada única
/// (`0600`, sem seguir link no destino, em diretório fechado a outros).
pub fn gravar(destino: &Path, valor: &Value) -> Result<usize, ErroArquivoCaixa> {
    use crate::arquivo_privado::{self, FalhaDaGravacao as F, Politica};

    if destino.file_name().is_none() {
        return Err(ErroArquivoCaixa::DestinoSemNomeDeArquivo);
    }
    // Como antes da #211: nome sem diretório (pai vazio) é recusado.
    if !destino.parent().is_some_and(Path::is_dir) {
        return Err(ErroArquivoCaixa::DiretorioDeDestinoAusente);
    }
    let bytes =
        serde_json::to_vec_pretty(valor).map_err(|_| ErroArquivoCaixa::EscritaDeArquivoFalhou)?;
    arquivo_privado::gravar(destino, &bytes, Politica::exclusiva()).map_err(
        |falha| match falha {
            F::SemNomeDeArquivo => ErroArquivoCaixa::DestinoSemNomeDeArquivo,
            F::DiretorioAusente => ErroArquivoCaixa::DiretorioDeDestinoAusente,
            F::DiretorioGravavelPorOutros => ErroArquivoCaixa::DiretorioDeDestinoGravavelPorOutros,
            F::DestinoJaExiste => ErroArquivoCaixa::DestinoJaExiste,
            F::DestinoNaoEhArquivoRegular
            | F::AcimaDoLimite
            | F::CriacaoFalhou
            | F::EscritaFalhou
            | F::ConfirmacaoFalhou => ErroArquivoCaixa::EscritaDeArquivoFalhou,
        },
    )?;
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
    #[cfg(unix)]
    #[test]
    fn diretorio_por_link_simbolico_e_aceito_e_aberto_a_outros_e_recusado() {
        use std::os::unix::fs::PermissionsExt;
        let dir = pasta();
        let real = dir.join("real");
        fs::create_dir(&real).unwrap();
        fs::set_permissions(&real, fs::Permissions::from_mode(0o755)).unwrap();
        std::os::unix::fs::symlink(&real, dir.join("atalho")).unwrap();
        let dado = serde_json::json!({});
        assert!(gravar(&dir.join("atalho/caixa.json"), &dado).is_ok());
        assert!(real.join("caixa.json").is_file());
        fs::set_permissions(&real, fs::Permissions::from_mode(0o777)).unwrap();
        let erro = gravar(&dir.join("atalho/outra.json"), &dado).unwrap_err();
        assert_eq!(
            erro.subcodigo(),
            "diretorio_do_arquivo_da_caixa_gravavel_por_outros"
        );
        assert!(!real.join("outra.json").exists());
        fs::remove_dir_all(dir).unwrap();
    }
}
