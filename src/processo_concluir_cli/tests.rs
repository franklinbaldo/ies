//! Saída JSON e humana de `processo concluir`: campos do contrato, nada de
//! URL, e a falha com a situação e a verificação.
use super::{humano, json_de_sucesso};
use crate::processo_concluir::{FalhaDoEnvio, FormaDaConclusao, ProcessoConcluido};
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
