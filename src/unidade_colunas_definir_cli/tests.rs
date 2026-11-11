//! RED/GREEN da saída JSON de `--definir` com resultados sintéticos.
use super::*;
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::nivel_de_detalhe::{ColunaDoDetalhe, PlanoDasColunas};
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};
use crate::visao_do_controle::{EstadoDasColunas, FalhaDoSalvar};

fn plano() -> PlanoDasColunas {
    PlanoDasColunas {
        antes: vec![ColunaDoDetalhe::Atribuicao, ColunaDoDetalhe::TipoProcesso],
        alvo: vec![ColunaDoDetalhe::Atribuicao, ColunaDoDetalhe::Anotacao],
        indisponiveis: vec![ColunaDoDetalhe::Prioridade],
    }
}

fn previa() -> ColunasDefinidas {
    ColunasDefinidas {
        unidade: "TESTE".to_owned(),
        plano: plano(),
        estado: EstadoDasColunas::Previa {
            campos_do_envio: vec![
                "sbmSalvar".to_owned(),
                "chkSinAtribuicao".to_owned(),
                "chkSinAnotacao".to_owned(),
                "hdnInfraTipoPagina".to_owned(),
            ],
        },
    }
}

fn conferidas() -> ColunasDefinidas {
    let feito = previa();
    ColunasDefinidas {
        unidade: feito.unidade,
        plano: feito.plano,
        estado: EstadoDasColunas::Conferidas {
            campos_do_envio: vec![
                "sbmSalvar".to_owned(),
                "chkSinAtribuicao".to_owned(),
                "chkSinAnotacao".to_owned(),
                "hdnInfraTipoPagina".to_owned(),
            ],
            depois: vec![ColunaDoDetalhe::Atribuicao, ColunaDoDetalhe::Anotacao],
            reconciliado_por: Some(erro_500()),
        },
    }
}

fn erro_500() -> FalhaDoSalvar {
    FalhaDoSalvar::Envio(ErroSubmissaoFormulario::Transporte(
        FalhaTransporte::Status(StatusHttpRecusado {
            status: 503,
            mensagem_do_servidor: Some("Serviço indisponível.".to_owned()),
        }),
    ))
}

#[test]
fn previa_publica_antes_alvo_diferenca_e_campos_sem_valores() {
    let valor = json_de_sucesso(&previa());
    assert_eq!(valor["unidade"], "TESTE");
    assert_eq!(valor["origem"], "web");
    assert_eq!(valor["confirmado"], false);
    assert_eq!(valor["alterado"], false);
    assert_eq!(valor["antes"], json!(["atribuicao", "tipo"]));
    assert_eq!(valor["alvo"], json!(["atribuicao", "anotacao"]));
    assert_eq!(valor["marcar"], json!(["anotacao"]));
    assert_eq!(valor["desmarcar"], json!(["tipo"]));
    assert_eq!(valor["indisponiveis"], json!(["prioridade"]));
    assert_eq!(valor["envio"]["metodo"], "POST");
    assert_eq!(valor["envio"]["acao"], "procedimento_configurar_detalhe");
    assert_eq!(valor["envio"]["campos"][0], "sbmSalvar");
    assert_eq!(valor["depois"], Value::Null);
    assert_eq!(valor["reconciliado_por_leitura"], false);
    assert!(!valor.to_string().contains("infra_hash"));
}

#[test]
fn salvo_e_reconciliado_publica_depois_e_a_falha_do_salvar() {
    let feito = conferidas();
    let valor = json_de_sucesso(&feito);
    assert_eq!(valor["alterado"], true);
    assert_eq!(valor["depois"], json!(["atribuicao", "anotacao"]));
    assert_eq!(valor["reconciliado_por_leitura"], true);
    assert_eq!(valor["falha_do_salvar"], format!("{:?}", erro_500()));
}

#[test]
fn alvo_ja_satisfeito_nao_publica_envio_nem_releitura_ficticios() {
    for confirmado in [false, true] {
        let feito = ColunasDefinidas {
            unidade: "TESTE".to_owned(),
            plano: plano(),
            estado: EstadoDasColunas::Inalteradas { confirmado },
        };
        let valor = serde_json::to_value(resultado_colunas(&feito)).unwrap();
        assert_eq!(valor["confirmado"], confirmado);
        assert_eq!(valor["alterado"], false);
        assert!(valor.get("envio").is_none());
        assert!(valor.get("depois").is_none());
        assert_eq!(valor["reconciliado_por_leitura"], false);
        assert!(valor.get("falha_do_salvar").is_none());
        assert_eq!(valor, json_de_sucesso(&feito));
    }
}

#[test]
fn saida_de_colunas_tipadas_preserva_json_pretty_em_previa_e_reconciliacao() {
    for reconciliar in [false, true] {
        let feito = if reconciliar { conferidas() } else { previa() };
        let mut bytes = Vec::new();
        serde_json::to_writer_pretty(&mut bytes, &resultado_colunas(&feito)).unwrap();
        let legado = json_de_sucesso(&feito);
        assert_eq!(
            std::str::from_utf8(&bytes).unwrap(),
            serde_json::to_string_pretty(&legado).unwrap()
        );
        let valor: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(valor["envio"]["metodo"], "POST");
        if reconciliar {
            assert_eq!(valor["falha_do_salvar"], format!("{:?}", erro_500()));
        } else {
            assert!(valor.get("falha_do_salvar").is_none());
            assert!(valor.get("depois").is_none());
        }
        assert!(!std::str::from_utf8(&bytes).unwrap().contains("infra_hash"));
    }
}
