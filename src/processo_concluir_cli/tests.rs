//! Saída JSON e humana de `processo concluir`: campos do contrato, nada de
//! URL, e a falha com a situação e a verificação.
use super::{humano, json_de_sucesso};
use crate::processo_concluir::{
    DesfechoDaConclusao, FalhaDoEnvio, FormaDaConclusao, ProcessoConcluido,
};
use serde_json::json;

fn previa() -> ProcessoConcluido {
    ProcessoConcluido {
        nup: "00001.000001/2026-01".into(),
        id_procedimento: "10".into(),
        unidade: "TESTE".into(),
        aberto_em: vec!["TESTE".into(), "TESTE_1_1".into()],
        requisicoes: 3,
        desfecho: DesfechoDaConclusao::Previa {
            forma: FormaDaConclusao::Tela,
        },
    }
}

/// O mesmo processo depois da escrita conferida pela leitura.
fn concluido(depois: Vec<String>, reconciliado_por: Option<FalhaDoEnvio>) -> ProcessoConcluido {
    ProcessoConcluido {
        requisicoes: 6,
        desfecho: DesfechoDaConclusao::Concluido {
            forma: FormaDaConclusao::Tela,
            depois,
            reconciliado_por,
        },
        ..previa()
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
    let feito = concluido(
        vec!["TESTE_1_1".into()],
        Some(FalhaDoEnvio::RedirecionamentoNaoReconhecido),
    );
    let valor = json_de_sucesso(&feito);
    assert_eq!(valor["reconciliado_por_leitura"], true);
}

#[test]
fn saida_tipificada_serializa_direto_sem_value_intermediario() {
    let feito = concluido(
        vec!["TESTE_1_1".into()],
        Some(FalhaDoEnvio::RedirecionamentoNaoReconhecido),
    );
    let mut bytes = Vec::new();
    serde_json::to_writer_pretty(&mut bytes, &super::resultado_conclusao(&feito)).unwrap();
    // O objeto JSON legado ordenava as chaves: manter também os bytes pretty.
    assert_eq!(
        std::str::from_utf8(&bytes).unwrap(),
        serde_json::to_string_pretty(&json_de_sucesso(&feito)).unwrap()
    );
    let recebido: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(recebido, json_de_sucesso(&feito));
    assert_eq!(
        recebido["falha_do_envio"],
        format!("{:?}", feito.reconciliado_por().unwrap())
    );
    let texto = String::from_utf8(bytes).unwrap();
    assert!(!texto.contains("infra_hash") && !texto.contains("PHPSESSID"));
    assert!(!texto.contains('\u{1b}'));
}

/// Contraprova (#695): o `falha_do_envio` do sucesso é o `Debug` do erro, e o
/// texto do SEI que entra nele já passou pelo saneamento. Uma validação da
/// Infra com URL assinada, `infra_hash` e cookie não vaza nenhum deles.
#[test]
fn falha_do_envio_no_sucesso_nao_vaza_segredo_do_texto_do_sei() {
    let hash = "ab".repeat(32);
    let html = format!(
        "<html><textarea id='txaInfraValidacao'>Recusado em \
         https://sei.exemplo.gov.br/sei/controlador.php?acao=x&amp;infra_hash={hash} \
         com PHPSESSID=segredo1</textarea></html>"
    );
    let mensagem = crate::transporte::validacao_da_infra::mensagem_de_validacao_da_infra(&html)
        .expect("a validação da Infra rende texto");
    let feito = concluido(
        Vec::new(),
        Some(FalhaDoEnvio::Envio(
            crate::formulario_web::ErroSubmissaoFormulario::ValidacaoRecusadaPeloSei(mensagem),
        )),
    );
    let texto = serde_json::to_string(&super::resultado_conclusao(&feito)).unwrap();
    assert!(texto.contains("ValidacaoRecusadaPeloSei"), "{texto}");
    for segredo in [
        hash.as_str(),
        "segredo1",
        "sei.exemplo.gov.br",
        "controlador.php",
    ] {
        assert!(!texto.contains(segredo), "{segredo} vazou: {texto}");
    }
}
