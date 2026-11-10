//! RED/GREEN da busca do destino no GO 5.0.3 (o que as cassettes e o Todos
//! mostram): o `selOrgao` com os órgãos, o filtro pelo órgão do prefixo da
//! sigla e o termo alargado (o último segmento sem `-<dígitos>`), com as
//! sequências exatas de requisições de cada caminho. Fixtures sintéticas na
//! estrutura gravada; ids e nomes inventados.
use super::destino::{orgao_da_sigla, termo_alargado};
use super::erro::{ErroEnviar, FalhaDaConsulta};
use super::tela::conferir_tela_de_envio;
use super::tests_apoio::*;
use super::{Destino, ProcessoEnviado};
use crate::processo_concluir::tests_apoio::*;

const ABC: &str = "ABC/GECONT-18141";
const ABC_CODIFICADA: &str = "ABC%2FGECONT-18141";

fn pares(lista: &[(&str, &str)]) -> Vec<(String, String)> {
    lista
        .iter()
        .map(|(v, r)| ((*v).to_owned(), (*r).to_owned()))
        .collect()
}

/// Como o GO responde a `GECONT` com "Todos" (39 itens gravados; aqui 3).
fn gecont_de_outros() -> String {
    resposta_xml(&[
        ("18440", "AGRODEFESA/GECONT-18440 - Gerencia"),
        ("18148", "CASACIVIL/GECONT-18148 - Gerencia"),
        ("13178", "ECONOMIA/GECONT GERAL-13178 - Gerencia"),
    ])
}

fn da_abc() -> String {
    resposta_xml(&[("18141", "ABC/GECONT-18141 - Gerencia")])
}

fn destino_abc() -> Destino {
    Destino {
        sigla: ABC.into(),
        descricao: "ABC/GECONT-18141 - Gerencia".into(),
        id_unidade: "18141".into(),
    }
}

fn vazia() -> String {
    resposta_xml(&[])
}

/// Prévia no GO com as `consultas` respondidas na ordem.
fn no_go(para: &str, consultas: Vec<String>) -> (Result<ProcessoEnviado, ErroEnviar>, Vec<Pedido>) {
    let mut depois = vec![ok(tela_de_envio_go(ID))];
    depois.extend(consultas.into_iter().map(ok));
    rodar_envio(
        para,
        false,
        false,
        com(abertura(arvore(&[UNIDADE], &enviar())), depois),
    )
}

fn filtrada() -> String {
    corpo_da_consulta_com(ABC_CODIFICADA, "28")
}

fn inteira() -> String {
    corpo_da_consulta_com(ABC_CODIFICADA, "")
}

fn alargada() -> String {
    corpo_da_consulta_com("GECONT", "")
}

fn filtrada_alargada() -> String {
    corpo_da_consulta_com("GECONT", "28")
}

#[test]
fn tela_publica_os_orgaos_do_selorgao() {
    let tela = conferir_tela_de_envio(&tela_de_envio_go(ID), ID).unwrap();
    assert_eq!(tela.orgao, "");
    assert_eq!(
        tela.orgaos,
        pares(&[
            ("null", ""),
            ("", "Todos"),
            ("28", "ABC"),
            ("31", "AGEHAB"),
            ("30", "CASA CIVIL"),
            ("5", "SEAD"),
        ])
    );
}

#[test]
fn orgao_e_o_unico_rotulo_igual_ao_prefixo() {
    let opcoes = pares(&[
        ("null", ""),
        ("", "Todos"),
        ("28", "ABC"),
        ("30", "CASA  CIVIL"),
        ("5", "SEAD"),
    ]);
    assert_eq!(orgao_da_sigla(ABC, &opcoes), Some("28"));
    assert_eq!(orgao_da_sigla("abc/CG-14723", &opcoes), Some("28"));
    assert_eq!(orgao_da_sigla(" ABC /X", &opcoes), Some("28"));
    assert_eq!(orgao_da_sigla("CASA CIVIL/X", &opcoes), Some("30"));
    // A sigla do GO é `CASACIVIL/…`: sem o espaço do rótulo, nenhum órgão.
    assert_eq!(orgao_da_sigla("CASACIVIL/GECONT-18148", &opcoes), None);
    for sem in ["ABC", "XYZ/GECONT-1", "/GECONT", "Todos/X", "null/X"] {
        assert_eq!(orgao_da_sigla(sem, &opcoes), None, "{sem}");
    }
    let repetido = pares(&[("1", "ABC"), ("2", "abc")]);
    assert_eq!(orgao_da_sigla("ABC/X", &repetido), None);
}

