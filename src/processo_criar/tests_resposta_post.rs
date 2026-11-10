//! Regressões sintéticas #245: resposta pós-POST sem prova não pode
//! encerrar o fluxo sem uma única releitura independente de Gerados.
use super::tests::{HASH, cadastro, lista, pedido, tela_controle};
use super::tests_candidatos::{
    arvore_com_propriedades, gerados_com_candidato, propriedades_correspondentes,
};
use super::*;
use crate::processo_concluir::tests_apoio::{NUP, ambiente, ok, r302, rodar_com, trabalhar};
use crate::transporte::FalhaTransporte;

#[test]
fn post_criador_responde_html_200_e_rele_gerados_sem_retry() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        ok("<html><body>resposta inesperada</body></html>".to_owned()),
        ok(tela_controle()),
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
    let erro = resultado.unwrap_err();
    assert_eq!(erro.saida(), 3);
    let diagnostico = erro.diagnostico_apos_post().expect("releitura obrigatoria");
    assert_eq!(diagnostico.causa_post, "resposta_inesperada");
    assert_eq!(diagnostico.causa_releitura, None);
    assert_eq!(diagnostico.candidatos_observados, Some(vec![]));
    assert_eq!(requests.len(), 5);
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
    assert_eq!(requests[4].metodo, "GET");
    assert_eq!(
        requests[4].url,
        crate::processo_concluir::tests_apoio::url_controle()
    );
}

#[test]
fn location_ambigua_rele_controle_sem_seguir_location() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        r302(&format!(
            "controlador.php?acao=procedimento_trabalhar&acao=procedimento_excluir&id_procedimento=10&infra_hash={HASH}"
        )),
        ok(tela_controle()),
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
    let erro = resultado.unwrap_err();
    assert_eq!(erro.saida(), 3);
    let diagnostico = erro.diagnostico_apos_post().expect("releitura obrigatoria");
    assert_eq!(diagnostico.causa_post, "localizacao_invalida");
    assert_eq!(diagnostico.causa_releitura, None);
    assert_eq!(requests.len(), 5);
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
    assert_eq!(
        requests[4].url,
        crate::processo_concluir::tests_apoio::url_controle()
    );
}

#[test]
fn retorno_inesperado_e_releitura_falha_preservam_as_duas_causas() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        ok("<html><body>nao e redirecionamento</body></html>".to_owned()),
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
    let diagnostico = resultado.unwrap_err().diagnostico_apos_post().unwrap();
    assert_eq!(diagnostico.causa_post, "resposta_inesperada");
    assert_eq!(diagnostico.causa_releitura, Some("rede"));
    assert_eq!(diagnostico.candidatos_observados, None);
    assert_eq!(requests.len(), 5);
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
}

// Casos 4.0 antigos preservados ao extrair as regressões do arquivo grande.
#[test]
fn redirecionamento_com_duas_acoes_nao_eh_seguido() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        r302(&format!(
            "controlador.php?acao=procedimento_trabalhar&acao=procedimento_excluir&id_procedimento=10&infra_hash={HASH}"
        )),
        ok(tela_controle()),
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
    assert_eq!(
        resultado
            .unwrap_err()
            .diagnostico_apos_post()
            .unwrap()
            .causa_post,
        "localizacao_invalida"
    );
    assert_eq!(requests.len(), 5);
    assert_eq!(
        requests.iter().filter(|req| req.metodo == "POST").count(),
        1
    );
}

#[test]
fn redirecionamento_com_id_invalido_duplicado_nao_eh_seguido() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        r302(&format!(
            "controlador.php?acao=procedimento_trabalhar&id_procedimento=10&id_procedimento=invalido&infra_hash={HASH}"
        )),
        ok(tela_controle()),
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
    assert_eq!(
        resultado
            .unwrap_err()
            .diagnostico_apos_post()
            .unwrap()
            .causa_post,
        "localizacao_invalida"
    );
    assert_eq!(requests.len(), 5);
    assert_eq!(
        requests.iter().filter(|req| req.metodo == "POST").count(),
        1
    );
}

/// #245: Location e árvore identificam o NUP, mas não demonstram por si
/// que o processo possui o tipo e a especificação enviados. A leitura
/// independente das propriedades é a prova antes de publicar confirmado.
#[test]
fn redirecionamento_valido_com_propriedades_divergentes_nao_confirma() {
    use crate::processo_concluir::tests_apoio::{NUP, UNIDADE, arvore, trabalhar};
    use crate::propriedades_processo::tests_apoio::{
        acao_da_barra, link_das_propriedades, tela_das_propriedades,
    };

    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        r302(&format!(
            "controlador.php?acao=procedimento_trabalhar&id_procedimento=10&infra_hash={HASH}"
        )),
        ok(trabalhar()),
        ok(arvore(
            &[UNIDADE],
            &[acao_da_barra(&link_das_propriedades(
                "procedimento_alterar",
                "10",
            ))],
        )),
        // Mesmo ID/NUP, mas tipo e especificação diferentes dos do pedido.
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
    let erro = resultado.expect_err("Location e árvore não provam o cadastro solicitado");
    assert_eq!(erro.saida(), 3);
    assert_eq!(erro.subcodigo(), "propriedades_divergentes");
    assert_eq!(requests.len(), 7); // 3 GET + 1 POST + tela + árvore + propriedades
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
    assert_eq!(requests[6].metodo, "GET");
}

