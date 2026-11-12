use super::{
    ConsultaDoControlador, ErroChaveDaConsulta, ErroConsultaDoControlador, ErroIdDaUrl, id_decimal,
    id_decimal_do_controlador, menciona_acao,
};
use crate::acao_do_sei::AcaoDoSei;
use crate::transporte::FalhaTransporte;

#[test]
fn le_acao_e_valor_unico() {
    let consulta =
        ConsultaDoControlador::ler("acao=arvore_visualizar&id_procedimento=42&infra_hash=ab")
            .unwrap();
    assert_eq!(consulta.acao(), Ok(AcaoDoSei::ArvoreVisualizar));
    assert_eq!(consulta.unico("id_procedimento"), Ok("42"));
}

#[test]
fn cada_recusa_tem_o_seu_erro() {
    let casos = [
        (
            "acao=arvore_visualizar&solto",
            ErroConsultaDoControlador::ParSemIgual,
        ),
        (
            "Acao=arvore_visualizar",
            ErroConsultaDoControlador::RecusadaPeloTransporte(
                FalhaTransporte::ChaveDeConsultaNaoCanonica,
            ),
        ),
        (
            "acao=arvore_visualizar;x",
            ErroConsultaDoControlador::RecusadaPeloTransporte(
                FalhaTransporte::SeparadorDeConsultaAmbiguo,
            ),
        ),
    ];
    for (texto, erro) in casos {
        assert_eq!(
            ConsultaDoControlador::ler(texto).err(),
            Some(erro),
            "{texto}"
        );
    }
}

#[test]
fn chave_ausente_repetida_e_acao_desconhecida() {
    let consulta = ConsultaDoControlador::ler("acao=nao_existe&id=1&id=2").unwrap();
    assert_eq!(
        consulta.unico("id_procedimento"),
        Err(ErroChaveDaConsulta::Ausente {
            chave: "id_procedimento"
        })
    );
    assert_eq!(
        consulta.unico("id"),
        Err(ErroChaveDaConsulta::Repetida { chave: "id" })
    );
    assert_eq!(consulta.acao(), Err(ErroChaveDaConsulta::AcaoDesconhecida));
    assert_eq!(consulta.valores("id").collect::<Vec<_>>(), ["1", "2"]);
}

#[test]
fn consulta_vazia_nao_tem_pares() {
    let consulta = ConsultaDoControlador::ler("").unwrap();
    assert_eq!(consulta.valores("acao").count(), 0);
    assert_eq!(
        ConsultaDoControlador::ler("acao=x&&id=1").err(),
        Some(ErroConsultaDoControlador::ParSemIgual)
    );
}

#[test]
fn url_sem_interrogacao_e_consulta_vazia() {
    for url in [
        "https://h/sei/controlador.php",
        "https://h/sei/controlador.php?",
        "https://h/sei/controlador.php?acao=arvore_visualizar&id=1",
        "https://h/sei/controlador.php?acao=x&&id=1",
        "",
    ] {
        let antes = ConsultaDoControlador::ler(url.split_once('?').map_or("", |(_, c)| c));
        let depois = ConsultaDoControlador::da_url(url.split_once('?').map(|(_, c)| c));
        assert_eq!(antes.map(|c| c.pares), depois.map(|c| c.pares), "{url}");
    }
}

#[test]
fn menciona_acao_seleciona_sem_conferir_o_resto() {
    let casos = [
        ("controlador.php?acao=arvore_visualizar&id=1", true),
        ("controlador.php?acao=outra&acao=arvore_visualizar", true),
        ("controlador.php?solto&acao=arvore_visualizar", true),
        ("controlador.php?acao=arvore_visualizar_x", false),
        ("controlador.php?xacao=arvore_visualizar", false),
        ("acao=arvore_visualizar", false),
    ];
    for (href, esperado) in casos {
        assert_eq!(
            menciona_acao(href, AcaoDoSei::ArvoreVisualizar),
            esperado,
            "{href}"
        );
    }
}

#[test]
fn id_decimal_devolve_o_id_unico_e_cada_falha_com_o_fato() {
    assert_eq!(
        id_decimal("controlador.php?acao=x&id_anexo=42", "id_anexo"),
        Ok("42")
    );
    let casos = [
        ("controlador.php", ErroIdDaUrl::SemConsulta),
        ("controlador.php?acao=x&solto", ErroIdDaUrl::ParSemIgual),
        (
            "controlador.php?acao=x",
            ErroIdDaUrl::ParametroAusente("id_anexo"),
        ),
        (
            "controlador.php?id_anexo=1&id_anexo=2",
            ErroIdDaUrl::ParametroRepetido("id_anexo"),
        ),
        (
            "controlador.php?id_anexo=4a",
            ErroIdDaUrl::IdNaoDecimal {
                chave: "id_anexo",
                valor: "4a".to_owned(),
            },
        ),
        (
            "controlador.php?id_anexo=",
            ErroIdDaUrl::IdNaoDecimal {
                chave: "id_anexo",
                valor: String::new(),
            },
        ),
    ];
    for (url, erro) in casos {
        assert_eq!(id_decimal(url, "id_anexo"), Err(erro), "{url}");
    }
}

#[test]
fn id_decimal_do_controlador_recusa_outra_pagina() {
    assert_eq!(
        id_decimal_do_controlador("outra.php?id_anexo=1", "id_anexo"),
        Err(ErroIdDaUrl::ForaDoControlador)
    );
    assert_eq!(
        id_decimal_do_controlador("controlador.php?id_anexo=1", "id_anexo"),
        Ok("1")
    );
}