#[test]
fn termo_alargado_e_o_ultimo_segmento_sem_o_numero() {
    for (sigla, termo) in [
        (ABC, Some("GECONT")),
        ("ECONOMIA/GECONT GERAL-13178", Some("GECONT GERAL")),
        ("SEAD/SUPTESTE-15887", Some("SUPTESTE")),
        ("ADCENTRAL/GR", Some("GR")),
        ("ADCENTRAL/GR/ARQUIVO /PROCE", Some("PROCE")),
        ("X/A-1B", Some("A-1B")),
        ("X/A-", Some("A-")),
        ("TESTE", None),
        ("ABC/", None),
        ("ABC/-123", None),
    ] {
        assert_eq!(termo_alargado(sigla).as_deref(), termo, "{sigla}");
    }
}

#[test]
fn filtro_pelo_orgao_acha_na_primeira_consulta() {
    let (r, feitos) = no_go(ABC, vec![da_abc()]);
    assert_eq!(r.unwrap().destino, destino_abc());
    assert_eq!(feitos, pedidos_da_busca(&[filtrada()]));
}

#[test]
fn alargado_no_orgao_e_a_segunda_consulta() {
    // A ordem do Todos: os dois termos no órgão do prefixo, depois os dois
    // no da tela. Do alargado vale só a sigla exata.
    let com_a_abc = resposta_xml(&[
        ("18440", "AGRODEFESA/GECONT-18440 - Gerencia"),
        ("18141", "abc/gecont-18141 - Gerencia"),
    ]);
    for no_orgao in [vazia(), gecont_de_outros()] {
        let (r, feitos) = no_go(ABC, vec![no_orgao, com_a_abc.clone()]);
        assert_eq!(r.unwrap().destino.id_unidade, "18141");
        assert_eq!(feitos, pedidos_da_busca(&[filtrada(), filtrada_alargada()]));
    }
}

#[test]
fn sem_a_sigla_no_orgao_volta_ao_orgao_da_tela() {
    let (r, feitos) = no_go(ABC, vec![vazia(), gecont_de_outros(), da_abc()]);
    assert_eq!(r.unwrap().destino, destino_abc());
    assert_eq!(
        feitos,
        pedidos_da_busca(&[filtrada(), filtrada_alargada(), inteira()])
    );
}

#[test]
fn quatro_consultas_no_maximo_e_so_a_sigla_exata() {
    let com_a_abc = resposta_xml(&[
        ("18440", "AGRODEFESA/GECONT-18440 - Gerencia"),
        ("18141", "abc/gecont-18141 - Gerencia"),
    ]);
    let quatro = pedidos_da_busca(&[filtrada(), filtrada_alargada(), inteira(), alargada()]);
    let (r, feitos) = no_go(ABC, vec![vazia(), vazia(), vazia(), com_a_abc]);
    assert_eq!(r.unwrap().destino.id_unidade, "18141");
    assert_eq!(feitos, quatro);
    // Inexistente (o `ABC/GECONT-18141` do GO, medido pelo Todos): não
    // oferecido, com a lista da última resposta (a alargada no da tela).
    let (r, feitos) = no_go(ABC, vec![vazia(), vazia(), vazia(), gecont_de_outros()]);
    assert_eq!(
        r,
        Err(ErroEnviar::DestinoNaoOferecidoPeloSei {
            oferecidos: vec![
                "AGRODEFESA/GECONT-18440 - Gerencia".into(),
                "CASACIVIL/GECONT-18148 - Gerencia".into(),
                "ECONOMIA/GECONT GERAL-13178 - Gerencia".into(),
            ]
        })
    );
    assert_eq!(feitos, quatro);
    // Itens sem a sigla exata também passam à consulta seguinte.
    let (r, feitos) = no_go(
        ABC,
        vec![
            gecont_de_outros(),
            gecont_de_outros(),
            gecont_de_outros(),
            vazia(),
        ],
    );
    assert_eq!(
        r,
        Err(ErroEnviar::DestinoNaoOferecidoPeloSei { oferecidos: vec![] })
    );
    assert_eq!(feitos, quatro);
    let dois = resposta_xml(&[
        ("1", "ABC/GECONT-18141 - Um"),
        ("2", "abc/gecont-18141 - Dois"),
    ]);
    let (r, feitos) = no_go(ABC, vec![vazia(), vazia(), vazia(), dois]);
    assert_eq!(
        r,
        Err(ErroEnviar::VariosDestinosComASigla {
            candidatos: vec![
                "ABC/GECONT-18141 - Um".into(),
                "abc/gecont-18141 - Dois".into()
            ]
        })
    );
    assert_eq!(feitos, quatro);
}

