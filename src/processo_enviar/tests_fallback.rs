//! BDD-333-03/04: fallback observado na CLI sem checkbox de remoção.
//! Respostas sintéticas; nenhuma instalação SEI é acessada.
use super::tests_apoio::*;
use super::{MetodoRemocao, PedidoDoEnvio, enviar_processo};
use crate::arvore_processo::tests_concluir::absoluta;
use crate::processo_concluir::tests_apoio::*;
use crate::propriedades_processo::tests_apoio::acao_da_barra;

fn link_anotacao() -> String {
    format!(
        "controlador.php?acao=anotacao_registrar&acao_origem=arvore_visualizar&id_procedimento={ID}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={HASH_ENVIAR}"
    )
}

fn action_anotacao() -> String {
    format!(
        "controlador.php?acao=anotacao_registrar&acao_origem=anotacao_registrar&id_procedimento={ID}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={HASH_ACTION_ENVIO}"
    )
}

fn form_anotacao(texto: &str) -> String {
    format!(
        r#"<html><form id="frmAnotacaoCadastro" method="post" action="controlador.php?acao=anotacao_registrar&acao_origem=anotacao_registrar&id_procedimento={ID}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={HASH_ACTION_ENVIO}">
        <input type="hidden" name="hdnInfraTipoPagina" value="2">
        <button type="submit" name="sbmRegistrarAnotacao" value="Salvar">Salvar</button>
        <input type="hidden" name="hdnIdProtocolo" value="{ID}">
        <textarea name="txaDescricao">{texto}</textarea>
        </form></html>"#
    )
}

fn respostas_envio(texto: &str, limpeza: Vec<Resposta>) -> Vec<Resposta> {
    let acoes = vec![pelo_envio(ID), acao_da_barra(&link_anotacao())];
    let sem_checkbox =
        tela_de_envio(Versao::V41, ID).replace(r#"name="chkSinRemoverAnotacoes""#, r#"name="""#);
    let mut depois = vec![ok(sem_checkbox), ok(form_anotacao(texto)), ok(um_destino())];
    depois.extend(limpeza);
    com(abertura(arvore(&[UNIDADE], &acoes)), depois)
}

fn executar(
    respostas: Vec<Resposta>,
) -> (
    Result<super::ProcessoEnviado, super::ErroEnviar>,
    Vec<Pedido>,
) {
    let amb = ambiente();
    rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        enviar_processo(
            sessao,
            &PedidoDoEnvio {
                nup,
                unidade: None,
                para: DESTINO,
                manter_aberto: false,
                remover_anotacoes: true,
                confirmar: true,
            },
            indice,
            rede,
        )
    })
}

#[test]
fn bdd_333_03_limpa_anotacao_confere_e_so_depois_envia() {
    // Dado envio sem checkbox e editor oferecido com anotação;
    // Quando executo ies processo enviar NUP --para SIGLA --remover-anotacoes --confirmar;
    // Então a anotação é limpa e conferida antes de tramitar, e o destino é comprovado.
    let (resultado, pedidos) = executar(respostas_envio(
        "Anotação original",
        vec![
            r302(&volta_a_arvore_do_envio()),
            ok(form_anotacao("")),
            r302(&volta_a_arvore_do_envio()),
            ok(arvore(&[DESTINO], &[])),
        ],
    ));
    let feito = resultado.expect("limpeza relida e destino confirmado");
    assert_eq!(feito.metodo_remocao, Some(MetodoRemocao::LimparAnotacao));
    assert!(feito.alterado);
    assert_eq!(feito.anotacao_depois, Some(false));
    assert_eq!(feito.requisicoes, 10);
    let posts: Vec<_> = pedidos.iter().filter(|p| p.metodo == "POST").collect();
    assert_eq!(posts.len(), 3, "consulta de destino, limpeza e envio");
    assert_eq!(posts[1].url, absoluta(&action_anotacao()));
    assert!(
        posts[1]
            .corpo
            .as_deref()
            .unwrap_or_default()
            .contains("txaDescricao=")
    );
    assert_eq!(posts[2].url, absoluta(&action_de_enviar()));
    assert!(
        !posts[2]
            .corpo
            .as_deref()
            .unwrap_or_default()
            .contains("chkSinRemoverAnotacoes")
    );
}

#[test]
fn bdd_333_04_anotacao_ja_vazia_nao_grava_limpeza() {
    // Dado anotação já vazia e editor oferecido;
    // Quando executo ies processo enviar NUP --para SIGLA --remover-anotacoes --confirmar;
    // Então não há POST de limpeza e a tramitação é verificada.
    let (resultado, pedidos) = executar(respostas_envio(
        "",
        vec![
            r302(&volta_a_arvore_do_envio()),
            ok(arvore(&[DESTINO], &[])),
        ],
    ));
    let feito = resultado.expect("nenhuma limpeza redundante");
    assert_eq!(feito.anotacao_antes, Some(false));
    assert_eq!(feito.anotacao_depois, Some(false));
    assert_eq!(feito.requisicoes, 8);
    assert!(
        !pedidos
            .iter()
            .any(|p| p.metodo == "POST" && p.url == absoluta(&action_anotacao()))
    );
}

