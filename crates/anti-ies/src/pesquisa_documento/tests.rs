use super::*;
mod ancora_documental;
mod coorte_processual;
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::observacao::{TabelaObservada, UnidadeExibida};

#[test]
fn campo_opcional_recusa_ocorrencias_duplicadas_incompletas() {
    for (corpo, nome, esperado) in [
        ("outro=1", "q", Some("")),
        ("q=", "q", Some("")),
        ("q=a+b", "q", Some("a b")),
        ("q", "q", None),
        ("q=%", "q", None),
        ("q=&q=", "q", None),
        ("q=&q", "q", None),
        ("q&q=", "q", None),
        ("hdnInicio=10&hdnInicio", "hdnInicio", None),
        ("hdnInicio&hdnInicio=10", "hdnInicio", None),
        (
            "txtDataInicio=01%2F01%2F2026",
            "txtDataInicio",
            Some("01/01/2026"),
        ),
        (
            "txtDataInicio=01%2F01%2F2026&txtDataInicio",
            "txtDataInicio",
            None,
        ),
    ] {
        assert_eq!(campo_opcional(corpo, nome, "").as_deref(), esperado, "{corpo}");
    }
}

#[test]
fn deslocamento_ausente_nao_equivale_a_controle_invalido() {
    for (corpo, esperado) in [
        ("q=", Some(0)),
        ("hdnInicio=0", Some(0)),
        ("hdnInicio=10", Some(10)),
        ("hdnInicio=%31%30", Some(10)),
        ("hdnInicio=", None),
        ("hdnInicio", None),
        ("hdnInicio=%ZZ", None),
        ("hdnInicio=%FF", None),
        ("hdnInicio=-1", None),
        ("hdnInicio=18446744073709551616", None),
        ("hdnInicio=10&hdnInicio=0", None),
        ("hdnInicio&hdnInicio=10", None),
    ] {
        assert_eq!(
            campo_opcional(corpo, "hdnInicio", "0").and_then(|s| s.parse::<u64>().ok()),
            esperado,
            "{corpo}"
        );
    }
}

#[test]
fn registro_negativo_na_unidade_nao_inventa_resultado_de_outro_termo() {
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
    iesei::pesquisa_documento::registrar_sem_resultado(
        &banco,
        &instalacao,
        "cenario",
        &unidade,
        "DOC-NAO-ACHADO",
    )
    .unwrap();
    let url = "/sei/controlador.php?acao=protocolo_pesquisar";
    let tela = responder_pesquisa("GET", url, "", &banco, &instalacao, "cenario");
    assert_eq!(tela.estado, Estado::Ok);
    assert!(tela.corpo.contains("frmPesquisaProtocolo"));
    assert!(tela.corpo.contains("optDocumentos"));
    assert!(tela.corpo.contains("name=\"q\""));
    let base =
        "rdoPesquisarEm=P&txtDataInicio=01%2F01%2F2026&hdnInicio=0&selOrgaoPesquisa%5B%5D=0";
    for extra in ["", "&q=&q", "&q&q=", "&hdnInicio", "&txtDataInicio"] {
        let resposta = responder_pesquisa(
            "POST",
            url,
            &format!("{base}{extra}"),
            &banco,
            &instalacao,
            "cenario",
        );
        assert_eq!(
            resposta.estado,
            if extra.is_empty() {
                Estado::DadosInsuficientes
            } else {
                Estado::RequisicaoInvalida
            },
            "{extra}"
        );
    }
    let buscar = |termo: &str| {
        responder_pesquisa(
            "POST",
            url,
            &format!("rdoPesquisarEm=D&chkSinDocumentosGerados=S&txtProtocoloPesquisa={termo}"),
            &banco,
            &instalacao,
            "cenario",
        )
    };
    let observado = buscar("DOC-NAO-ACHADO");
    assert_eq!(observado.estado, Estado::Ok);
    assert!(observado.corpo.contains("pesquisaSemResultado"));
    assert_eq!(buscar("OUTRO").estado, Estado::DadosInsuficientes);
    assert_eq!(
        responder_pesquisa(
            "POST",
            url,
            "rdoPesquisarEm=D&chkSinDocumentosGerados=S&txtProtocoloPesquisa=DOC-NAO-ACHADO&txtDataInicio=01%2F01%2F2026",
            &banco,
            &instalacao,
            "cenario",
        )
        .estado,
        Estado::DadosInsuficientes
    );
    // A observação negativa de protocolo não prova a mesma consulta com q.
    for (criterios, esperado) in [
        ("txtProtocoloPesquisa=DOC-NAO-ACHADO&q=", Estado::Ok),
        (
            "txtProtocoloPesquisa=DOC-NAO-ACHADO&selSeriePesquisa=42",
            Estado::DadosInsuficientes,
        ),
        (
            "txtProtocoloPesquisa=DOC-NAO-ACHADO&selSeriePesquisa=&selSeriePesquisa",
            Estado::DadosInsuficientes,
        ),
        (
            "txtProtocoloPesquisa=DOC-NAO-ACHADO&hdnInicio=10&hdnInicio=0",
            Estado::RequisicaoInvalida,
        ),
        (
            "txtProtocoloPesquisa=DOC-NAO-ACHADO&hdnInicio=%ZZ",
            Estado::RequisicaoInvalida,
        ),
        (
            "txtProtocoloPesquisa=DOC-NAO-ACHADO&q=contrato",
            Estado::DadosInsuficientes,
        ),
        (
            "txtProtocoloPesquisa=&q=contrato",
            Estado::DadosInsuficientes,
        ),
        (
            "txtProtocoloPesquisa=DOC-NAO-ACHADO&q=licita%E7%E3o",
            Estado::DadosInsuficientes,
        ),
        (
            "txtProtocoloPesquisa=DOC-NAO-ACHADO&q=&q=contrato",
            Estado::RequisicaoInvalida,
        ),
    ] {
        let resposta = responder_pesquisa(
            "POST",
            url,
            &format!("rdoPesquisarEm=D&chkSinDocumentosGerados=S&{criterios}"),
            &banco,
            &instalacao,
            "cenario",
        );
        assert_eq!(resposta.estado, esperado, "{criterios}");
    }
    // A segunda página contradiz o zero explícito, ainda que não haja
    // primeira página positiva. Nenhum deslocamento pode renderizá-la.
    iesei::pesquisa_documento::registrar_pagina(
        &mut banco,
        &instalacao,
        "cenario",
        &unidade,
        "DOC-NAO-ACHADO",
        &iesei::pesquisa_documento::PaginaObservada {
            pagina: 2,
            inicio: 11,
            fim: 20,
            total_declarado: 20,
            linhas: vec![],
        },
    )
    .unwrap();
    for deslocamento in [0, 10] {
        let resposta = responder_pesquisa(
            "POST",
            url,
            &format!(
                "rdoPesquisarEm=D&chkSinDocumentosGerados=S&txtProtocoloPesquisa=DOC-NAO-ACHADO&hdnInicio={deslocamento}"
            ),
            &banco,
            &instalacao,
            "cenario",
        );
        assert_eq!(resposta.estado, Estado::DadosInsuficientes);
        assert!(!resposta.corpo.contains("pesquisaSemResultado"));
    }
    assert_eq!(
        responder_pesquisa(
            "POST",
            url,
            "rdoPesquisarEm=P&txtProtocoloPesquisa=DOC-NAO-ACHADO",
            &banco,
            &instalacao,
            "cenario",
        )
        .estado,
        Estado::RequisicaoInvalida
    );
}
