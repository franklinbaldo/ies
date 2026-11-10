//! Saída JSON e humana de `processo reabrir`: campos do contrato, nada de
//! URL, e a falha com a situação e a verificação.
use super::{humano, json_de_sucesso};
use crate::arvore_processo::{FormaDaReabertura, MotivoAbertoEm};
use crate::processo_concluir::{FalhaDoEnvio, VerificacaoPorLeitura};
use crate::processo_reabrir::{ErroReabrir, ProcessoReaberto};
use serde_json::json;

fn previa() -> ProcessoReaberto {
    ProcessoReaberto {
        nup: "00001.000001/2026-01".into(),
        id_procedimento: "10".into(),
        unidade: "TESTE".into(),
        confirmado: false,
        alterado: false,
        oferecido: true,
        forma: None,
        aberto_em: vec!["TESTE_1_1".into()],
        depois: None,
        reconciliado_por: None,
        requisicoes: 3,
    }
}

#[test]
fn previa_publica_o_contrato_sem_url() {
    assert_eq!(
        json_de_sucesso(&previa()),
        json!({"nup": "00001.000001/2026-01", "id_procedimento": "10", "unidade": "TESTE",
               "origem": "web", "confirmado": false, "alterado": false,
               "reabertura_oferecida": true, "forma": null, "aberto_em": ["TESTE_1_1"],
               "depois": null, "reconciliado_por_leitura": false, "requisicoes": 3})
    );
    let texto = humano(&previa());
    assert!(texto.contains("Aberto em: TESTE_1_1"), "{texto}");
    assert!(
        texto.contains("Para reabrir, repita com --confirmar"),
        "{texto}"
    );
    assert!(texto.ends_with("Requisições: 3"), "{texto}");
}

#[test]
fn ja_aberto_e_reaberto_na_saida_humana() {
    let ja = ProcessoReaberto {
        oferecido: false,
        aberto_em: vec!["TESTE".into()],
        ..previa()
    };
    assert!(humano(&ja).contains("Nada a fazer"));
    assert_eq!(json_de_sucesso(&ja)["reabertura_oferecida"], false);
    let feito = ProcessoReaberto {
        confirmado: true,
        alterado: true,
        forma: Some(FormaDaReabertura::Variavel),
        depois: Some(vec!["TESTE".into(), "TESTE_1_1".into()]),
        reconciliado_por: Some(FalhaDoEnvio::RedirecionamentoNaReabertura),
        requisicoes: 6,
        ..previa()
    };
    let texto = humano(&feito);
    assert!(
        texto.contains("Reaberto. Aberto em, relido: TESTE, TESTE_1_1"),
        "{texto}"
    );
    assert!(texto.contains("redirecionamento_na_reabertura"), "{texto}");
    let valor = json_de_sucesso(&feito);
    assert_eq!(valor["forma"], "variavel");
    assert_eq!(valor["reconciliado_por_leitura"], true);
    assert_eq!(
        valor["falha_do_envio"],
        json!({"subcodigo": "redirecionamento_na_reabertura", "status_http": 302})
    );
}

#[test]
fn falhas_trazem_a_situacao_e_a_verificacao() {
    let erro = ErroReabrir::EfeitoIncertoNaReabertura {
        aberto_em: vec![],
        falha_do_envio: None,
        verificacao: Box::new(VerificacaoPorLeitura::SituacaoNaoLida(
            MotivoAbertoEm::HtmlDaRaizNaoPublicado,
        )),
    };
    assert_eq!(erro.saida(), 3);
    let valor = json_de_falha(&erro);
    assert_eq!(valor["codigo"], "efeito_incerto");
    assert_eq!(valor["subcodigo"], "efeito_incerto_na_reabertura");
    assert_eq!(valor["aberto_em"], json!([]));
    assert_eq!(valor["verificacao"], "html_da_raiz_nao_publicado");
    let nao = ErroReabrir::ReaberturaNaoAplicada {
        aberto_em: vec![],
        depois: vec!["OUTRA".into()],
        falha_do_envio: Some(FalhaDoEnvio::RedirecionamentoNaReabertura),
    };
    let valor = json_de_falha(&nao);
    assert_eq!(
        (nao.saida(), valor["codigo"].as_str()),
        (1, Some("escrita_nao_aplicada"))
    );
    assert_eq!(valor["depois"], json!(["OUTRA"]));
    assert_eq!(valor["status_http"], 302);
    assert_eq!(
        valor["falha_do_envio"]["subcodigo"],
        "redirecionamento_na_reabertura"
    );
    let ja = ErroReabrir::UnidadeDaSessaoJaNaSituacao {
        aberto_em: vec!["TESTE".into()],
    };
    assert_eq!(json_de_falha(&ja)["aberto_em"], json!(["TESTE"]));
}

