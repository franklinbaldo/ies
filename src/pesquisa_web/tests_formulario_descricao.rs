use super::*;

fn com_textos() -> String {
    observado().replace(
        "<input type='hidden' id='hdnInicio'",
        "<input type='text' name='txtDescricaoPesquisa' maxlength='250'><input type='text' name='txtObservacaoPesquisa' maxlength='250'><input type='hidden' id='hdnInicio'",
    )
}

#[test]
fn descricao_e_observacao_permanecem_nos_dois_posts() {
    let url = url_servida();
    let html = HtmlServido::de_teste(&com_textos());
    let filtros = FiltrosDaPesquisa {
        descricao: Some("Ação + &"),
        observacao: Some(" outra "),
        ..Default::default()
    };
    let tela = tela_iso(&url, &html);
    let inicial = derivar_pesquisa_em_filtrada(
        PesquisarEm::Processos,
        &instalacao(),
        &tela,
        ("UNIDADE TESTE", ""),
        filtros,
    )
    .unwrap();
    let seguinte = derivar_pagina_em_filtrada(
        PesquisarEm::Processos,
        &instalacao(),
        &tela,
        ("UNIDADE TESTE", ""),
        10,
        filtros,
    )
    .unwrap();
    for (envio, inicio) in [(inicial, "hdnInicio=0"), (seguinte, "hdnInicio=10")] {
        let corpo = String::from_utf8(envio.corpo().to_vec()).unwrap();
        let pares: Vec<_> = corpo.split('&').collect();
        assert!(pares.contains(&"txtDescricaoPesquisa=A%E7%E3o+%2B+%26"));
        assert!(pares.contains(&"txtObservacaoPesquisa=+outra+"));
        assert!(pares.contains(&"txtProtocoloPesquisa="));
        assert!(pares.contains(&inicio));
    }
}

#[test]
fn descricao_respeita_controle_servido_e_limite_utf16() {
    let url = url_servida();
    let texto = "😀".repeat(126);
    let filtros = FiltrosDaPesquisa {
        descricao: Some(&texto),
        ..Default::default()
    };
    for html in [
        observado(),
        com_textos(),
        com_textos().replace("maxlength='250'", "maxlength='x'"),
        com_textos().replace(
            "name='txtDescricaoPesquisa'",
            "name='txtDescricaoPesquisa' disabled",
        ),
    ] {
        let html = HtmlServido::de_teste(&html);
        let erro = derivar_pesquisa_em_filtrada(
            PesquisarEm::Processos,
            &instalacao(),
            &tela_iso(&url, &html),
            ("UNIDADE TESTE", ""),
            filtros,
        )
        .unwrap_err();
        assert!(matches!(erro, ErroPesquisaProtocolo::Filtro(_)));
    }
}
