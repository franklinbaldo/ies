//! Regressões sintéticas #245: resposta pós-POST sem prova não pode
//! encerrar o fluxo sem uma única releitura independente de Gerados.
use super::*;
use super::tests::{cadastro, lista, pedido, tela_controle, HASH};
use crate::processo_concluir::tests_apoio::{ambiente, ok, r302, rodar_com};
use crate::transporte::FalhaTransporte;

#[test]
fn post_criador_responde_html_200_e_rele_gerados_sem_retry() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()), ok(lista(false)), ok(cadastro()),
        ok("<html><body>resposta inesperada</body></html>".to_owned()),
        ok(tela_controle()),
    ];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(sessao.origem, sessao.url_do_controle, sessao.cookie, &pedido(true), rede)
    });
    let erro = resultado.unwrap_err();
    assert_eq!(erro.saida(), 3);
    let diagnostico = erro.diagnostico_apos_post().expect("releitura obrigatoria");
    assert_eq!(diagnostico.causa_post, "resposta_inesperada_apos_post_criador");
    assert_eq!(diagnostico.causa_releitura, None);
    assert_eq!(diagnostico.candidatos_observados, Some(vec![]));
    assert_eq!(requests.len(), 5);
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
    assert_eq!(requests[4].metodo, "GET");
    assert_eq!(requests[4].url, crate::processo_concluir::tests_apoio::url_controle());
}

#[test]
fn location_ambigua_rele_controle_sem_seguir_location() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()), ok(lista(false)), ok(cadastro()),
        r302(&format!(
            "controlador.php?acao=procedimento_trabalhar&acao=procedimento_excluir&id_procedimento=10&infra_hash={HASH}"
        )),
        ok(tela_controle()),
    ];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(sessao.origem, sessao.url_do_controle, sessao.cookie, &pedido(true), rede)
    });
    let erro = resultado.unwrap_err();
    assert_eq!(erro.saida(), 3);
    let diagnostico = erro.diagnostico_apos_post().expect("releitura obrigatoria");
    assert_eq!(diagnostico.causa_post, "localizacao_invalida_apos_post_criador");
    assert_eq!(diagnostico.causa_releitura, None);
    assert_eq!(requests.len(), 5);
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
    assert_eq!(requests[4].url, crate::processo_concluir::tests_apoio::url_controle());
}

#[test]
fn retorno_inesperado_e_releitura_falha_preservam_as_duas_causas() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()), ok(lista(false)), ok(cadastro()),
        ok("<html><body>nao e redirecionamento</body></html>".to_owned()),
        Err(FalhaTransporte::Rede),
    ];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(sessao.origem, sessao.url_do_controle, sessao.cookie, &pedido(true), rede)
    });
    let diagnostico = resultado.unwrap_err().diagnostico_apos_post().unwrap();
    assert_eq!(diagnostico.causa_post, "resposta_inesperada_apos_post_criador");
    assert_eq!(diagnostico.causa_releitura, Some("falha_de_rede"));
    assert_eq!(diagnostico.candidatos_observados, None);
    assert_eq!(requests.len(), 5);
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
}
