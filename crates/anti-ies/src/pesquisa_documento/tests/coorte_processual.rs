use super::*;

#[test]
fn coorte_processual_contraditoria_recusa_ambas_as_paginas() {
    use iesei::pesquisa_processo::{Criterios, Pagina, registrar_pagina, registrar_sem_resultado};

    for zero_primeiro in [true, false] {
        let mut banco = Connection::open_in_memory().unwrap();
        banco_cassettes::iniciar(&banco).unwrap();
        let instalacao = Instalacao::interpretar("https://sei.test/sei").unwrap();
        let unidade = UnidadeExibida::nova("TESTE").unwrap();
        let criterios = Criterios {
            tipo_id: "",
            de: "01/01/2026",
            ate: "",
        };
        if zero_primeiro {
            registrar_sem_resultado(&banco, &instalacao, "c", &unidade, criterios).unwrap();
        }
        registrar_pagina(
            &mut banco,
            &instalacao,
            "c",
            &unidade,
            criterios,
            &Pagina {
                numero: 2,
                inicio: 3,
                fim: 4,
                total: 4,
                ids: vec![],
            },
        )
        .unwrap();
        if !zero_primeiro {
            registrar_sem_resultado(&banco, &instalacao, "c", &unidade, criterios).unwrap();
        }
        for inicio in [0, 2] {
            let corpo = format!(
                "txtDataInicio=01%2F01%2F2026&txtDataFim=&selTipoProcedimentoPesquisa=&hdnInicio={inicio}&txtProtocoloPesquisa=&q=&chkSinConsiderarDocumentos=&selOrgaoPesquisa%5B%5D=0&hdnIdAssunto=&hdnIdUnidade=&hdnIdContato=&hdnIdAssinante=&hdnIdUsuarioGerador1=&hdnIdUsuarioGerador2=&hdnIdUsuarioGerador3="
            );
            let resposta = periodo_processual(&corpo, &banco, &instalacao, "c", &unidade);
            assert_eq!(resposta.estado, Estado::DadosInsuficientes);
            assert!(!resposta.corpo.contains("pesquisaSemResultado"));
            assert!(!resposta.corpo.contains("pesquisaTituloRegistro"));
        }
    }
}

#[test]
fn texto_processual_nao_herda_zero_do_periodo() {
    use iesei::pesquisa_processo::{Criterios, registrar_sem_resultado};

    let banco = Connection::open_in_memory().unwrap();
    banco_cassettes::iniciar(&banco).unwrap();
    let instalacao = Instalacao::interpretar("https://sei.test/sei").unwrap();
    let unidade = UnidadeExibida::nova("TESTE").unwrap();
    let criterios = Criterios {
        tipo_id: "",
        de: "01/01/2026",
        ate: "",
    };
    registrar_sem_resultado(&banco, &instalacao, "c", &unidade, criterios).unwrap();
    let base = "txtDataInicio=01%2F01%2F2026&hdnInicio=0&selOrgaoPesquisa%5B%5D=0";
    for (extra, esperado) in [
        ("", Estado::Ok),
        ("&txtDescricaoPesquisa=&txtObservacaoPesquisa=", Estado::Ok),
        ("&txtDescricaoPesquisa=contrato", Estado::DadosInsuficientes),
        (
            "&txtObservacaoPesquisa=contrato",
            Estado::DadosInsuficientes,
        ),
        (
            "&txtDescricaoPesquisa=&txtDescricaoPesquisa",
            Estado::RequisicaoInvalida,
        ),
        ("&txtObservacaoPesquisa", Estado::RequisicaoInvalida),
    ] {
        let resposta = periodo_processual(
            &format!("{base}{extra}"),
            &banco,
            &instalacao,
            "c",
            &unidade,
        );
        assert_eq!(resposta.estado, esperado, "{extra}");
    }
}
