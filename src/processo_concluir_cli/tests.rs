//! Saída JSON e humana de `processo concluir`: campos do contrato, nada de
//! URL, e a falha com a situação e a verificação.
use super::{humano, json_de_sucesso};
use crate::arvore_processo::MotivoAbertoEm;
use crate::processo_concluir::{
    ErroConcluir, FalhaDoEnvio, FormaDaConclusao, ProcessoConcluido, VerificacaoPorLeitura,
};
use serde_json::json;

fn previa() -> ProcessoConcluido {
    ProcessoConcluido {
        nup: "00001.000001/2026-01".into(),
        id_procedimento: "10".into(),
        unidade: "TESTE".into(),
        confirmado: false,
        alterado: false,
        forma: Some(FormaDaConclusao::Tela),
        aberto_em: vec!["TESTE".into(), "TESTE_1_1".into()],
        depois: None,
        reconciliado_por: None,
        requisicoes: 3,
    }
}

#[test]
fn previa_publica_o_contrato_sem_url() {
    let valor = json_de_sucesso(&previa());
    assert_eq!(
        valor,
        json!({"nup": "00001.000001/2026-01", "id_procedimento": "10", "unidade": "TESTE",
               "origem": "web", "confirmado": false, "alterado": false, "forma": "tela",
               "aberto_em": ["TESTE", "TESTE_1_1"], "depois_esperado": ["TESTE_1_1"],
               "depois": null, "reconciliado_por_leitura": false, "requisicoes": 3})
    );
    let texto = humano(&previa());
    assert!(texto.ends_with("Requisições: 3"), "{texto}");
    assert!(texto.contains("Para concluir, repita com --confirmar"));
}

#[test]
fn reconciliado_traz_a_falha_do_envio() {
    let feito = ProcessoConcluido {
        confirmado: true,
        alterado: true,
        depois: Some(vec!["TESTE_1_1".into()]),
        reconciliado_por: Some(FalhaDoEnvio::RedirecionamentoNaoReconhecido),
        requisicoes: 6,
        ..previa()
    };
    let valor = json_de_sucesso(&feito);
    assert_eq!(valor["reconciliado_por_leitura"], true);
}

#[test]
fn nao_confirmado_sai_com_um_e_a_verificacao() {
    let erro = ErroConcluir::NaoConfirmadoNaConclusao {
        aberto_em: vec!["TESTE".into()],
        falha_do_envio: None,
        verificacao: Box::new(VerificacaoPorLeitura::SituacaoNaoLida(
            MotivoAbertoEm::HtmlDaRaizNaoPublicado,
        )),
    };
    let valor = json_de_falha(&erro);
    assert_eq!(valor["codigo"], "nao_confirmado");
    assert_eq!(valor["aberto_em"], json!(["TESTE"]));
    let nao = ErroConcluir::ConclusaoNaoAplicada {
        aberto_em: vec!["TESTE".into()],
        depois: vec!["TESTE".into()],
        falha_do_envio: None,
    };
    assert_eq!(nao.codigo(), "escrita_nao_aplicada");
}

#[test]
fn saida_tipificada_serializa_direto_sem_value_intermediario() {
    let feito = ProcessoConcluido {
        confirmado: true,
        alterado: true,
        depois: Some(vec!["TESTE_1_1".into()]),
        reconciliado_por: Some(FalhaDoEnvio::RedirecionamentoNaoReconhecido),
        ..previa()
    };
    let mut bytes = Vec::new();
    serde_json::to_writer_pretty(&mut bytes, &super::resultado_conclusao(&feito)).unwrap();
    // O objeto JSON legado ordenava as chaves: manter também os bytes pretty.
    assert_eq!(
        std::str::from_utf8(&bytes).unwrap(),
        serde_json::to_string_pretty(&json_de_sucesso(&feito)).unwrap()
    );
    let recebido: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(recebido, json_de_sucesso(&feito));
    assert_eq!(recebido["falha_do_envio"]["status_http"], 302);
    let texto = String::from_utf8(bytes).unwrap();
    assert!(!texto.contains("infra_hash") && !texto.contains("PHPSESSID"));
    assert!(!texto.contains('\u{1b}'));
}

#[test]
fn erro_tipado_da_conclusao_publica_situacao_e_resposta_sem_json() {
    let _erro = ErroConcluir::ConclusaoNaoAplicada {
        aberto_em: vec!["TESTE".into()],
        depois: vec!["OUTRA".into()],
        falha_do_envio: Some(FalhaDoEnvio::RedirecionamentoNaoReconhecido),
    };
}