#[test]
fn redirecionamento_valido_get_verificador_503_preserva_causa_http() {
    // O POST recebeu um Location esperado, mas o GET verificador foi
    // recusado por HTTP 503. A criação pode ter acontecido: saída 3,
    // preservando o status da leitura sem repetir a escrita.
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        r302(&format!(
            "controlador.php?acao=procedimento_trabalhar&id_procedimento=10&infra_hash={HASH}"
        )),
        Err(FalhaTransporte::StatusHttpDeErroDoServidor(
            crate::transporte::StatusHttpRecusado::sem_mensagem(503),
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
    let erro = resultado.expect_err("sem GET verificador não se confirma a criação");
    assert_eq!(erro.saida(), 3);
    assert_eq!(erro.status_http().map(|status| status.status), Some(503));
    assert_eq!(requests.len(), 5); // 3 GET + 1 POST criador + 1 GET verificador
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
    assert_eq!(requests[4].metodo, "GET");
}

#[test]
fn redirecionamento_valido_get_verificador_sem_rede_preserva_causa() {
    let amb = ambiente();
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        r302(&format!(
            "controlador.php?acao=procedimento_trabalhar&id_procedimento=10&infra_hash={HASH}"
        )),
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
    let erro = resultado.expect_err("falha da leitura é incerta, não recusa do POST");
    assert_eq!(erro.saida(), 3);
    assert_eq!(erro.subcodigo(), "rede");
    assert_eq!(requests.len(), 5);
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
}

/// O redirecionamento de um POST criador não prova criação nova se o
/// mesmo ID já estava na fotografia completa dos Gerados anterior ao POST.
/// A leitura das propriedades poderia coincidir até em conta compartilhada.
#[test]
fn location_para_id_preexistente_nao_confirma_criacao() {
    let amb = ambiente();
    let respostas = vec![
        ok(gerados_com_candidato()),
        ok(lista(false)),
        ok(cadastro()),
        r302(&format!(
            "controlador.php?acao=procedimento_trabalhar&id_procedimento=10&infra_hash={HASH}"
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
    let erro = resultado.expect_err("ID preexistente não comprova nova criação");
    assert_eq!(erro.saida(), 3);
    assert_eq!(erro.subcodigo(), "identidade_preexistente");
    assert_eq!(requests.len(), 4); // 3 GET de preparação + 1 POST, zero GET inútil
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
}

#[test]
fn nivel_restrito_concorrente_nao_prova_criacao_publica() {
    // #245: a conta compartilhada pode ganhar processo com mesmo tipo e
    // especificação, mas nível de acesso diferente do POST.
    let amb = ambiente();
    let propriedades = propriedades_correspondentes()
        .replace("value=\"0\" checked=\"checked\" class", "value=\"0\" class")
        .replace(
            "value=\"1\"  class=\"infraRadio\"",
            "value=\"1\" checked=\"checked\" class=\"infraRadio\"",
        );
    let respostas = vec![
        ok(tela_controle()),
        ok(lista(false)),
        ok(cadastro()),
        Err(FalhaTransporte::Rede),
        ok(gerados_com_candidato()),
        ok(trabalhar()),
        ok(arvore_com_propriedades()),
        ok(propriedades),
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
    let erro = resultado.expect_err("nível divergente não comprova a criação");
    assert_eq!(erro.saida(), 3);
    assert_eq!(
        erro.diagnostico_apos_post().unwrap().candidatos_observados,
        Some(vec![NUP.to_owned()])
    );
    assert_eq!(requests.len(), 8); // 3 GET iniciais + 1 POST + 1 GET Controle + 3 GET de prova
    assert_eq!(requests.iter().filter(|r| r.metodo == "POST").count(), 1);
}

#[test]
fn location_com_fragmento_da_arvore_identifica_o_processo_criado() {
    // UnB 4.0.9 (cassette documento-criar-corpo-unb, interação 5): o 302 do
    // procedimento_gerar termina em `#ID-<id>`, a âncora da árvore.
    let instalacao =
        crate::instalacao::Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let envio = format!(
        "https://sei.exemplo.gov.br/sei/controlador.php?acao=procedimento_gerar&acao_origem=procedimento_gerar&infra_sistema=100000100&infra_unidade_atual=110000846&infra_hash={HASH}"
    );
    let localizacao = format!(
        "controlador.php?acao=procedimento_trabalhar&acao_origem=procedimento_gerar&acao_retorno=procedimento_escolher_tipo&id_procedimento=51229&atualizar_arvore=1&infra_sistema=100000100&infra_unidade_atual=110000846&infra_hash={HASH}#ID-51229"
    );
    let r = super::links::retorno_da_criacao(&instalacao, &localizacao, &envio);
    assert_eq!(r.map(|(_, id)| id), Ok("51229".to_owned()));
}
