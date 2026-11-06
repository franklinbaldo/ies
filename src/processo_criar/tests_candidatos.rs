//! RED #245: a resposta inconclusiva exige conferência independente dos
//! candidatos da caixa Gerados; não basta observar a linha nova.
use super::tests::{cadastro, lista, pedido, tela_controle};
use super::*;
use crate::processo_concluir::tests_apoio::{
    NUP, UNIDADE, ambiente, arvore, ok, rodar_com, trabalhar, trabalhar_controle,
};
use crate::propriedades_processo::tests_apoio::{
    acao_da_barra, link_das_propriedades, tela_das_propriedades,
};
use crate::transporte::FalhaTransporte;

fn gerados_com_candidato() -> String {
    tela_controle().replace(
        "<table id=\"tblProcessosGerados\"><caption>0 registros:</caption></table>",
        &format!(
            "<table id=\"tblProcessosGerados\"><caption>1 registros - 1 a 1):</caption>\
             <tr id=\"P10\"><td><a href=\"{}\" class=\"processoVisualizado\">{NUP}</a></td></tr></table>",
            trabalhar_controle("10")
        ),
    )
}

fn arvore_com_propriedades() -> String {
    arvore(
        &[UNIDADE],
        &[acao_da_barra(&link_das_propriedades(
            "procedimento_alterar",
            "10",
        ))],
    )
}

fn propriedades_correspondentes() -> String {
    tela_das_propriedades(NUP, "10")
        .replace("Tipo Sintetico: Teste", "Administração Geral: Ensaio")
        .replace("Especificação sintética &amp; teste", "Ensaio automatizado issue 245")
}

#[test]
fn post_falha_mas_propriedades_conferidas_provam_criacao_unica() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        Err(FalhaTransporte::Rede),
        ok(gerados_com_candidato()),
        ok(trabalhar()),
        ok(arvore_com_propriedades()),
        ok(propriedades_correspondentes()),
    ];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(sessao.origem, sessao.url_do_controle, sessao.cookie, &pedido(true), rede)
    });
    let feito = resultado.expect("um candidato comprovado nao permanece incerto");
    assert!(feito.confirmado);
    assert_eq!(feito.nup.as_deref(), Some(NUP));
    assert_eq!(feito.requisicoes, 8);
    assert_eq!(requests.len(), 8); // 3 GET + 1 POST + 1 Controle + 3 GET de processo
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
    assert!(requests[5..].iter().all(|r| r.metodo == "GET"));
    assert!(requests[5].url.contains("acao=procedimento_trabalhar"));
    assert!(requests[6].url.contains("acao=procedimento_visualizar"));
    assert!(requests[7].url.contains("acao=procedimento_alterar"));
}

#[test]
fn candidato_com_propriedades_de_outro_tipo_nao_e_prova_de_criacao() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        Err(FalhaTransporte::Rede),
        ok(gerados_com_candidato()),
        ok(trabalhar()),
        ok(arvore_com_propriedades()),
        ok(tela_das_propriedades(NUP, "10")),
    ];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(sessao.origem, sessao.url_do_controle, sessao.cookie, &pedido(true), rede)
    });
    assert!(resultado.is_err());
    assert_eq!(requests.len(), 8);
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
}

#[test]
fn falha_da_leitura_de_propriedades_apos_post_nao_repete_a_escrita() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        Err(FalhaTransporte::Rede),
        ok(gerados_com_candidato()),
        ok(trabalhar()),
        ok(arvore_com_propriedades()),
        Err(FalhaTransporte::Rede),
    ];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(sessao.origem, sessao.url_do_controle, sessao.cookie, &pedido(true), rede)
    });
    let erro = resultado.expect_err("falha da leitura nao e confirmacao");
    assert_eq!(erro.saida(), 3);
    assert_eq!(requests.len(), 8);
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
}