#[test]
fn bdd_333_04_limpeza_inconclusiva_falha_sem_tramitar() {
    // Dado o envio sem checkbox e a anotação preenchida;
    // Quando executo ies processo enviar NUP --para SIGLA --remover-anotacoes --confirmar;
    // Então uma releitura inconclusiva retorna saída 3 sem enviar nem repetir limpeza.
    let (resultado, pedidos) = executar(respostas_envio(
        "Antes",
        vec![
            r302(&volta_a_arvore_do_envio()),
            Err(crate::transporte::FalhaTransporte::Rede),
        ],
    ));
    let erro = resultado.expect_err("sem releitura não há envio autorizado");
    assert!(matches!(
        erro,
        super::ErroEnviar::NaoConfirmadoNaLimpezaDaAnotacao
    ));
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.metodo == "POST" && p.url == absoluta(&action_anotacao()))
            .count(),
        1
    );
    assert!(
        !pedidos
            .iter()
            .any(|p| { p.metodo == "POST" && p.url == absoluta(&action_de_enviar()) })
    );
}

#[test]
fn bdd_333_03_anotacao_continua_preenchida_nao_tramita() {
    // Dado a tentativa única de salvar a descrição vazia;
    // Quando executo ies processo enviar NUP --para SIGLA --remover-anotacoes --confirmar;
    // Então uma anotação ainda preenchida impede o envio.
    let (resultado, pedidos) = executar(respostas_envio(
        "Antes",
        vec![
            r302(&volta_a_arvore_do_envio()),
            ok(form_anotacao("Ainda presente")),
        ],
    ));
    assert!(matches!(
        resultado,
        Err(super::ErroEnviar::AnotacaoNaoLimpaAntesDeEnviar)
    ));
    assert!(
        !pedidos
            .iter()
            .any(|p| { p.metodo == "POST" && p.url == absoluta(&action_de_enviar()) })
    );
}

#[test]
fn bdd_333_04_envio_incerto_preserva_a_limpeza_ja_comprovada_na_saida() {
    // Dado que a limpeza foi relida como vazia, mas a releitura do envio falhou;
    // Quando a confirmação termina com efeito incerto na tramitação;
    // Então a CLI informa que a anotação já está limpa, sem repetir os POSTs.
    let (resultado, pedidos) = executar(respostas_envio(
        "Antes",
        vec![
            r302(&volta_a_arvore_do_envio()),
            ok(form_anotacao("")),
            r302(&volta_a_arvore_do_envio()),
            Err(crate::transporte::FalhaTransporte::Rede),
        ],
    ));
    let erro = resultado.expect_err("o destino não foi comprovado");
    let saida = serde_json::to_value(crate::processo_enviar_cli::json_de_falha(&erro))
        .expect("erro tipado serializável");
    assert_eq!(saida["codigo"], "nao_confirmado");
    assert_eq!(
        saida["anotacao_depois"], false,
        "limpeza confirmada não pode desaparecer da falha"
    );
    for link in [action_anotacao(), action_de_enviar()] {
        assert_eq!(
            pedidos
                .iter()
                .filter(|p| p.metodo == "POST" && p.url == absoluta(&link))
                .count(),
            1,
            "cada escrita é única"
        );
    }
}

#[test]
fn bdd_333_05_sem_checkbox_e_sem_editor_recusa_na_previa() {
    // Dado que a instalação não oferece nem checkbox nem edição da anotação;
    // Quando executo ies processo enviar NUP --para SIGLA --remover-anotacoes;
    // Então a CLI recusa a remoção não oferecida, sem tramitar o processo.
    let acoes = vec![pelo_envio(ID), acao_da_barra(&link_anotacao())];
    let sem_checkbox =
        tela_de_envio(Versao::V41, ID).replace(r#"name="chkSinRemoverAnotacoes""#, r#"name="""#);
    let formulario_sem_edicao = form_anotacao("Antes").replace(
        r#"<button type="submit" name="sbmRegistrarAnotacao" value="Salvar">Salvar</button>"#,
        "<span>Somente leitura</span>",
    );
    let respostas = com(
        abertura(arvore(&[UNIDADE], &acoes)),
        vec![ok(sem_checkbox), ok(formulario_sem_edicao)],
    );
    let amb = ambiente();
    let (resultado, pedidos) = rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        enviar_processo(
            sessao,
            &PedidoDoEnvio {
                nup,
                unidade: None,
                para: DESTINO,
                manter_aberto: false,
                remover_anotacoes: true,
                confirmar: false,
            },
            indice,
            rede,
        )
    });
    assert!(matches!(
        resultado,
        Err(super::ErroEnviar::RemocaoDeAnotacoesNaoOferecida)
    ));
    assert!(pedidos.iter().all(|pedido| pedido.metodo != "POST"));
}
