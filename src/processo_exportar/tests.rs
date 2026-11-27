//! Hipóteses sintéticas dos leitores usados pelo comando.
use super::{Formato, endereco_download, submissao};
use crate::formulario_web::{CodificacaoDaTela, TelaServida};
use crate::instalacao::Instalacao;
use crate::tela_html::HtmlServido;

fn url(acao: &str, id: &str) -> String {
    format!(
        "controlador.php?acao={acao}&id_procedimento={id}&infra_hash={}",
        "a".repeat(64)
    )
}

#[test]
fn todos_gera_post_sem_botao_e_recusa_id_alheio() {
    let origem = Instalacao::interpretar("https://sei.example.org/sei").unwrap();
    let action = url("procedimento_gerar_pdf", "123");
    let absoluto = format!("{}/{action}", origem.url_base());
    let html = HtmlServido::de_teste(&format!(
        r#"<form id="frmProcedimentoPdf" method="post" action="{action}">
        <input type="radio" name="rdoTipo" value="T" checked>
        <input type="radio" name="rdoTipo" value="A">
        <input type="hidden" name="hdnFlagGerar" value="0">
        <button type="submit" name="btnGerar">Gerar</button></form>"#
    ));
    let tela = TelaServida {
        url: &absoluto,
        html: &html,
        codificacao: CodificacaoDaTela::Utf8,
    };
    let post = submissao(&origem, &tela, "123", Formato::Pdf, None, None).unwrap();
    assert_eq!(post.corpo(), b"rdoTipo=T&hdnFlagGerar=1");
    assert!(submissao(&origem, &tela, "456", Formato::Pdf, None, None).is_err());
}

#[test]
fn download_exige_link_da_mesma_instalacao_e_processo() {
    let origem = Instalacao::interpretar("https://sei.example.org/sei").unwrap();
    let link = url("exibir_arquivo", "123");
    for (publicado, valido) in [
        (link.clone(), true),
        (url("exibir_arquivo", "456"), false),
        (format!("https://outro.example.org/sei/{link}"), false),
        (url("procedimento_gerar_pdf", "123"), false),
    ] {
        let html = HtmlServido::de_teste(&format!(
            "<iframe id=\"ifrDownload\"></iframe><script>\ndocument.getElementById('ifrDownload').src = '{publicado}';\n</script>"
        ));
        assert_eq!(endereco_download(&origem, &html, "123").is_ok(), valido);
    }
    assert!(
        endereco_download(
            &origem,
            &HtmlServido::de_teste("<iframe id=\"ifrDownload\"></iframe>"),
            "123"
        )
        .is_err()
    );
}

#[test]
fn download_sintetico_publica_bytes_e_recusa_respostas_divergentes() {
    use super::gravar;
    use crate::anexo_documento::LimiteDoAnexo;
    use crate::transporte::resposta_binaria::RespostaBinaria;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    let instante = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let pasta = std::env::temp_dir().join(format!(
        "ies-exportar-{}-{}",
        std::process::id(),
        instante.as_nanos()
    ));
    fs::create_dir(&pasta).unwrap();
    let pdf = b"%PDF-1.7\nSINTETICO\n";
    let zip = b"PK\x05\x06\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
    let resposta = |status, mime: &str, tamanho: usize, corpo| RespostaBinaria {
        status,
        cabecalhos: vec![
            ("content-type".into(), mime.into()),
            ("content-length".into(), tamanho.to_string()),
        ],
        corpo,
    };
    let limite = LimiteDoAnexo::novo(100).unwrap();
    let destino = pasta.join("resultado");
    for (formato, mime, bytes, sha) in [
        (
            Formato::Pdf,
            "application/pdf",
            pdf.as_slice(),
            "e849e57051f3fa640beb4b379b78b76496d4ec0c948668e8fc1e98c78953bcfe",
        ),
        (
            Formato::Zip,
            "application/zip",
            zip.as_slice(),
            "8739c76e681f900923b900c9df0ef75cf421d39cabb54650c4b9ad19b6a76d85",
        ),
    ] {
        let resultado = gravar(
            resposta(200, mime, bytes.len(), bytes),
            formato,
            &destino,
            limite,
        )
        .unwrap();
        assert_eq!(fs::read(&destino).unwrap(), bytes);
        assert_eq!(resultado.sha256, sha);
        assert_eq!(resultado.tamanho_bytes, u64::try_from(bytes.len()).unwrap());
        assert!(
            gravar(
                resposta(200, mime, bytes.len(), bytes),
                formato,
                &destino,
                limite
            )
            .is_err()
        );
        assert_eq!(fs::read(&destino).unwrap(), bytes);
        fs::remove_file(&destino).unwrap();
    }
    for (status, mime, tamanho, bytes, maximo) in [
        (503, "application/pdf", pdf.len(), pdf.as_slice(), 100),
        (200, "text/html", pdf.len(), pdf.as_slice(), 100),
        (200, "application/pdf", zip.len(), zip.as_slice(), 100),
        (200, "application/pdf", pdf.len() + 1, pdf.as_slice(), 100),
        (200, "application/pdf", pdf.len(), pdf.as_slice(), 8),
    ] {
        assert!(
            gravar(
                resposta(status, mime, tamanho, bytes),
                Formato::Pdf,
                &destino,
                LimiteDoAnexo::novo(maximo).unwrap()
            )
            .is_err()
        );
        assert_eq!(fs::read_dir(&pasta).unwrap().count(), 0);
    }
    fs::remove_dir(&pasta).unwrap();
}

