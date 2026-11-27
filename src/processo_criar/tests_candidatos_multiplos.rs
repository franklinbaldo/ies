//! Reconciliação de múltiplos candidatos e causas de releituras incompletas.
use super::*;

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