#[test]
fn varios_no_orgao_e_recusa_sem_voltar_ao_da_tela() {
    let dois = resposta_xml(&[
        ("1", "ABC/GECONT-18141 - Um"),
        ("2", "ABC/GECONT-18141 - Dois"),
    ]);
    let (r, feitos) = no_go(ABC, vec![dois]);
    assert!(
        matches!(r, Err(ErroEnviar::VariosDestinosComASigla { .. })),
        "{r:?}"
    );
    assert_eq!(feitos, pedidos_da_busca(&[filtrada()]));
}

#[test]
fn falha_de_uma_consulta_encerra_a_busca() {
    let (r, feitos) = no_go(ABC, vec!["<html>login</html>".into()]);
    assert_eq!(
        r,
        Err(ErroEnviar::Consulta(
            FalhaDaConsulta::RespostaDoAutocompletarNaoXml
        ))
    );
    assert_eq!(feitos, pedidos_da_busca(&[filtrada()]));
    let (r, feitos) = no_go(ABC, vec![vazia(), vazia(), "<html/>".into()]);
    assert_eq!(
        r,
        Err(ErroEnviar::Consulta(
            FalhaDaConsulta::RespostaDoAutocompletarNaoXml
        ))
    );
    assert_eq!(
        feitos,
        pedidos_da_busca(&[filtrada(), filtrada_alargada(), inteira()])
    );
}

#[test]
fn sem_orgao_do_prefixo_a_busca_e_no_orgao_da_tela() {
    // `CASACIVIL` não é rótulo do `selOrgao` (é `CASA CIVIL`).
    let sigla = "CASACIVIL/GECONT-18148";
    let codificada = "CASACIVIL%2FGECONT-18148";
    let (r, feitos) = no_go(sigla, vec![gecont_de_outros()]);
    assert_eq!(r.unwrap().destino.id_unidade, "18148");
    assert_eq!(
        feitos,
        pedidos_da_busca(&[corpo_da_consulta_com(codificada, "")])
    );
    let (r, feitos) = no_go(sigla, vec![vazia(), gecont_de_outros()]);
    assert_eq!(r.unwrap().destino.id_unidade, "18148");
    assert_eq!(
        feitos,
        pedidos_da_busca(&[corpo_da_consulta_com(codificada, ""), alargada()])
    );
    let outra = resposta_xml(&[("12318", "CASACIVIL/COMSET-12318 - Outra")]);
    let (r, feitos) = no_go(sigla, vec![outra, gecont_de_outros()]);
    assert_eq!(r.unwrap().destino.id_unidade, "18148");
    assert_eq!(
        feitos,
        pedidos_da_busca(&[corpo_da_consulta_com(codificada, ""), alargada()])
    );
    // Sem `/`, uma consulta só, como sempre (vazia: não oferecido).
    let (r, feitos) = no_go("TESTE", vec![vazia()]);
    assert_eq!(
        r,
        Err(ErroEnviar::DestinoNaoOferecidoPeloSei { oferecidos: vec![] })
    );
    assert_eq!(
        feitos,
        pedidos_da_busca(&[corpo_da_consulta_com("TESTE", "")])
    );
}
