//! BDD-60-05: a versão escolhida pode perder acesso depois do histórico.
//! Hipótese temporal sintética; nenhuma cassette observa essa corrida.
use super::*;

#[test]
fn bdd_60_05_cli_recusa_versao_revogada_entre_lista_e_corpo() {
    for ocultar in [false, true] {
        let mut argumentos = vec![
            "--json",
            "documento",
            DOC_A,
            "--processo",
            NUP_A,
            "--versao",
            "1",
            "--sem-cache",
        ];
        if ocultar {
            argumentos.push("--ocultar-inacessiveis");
        }
        let (saida, trilha) = apoio::executar_versao_revogada_ao_visualizar(&argumentos);
        assert_eq!(
            saida.status.code(),
            Some(1),
            "a versão recusada não pode ser substituída: stderr={} trilha={trilha:?}",
            String::from_utf8_lossy(&saida.stderr)
        );
        falha_cli::falha_por(&saida, "Documento", "Status");
        assert_eq!(
            trilha
                .iter()
                .filter(|pedido| pedido.as_str() == "GET documento_versao_listar")
                .count(),
            1,
            "a lista deve ser publicada antes da recusa: {trilha:?}"
        );
        assert_eq!(
            trilha
                .iter()
                .filter(|pedido| pedido.as_str() == "GET documento_visualizar")
                .count(),
            1,
            "um GET da versão 1, sem retry/fallback para a versão 2: {trilha:?}"
        );
        assert!(
            trilha.iter().all(|pedido| {
                pedido.starts_with("GET ") || pedido == "POST protocolo_pesquisa_rapida"
            }),
            "a recusa não autoriza escrita: {trilha:?}"
        );
    }
}

/// Uma recusa após o histórico não pode criar foto nem alterar outros
/// arquivos locais, inclusive com a opção de ocultar inacessíveis.
#[test]
fn bdd_60_05_cli_versao_revogada_nao_cria_foto() {
    struct PastaTemporaria(std::path::PathBuf);
    impl Drop for PastaTemporaria {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    for ocultar in [false, true] {
        let agora = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let pasta = PastaTemporaria(std::env::temp_dir().join(format!(
            "ies-bdd60-revogacao-foto-{}-{agora}",
            std::process::id()
        )));
        std::fs::create_dir(&pasta.0).unwrap();
        // Uma foto v1 já existente seria recusada na prévia, ANTES da rede.
        // Um rascunho de nome independente permite exercitar a revogação.
        let rascunho = pasta.0.join("rascunho.html");
        let conteudo_local = b"<html><body>rascunho privado</body></html>";
        std::fs::write(&rascunho, conteudo_local).unwrap();
        let dir = pasta.0.to_str().unwrap();
        let mut argumentos = vec![
            "--json",
            "documento",
            DOC_A,
            "--processo",
            NUP_A,
            "--versao",
            "1",
            "--saida",
            dir,
            "--sem-cache",
        ];
        if ocultar {
            argumentos.push("--ocultar-inacessiveis");
        }

        let (saida, trilha) = apoio::executar_versao_revogada_ao_visualizar(&argumentos);
        assert_eq!(
            saida.status.code(),
            Some(1),
            "a recusa não pode ser ocultada: stderr={} trilha={trilha:?}",
            String::from_utf8_lossy(&saida.stderr)
        );
        falha_cli::falha_por(&saida, "Documento", "Status");
        assert_eq!(std::fs::read(&rascunho).unwrap(), conteudo_local);
        // Conferir toda a pasta: um nome inesperado também seria uma foto indevida.
        let arquivos: Vec<_> = std::fs::read_dir(&pasta.0)
            .unwrap()
            .map(|entrada| entrada.unwrap().file_name())
            .collect();
        assert_eq!(
            arquivos,
            [std::ffi::OsString::from("rascunho.html")],
            "a recusa não pode criar qualquer novo arquivo local"
        );
        assert_eq!(
            trilha
                .iter()
                .filter(|pedido| pedido.as_str() == "GET documento_versao_listar")
                .count(),
            1,
            "histórico antes da negativa: {trilha:?}"
        );
        assert_eq!(
            trilha
                .iter()
                .filter(|pedido| pedido.as_str() == "GET documento_visualizar")
                .count(),
            1,
            "sem retry nem fallback para outra revisão: {trilha:?}"
        );
        assert!(
            trilha.iter().all(|pedido| {
                pedido.starts_with("GET ") || pedido == "POST protocolo_pesquisa_rapida"
            }),
            "não executar POST mutador: {trilha:?}"
        );
    }
}