#[test]
fn erro_tipado_da_conclusao_conserva_verificacao_sem_situacao_vazia() {
    let _erro = ErroConcluir::NaoConfirmadoNaConclusao {
        aberto_em: Vec::new(),
        falha_do_envio: None,
        verificacao: Box::new(VerificacaoPorLeitura::SituacaoNaoLida(
            MotivoAbertoEm::HtmlDaRaizNaoPublicado,
        )),
    };
}

#[test]
fn falha_serializavel_preserva_goldens_compactos_sem_value() {
    let casos = [
        (
            ErroConcluir::UnidadeDaSessaoNaoPublicada,
            r#"{"codigo":"fonte_invalida","erro":"a abertura não publicou a unidade da sessão; nada foi concluído","retentavel":false}"#,
        ),
        (
            ErroConcluir::SituacaoDoProcessoNaoLidaAntesDeConcluir {
                motivo: MotivoAbertoEm::HtmlDaRaizNaoPublicado,
            },
            r#"{"codigo":"fonte_invalida","erro":"a árvore não publica onde o processo está aberto, e a conclusão não poderia ser conferida; nada foi concluído","motivo":"html_da_raiz_nao_publicado","retentavel":false}"#,
        ),
        (
            ErroConcluir::ConclusaoNaoAplicada {
                aberto_em: vec![],
                depois: vec![],
                falha_do_envio: None,
            },
            r#"{"aberto_em":[],"codigo":"escrita_nao_aplicada","depois":[],"erro":"a árvore lida depois de concluir ainda lista a unidade da sessão: o processo não foi concluído","retentavel":false}"#,
        ),
        (
            ErroConcluir::NaoConfirmadoNaConclusao {
                aberto_em: vec!["TESTE".into()],
                falha_do_envio: None,
                verificacao: Box::new(VerificacaoPorLeitura::Lida(vec![])),
            },
            r#"{"aberto_em":["TESTE"],"codigo":"nao_confirmado","erro":"não se sabe se o processo foi concluído; confira com `ies processo NUP` antes de repetir","retentavel":false}"#,
        ),
        (
            ErroConcluir::NaoConfirmadoNaConclusao {
                aberto_em: vec!["TESTE".into()],
                falha_do_envio: Some(FalhaDoEnvio::RedirecionamentoNaoReconhecido),
                verificacao: Box::new(VerificacaoPorLeitura::Lida(vec![])),
            },
            r#"{"aberto_em":["TESTE"],"codigo":"nao_confirmado","erro":"não se sabe se o processo foi concluído; confira com `ies processo NUP` antes de repetir","falha_do_envio":{"status_http":302},"retentavel":false,"status_http":302}"#,
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

fn json_de_falha(erro: &ErroConcluir) -> serde_json::Value {
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
            r#"{"aberto_em":[],"codigo":"nao_confirmado","erro":"não se sabe se o processo foi concluído; confira com `ies processo NUP` antes de repetir","falha_do_envio":{"mensagem_do_servidor":"Mensagem saneada."},"mensagem_do_servidor":"Mensagem saneada.","retentavel":false}"#,
        ),
        (
            FalhaDoEnvio::Transporte(FalhaTransporte::StatusHttpDeErroDoServidor(
                StatusHttpRecusado {
                    status: 500,
                    mensagem_do_servidor: Some("Mensagem saneada.".into()),
                },
            )),
            r#"{"aberto_em":[],"codigo":"nao_confirmado","erro":"não se sabe se o processo foi concluído; confira com `ies processo NUP` antes de repetir","falha_do_envio":{"mensagem_do_servidor":"Mensagem saneada.","status_http":500},"mensagem_do_servidor":"Mensagem saneada.","retentavel":false,"status_http":500}"#,
        ),
    ];
    for (falha, esperado) in casos {
        let erro = ErroConcluir::NaoConfirmadoNaConclusao {
            aberto_em: vec![],
            falha_do_envio: Some(falha),
            verificacao: Box::new(VerificacaoPorLeitura::SituacaoNaoLida(
                MotivoAbertoEm::HtmlDaRaizNaoPublicado,
            )),
        };
        assert_eq!(
            serde_json::to_string(&super::json_de_falha(&erro)).unwrap(),
            esperado
        );
    }
}
