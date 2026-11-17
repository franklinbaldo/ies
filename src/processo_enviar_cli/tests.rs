//! Saída JSON e humana de `processo enviar`: campos do contrato, nada de
//! URL, e as falhas com as listas que o SEI publicou.
use super::{humano, json_de_sucesso, sigla_de_destino};
use crate::processo_concluir::FalhaDoEnvio;
use crate::processo_enviar::{
    DesfechoDoEnvio, Destino, MetodoRemocao, ProcessoEnviado, RemocaoDaAnotacao,
};
use serde_json::json;

fn previa() -> ProcessoEnviado {
    ProcessoEnviado {
        nup: "00001.000001/2026-01".into(),
        id_procedimento: "10".into(),
        unidade: "TESTE".into(),
        manter_aberto: false,
        remocao: None,
        anotacao_depois: None,
        destino: Destino {
            sigla: "TESTE_1_1".into(),
            descricao: "TESTE_1_1 - Unidade 1.1".into(),
            id_unidade: "110000002".into(),
        },
        aberto_em: vec!["TESTE".into()],
        desfecho: DesfechoDoEnvio::Previa,
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
               "aberto_em": ["TESTE"], "depois_esperado": ["TESTE_1_1"],
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
        manter_aberto: true,
        desfecho: DesfechoDoEnvio::Enviado {
            depois: vec!["TESTE".into(), "TESTE_1_1".into()],
            reconciliado_por: Some(FalhaDoEnvio::RespostaDoEnvioSemRedirecionamento),
        },
        requisicoes: 7,
        ..previa()
    };
    let valor = json_de_sucesso(&feito);
    assert_eq!(valor["depois_esperado"], json!(["TESTE", "TESTE_1_1"]));
    assert_eq!(valor["reconciliado_por_leitura"], true);
    assert!(humano(&feito).contains("Enviado. Aberto em, relido: TESTE, TESTE_1_1"));
}

#[test]
fn sigla_de_destino_e_conferida_antes_de_tudo() {
    assert_eq!(sigla_de_destino(" TESTE ").unwrap(), "TESTE");
    assert_eq!(
        sigla_de_destino("ADCENTRAL/GR/ARQUIVO /PROCE").unwrap(),
        "ADCENTRAL/GR/ARQUIVO /PROCE"
    );
    for ruim in ["", "   ", "A\tB", &"X".repeat(101)] {
        assert!(sigla_de_destino(ruim).is_err(), "{ruim:?}");
    }
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
        let ((depurado, sugestoes), pedidos) =
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
                let sugestoes = super::sugestoes_guardadas(&erro, sessao, indice, DESTINO);
                (format!("{erro:?}"), sugestoes)
            });
        // Quatro GET de navegação e um POST apenas de consulta.
        // Mesmo confirmar=true não faz POST de escrita sem destino do SEI.
        assert_eq!(
            pedidos.iter().map(|p| p.metodo).collect::<Vec<_>>(),
            vec!["GET", "GET", "GET", "GET", "POST"],
        );
        assert_eq!(pedidos.len(), 5);
        assert!(
            depurado.starts_with("DestinoNaoOferecidoPeloSei")
                && depurado.contains("OUTRA - Destino publicado pelo SEI"),
            "{depurado}"
        );
        let esperadas: &[&str] = if unidade_da_lista == Some(ativa.as_str()) {
            &["TESTE_1_1 - Unidade guardada"]
        } else {
            &[]
        };
        assert_eq!(sugestoes, esperadas);
    }
}

#[test]
fn saida_tipificada_serializa_destino_sem_value_intermediario() {
    let feito = ProcessoEnviado {
        desfecho: DesfechoDoEnvio::Enviado {
            depois: vec!["TESTE_1_1".into()],
            reconciliado_por: Some(FalhaDoEnvio::RedirecionamentoNaoReconhecidoAposEnviar),
        },
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
    assert_eq!(
        recebido["falha_do_envio"],
        format!("{:?}", feito.reconciliado_por().unwrap())
    );
    assert!(!String::from_utf8(bytes).unwrap().contains("infra_hash"));
}

#[test]
fn bdd_previa_publica_opcao_somente_se_solicitada() {
    let mut feito = previa();
    assert!(json_de_sucesso(&feito).get("remover_anotacoes").is_none());
    feito.remocao = Some(RemocaoDaAnotacao {
        metodo: MetodoRemocao::CheckboxNoEnvio,
        antes: false,
    });
    assert_eq!(json_de_sucesso(&feito)["remover_anotacoes"], true);
    assert!(humano(&feito).contains("Remover anotações no envio: sim"));
}
