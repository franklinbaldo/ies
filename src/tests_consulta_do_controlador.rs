use super::{
    ConsultaDoControlador, ErroAcaoDaUrl, ErroConsultaDoControlador, ErroIdDaUrl,
    ErroParametroDaUrl, id_decimal, id_decimal_do_controlador, menciona_acao,
};
use crate::acao_do_sei::AcaoDoSei;

#[test]
fn le_acao_e_valor_unico() {
    let consulta =
        ConsultaDoControlador::ler("acao=arvore_visualizar&id_procedimento=42&infra_hash=ab")
            .unwrap();
    assert_eq!(
        consulta.acao(),
        Ok(AcaoDoSei::ArvoreVisualizar.como_na_url())
    );
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
            ErroConsultaDoControlador::ChaveDeConsultaNaoCanonica,
        ),
        (
            "acao=arvore_visualizar;x",
            ErroConsultaDoControlador::SeparadorDeConsultaAmbiguo,
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
fn parametro_ausente_repetido_e_acao_como_servida() {
    let consulta = ConsultaDoControlador::ler("acao=nao_existe&id=1&id=2").unwrap();
    assert_eq!(
        consulta.unico("id_procedimento"),
        Err(ErroParametroDaUrl::ParametroAusente("id_procedimento"))
    );
    assert_eq!(
        consulta.unico("id"),
        Err(ErroParametroDaUrl::ParametroRepetido("id"))
    );
    assert_eq!(consulta.acao(), Ok("nao_existe"));
    assert_eq!(
        ConsultaDoControlador::ler("id=1").unwrap().acao(),
        Err(ErroAcaoDaUrl::AcaoAusente)
    );
    assert_eq!(
        ConsultaDoControlador::ler("acao=a&acao=a").unwrap().acao(),
        Err(ErroAcaoDaUrl::AcaoRepetida)
    );
    assert_eq!(
        ConsultaDoControlador::ler("acao_origem=a&acao_origem=b")
            .unwrap()
            .acao_de_origem(),
        Err(ErroAcaoDaUrl::AcaoRepetida)
    );
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
