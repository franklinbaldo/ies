use super::*;

// Hipótese sintética: o controle q já consta da fixture estrutural.
#[test]
fn texto_livre_e_protocolo_sao_controles_distintos() {
    let url = url_servida();
    let html = HtmlServido::de_teste(&observado());
    let filtros = FiltrosDaPesquisa {
        texto: Some("contrato"),
        ..Default::default()
    };
    for termo in ["", "0001"] {
        let envio = derivar_pesquisa_em_filtrada(
            PesquisarEm::Documentos,
            &instalacao(),
            &tela_iso(&url, &html),
            ("UNIDADE TESTE", termo),
            filtros,
        )
        .unwrap();
        let corpo = String::from_utf8(envio.corpo().to_vec()).unwrap();
        assert!(corpo.split('&').any(|par| par == "q=contrato"));
        assert!(corpo.split('&').any(|par| par.split_once('=') == Some(("txtProtocoloPesquisa", termo))));
    }
}
