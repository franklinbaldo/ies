//! RED/GREEN da conferência da tela "Enviar Processo", da consulta que o
//! script declara e dos pares do POST, nas duas famílias observadas.
use super::erro::ErroTelaDeEnvio as E;
use super::tela::{
    ENVIAR_PROCESSO, TelaDeEnvio, campos_da_consulta, campos_do_envio, conferir_tela_de_envio,
    redirecionamento_esperado,
};
use super::tests_apoio::*;
use crate::arvore_processo::tests_concluir::absoluta;
use crate::arvore_processo::tests_tela::instalacao;
use crate::erro_html::ErroHtml;
use crate::formulario_web::{CodificacaoDaTela, TelaServida, derivar_submissao};
use crate::processo_concluir::tests_apoio::{ID, NUP};

fn conferida(versao: Versao) -> TelaDeEnvio {
    conferir_tela_de_envio(&tela_de_envio(versao, ID), ID).unwrap()
}

#[test]
fn tela_das_duas_familias_e_conferida() {
    let v40 = conferida(Versao::V40);
    assert_eq!(v40.orgao, "");
    assert_eq!(v40.textos, ["txtPrazo", "txtDias"]);
    assert_eq!(v40.autocompletar, autocompletar());
    assert!(!v40.unidade_atual);
    let v41 = conferida(Versao::V41);
    assert_eq!(
        v41.textos,
        [
            "txtPrazoRetornoProgramado",
            "txtDiasRetornoProgramado",
            "txtPrazoReaberturaProgramada",
            "txtDiasReaberturaProgramada"
        ]
    );
    assert!(v41.unidade_atual);
}

#[test]
fn consulta_leva_o_corpo_do_prepararexecucao() {
    let v40 = conferida(Versao::V40);
    assert_eq!(
        campos_da_consulta(&v40, DESTINO),
        [("palavras_pesquisa", DESTINO), ("id_orgao", "")]
    );
    let v41 = conferida(Versao::V41);
    assert_eq!(
        campos_da_consulta(&v41, DESTINO),
        [
            ("palavras_pesquisa", DESTINO),
            ("id_orgao", ""),
            ("unidade_atual", "0")
        ]
    );
}