#[test]
fn saida_tipificada_serializa_direto_e_preserva_campos_opcionais() {
    let previa = previa();
    let mut bytes = Vec::new();
    serde_json::to_writer_pretty(&mut bytes, &super::resultado_reabertura(&previa)).unwrap();
    // O objeto JSON legado ordenava as chaves: manter também os bytes pretty.
    assert_eq!(
        std::str::from_utf8(&bytes).unwrap(),
        serde_json::to_string_pretty(&json_de_sucesso(&previa)).unwrap()
    );
    let recebido: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(recebido, json_de_sucesso(&previa));
    assert!(recebido.get("falha_do_envio").is_none());
    assert!(recebido["forma"].is_null());
    assert!(!String::from_utf8(bytes).unwrap().contains("infra_hash"));
}

#[test]
fn erro_tipado_da_reabertura_preserva_situacao_resposta_e_verificacao() {
    let erro = ErroReabrir::EfeitoIncertoNaReabertura {
        aberto_em: vec!["TESTE_1_1".into()],
        falha_do_envio: Some(FalhaDoEnvio::RedirecionamentoNaReabertura),
        verificacao: Box::new(VerificacaoPorLeitura::SituacaoNaoLida(
            MotivoAbertoEm::HtmlDaRaizNaoPublicado,
        )),
    };
    assert_eq!(
        super::linhas_da_falha(&erro),
        vec![
            format!("IES: {} [{}]", erro.mensagem(), erro.subcodigo()),
            "IES: aberto_em: TESTE_1_1".to_owned(),
            "IES: resposta da escrita: redirecionamento_na_reabertura".to_owned(),
            "IES: verificação: html_da_raiz_nao_publicado".to_owned(),
        ]
    );
}

#[test]
fn erro_tipado_da_reabertura_preserva_depois_sem_json() {
    let erro = ErroReabrir::ReaberturaNaoAplicada {
        aberto_em: vec![],
        depois: vec!["OUTRA".into()],
        falha_do_envio: None,
    };
    assert_eq!(
        super::linhas_da_falha(&erro),
        vec![
            format!("IES: {} [{}]", erro.mensagem(), erro.subcodigo()),
            "IES: aberto_em: (nenhuma unidade)".to_owned(),
            "IES: depois: OUTRA".to_owned(),
        ]
    );
}

