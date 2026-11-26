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
    let post = submissao(&origem, &tela, "123", Formato::Pdf, None).unwrap();
    assert_eq!(post.corpo(), b"rdoTipo=T&hdnFlagGerar=1");
    assert!(submissao(&origem, &tela, "456", Formato::Pdf, None).is_err());
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
