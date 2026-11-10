//! Saída JSON e humana de `processo enviar`: campos do contrato, nada de
//! URL, e as falhas com as listas que o SEI publicou.
use super::{humano, json_de_sucesso, sigla_de_destino};
use crate::erro_publico::Rotulo as _;
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
        remover_anotacoes: false,
        anotacao_antes: None,
        anotacao_depois: None,
        metodo_remocao: None,
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
    assert!(humano(&feito).contains("Enviado. Aberto em, relido: TESTE, TESTE_1_1"));
}

#[test]
fn falhas_levam_as_listas() {
    let oferecidos = json_de_falha(&ErroEnviar::DestinoNaoOferecidoPeloSei {
        oferecidos: vec!["OUTRA - Outra".into()],
    });
    assert_eq!(oferecidos["oferecidos"], json!(["OUTRA - Outra"]));
    let incerto = ErroEnviar::EfeitoIncertoNoEnvio {
        aberto_em: vec!["TESTE".into()],
        anotacao_depois: None,
        falha_do_envio: Some(FalhaDoEnvio::RedirecionamentoNaoReconhecidoAposEnviar),
        verificacao: Box::new(VerificacaoPorLeitura::LeituraFalhou(
            crate::processo_obter::ErroProcessoObter::Transporte(FalhaTransporte::Rede),
        )),
    };
    let valor = json_de_falha(&incerto);
    assert_eq!(valor["codigo"], "efeito_incerto");
    assert_eq!(valor["status_http"], 302);
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

#[test]
fn erro_de_envio_exibe_sugestoes_guardadas_no_modo_humano() {
    // A apresentação humana recebe strings tipadas, não serde_json::Value.
    let sugeridas = vec![
        "ABC/GECONT - Contabilidade".to_owned(),
        "ABC/GECONT-18141 - Contabilidade regional".to_owned(),
    ];
    assert_eq!(
        super::texto_das_sugestoes(&sugeridas).as_deref(),
        Some(concat!(
            "IES: sugestões guardadas (não confirmadas pelo SEI): ",
            "ABC/GECONT - Contabilidade, ",
            "ABC/GECONT-18141 - Contabilidade regional"
        ))
    );
    assert_eq!(super::texto_das_sugestoes(&[]), None);
}

#[test]
fn sugestoes_guardadas_compoem_erro_web_sem_consulta_extra() {
    // Caso de Todos tests/test_destinos_indice_010.py @323eb01:
    // apenas o autocomplete do SEI escolhe o destino. A lista SQLite
    // pode sugerir, mas não acrescenta HTTP, nem autoriza escrita.
    use crate::indice_local::{UnidadeGuardada, unidade_dos_links};
    use crate::processo_concluir::tests_apoio::{
        ID, ORIGEM, UNIDADE, abertura, ambiente, arvore, ok, rodar_com, url_controle,
    };
    use crate::processo_enviar::tests_apoio::{
        DESTINO, Versao, enviar, resposta_xml, tela_de_envio,
    };
    use crate::processo_enviar::{PedidoDoEnvio, enviar_processo};

    let instalacao =
        crate::instalacao::Instalacao::interpretar(ORIGEM).expect("origem sintética HTTPS");
    let chave = instalacao.url_base();
    let ativa = unidade_dos_links(&url_controle()).expect("unidade da sessão sintética");
    let guardada = UnidadeGuardada {
        id: "900".into(),
        sigla: DESTINO.into(),
        descricao: "Unidade guardada".into(),
        orgao: "Sintético".into(),
    };

    for (unidade_da_lista, confirmar) in [
        (Some(ativa.as_str()), false),
        (Some("999"), true),
        (None, false),
    ] {
        let ambiente = ambiente();
        if let Some(unidade) = unidade_da_lista {
            assert!(ambiente.indice.guardar_lista_de_unidades(
                &chave,
                unidade,
                std::slice::from_ref(&guardada),
                1,
            ));
        }
        let mut respostas = abertura(arvore(&[UNIDADE], &enviar()));
        respostas.extend([
            ok(tela_de_envio(Versao::V41, ID)),
            ok(resposta_xml(&[(
                "901",
                "OUTRA - Destino publicado pelo SEI",
            )])),
        ]);
        let ((valor, mensagem, _subcodigo, saida), pedidos) =
            rodar_com(&ambiente, respostas, |sessao, nup, indice, roteiro| {
                let pedido = PedidoDoEnvio {
                    nup,
                    unidade: None,
                    para: DESTINO,
                    manter_aberto: false,
                    remover_anotacoes: false,
                    confirmar,
                };
                let erro = enviar_processo(sessao, &pedido, indice, roteiro).unwrap_err();
                let projecao = super::json_de_falha_com_sugestoes(&erro, sessao, indice, DESTINO);
                assert_ne!(
                    std::any::type_name_of_val(&projecao),
                    std::any::type_name::<serde_json::Value>()
                );
                let valor = serde_json::to_value(projecao).unwrap();
                (
                    valor.clone(),
                    super::texto_das_sugestoes(&super::sugestoes_guardadas(
                        &erro, sessao, indice, DESTINO,
                    )),
                    erro.rotulo().to_owned(),
                    erro.saida(),
                )
            });
        // Quatro GET de navegação e um POST apenas de consulta.
        // Mesmo confirmar=true não faz POST de escrita sem destino do SEI.
        assert_eq!(
            pedidos.iter().map(|p| p.metodo).collect::<Vec<_>>(),
            vec!["GET", "GET", "GET", "GET", "POST"],
        );
        assert_eq!(pedidos.len(), 5);
        assert_eq!(saida, 1);
        let esperado = if unidade_da_lista == Some(ativa.as_str()) {
            r#"{"codigo":"fonte_invalida","erro":"o SEI não oferece unidade de destino com essa sigla; nada foi enviado","oferecidos":["OUTRA - Destino publicado pelo SEI"],"retentavel":false,"sugestoes_guardadas":["TESTE_1_1 - Unidade guardada"]}"#
        } else {
            r#"{"codigo":"fonte_invalida","erro":"o SEI não oferece unidade de destino com essa sigla; nada foi enviado","oferecidos":["OUTRA - Destino publicado pelo SEI"],"retentavel":false}"#
        };
        assert_eq!(valor.to_string(), esperado);
        assert_eq!(
            valor["oferecidos"],
            json!(["OUTRA - Destino publicado pelo SEI"]),
        );
        if unidade_da_lista == Some(ativa.as_str()) {
            assert_eq!(
                valor["sugestoes_guardadas"],
                json!(["TESTE_1_1 - Unidade guardada"]),
            );
            assert_eq!(
                mensagem.as_deref(),
                Some(
                    "IES: sugestões guardadas (não confirmadas pelo SEI): TESTE_1_1 - Unidade guardada"
                ),
            );
        } else {
            assert!(valor.get("sugestoes_guardadas").is_none());
            assert_eq!(mensagem, None);
        }
    }
}

#[test]
fn saida_tipificada_serializa_destino_sem_value_intermediario() {
    let feito = ProcessoEnviado {
        confirmado: true,
        alterado: true,
        depois: Some(vec!["TESTE_1_1".into()]),
        reconciliado_por: Some(FalhaDoEnvio::RedirecionamentoNaoReconhecidoAposEnviar),
        ..previa()
    };
    let mut bytes = Vec::new();
    serde_json::to_writer_pretty(&mut bytes, &super::resultado_envio(&feito)).unwrap();
    // O objeto JSON legado ordenava as chaves: manter também os bytes pretty.
    assert_eq!(
        std::str::from_utf8(&bytes).unwrap(),
        serde_json::to_string_pretty(&json_de_sucesso(&feito)).unwrap()
    );
    let recebido: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(recebido, json_de_sucesso(&feito));
    assert_eq!(recebido["destino"]["id_unidade"], "110000002");
    assert_eq!(recebido["falha_do_envio"]["status_http"], 302);
    assert!(!String::from_utf8(bytes).unwrap().contains("infra_hash"));
}

#[test]
fn falha_de_envio_humana_usa_causas_tipadas_sem_json() {
    let nao_aplicado = ErroEnviar::EnvioNaoAplicado {
        aberto_em: vec!["TESTE".into()],
        depois: vec!["OUTRA".into()],
        falha_do_envio: Some(FalhaDoEnvio::RedirecionamentoNaoReconhecidoAposEnviar),
    };
    let linhas = super::linhas_da_falha(&nao_aplicado);
    assert_eq!(linhas[1], "IES: aberto_em: TESTE");
    assert_eq!(linhas[2], "IES: depois: OUTRA");
    let incerto = ErroEnviar::EfeitoIncertoNoEnvio {
        aberto_em: vec![],
        anotacao_depois: None,
        falha_do_envio: None,
        verificacao: Box::new(VerificacaoPorLeitura::SituacaoNaoLida(
            crate::arvore_processo::MotivoAbertoEm::HtmlDaRaizNaoPublicado,
        )),
    };
    let linhas = super::linhas_da_falha(&incerto);
    assert_eq!(linhas[1], "IES: aberto_em: (nenhuma unidade)");
    assert_eq!(linhas[2], "IES: verificação: html_da_raiz_nao_publicado");
    assert_eq!(incerto.saida(), 3);
}

#[test]
fn falhas_do_envio_serializam_direto_com_goldens_fixos() {
    let casos = [
        (
            ErroEnviar::DestinoNaoOferecidoPeloSei { oferecidos: vec![] },
            r#"{"codigo":"fonte_invalida","erro":"o SEI não oferece unidade de destino com essa sigla; nada foi enviado","oferecidos":[],"retentavel":false}"#,
        ),
        (
            ErroEnviar::VariosDestinosComASigla {
                candidatos: vec!["TESTE - A".into(), "TESTE - B".into()],
            },
            r#"{"candidatos":["TESTE - A","TESTE - B"],"codigo":"fonte_invalida","erro":"o SEI oferece mais de uma unidade com essa sigla; nada foi enviado","retentavel":false}"#,
        ),
        (
            ErroEnviar::EnvioNaoAplicado {
                aberto_em: vec![],
                depois: vec![],
                falha_do_envio: None,
            },
            r#"{"aberto_em":[],"codigo":"escrita_nao_aplicada","depois":[],"erro":"a árvore lida depois de enviar não lista o destino: o processo não foi enviado","retentavel":false}"#,
        ),
        (
            ErroEnviar::EfeitoIncertoNoEnvio {
                aberto_em: vec![],
                anotacao_depois: None,
                falha_do_envio: Some(FalhaDoEnvio::RedirecionamentoNaoReconhecidoAposEnviar),
                verificacao: Box::new(VerificacaoPorLeitura::Lida(vec![])),
            },
            r#"{"aberto_em":[],"codigo":"efeito_incerto","erro":"não se sabe se o processo foi enviado; confira com `ies processo NUP` antes de repetir","falha_do_envio":{"status_http":302},"retentavel":false,"status_http":302,"verificacao":null}"#,
        ),
    ];
    for (erro, esperado) in casos {
        let projecao = super::json_de_falha(&erro);
        assert_eq!(serde_json::to_string(&projecao).unwrap(), esperado);
        assert_ne!(
            std::any::type_name_of_val(&projecao),
            std::any::type_name::<serde_json::Value>()
        );
    }
}

fn json_de_falha(erro: &ErroEnviar) -> serde_json::Value {
    serde_json::to_value(super::json_de_falha(erro)).unwrap()
}

#[test]
fn bdd_previa_publica_opcao_somente_se_solicitada() {
    let mut feito = previa();
    assert!(json_de_sucesso(&feito).get("remover_anotacoes").is_none());
    feito.remover_anotacoes = true;
    assert_eq!(json_de_sucesso(&feito)["remover_anotacoes"], true);
    assert!(humano(&feito).contains("Remover anotações no envio: sim"));
}
