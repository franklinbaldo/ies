//! Exportação local de caixa verificada, nunca sobrepõe arquivo preexistente.
use serde_json::Value;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub enum ErroArquivoCaixa {
    DestinoSemNomeDeArquivo,
    DiretorioDeDestinoAusente,
    DestinoJaExiste,
    EscritaDeArquivoFalhou(#[source] crate::arquivo_privado::FalhaDaGravacao),
    /// O valor não pôde ser serializado como JSON antes da gravação.
    SerializacaoDoJsonFalhou(#[source] serde_json::Error),
}
/// Grava JSON apenas em arquivo novo pela gravação privada única
/// (`0600`, sem seguir link no destino). O diretório é da pessoa: o modo
/// dele não é conferido.
pub fn gravar(destino: &Path, valor: &Value) -> Result<usize, ErroArquivoCaixa> {
    use crate::arquivo_privado::{self, FalhaDaGravacao, Politica};

    if destino.file_name().is_none() {
        return Err(ErroArquivoCaixa::DestinoSemNomeDeArquivo);
    }
    // Como antes da #211: nome sem diretório (pai vazio) é recusado.
    if !destino
        .parent()
        .is_some_and(|pai| !pai.as_os_str().is_empty())
    {
        return Err(ErroArquivoCaixa::DiretorioDeDestinoAusente);
    }
    let bytes =
        serde_json::to_vec_pretty(valor).map_err(ErroArquivoCaixa::SerializacaoDoJsonFalhou)?;
    arquivo_privado::gravar(
        destino,
        &bytes,
        Politica::exclusiva().diretorio_do_usuario(),
    )
    .map_err(|falha| match falha {
        FalhaDaGravacao::SemNomeDeArquivo => ErroArquivoCaixa::DestinoSemNomeDeArquivo,
        FalhaDaGravacao::DiretorioAusente
        | FalhaDaGravacao::NivelDoCaminhoNaoEhDiretorio => ErroArquivoCaixa::DiretorioDeDestinoAusente,
        FalhaDaGravacao::DestinoJaExiste => ErroArquivoCaixa::DestinoJaExiste,
        causa => ErroArquivoCaixa::EscritaDeArquivoFalhou(causa),
    })?;
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
        assert!(matches!(
            gravar(&nome, &dado),
            Err(ErroArquivoCaixa::DestinoJaExiste)
        ));
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
        assert!(matches!(
            gravar(&dir.join("ausente/caixa.json"), &serde_json::json!({})),
            Err(ErroArquivoCaixa::DiretorioDeDestinoAusente)
        ));
        fs::remove_dir_all(dir).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn exportacao_preserva_causa_quando_ancestral_e_arquivo() {
        let dir = pasta();
        let ancestral = dir.join("regular");
        fs::write(&ancestral, b"SENTINELA").unwrap();
        let erro = gravar(
            &ancestral.join("caixa.json"),
            &serde_json::json!({"codigo": 1}),
        )
        .unwrap_err();
        let causa = std::error::Error::source(&erro)
            .and_then(|erro| erro.downcast_ref::<crate::arquivo_privado::FalhaDaGravacao>());
        assert_eq!(
            causa,
            Some(
                &crate::arquivo_privado::FalhaDaGravacao::DiretorioNaoConsultado(
                    std::io::ErrorKind::NotADirectory
                )
            )
        );
        assert_eq!(fs::read(&ancestral).unwrap(), b"SENTINELA");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
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
        assert!(matches!(
            gravar(&link, &serde_json::json!({})),
            Err(ErroArquivoCaixa::DestinoJaExiste)
        ));
        assert_eq!(fs::read_to_string(&alvo).unwrap(), "NAO ALTERAR");
        fs::remove_dir_all(dir).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn diretorio_da_pessoa_por_link_e_gravavel_por_outros_e_aceito() {
        // O arquivo da caixa é escolhido pela pessoa: o modo do diretório
        // é dela; só valem a criação exclusiva e o nome final.
        use std::os::unix::fs::PermissionsExt;
        let dir = pasta();
        let real = dir.join("real");
        fs::create_dir(&real).unwrap();
        std::os::unix::fs::symlink(&real, dir.join("atalho")).unwrap();
        let dado = serde_json::json!({});
        for bits in [0o755, 0o775, 0o777] {
            fs::set_permissions(&real, fs::Permissions::from_mode(bits)).unwrap();
            let nome = format!("atalho/caixa-{bits:o}.json");
            assert!(gravar(&dir.join(&nome), &dado).is_ok(), "{bits:o}");
            assert!(matches!(
                gravar(&dir.join(&nome), &dado),
                Err(ErroArquivoCaixa::DestinoJaExiste)
            ));
        }
        assert_eq!(fs::read_dir(&real).unwrap().count(), 3);
        fs::remove_dir_all(dir).unwrap();
    }
}
