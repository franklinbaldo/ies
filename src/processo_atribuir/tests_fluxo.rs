//! RED/GREEN: prévia Web com orçamento exato, usando apenas respostas sintéticas.
//! Caso derivado de `Todos tests/test_ciclo_atribuir_010.py` @323eb01.
use super::fluxo::{ErroProcessoAtribuir as E, PedidoDaAtribuicao, executar_previa};
use crate::processo_concluir::tests_apoio::*;
use crate::propriedades_processo::tests_apoio::acao_da_barra;
use crate::transporte::FalhaTransporte;

fn link() -> String {
    format!(
        "controlador.php?acao=procedimento_atribuicao_cadastrar&acao_origem=arvore_visualizar&id_procedimento={ID}&infra_hash={}",
        "d".repeat(64)
    )
}

fn acao() -> Vec<String> {
    vec![acao_da_barra(&link())]
}

fn formulario() -> String {
    format!(
        r#"<html><body><form id="frmAtividadeAtribuir" method="post" action="controlador.php?acao=procedimento_atribuicao_cadastrar&id_procedimento={ID}&infra_hash={h}">
<input type="hidden" name="hdnInfraTipoPagina" value="2">
<button type="submit" name="sbmSalvar" value="Salvar">Salvar</button>
<button type="button" name="btnCancelar" value="Cancelar">Cancelar</button>
<select id="selAtribuicao" name="selAtribuicao">
<option value="null" selected>&nbsp;</option>
<option value="100000035">aluno2 - aluno2</option>
<option value="100000036">aluno20 - aluno20</option>
</select>
<input type="hidden" name="hdnIdProtocolo" value="{ID}">
</form></body></html>"#,
        h = "f".repeat(64)
    )
}

fn arvore_atual(atual: &str) -> String {
    let mut html = arvore(&[UNIDADE], &acao());
    if !atual.is_empty() {
        let antes = format!(
            "Processo aberto somente na unidade {}.<br />",
            ancora(UNIDADE)
        );
        let depois = format!(
            "Processo aberto somente na unidade {} (atribuído para <a alt=\"{atual}\" title=\"{atual}\" class=\"ancoraSigla\">{atual}</a>).<br />",
            ancora(UNIDADE)
        );
        assert!(html.contains(&antes));
        html = html.replace(&antes, &depois);
    }
    html
}

fn respostas_previa(atual: &str) -> Vec<Resposta> {
    let mut respostas = abertura(arvore_atual(atual));
    respostas.push(ok(formulario()));
    respostas
}

fn pedidos_previa() -> Vec<Pedido> {
    let mut pedidos = pedidos_da_abertura();
    pedidos.push(get(&format!("{ORIGEM}/{}", link())));
    pedidos
}

fn rodar(
    usuario: Option<&str>,
    respostas: Vec<Resposta>,
) -> (Result<super::fluxo::ProcessoAtribuido, E>, Vec<Pedido>) {
    let amb = ambiente();
    rodar_com(&amb, respostas, |sessao, nup, indice, envio| {
        executar_previa(
            sessao,
            &PedidoDaAtribuicao {
                nup,
                unidade: None,
                usuario,
                confirmar: false,
            },
            indice,
            envio,
        )
    })
}

#[test]
fn previa_seleciona_login_exato_com_quatro_gets_e_nenhum_post() {
    let (resultado, pedidos) = rodar(Some("aluno2"), respostas_previa("aluno20"));
    let feito = resultado.unwrap();
    assert_eq!(feito.usuario_atual, "aluno20");
    assert_eq!(feito.usuario_pedido.as_deref(), Some("aluno2"));
    assert_eq!(feito.valor_da_opcao, "100000035");
    assert_eq!(feito.requisicoes, 4);
    assert!(!feito.confirmado && !feito.alterado);
    assert_eq!(pedidos, pedidos_previa());
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 0);
}

#[test]
fn remocao_exige_atribuicao_observada_sem_enviar_post() {
    let (resultado, pedidos) = rodar(None, respostas_previa("aluno20"));
    let feito = resultado.unwrap();
    assert_eq!(feito.valor_da_opcao, "null");
    assert_eq!(feito.requisicoes, 4);
    assert_eq!(pedidos, pedidos_previa());
}

#[test]
fn barra_ausente_recusa_com_somente_tres_gets() {
    let (resultado, pedidos) = rodar(Some("aluno2"), abertura(arvore(&[UNIDADE], &[])));
    assert_eq!(resultado, Err(E::AtribuicaoNaoOferecidaNaBarra));
    assert_eq!(pedidos, pedidos_da_abertura());
}

#[test]
fn tela_recusada_preserva_o_status_http_e_nao_faz_post() {
    let mut respostas = abertura(arvore_atual("aluno20"));
    respostas.push(Err(FalhaTransporte::Rede));
    let (resultado, pedidos) = rodar(Some("aluno2"), respostas);
    let erro = resultado.unwrap_err();
    assert_eq!(erro, E::Tela(FalhaTransporte::Rede));
    assert_eq!(pedidos, pedidos_previa());
}