#[test]
fn falha_serializavel_preserva_goldens_compactos_sem_value() {
    let casos = [
        (
            ErroReabrir::UnidadeDaSessaoNaoPublicada,
            r#"{"codigo":"fonte_invalida","erro":"a abertura não publicou a unidade da sessão; nada foi reaberto","retentavel":false,"subcodigo":"unidade_da_sessao_nao_publicada"}"#,
        ),
        (
            ErroReabrir::SituacaoDoProcessoNaoLidaAntesDeReabrir {
                motivo: MotivoAbertoEm::HtmlDaRaizNaoPublicado,
            },
            r#"{"codigo":"fonte_invalida","erro":"a árvore não publica onde o processo está aberto, e a reabertura não poderia ser conferida; nada foi reaberto","motivo":"html_da_raiz_nao_publicado","retentavel":false,"subcodigo":"situacao_do_processo_nao_lida_antes_de_reabrir"}"#,
        ),
        (
            ErroReabrir::ReaberturaNaoAplicada {
                aberto_em: vec![],
                depois: vec![],
                falha_do_envio: None,
            },
            r#"{"aberto_em":[],"codigo":"escrita_nao_aplicada","depois":[],"erro":"a árvore lida depois de reabrir não lista a unidade da sessão: o processo não foi reaberto","retentavel":false,"subcodigo":"reabertura_nao_aplicada"}"#,
        ),
        (
            ErroReabrir::EfeitoIncertoNaReabertura {
                aberto_em: vec!["TESTE".into()],
                falha_do_envio: None,
                verificacao: Box::new(VerificacaoPorLeitura::Lida(vec![])),
            },
            r#"{"aberto_em":["TESTE"],"codigo":"efeito_incerto","erro":"não se sabe se o processo foi reaberto; confira com `ies processo NUP` antes de repetir","retentavel":false,"subcodigo":"efeito_incerto_na_reabertura","verificacao":null}"#,
        ),
        (
            ErroReabrir::EfeitoIncertoNaReabertura {
                aberto_em: vec!["TESTE".into()],
                falha_do_envio: Some(FalhaDoEnvio::RedirecionamentoNaReabertura),
                verificacao: Box::new(VerificacaoPorLeitura::Lida(vec![])),
            },
            r#"{"aberto_em":["TESTE"],"codigo":"efeito_incerto","erro":"não se sabe se o processo foi reaberto; confira com `ies processo NUP` antes de repetir","falha_do_envio":{"status_http":302,"subcodigo":"redirecionamento_na_reabertura"},"retentavel":false,"status_http":302,"subcodigo":"efeito_incerto_na_reabertura","verificacao":null}"#,
        ),
    ];
    for (erro, esperado) in casos {
        let projecao = super::json_de_falha(&erro);
        assert_eq!(serde_json::to_string(&projecao).unwrap(), esperado);
        // RED: o contrato deve ser uma projeção Serialize, não um Value.
        assert_ne!(
            std::any::type_name_of_val(&projecao),
            std::any::type_name::<serde_json::Value>()
        );
        assert!(!esperado.contains('\u{1b}'));
    }
}

fn json_de_falha(erro: &ErroReabrir) -> serde_json::Value {
    serde_json::to_value(super::json_de_falha(erro)).unwrap()
}

#[test]
fn falha_http_preserva_mensagem_status_e_identidade_original() {
    use crate::formulario_web::ErroSubmissaoFormulario;
    use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

    let casos = [
        (
            FalhaDoEnvio::Envio(ErroSubmissaoFormulario::ValidacaoRecusadaPeloSei(
                StatusHttpRecusado {
                    status: 200,
                    mensagem_do_servidor: Some("Mensagem saneada.".into()),
                },
            )),
            r#"{"aberto_em":[],"codigo":"efeito_incerto","erro":"não se sabe se o processo foi reaberto; confira com `ies processo NUP` antes de repetir","falha_do_envio":{"mensagem_do_servidor":"Mensagem saneada.","subcodigo":"validacao_recusada_pelo_sei"},"mensagem_do_servidor":"Mensagem saneada.","retentavel":false,"subcodigo":"efeito_incerto_na_reabertura","verificacao":"html_da_raiz_nao_publicado"}"#,
        ),
        (
            FalhaDoEnvio::Transporte(FalhaTransporte::StatusHttpDeErroDoServidor(
                StatusHttpRecusado {
                    status: 500,
                    mensagem_do_servidor: Some("Mensagem saneada.".into()),
                },
            )),
            r#"{"aberto_em":[],"codigo":"efeito_incerto","erro":"não se sabe se o processo foi reaberto; confira com `ies processo NUP` antes de repetir","falha_do_envio":{"mensagem_do_servidor":"Mensagem saneada.","status_http":500,"subcodigo":"status_http_de_erro_do_servidor"},"mensagem_do_servidor":"Mensagem saneada.","retentavel":false,"status_http":500,"subcodigo":"efeito_incerto_na_reabertura","verificacao":"html_da_raiz_nao_publicado"}"#,
        ),
    ];
    for (falha, esperado) in casos {
        let erro = ErroReabrir::EfeitoIncertoNaReabertura {
            aberto_em: vec![],
            falha_do_envio: Some(falha),
            verificacao: Box::new(VerificacaoPorLeitura::SituacaoNaoLida(
                MotivoAbertoEm::HtmlDaRaizNaoPublicado,
            )),
        };
        assert_eq!(erro.saida(), 3);
        assert_eq!(
            serde_json::to_string(&super::json_de_falha(&erro)).unwrap(),
            esperado
        );
    }
}