#[test]
fn post_derivado_pelo_primitivo_na_ordem_do_documento() {
    for versao in [Versao::V40, Versao::V41] {
        for manter in [false, true] {
            let html = tela_de_envio(versao, ID);
            let tela = conferir_tela_de_envio(&html, ID).unwrap();
            let campos = campos_do_envio(&tela, &destino(), manter, false);
            let pares: Vec<(&str, &str)> = campos.iter().map(|(n, v)| (*n, v.as_str())).collect();
            let url = url_da_tela();
            let servida = TelaServida {
                url: &url,
                html: &html,
                codificacao: CodificacaoDaTela::Iso88591,
            };
            let s =
                derivar_submissao(&ENVIAR_PROCESSO, &instalacao(), &servida, "", &pares).unwrap();
            assert_eq!(
                String::from_utf8(s.corpo().to_vec()).unwrap(),
                corpo_do_envio(versao, manter)
            );
        }
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Uma tabela de casos.
fn recusas_da_tela_antes_de_qualquer_post() {
    let base = tela_de_envio(Versao::V41, ID);
    let trocar = |de: &str, para: &str| {
        assert!(base.contains(de), "{de}");
        base.replacen(de, para, 1)
    };
    let casos = [
        (
            trocar("frmAtividadeListar", "frmOutro"),
            E::Html(ErroHtml::ausente("formulario_de_envio_ausente")),
        ),
        (
            trocar(
                "name=\"hdnIdProtocolos\" value=\"10\"",
                "name=\"hdnIdProtocolos\" value=\"11\"",
            ),
            E::TelaDeEnvioDeOutroProcesso,
        ),
        (
            trocar(
                "<input type=\"hidden\" id=\"hdnUnidades\"",
                "<input type=\"text\" name=\"txtNovo\" value=\"\" /><input type=\"hidden\" id=\"hdnUnidades\"",
            ),
            E::Html(ErroHtml::valor("campo_nao_previsto_na_tela_de_envio")),
        ),
        (
            trocar(
                "name=\"chkSinManterAberto\" class",
                "name=\"chkSinManterAberto\" checked=\"checked\" class",
            ),
            E::TelaDeEnvioComOpcoesPreenchidas,
        ),
        (
            trocar(
                "name=\"chkSinEnviarEmailNotificacao\" class",
                "name=\"chkSinEnviarEmailNotificacao\" checked class",
            ),
            E::TelaDeEnvioComOpcoesPreenchidas,
        ),
        (
            trocar(
                "id=\"optDiasRetornoProgramado\" value=\"2\"",
                "id=\"optDiasRetornoProgramado\" value=\"2\" checked=\"checked\"",
            ),
            E::TelaDeEnvioComOpcoesPreenchidas,
        ),
        (
            trocar(
                "name=\"txtDiasRetornoProgramado\" class=\"infraText\" value=\"\"",
                "name=\"txtDiasRetornoProgramado\" class=\"infraText\" value=\"5\"",
            ),
            E::TelaDeEnvioComOpcoesPreenchidas,
        ),
        (
            trocar(
                "name=\"hdnUnidades\" value=\"\"",
                "name=\"hdnUnidades\" value=\"1±X\"",
            ),
            E::TelaDeEnvioComOpcoesPreenchidas,
        ),
        (
            trocar(
                "multiple=\"multiple\" class=\"infraSelect\" tabindex=\"504\">\n",
                "multiple=\"multiple\" class=\"infraSelect\" tabindex=\"504\">\n<option value=\"9\">X - Y</option>\n",
            ),
            E::TelaDeEnvioComOpcoesPreenchidas,
        ),
        (
            trocar(
                &format!("<option value=\"{ID}\">"),
                &format!("<option value=\"{ID}\" selected=\"selected\">"),
            ),
            E::TelaDeEnvioComOpcoesPreenchidas,
        ),
        (
            trocar(
                "<option value=\"\" selected=\"selected\">Todos</option>",
                "<option value=\"\">Todos</option>",
            ),
            E::OrgaoDaTelaDeEnvioNaoSelecionado,
        ),
        (
            trocar(
                "objAutoCompletarUnidade = new infraAjaxAutoCompletar('hdnIdUnidade','txtUnidade','",
                "objAutoCompletarUnidade = new infraAjaxAutoCompletar('hdnIdOutro','txtUnidade','",
            ),
            E::AutocompletarDeUnidadeNaoDeclarado,
        ),
        (
            trocar("+'&unidade_atual=0';", "+'&unidade_atual=1';"),
            E::Html(ErroHtml::valor(
                "consulta_do_autocompletar_fora_da_forma_observada",
            )),
        ),
    ];
    for (html, erro) in casos {
        assert_eq!(conferir_tela_de_envio(&html, ID), Err(erro), "{erro:?}");
    }
    // Duas declarações do autocomplete: nenhuma é escolhida.
    let linha = format!(
        "objAutoCompletarUnidade = new infraAjaxAutoCompletar('hdnIdUnidade','txtUnidade','{}');",
        autocompletar()
    );
    let repetida = base.replacen(&linha, &format!("{linha}\n{linha}"), 1);
    assert_eq!(
        conferir_tela_de_envio(&repetida, ID),
        Err(E::AutocompletarDeUnidadeNaoDeclarado)
    );
    let _ = NUP;
}

#[test]
fn redirecionamento_do_envio_observado() {
    let post = absoluta(&action_de_enviar());
    let esperado = |l: Option<&str>| redirecionamento_esperado(l, &post, ID);
    let volta = volta_a_arvore_do_envio();
    assert!(esperado(Some(&volta)));
    // `http://` no host e na pasta do POST: o mesmo controlador (#766).
    assert!(esperado(Some(
        &absoluta(&volta).replacen("https://", "http://", 1)
    )));
    assert!(esperado(Some(&absoluta(&volta))));
    for outro in [
        volta.replace("acao_origem=procedimento_enviar", "acao_origem=x"),
        volta.replace("id_procedimento=10", "id_procedimento=11"),
        volta.replace("acao=arvore_visualizar", "acao=procedimento_enviar"),
        format!("https://outro/{volta}"),
        absoluta(&volta).replacen(
            "https://sei.exemplo.gov.br",
            "http://outro.exemplo.gov.br",
            1,
        ),
    ] {
        assert!(!esperado(Some(&outro)), "{outro}");
    }
    assert!(!esperado(None));
}

#[test]
fn bdd_checkbox_de_remocao_e_oferta_observada_da_tela() {
    // Dado um controle habilitado nas famílias 4.0 e 4.1/5.0.
    for versao in [Versao::V40, Versao::V41] {
        let html = tela_de_envio(versao, ID);
        let tela = conferir_tela_de_envio(&html, ID).unwrap();
        assert!(tela.remover_anotacoes_oferecida);
        // A ausência de name significa que não há controle enviável.
        let sem_opcao = html.replace(r#"name="chkSinRemoverAnotacoes""#, r#"name="""#);
        assert!(
            !conferir_tela_de_envio(&sem_opcao, ID)
                .unwrap()
                .remover_anotacoes_oferecida
        );
    }
}

#[test]
fn bdd_remover_anotacoes_deriva_checkbox_do_formulario_servido() {
    // Dado o checkbox observado nas versões 4.0 e 4.1/5.0,
    // quando preparo o POST do pedido explícito de remoção,
    // então apenas a opção solicitada é marcada no corpo, na ordem da tela.
    for versao in [Versao::V40, Versao::V41] {
        for manter_aberto in [false, true] {
            let html = tela_de_envio(versao, ID);
            let tela = conferir_tela_de_envio(&html, ID).unwrap();
            let campos = campos_do_envio(&tela, &destino(), manter_aberto, true);
            let pares: Vec<(&str, &str)> = campos.iter().map(|(n, v)| (*n, v.as_str())).collect();
            let url = url_da_tela();
            let servida = TelaServida {
                url: &url,
                html: &html,
                codificacao: CodificacaoDaTela::Iso88591,
            };
            let envio =
                derivar_submissao(&ENVIAR_PROCESSO, &instalacao(), &servida, "", &pares).unwrap();
            let esperado = corpo_do_envio(versao, manter_aberto).replacen(
                "txtPrazo",
                "chkSinRemoverAnotacoes=on&txtPrazo",
                1,
            );
            assert_eq!(String::from_utf8(envio.corpo().to_vec()).unwrap(), esperado);
        }
    }
}

// Teste técnico do cenário CLI BDD-333-03 com HTML originado do SQLite.
// A fonte do formulário é o anti-IES, não uma fixture HTML copiada.
#[test]
fn anti_ies_gera_formulario_que_o_leitor_ies_compreende() {
    use iesei::banco_cassettes::iniciar;
    use iesei::instalacao::Instalacao;
    use iesei::processos::{Destino as DestinoSimulado, ProcessoSimulado, guardar};
    use rusqlite::Connection;

    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let origem = Instalacao::interpretar("https://sei.exemplo/sei").unwrap();
    guardar(
        &mut banco,
        &origem,
        "teste-leitor",
        &ProcessoSimulado {
            id: "10".into(),
            numero: "00001.000001/2026-01".into(),
            unidade_origem: "ORIGEM".into(),
            aberto_em: vec!["ORIGEM".into()],
            anotacao_origem: Some("Texto".into()),
            destinos: vec![DestinoSimulado {
                id: "2".into(),
                sigla: "DESTINO".into(),
                descricao: "DESTINO - Unidade de testes".into(),
            }],
        },
    )
    .unwrap();
    let resposta = anti_ies::processo_envio::responder(
        "GET",
        "/sei/controlador.php?acao=procedimento_enviar&id_procedimento=10",
        "",
        &mut banco,
        &origem,
        "teste-leitor",
    );
    assert_eq!(resposta.estado, anti_ies::Estado::Ok);
    let tela = conferir_tela_de_envio(&resposta.corpo, "10").unwrap();
    assert_eq!(tela.orgao, "");
    assert!(tela.remover_anotacoes_oferecida);
    assert_eq!(tela.textos, ["txtPrazo", "txtDias"]);
}

#[test]
fn anti_ies_autocomplete_e_lido_pelo_parser_real_do_ies() {
    use iesei::banco_cassettes::iniciar;
    use iesei::instalacao::Instalacao;
    use iesei::processos::{Destino, ProcessoSimulado, guardar};
    use rusqlite::Connection;

    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let origem = Instalacao::interpretar("https://sei.exemplo/sei").unwrap();
    guardar(
        &mut banco,
        &origem,
        "teste-autocomplete",
        &ProcessoSimulado {
            id: "10".into(),
            numero: "00001.000001/2026-01".into(),
            unidade_origem: "ORIGEM".into(),
            aberto_em: vec!["ORIGEM".into()],
            anotacao_origem: Some("Texto".into()),
            destinos: vec![Destino {
                id: "2".into(),
                sigla: "DESTINO".into(),
                descricao: "DESTINO - Unidade de testes".into(),
            }],
        },
    )
    .unwrap();
    let resposta = anti_ies::processo_envio::autocompletar(
        "POST",
        "/sei/controlador_ajax.php?acao_ajax=unidade_auto_completar_envio_processo",
        "palavras_pesquisa=DES&id_orgao=",
        &banco,
        &origem,
        "teste-autocomplete",
    );
    assert_eq!(resposta.estado, anti_ies::Estado::Ok);
    let itens = super::destino::itens_do_autocompletar(&resposta.corpo).unwrap();
    let escolhido = super::destino::escolher_destino("DESTINO", &itens).unwrap();
    assert_eq!(escolhido.id_unidade, "2");
    assert_eq!(escolhido.sigla, "DESTINO");
}