#[test]
fn http_403_publica_status_e_mensagem_sem_repetir_requisicao() {
    let mut respostas = abertura(arvore_atual("aluno20"));
    respostas.push(Ok((
        403,
        "<html><body><div id=\"divInfraExcecao\">Acesso negado.</div></body></html>".to_owned(),
        None,
    )));
    let (resultado, pedidos) = rodar(Some("aluno2"), respostas);
    let erro = resultado.unwrap_err();
    assert_eq!(erro.status_http().map(|h| h.status), Some(403));
    assert_eq!(
        erro.status_http()
            .and_then(|h| h.mensagem_do_servidor.as_deref()),
        Some("Acesso negado.")
    );
    let publico = crate::processo_concluir_cli::json_base_da_falha(
        "atribuicao_recusada",
        erro.subcodigo(),
        erro.mensagem(),
        erro.status_http().cloned(),
    );
    assert_eq!(publico["status_http"], 403);
    assert_eq!(publico["mensagem_do_servidor"], "Acesso negado.");
    assert_eq!(pedidos, pedidos_previa());
}

#[test]
fn confirmar_login_inexistente_falha_antes_do_post() {
    let (resultado, pedidos) =
        rodar_confirmacao(Some("login_ausente"), respostas_previa("aluno20"));
    assert_eq!(resultado.unwrap_err().subcodigo(), "usuario_nao_atribuivel");
    assert_eq!(pedidos, pedidos_previa());
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 0);
}

fn rodar_confirmacao(
    usuario: Option<&str>,
    respostas: Vec<Resposta>,
) -> (Result<super::fluxo::ProcessoAtribuido, E>, Vec<Pedido>) {
    let amb = ambiente();
    rodar_com(&amb, respostas, |sessao, nup, indice, envio| {
        executar_previa(
            sessao,
            &PedidoDaAtribuicao {
                nup,
                unidade: None,
                usuario,
                confirmar: true,
            },
            indice,
            envio,
        )
    })
}

fn pedidos_confirmacao(valor: &str) -> Vec<Pedido> {
    let mut esperados = pedidos_previa();
    esperados.push(post(
        &format!(
            "{ORIGEM}/controlador.php?acao=procedimento_atribuicao_cadastrar&id_procedimento={ID}&infra_hash={}",
            "f".repeat(64)
        ),
        &format!(
            "hdnInfraTipoPagina=2&sbmSalvar=Salvar&selAtribuicao={valor}&hdnIdProtocolo={ID}"
        ),
    ));
    esperados.push(get(&url_arvore()));
    esperados
}

// Caso de Todos tests/test_ciclo_atribuir_010.py @323eb01: escrita única.
#[test]
fn confirmar_atribuicao_faz_um_post_e_verifica_por_arvore_nova() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(r302("controlador.php?acao=arvore_visualizar"));
    respostas.push(ok(arvore_atual("aluno2")));
    let (resultado, pedidos) = rodar_confirmacao(Some("aluno2"), respostas);
    let feito = resultado.unwrap();
    assert!(feito.confirmado && feito.alterado);
    assert_eq!(feito.requisicoes, 6);
    assert_eq!(pedidos, pedidos_confirmacao("100000035"));
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

// Caso de Todos tests/test_ciclo_atribuir_010.py @323eb01: remover = null.
#[test]
fn confirmar_remocao_posta_null_e_verifica_ausencia_de_atribuicao() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(r302("controlador.php?acao=arvore_visualizar"));
    respostas.push(ok(arvore_atual("")));
    let (resultado, pedidos) = rodar_confirmacao(None, respostas);
    let feito = resultado.unwrap();
    assert!(feito.confirmado && feito.alterado);
    assert_eq!(feito.requisicoes, 6);
    assert_eq!(pedidos, pedidos_confirmacao("null"));
}

#[test]
fn falha_do_post_e_releitura_sem_prova_sai_tres_sem_retry() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(Err(FalhaTransporte::Rede));
    respostas.push(ok(arvore_atual("aluno20")));
    let (resultado, pedidos) = rodar_confirmacao(Some("aluno2"), respostas);
    let erro = resultado.unwrap_err();
    assert_eq!(erro.subcodigo(), "efeito_incerto_na_atribuicao");
    assert_eq!(erro.saida(), 3);
    assert_eq!(pedidos, pedidos_confirmacao("100000035"));
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

#[test]
fn falha_do_post_com_prova_independente_confirma_sem_segundo_post() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(Err(FalhaTransporte::Rede));
    respostas.push(ok(arvore_atual("aluno2")));
    let (resultado, pedidos) = rodar_confirmacao(Some("aluno2"), respostas);
    let feito = resultado.unwrap();
    assert!(feito.confirmado && feito.alterado);
    assert_eq!(feito.requisicoes, 6);
    assert_eq!(pedidos, pedidos_confirmacao("100000035"));
}
