//! RED/GREEN da saída JSON de `--definir` com resultados sintéticos.
use super::*;
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::nivel_de_detalhe::{ColunaDoDetalhe as C, PlanoDasColunas};
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};
use crate::visao_do_controle::{FalhaDoSalvar, VerificacaoDasColunas};

fn plano() -> PlanoDasColunas {
    PlanoDasColunas {
        antes: vec![C::Atribuicao, C::TipoProcesso],
        alvo: vec![C::Atribuicao, C::Anotacao],
        indisponiveis: vec![C::Prioridade],
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
            depois: vec![C::Atribuicao, C::Anotacao],
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
    assert_eq!(valor["falha_do_salvar"]["status_http"], 503);
    assert_eq!(
        valor["falha_do_salvar"]["mensagem_do_servidor"],
        "Serviço indisponível."
    );
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
        assert_eq!(valor["envio"], Value::Null);
        assert_eq!(valor["depois"], Value::Null);
        assert_eq!(valor["reconciliado_por_leitura"], false);
        assert!(valor.get("falha_do_salvar").is_none());
        assert_eq!(valor, json_de_sucesso(&feito));
    }
}

#[test]
fn nao_confirmado_publica_codigo_causa_e_verificacao() {
    let erro = ErroDefinirColunas::NaoConfirmadoNaConfiguracaoDeColunas {
        plano: Box::new(plano()),
        falha_do_salvar: Some(erro_500()),
        verificacao: Box::new(VerificacaoDasColunas::LeituraFalhou(
            ErroVisaoDoControle::Transporte(FalhaTransporte::Rede),
        )),
    };
    let valor = json_de_falha(&erro);
    assert_eq!(valor["codigo"], "nao_confirmado");
    assert_eq!(valor["retentavel"], false);
    assert_eq!(valor["antes"], json!(["atribuicao", "tipo"]));
    assert_eq!(valor["alvo"], json!(["atribuicao", "anotacao"]));
    assert_eq!(valor["status_http"], 503);
    assert_eq!(valor["mensagem_do_servidor"], "Serviço indisponível.");
    assert_eq!(valor["observadas"], Value::Null);
}

#[test]
fn coluna_indisponivel_e_leitura_sao_fonte_invalida() {
    let erro = ErroDefinirColunas::ColunaIndisponivelNestaInstalacao {
        coluna: C::Prioridade,
    };
    let valor = json_de_falha(&erro);
    assert_eq!(valor["codigo"], "fonte_invalida");
    assert_eq!(valor["coluna"], "prioridade");
}

#[test]
fn falha_do_salvar_sem_sinal_de_gravacao_sai_so_com_o_subcodigo() {
    let erro = ErroDefinirColunas::NaoConfirmadoNaConfiguracaoDeColunas {
        plano: Box::new(plano()),
        falha_do_salvar: Some(FalhaDoSalvar::NivelDevolvidoSemSinalDeGravacao),
        verificacao: Box::new(VerificacaoDasColunas::LeituraFalhou(
            ErroVisaoDoControle::Transporte(FalhaTransporte::Rede),
        )),
    };
    let valor = json_de_falha(&erro);
    assert_eq!(valor.get("status_http"), None);
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
            assert_eq!(valor["falha_do_salvar"]["status_http"], 503);
        } else {
            assert!(valor.get("falha_do_salvar").is_none());
            assert!(valor["depois"].is_null());
        }
        assert!(!std::str::from_utf8(&bytes).unwrap().contains("infra_hash"));
    }
}

#[test]
fn falhas_de_colunas_sao_apresentadas_sem_interpretar_json() {
    let antes = plano();
    let erro = ErroDefinirColunas::NaoConfirmadoNaConfiguracaoDeColunas {
        plano: Box::new(antes.clone()),
        falha_do_salvar: Some(erro_500()),
        verificacao: Box::new(VerificacaoDasColunas::Marcadas(vec![C::Atribuicao])),
    };
    let linhas = linhas_da_falha(&erro);
    assert_eq!(linhas[1], "IES: antes: atribuicao, tipo");
    assert_eq!(linhas[2], "IES: alvo: atribuicao, anotacao");
    assert_eq!(linhas[3], "IES: observadas: atribuicao");

    let diferente = ErroDefinirColunas::MarcasDiferentesDoAlvoAposSalvar {
        plano: Box::new(antes),
        observadas: vec![],
    };
    let linhas = linhas_da_falha(&diferente);
    assert_eq!(linhas[3], "IES: observadas: (nenhuma)");
    assert_eq!(linhas.len(), 4);

    let so_leitura = ErroDefinirColunas::ColunaIndisponivelNestaInstalacao {
        coluna: C::Prioridade,
    };
    assert_eq!(linhas_da_falha(&so_leitura).len(), 1);
}