#[test]
fn selecao_por_identidade_lexical_e_nao_por_titulo() {
    use super::{ErroSelecao, selecao_da_arvore};
    use crate::arvore_processo::{Cancelamento, DocumentoNaArvore, NaturezaDoDocumento};

    let doc = |id: &str, numero: &str| DocumentoNaArvore {
        id: id.into(),
        rotulo: format!("Anexo ({numero})"),
        numero_sei: Some(numero.into()),
        pasta: None,
        natureza: NaturezaDoDocumento {
            icone: "documento_pdf".into(),
            cancelamento: Cancelamento::Ativo,
            dica: None,
        },
    };
    let docs = [doc("11", "0001"), doc("22", "0002")];
    let origem = Instalacao::interpretar("https://sei.example.org/sei").unwrap();
    assert_eq!(
        selecao_da_arvore(&docs, &["0002".into(), "0001".into()], "", &origem).unwrap(),
        "22±0002 - Anexo ¥11±0001 - Anexo "
    );
    assert!(matches!(
        selecao_da_arvore(&docs, &["22".into()], "", &origem),
        Err(ErroSelecao::DocumentoSelecionadoForaDaArvore)
    ));
    assert!(matches!(
        selecao_da_arvore(&docs, &["2".into()], "", &origem),
        Err(ErroSelecao::DocumentoSelecionadoForaDaArvore)
    ));
    assert!(matches!(
        selecao_da_arvore(&docs, &["0002".into(), "0002".into()], "", &origem),
        Err(ErroSelecao::DocumentoSelecionadoRepetido)
    ));
    assert!(matches!(
        selecao_da_arvore(&[doc("11", "0001"), doc("22", "0001")], &["0001".into()], "", &origem),
        Err(ErroSelecao::IdentidadeDocumentalAmbigua)
    ));
}

#[test]
fn selecao_recusa_fechado_sem_excluir_anexo_recuperavel() {
    use super::{ErroSelecao, selecao_da_arvore};
    use crate::arvore_processo::tests::{NUP, arvore, no_documento, no_processo};
    use crate::arvore_processo::tests_conteudo::{com_src, src_anexo};
    use crate::arvore_processo::ler_arvore;

    let origem = Instalacao::interpretar("https://sei.example.org/sei").unwrap();
    let html = arvore(&[
        no_processo(0, "10", NUP),
        no_documento(1, "21", "10", "0000101"),
        no_documento(2, "22", "10", "0000102"),
    ]);
    let html = com_src(&html, 2, &src_anexo());
    let lida = ler_arvore(&html, NUP, "10").unwrap();
    let docs = &lida.documentos;
    assert!(selecao_da_arvore(docs, &["0000102".into()], &html, &origem).is_ok());
    for pedidos in [vec!["0000101".into()], vec!["0000102".into(), "0000101".into()]] {
        assert!(matches!(
            selecao_da_arvore(docs, &pedidos, &html, &origem),
            Err(ErroSelecao::DocumentoSelecionadoInacessivel)
        ));
    }
    // Conteúdo vazio não basta: barra publicada ou a3 não fechado.
    for outro in [
        html.replace("Nos[1].acoes = '';", "Nos[1].acoes = '<span>Ação</span>';"),
        html.replacen("\"about:blank\",\"ifrVisualizacao\"", "\"outra\",\"ifrVisualizacao\"", 2),
        html.replace("Nos[1].src = '';\n", ""),
    ] {
        assert!(selecao_da_arvore(docs, &["0000101".into()], &outro, &origem).is_ok());
    }
}
