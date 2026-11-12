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
        .replace("Tipo Sintetico: Teste", "Administração Geral: Ensaio")
        .replace(
            "Especificação sintética &amp; teste",
            "Ensaio automatizado issue 245",
        )
}

#[test]
fn dois_candidatos_web_comprovados_sao_publicados_como_ambiguos() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        Err(FalhaTransporte::Rede),
        ok(gerados_com_dois_candidatos()),
        ok(trabalhar()),
        ok(arvore_com_propriedades()),
        ok(propriedades_correspondentes()),
        ok(crate::arvore_processo::tests_tela::tela_do_processo(
            UNIDADE,
            &[crate::arvore_processo::tests_tela::src_da_arvore("11")],
        )),
        ok(arvore_com_propriedades_de("11", SEGUNDO_NUP)),
        ok(propriedades_correspondentes_de("11", SEGUNDO_NUP)),
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
    let erro = resultado.expect_err("duas provas nao permitem atribuir o POST");
    let diagnostico = erro.diagnostico_apos_post().expect("diagnostico tipado");
    assert_eq!(
        diagnostico.candidatos_observados,
        Some(vec![NUP.to_owned(), SEGUNDO_NUP.to_owned()])
    );
    assert_eq!(
        diagnostico.candidatos_compativeis,
        Some(vec![NUP.to_owned(), SEGUNDO_NUP.to_owned()])
    );
    assert_eq!(
        diagnostico.evidencia(),
        Some(crate::processo_criar_fluxo::erro_fluxo::EvidenciaDeCoincidencia::Multiplas)
    );
    assert_eq!(requests.len(), 11); // 3 GET + 1 POST + 1 Controle + 3 GET por candidato
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
    assert!(requests[4..].iter().all(|r| r.metodo == "GET"));
}

#[test]
fn dois_candidatos_mas_so_um_confere_exigem_todas_as_leituras() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        Err(FalhaTransporte::Rede),
        ok(gerados_com_dois_candidatos()),
        ok(trabalhar()),
        ok(arvore_com_propriedades()),
        ok(propriedades_correspondentes()),
        ok(crate::arvore_processo::tests_tela::tela_do_processo(
            UNIDADE,
            &[crate::arvore_processo::tests_tela::src_da_arvore("11")],
        )),
        ok(arvore_com_propriedades_de("11", SEGUNDO_NUP)),
        ok(tela_das_propriedades(SEGUNDO_NUP, "11")), // tipo diferente
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
    let erro = resultado.expect_err("uma coincidencia e insuficiente para demonstrar autoria");
    let diagnostico = erro.diagnostico_apos_post().unwrap();
    assert_eq!(diagnostico.candidatos_compativeis, Some(vec![NUP.into()]));
    assert_eq!(requests.len(), 11);
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
}

#[test]
fn falha_no_segundo_candidato_preserva_primeira_coincidencia_sem_declarar_aplicacao() {
    // A primeira conferência de tipo e especificação já foi observada.
    // A segunda leitura falha: a autoria e a unicidade NÃO foram provadas.
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        Err(FalhaTransporte::Rede),
        ok(gerados_com_dois_candidatos()),
        ok(trabalhar()),
        ok(arvore_com_propriedades()),
        ok(propriedades_correspondentes()),
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
    let erro = resultado.expect_err("a segunda leitura falhou; efeito segue incerto");
    let diagnostico = erro
        .diagnostico_apos_post()
        .expect("duas causas e provas parciais");
    assert_eq!(
        diagnostico.candidatos_observados,
        Some(vec![NUP.to_owned(), SEGUNDO_NUP.to_owned()])
    );
    assert_eq!(
        diagnostico.candidatos_compativeis,
        Some(vec![NUP.to_owned()])
    ); // prova individual já observada, sem classificar autoria nem unicidade
    assert_eq!(requests.len(), 9); // 3 GET + 1 POST + 1 Controle + 3 GET + 1 GET falho
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
}

#[test]
fn falha_apos_fotografia_preserva_candidatos_ja_observados() {
    // A fotografia posterior foi lida e revelou NUP novo, mas a leitura
    // independente da tela do candidato falhou. Null significaria
    // indevidamente que nenhum candidato foi observado.
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        Err(FalhaTransporte::Rede),
        ok(gerados_com_candidato()),
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
    let erro = resultado.expect_err("falha da conferência não comprova criação");
    let diagnostico = erro
        .diagnostico_apos_post()
        .expect("diagnóstico com ambas causas");
    assert_eq!(
        diagnostico.candidatos_observados,
        Some(vec![NUP.to_owned()])
    );
    assert_eq!(diagnostico.candidatos_compativeis, None);
    assert_eq!(requests.len(), 6); // 3 GET + 1 POST + Controle + tela do candidato
    assert_eq!(
        requests.iter().filter(|req| req.metodo == "POST").count(),
        1
    );
}

#[test]
fn falha_http_na_releitura_apos_post_sem_resposta_preserva_status_503() {
    // O POST ficou sem resposta e o GET posterior recebeu HTTP 503.
    // As duas causas e a mensagem saneada do GET são observações distintas.
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        Err(FalhaTransporte::Rede),
        Err(FalhaTransporte::Status(
            crate::transporte::StatusHttpRecusado {
                status: 503,
                mensagem_do_servidor: Some("Controle indisponível".to_owned()),
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
    let erro = resultado.expect_err("a escrita não foi comprovada");
    let diagnostico = erro.diagnostico_apos_post().expect("duas causas");
    assert_eq!(
        diagnostico
            .releitura_falhou
            .as_ref()
            .and_then(|falha| falha.http.as_ref())
            .map(|http| http.status),
        Some(503)
    );
    assert_eq!(diagnostico.candidatos_observados, None);
    // RED: status do GET era perdido porque só se consultava a causa do POST.
    assert_eq!(erro.status_http().map(|h| h.status), Some(503));
    assert_eq!(
        erro.status_http()
            .and_then(|h| h.mensagem_do_servidor.as_deref()),
        Some("Controle indisponível")
    );
    assert_eq!(requests.len(), 5); // 3 GET prévios + 1 POST + 1 GET falho
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
}

#[test]
fn post_503_e_releitura_502_sao_status_distintos_sem_retry() {
    // O POST pode retornar 503 enquanto a leitura posterior retorna 502.
    // Nunca ocultar o segundo fato pela existência de status no primeiro.
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        Err(FalhaTransporte::Status(
            crate::transporte::StatusHttpRecusado::sem_mensagem(503),
        )),
        Err(FalhaTransporte::Status(
            crate::transporte::StatusHttpRecusado {
                status: 502,
                mensagem_do_servidor: Some("Falha de leitura saneada".to_owned()),
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
    let erro = resultado.expect_err("duas falhas nao confirmam criacao");
    assert_eq!(erro.status_http().map(|http| http.status), Some(503));
    let diagnostico = erro.diagnostico_apos_post().expect("duas causas");
    let http_releitura = diagnostico
        .releitura_falhou
        .and_then(|falha| falha.http)
        .expect("HTTP da releitura");
    assert_eq!(http_releitura.status, 502);
    assert_eq!(
        http_releitura.mensagem_do_servidor.as_deref(),
        Some("Falha de leitura saneada")
    );
    assert_eq!(requests.len(), 5); // 3 GET + POST único + GET posterior
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
}
