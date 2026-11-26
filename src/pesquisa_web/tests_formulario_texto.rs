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
        assert!(
            corpo
                .split('&')
                .any(|par| par.split_once('=') == Some(("txtProtocoloPesquisa", termo)))
        );
    }
}

#[test]
fn texto_livre_permanece_no_post_da_pagina_em_iso() {
    let url = url_servida();
    let html = HtmlServido::de_teste(&observado());
    let filtros = FiltrosDaPesquisa {
        texto: Some("licitação + contrato"),
        ..Default::default()
    };
    let envio = derivar_pagina_em_filtrada(
        PesquisarEm::Documentos,
        &instalacao(),
        &tela_iso(&url, &html),
        ("UNIDADE TESTE", ""),
        10,
        filtros,
    )
    .unwrap();
    let corpo = String::from_utf8(envio.corpo().to_vec()).unwrap();
    let pares: Vec<_> = corpo.split('&').collect();
    assert!(pares.contains(&"q=licita%E7%E3o+%2B+contrato"));
    assert!(pares.contains(&"txtProtocoloPesquisa="));
    assert!(pares.contains(&"hdnInicio=10"));
    assert!(!pares.iter().any(|par| par.starts_with("sbmPesquisar=")));
}

#[test]
fn texto_livre_nao_sai_se_a_tela_nao_publica_q() {
    let url = url_servida();
    let html = HtmlServido::de_teste(&observado().replace("name='q'", "name='outro'"));
    let filtros = FiltrosDaPesquisa {
        texto: Some("contrato"),
        ..Default::default()
    };
    let erro = derivar_pesquisa_em_filtrada(
        PesquisarEm::Documentos,
        &instalacao(),
        &tela_iso(&url, &html),
        ("UNIDADE TESTE", ""),
        filtros,
    )
    .unwrap_err();
    assert!(matches!(
        erro,
        ErroPesquisaProtocolo::Submissao(
            ErroSubmissaoFormulario::CampoNaoPublicadoPeloFormulario { campo }
        ) if campo == "q"
    ));
}
