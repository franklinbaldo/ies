//! Saída JSON e humana de `processo enviar`: campos do contrato, nada de
//! URL, e as falhas com as listas que o SEI publicou.
use super::{humano, json_de_falha, json_de_sucesso, sigla_de_destino};
use crate::processo_concluir::{FalhaDoEnvio, VerificacaoPorLeitura};
use crate::processo_enviar::{Destino, ErroEnviar, ProcessoEnviado};
use crate::transporte::FalhaTransporte;
use serde_json::json;

fn previa() -> ProcessoEnviado {
    ProcessoEnviado {
        nup: "00001.000001/2026-01".into(),
        id_procedimento: "10".into(),
        unidade: "TESTE".into(),
        confirmado: false,
        alterado: false,
        manter_aberto: false,
        destino: Destino {
            sigla: "TESTE_1_1".into(),
            descricao: "TESTE_1_1 - Unidade 1.1".into(),
            id_unidade: "110000002".into(),
        },
        aberto_em: vec!["TESTE".into()],
        depois: None,
        reconciliado_por: None,
        requisicoes: 5,
    }
}

#[test]
fn previa_publica_o_contrato_sem_url() {
    assert_eq!(
        json_de_sucesso(&previa()),
        json!({"nup": "00001.000001/2026-01", "id_procedimento": "10", "unidade": "TESTE",
               "origem": "web", "confirmado": false, "alterado": false, "manter_aberto": false,
               "destino": {"sigla": "TESTE_1_1", "descricao": "TESTE_1_1 - Unidade 1.1",
                           "id_unidade": "110000002"},
               "aberto_em": ["TESTE"], "depois_esperado": ["TESTE_1_1"], "depois": null,
               "reconciliado_por_leitura": false, "requisicoes": 5})
    );
    let texto = humano(&previa());
    assert!(
        texto.contains("Destino: TESTE_1_1 - Unidade 1.1"),
        "{texto}"
    );
    assert!(
        texto.contains("Para enviar, repita com --confirmar"),
        "{texto}"
    );
    assert!(texto.ends_with("Requisições: 5"), "{texto}");
}

#[test]
fn enviado_e_reconciliado() {
    let feito = ProcessoEnviado {
        confirmado: true,
        alterado: true,
        manter_aberto: true,
        depois: Some(vec!["TESTE".into(), "TESTE_1_1".into()]),
        reconciliado_por: Some(FalhaDoEnvio::RespostaDoEnvioSemRedirecionamento),
        requisicoes: 7,
        ..previa()
    };
    let valor = json_de_sucesso(&feito);
    assert_eq!(valor["depois_esperado"], json!(["TESTE", "TESTE_1_1"]));
    assert_eq!(valor["reconciliado_por_leitura"], true);
    assert_eq!(
        valor["falha_do_envio"]["subcodigo"],
        "resposta_do_envio_sem_redirecionamento"
    );
    assert!(humano(&feito).contains("Enviado. Aberto em, relido: TESTE, TESTE_1_1"));
}

#[test]
fn falhas_levam_as_listas() {
    let oferecidos = json_de_falha(&ErroEnviar::DestinoNaoOferecidoPeloSei {
        oferecidos: vec!["OUTRA - Outra".into()],
    });
    assert_eq!(oferecidos["subcodigo"], "destino_nao_oferecido_pelo_sei");
    assert_eq!(oferecidos["oferecidos"], json!(["OUTRA - Outra"]));
    let incerto = ErroEnviar::EfeitoIncertoNoEnvio {
        aberto_em: vec!["TESTE".into()],
        falha_do_envio: Some(FalhaDoEnvio::RedirecionamentoNaoReconhecidoAposEnviar),
        verificacao: Box::new(VerificacaoPorLeitura::LeituraFalhou(
            crate::processo_obter::ErroProcessoObter::Transporte(FalhaTransporte::Rede),
        )),
    };
    let valor = json_de_falha(&incerto);
    assert_eq!(valor["codigo"], "efeito_incerto");
    assert_eq!(valor["status_http"], 302);
    assert_eq!(
        valor["falha_do_envio"]["subcodigo"],
        "redirecionamento_nao_reconhecido_apos_enviar"
    );
    assert!(valor["verificacao"].is_string());
    assert_eq!(incerto.saida(), 3);
}

#[test]
fn sigla_de_destino_e_conferida_antes_de_tudo() {
    let um = |s: &str| vec![s.to_owned()];
    assert_eq!(sigla_de_destino(&um(" TESTE ")), Ok("TESTE"));
    assert_eq!(
        sigla_de_destino(&um("ADCENTRAL/GR/ARQUIVO /PROCE")),
        Ok("ADCENTRAL/GR/ARQUIVO /PROCE")
    );
    for ruim in ["", "  ", "A\nB", &"X".repeat(101)] {
        assert_eq!(
            sigla_de_destino(&um(ruim)),
            Err("sigla_de_destino_invalida"),
            "{ruim:?}"
        );
    }
    assert_eq!(
        sigla_de_destino(&["A".into(), "B".into()]),
        Err("mais_de_um_destino_nao_suportado")
    );
}
