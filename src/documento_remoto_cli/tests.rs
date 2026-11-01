use super::*;
use crate::documento_transporte::ErroDocumentoTransporte;
use clap::Parser;
fn cli(args: &[&str]) -> crate::Documento {
    let parsed = Cli::try_parse_from(args).unwrap();
    let Domain::Documento(cmd) = parsed.command else {
        panic!("documento esperado")
    };
    cmd
}
fn html(numero: &str) -> String {
    format!(
        "<html><head><title>SEI/UnB - {numero} - Ofício</title></head><body><p>Texto seguro</p></body></html>"
    )
}
#[test]
fn comando_opt_in_valida_numero_sem_perder_zeros() {
    let c = cli(&[
        "ies",
        "documento",
        "0007",
        "--url-documento-file",
        "/tmp/link",
    ]);
    assert_eq!(conferir(&c, None).unwrap().as_str(), "0007");
}
#[test]
fn recusa_multiplos_documentos_versao_e_contexto_que_nao_foram_comprovados() {
    let muitos = cli(&[
        "ies",
        "documento",
        "0007",
        "0008",
        "--url-documento-file",
        "/tmp/link",
    ]);
    assert_eq!(
        conferir(&muitos, None),
        Err(ErroDocumentoRemoto::QuantidadeDeDocumentos)
    );
    let versao = cli(&[
        "ies",
        "documento",
        "0007",
        "--url-documento-file",
        "/tmp/link",
        "--versao",
        "2",
    ]);
    assert_eq!(
        conferir(&versao, None),
        Err(ErroDocumentoRemoto::OpcoesNaoComprovadas)
    );
    let c = cli(&[
        "ies",
        "documento",
        "0007",
        "--url-documento-file",
        "/tmp/link",
    ]);
    assert_eq!(
        conferir(&c, Some("OUTRA")),
        Err(ErroDocumentoRemoto::OpcoesNaoComprovadas)
    );
}
#[test]
fn numero_visivel_do_html_precisa_ser_exatamente_o_solicitado() {
    let numero = documento::NumeroDocumento::novo("0007").unwrap();
    assert_eq!(
        selecionar(&html("0007"), &numero).unwrap().1,
        "Texto seguro"
    );
    assert_eq!(
        selecionar(&html("7"), &numero),
        Err(ErroDocumentoRemoto::NumeroDoHtmlDiferenteDoSolicitado)
    );
}
#[test]
fn subcodigos_especificos_nao_fazem_classificacao_por_mensagem() {
    assert_eq!(
        ErroDocumentoRemoto::NumeroDoHtmlDiferenteDoSolicitado.subcodigo(),
        "numero_do_html_diferente_do_solicitado"
    );
    assert_eq!(
        ErroDocumentoTransporte::UrlNaoEhVisualizacaoAssinada.subcodigo(),
        "url_documento_sem_visualizacao_assinada"
    );
}
