//! RED #245: a resposta inconclusiva exige conferência independente dos
//! candidatos da caixa Gerados; não basta observar a linha nova.
use super::tests::{cadastro, lista, pedido, tela_controle};
use super::*;
use crate::acao_do_sei::AcaoDoSei;
use crate::processo_concluir::tests_apoio::{
    NUP, UNIDADE, ambiente, arvore, ok, rodar_com, trabalhar, trabalhar_controle,
};
use crate::propriedades_processo::tests_apoio::{
    acao_da_barra, link_das_propriedades, tela_das_propriedades,
};
use crate::transporte::FalhaTransporte;

pub(super) fn gerados_com_candidato() -> String {
    tela_controle().replace(
        "<table id=\"tblProcessosGerados\"><caption>0 registros:</caption></table>",
        &format!(
            "<table id=\"tblProcessosGerados\"><caption>Processos (1 registros - 1 a 1):</caption>\
             <tr id=\"P10\"><td><a href=\"{}\" class=\"processoVisualizado\">{NUP}</a></td></tr></table>",
            trabalhar_controle("10")
        ),
    )
}

pub(super) fn arvore_com_propriedades() -> String {
    arvore(
        &[UNIDADE],
        &[acao_da_barra(&link_das_propriedades(
            AcaoDoSei::ProcedimentoAlterar,
            "10",
        ))],
    )
}

pub(super) fn propriedades_correspondentes() -> String {
    tela_das_propriedades(NUP, "10")
        .replace("100000002", "12")
        .replace("Tipo Sintetico: Teste", "Administração Geral: Ensaio")
        .replace(
            "Especificação sintética &amp; teste",
            "Ensaio automatizado issue 245",
        )
}

#[test]
fn post_falha_e_propriedades_iguais_identificam_residuo_nao_autoria() {
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
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido(true),
            rede,
        )
    });
    let erro = resultado.expect_err("um candidato nao comprova autoria de POST sem resposta");
    let diagnostico = erro.diagnostico_apos_post().unwrap();
    assert_eq!(diagnostico.candidatos_compativeis, Some(vec![NUP.into()]));
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
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido(true),
            rede,
        )
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
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido(true),
            rede,
        )
    });
    let _erro = resultado.expect_err("falha da leitura nao e confirmacao");
    assert_eq!(requests.len(), 8);
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
}

#[test]
fn recusa_explicita_da_infra_nao_vira_criacao_por_candidato_concorrente() {
    let amb = ambiente();
    let recusa = "<html><body><textarea id=\"txaInfraValidacao\" name=\"txaInfraValidacao\">Criacao recusada.</textarea></body></html>";
    // A conta compartilhada pode receber candidato de outro agente;
    // depois da recusa nenhuma releitura deve tentar atribuí-lo a este POST.
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        ok(recusa.to_owned()),
    ];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido(true),
            rede,
        )
    });
    let erro = resultado.expect_err("recusa explicita nao confirma escrita");
    assert!(erro.nao_aplicada_comprovada()); // prova expressa de recusa na tela do SEI
    assert_eq!(requests.len(), 4); // 3 GET + 1 POST, sem consulta pos-recusa
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
}

#[test]
fn http_403_no_post_nao_vira_criacao_por_candidato_concorrente() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        Err(FalhaTransporte::Status(
            crate::transporte::StatusHttpRecusado {
                status: 403,
                mensagem_do_servidor: Some("Acesso negado.".to_owned()),
            },
        )),
    ];
    let (resultado, requests) = rodar_com(&amb, respostas, |sessao, _, _, rede| {
        criar(
            sessao.origem,
            sessao.url_do_controle,
            sessao.cookie,
            &pedido(true),
            rede,
        )
    });
    let erro = resultado.expect_err("recusa HTTP 403 nao confirma escrita");
    assert!(!erro.nao_aplicada_comprovada()); // 403 por si não prova ausência de efeito
    // HTTP 403 não prova não aplicação: o efeito segue incerto, saída 3.
    assert_eq!(erro.status_http().map(|status| status.status), Some(403));
    assert_eq!(requests.len(), 4); // 3 GET + 1 POST, sem consulta pos-recusa
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
}

pub(super) const SEGUNDO_NUP: &str = "00000.000011/2026-11";

/// Dois processos de ensaio distintos apareceram entre as fotografias.
/// Ambos podem ter sido criados por outros agentes na mesma unidade.
pub(super) fn gerados_com_dois_candidatos() -> String {
    tela_controle().replace(
        "<table id=\"tblProcessosGerados\"><caption>0 registros:</caption></table>",
        &format!(
            "<table id=\"tblProcessosGerados\"><caption>Processos (2 registros - 1 a 2):</caption>\
             <tr id=\"P10\"><td><a href=\"{}\" class=\"processoVisualizado\">{NUP}</a></td></tr>\
             <tr id=\"P11\"><td><a href=\"{}\" class=\"processoVisualizado\">{SEGUNDO_NUP}</a></td></tr></table>",
            trabalhar_controle("10"),
            trabalhar_controle("11")
        ),
    )
}

fn arvore_com_propriedades_de(id: &str, nup: &str) -> String {
    crate::arvore_processo::tests_concluir::arvore_para_concluir(
        id,
        nup,
        &crate::processo_concluir::tests_apoio::situacao(&[UNIDADE]),
        &[acao_da_barra(&link_das_propriedades(
            AcaoDoSei::ProcedimentoAlterar,
            id,
        ))],
    )
}

fn propriedades_correspondentes_de(id: &str, nup: &str) -> String {
    tela_das_propriedades(nup, id)
        .replace("100000002", "12")
        .replace("Tipo Sintetico: Teste", "Administração Geral: Ensaio")
        .replace(
            "Especificação sintética &amp; teste",
            "Ensaio automatizado issue 245",
        )
}

#[path = "tests_candidatos_multiplos.rs"]
mod tests_candidatos_multiplos;
