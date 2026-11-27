use super::{FalhaPrevia, Leitura, conferir_membros_zip, executar_com_falha_previa, falha};

// Mesmo pedido e estado, com e sem mutação do download: o ZIP errado
// tem cardinalidade correta, formato legível e o mesmo tamanho do bom.
#[test]
fn exportacao_seletiva_recusa_membro_alheio_sem_publicar() {
    for mutante in [false, true] {
        let (saidas, eventos, documentos, arquivos) = executar_com_falha_previa(
            None,
            mutante.then_some(FalhaPrevia::ZipMembroAlheio),
            false,
            Leitura::ZipSelecao(&["050000001", "050000002"]),
            &[b"%PDF-1.4\n2 0 obj\n%%EOF\n"],
        );
        assert_eq!(documentos, 2);
        for saida in &saidas[..3] {
            assert!(saida.status.success(), "HTTP={eventos:?}");
        }
        for prefixo in ["POST procedimento_gerar_zip ", "GET exibir_arquivo "] {
            let chamadas: Vec<_> = eventos
                .iter()
                .filter(|e| e.starts_with(prefixo))
                .map(String::as_str)
                .collect();
            let esperado = format!("{prefixo}200");
            assert_eq!(chamadas, [esperado.as_str()]);
        }
        assert_eq!(
            eventos.iter().any(|e| e == "zip membro alheio projetado"),
            mutante
        );
        if mutante {
            falha(&saidas[3], "SelecaoZipDivergente");
            assert_eq!(arquivos, vec![None]);
        } else {
            assert!(saidas[3].status.success(), "HTTP={eventos:?}");
            conferir_membros_zip(
                arquivos.into_iter().next().flatten().unwrap(),
                &[
                    ("050000001.pdf", b"%PDF-1.4\n1 0 obj\n%%EOF\n"),
                    ("050000002.pdf", b"%PDF-1.4\n2 0 obj\n%%EOF\n"),
                ],
            );
        }
    }
}
