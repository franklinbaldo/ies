//! Saída JSON e humana de `processo reabrir`: campos do contrato, nada de
//! URL, e a falha com a situação e a verificação.
use super::{humano, json_de_sucesso};
use crate::arvore_processo::{FormaDaReabertura, MotivoAbertoEm};
use crate::processo_concluir::{FalhaDoEnvio, VerificacaoPorLeitura};
use crate::processo_reabrir::{DesfechoDaReabertura, ErroReabrir, ProcessoReaberto};
use serde_json::json;

fn previa() -> ProcessoReaberto {
    ProcessoReaberto {
        nup: "00001.000001/2026-01".into(),
        id_procedimento: "10".into(),
        unidade: "TESTE".into(),
        aberto_em: vec!["TESTE_1_1".into()],
        requisicoes: 3,
        desfecho: DesfechoDaReabertura::Previa,
    }
}

#[test]
fn previa_publica_o_contrato_sem_url() {
    assert_eq!(
        json_de_sucesso(&previa()),
        json!({"nup": "00001.000001/2026-01", "id_procedimento": "10", "unidade": "TESTE",
               "origem": "web", "confirmado": false, "alterado": false,
               "reabertura_oferecida": true, "aberto_em": ["TESTE_1_1"],
               "reconciliado_por_leitura": false, "requisicoes": 3})
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
        aberto_em: vec!["TESTE".into()],
        desfecho: DesfechoDaReabertura::JaAbertoNaUnidade { confirmado: false },
        ..previa()
    };
    assert!(humano(&ja).contains("Nada a fazer"));
    assert_eq!(json_de_sucesso(&ja)["reabertura_oferecida"], false);
    let feito = ProcessoReaberto {
        requisicoes: 6,
        desfecho: DesfechoDaReabertura::Reaberto {
            forma: FormaDaReabertura::Variavel,
            depois: vec!["TESTE".into(), "TESTE_1_1".into()],
            reconciliado_por: Some(FalhaDoEnvio::RedirecionamentoNaReabertura),
        },
        ..previa()
    };
    let texto = humano(&feito);
    assert!(
        texto.contains("Reaberto. Aberto em, relido: TESTE, TESTE_1_1"),
        "{texto}"
    );
    let valor = json_de_sucesso(&feito);
    assert_eq!(valor["forma"], "variavel");
    assert_eq!(valor["reconciliado_por_leitura"], true);
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
    assert!(recebido.get("forma").is_none());
    assert!(!String::from_utf8(bytes).unwrap().contains("infra_hash"));
}

#[test]
fn erro_tipado_da_reabertura_preserva_situacao_resposta_e_verificacao() {
    let _erro = ErroReabrir::NaoConfirmadoNaReabertura {
        aberto_em: vec!["TESTE_1_1".into()],
        falha_do_envio: Some(FalhaDoEnvio::RedirecionamentoNaReabertura),
        verificacao: Box::new(VerificacaoPorLeitura::SituacaoNaoLida(
            MotivoAbertoEm::HtmlDaRaizNaoPublicado,
        )),
    };
}

#[test]
fn erro_tipado_da_reabertura_preserva_depois_sem_json() {
    let _erro = ErroReabrir::ReaberturaNaoAplicada {
        aberto_em: vec![],
        depois: vec!["OUTRA".into()],
        falha_do_envio: None,
    };
}
