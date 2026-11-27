use super::*;

#[test]
fn pagina_documental_exige_ancora_na_primeira_posicao() {
    use iesei::pesquisa_documento::{PaginaObservada, registrar_pagina};

    let mut banco = Connection::open_in_memory().unwrap();
    banco_cassettes::iniciar(&banco).unwrap();
    let instalacao = Instalacao::interpretar("https://sei.test/sei").unwrap();
    let unidade = UnidadeExibida::nova("TESTE").unwrap();
    banco_cassettes::guardar(
        &mut banco,
        "cenario",
        &ControleResumido {
            instalacao: instalacao.clone(),
            unidade: unidade.clone(),
            recebidos: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
            gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
        },
    )
    .unwrap();

    for (termo, inicio, total, esperado_zero, esperado_dez) in [
        ("integra", 1, 20, Estado::Ok, Estado::Ok),
        (
            "deslocada",
            2,
            20,
            Estado::DadosInsuficientes,
            Estado::DadosInsuficientes,
        ),
        ("divergente", 1, 21, Estado::Ok, Estado::DadosInsuficientes),
    ] {
        for pagina in [
            PaginaObservada {
                pagina: 1,
                inicio,
                fim: 10,
                total_declarado: total,
                linhas: vec![],
            },
            PaginaObservada {
                pagina: 2,
                inicio: 11,
                fim: 20,
                total_declarado: 20,
                linhas: vec![],
            },
        ] {
            registrar_pagina(&mut banco, &instalacao, "cenario", &unidade, termo, &pagina).unwrap();
        }
        for (offset, esperado) in [(0, esperado_zero), (10, esperado_dez)] {
            let resposta = responder_pesquisa(
                "POST",
                "/sei/controlador.php?acao=protocolo_pesquisar",
                &format!(
                    "rdoPesquisarEm=D&chkSinDocumentosGerados=S&txtProtocoloPesquisa={termo}&hdnInicio={offset}"
                ),
                &banco,
                &instalacao,
                "cenario",
            );
            assert_eq!(resposta.estado, esperado, "{termo}/{offset}");
            assert!(!resposta.corpo.contains("pesquisaSemResultado"));
        }
    }

    // Critérios exclusivos de processo não podem herdar uma página documental.
    for deslocamento in [0, 10] {
        verificar_filtros_processuais(&banco, &instalacao, "integra", deslocamento, false);
    }
    iesei::pesquisa_documento::registrar_sem_resultado(
        &banco,
        &instalacao,
        "cenario",
        &unidade,
        "vazia",
    )
    .unwrap();
    verificar_filtros_processuais(&banco, &instalacao, "vazia", 0, true);
}

fn verificar_filtros_processuais(
    banco: &Connection,
    instalacao: &Instalacao,
    termo: &str,
    deslocamento: u64,
    vazio: bool,
) {
    let base = format!(
        "rdoPesquisarEm=D&chkSinDocumentosGerados=S&txtProtocoloPesquisa={termo}&hdnInicio={deslocamento}"
    );
    for (extra, esperado) in [
        ("", Estado::Ok),
        ("&txtDescricaoPesquisa=", Estado::Ok),
        ("&txtObservacaoPesquisa=", Estado::Ok),
        ("&txtDescricaoPesquisa=&txtObservacaoPesquisa=", Estado::Ok),
        ("&txtDescricaoPesquisa=contrato", Estado::DadosInsuficientes),
        ("&txtObservacaoPesquisa=contrato", Estado::DadosInsuficientes),
        ("&txtDescricaoPesquisa=+", Estado::DadosInsuficientes),
        ("&txtObservacaoPesquisa=%20", Estado::DadosInsuficientes),
        (
            "&txtDescricaoPesquisa=&txtDescricaoPesquisa",
            Estado::DadosInsuficientes,
        ),
        (
            "&txtObservacaoPesquisa=&txtObservacaoPesquisa=",
            Estado::DadosInsuficientes,
        ),
        ("&txtDescricaoPesquisa", Estado::DadosInsuficientes),
        ("&txtObservacaoPesquisa=%ZZ", Estado::DadosInsuficientes),
    ] {
        let resposta = responder_pesquisa(
            "POST",
            "/sei/controlador.php?acao=protocolo_pesquisar",
            &format!("{base}{extra}"),
            banco,
            instalacao,
            "cenario",
        );
        assert_eq!(resposta.estado, esperado, "{termo}/{deslocamento}/{extra}");
        assert_eq!(
            resposta.corpo.contains("pesquisaSemResultado"),
            vazio && esperado == Estado::Ok,
            "{termo}/{deslocamento}/{extra}"
        );
    }
}
