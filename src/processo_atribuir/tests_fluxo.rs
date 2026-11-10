//! RED/GREEN: prévia Web com orçamento exato, usando apenas respostas sintéticas.
//! Caso derivado de `Todos tests/test_ciclo_atribuir_010.py` @323eb01.
use super::fluxo::{ErroProcessoAtribuir, EstadoDaAtribuicao, PedidoDaAtribuicao, executar_previa};
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
) -> (
    Result<super::fluxo::ProcessoAtribuido, ErroProcessoAtribuir>,
    Vec<Pedido>,
) {
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
    assert_eq!(feito.estado, EstadoDaAtribuicao::Previa);
    assert_eq!(pedidos, pedidos_previa());
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 0);
}

// Caso de diagnóstico UFRJ: árvore expôs PR-P mas a prévia recusou
// situacao_da_atribuicao_nao_observada (runs 42229686084, 42230157551).
// A divergência da unidade da tela é uma HIPÓTESE sintética, não fato live.
#[test]
fn previa_com_unidade_da_arvore_diferente_publica_fatos_e_tentativas_sem_post() {
    let (resultado, pedidos) = rodar(Some("aluno2"), abertura(arvore(&["PR-P"], &acao())));
    let erro = resultado.unwrap_err();
    assert!(matches!(
        &erro,
        ErroProcessoAtribuir::SituacaoDaAtribuicaoNaoObservada {
            unidade,
            aberto_em: Some(siglas),
            motivo: None,
            requisicoes: 3,
        } if unidade == "TESTE" && siglas.len() == 1 && siglas[0] == "PR-P"
    ));
    assert_eq!(pedidos, pedidos_da_abertura());
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 0);
}

// #264: um único estado tipado comanda `confirmado` e `alterado`.
#[test]
fn estado_tipado_comanda_confirmado_e_alterado() {
    let (previa, pedidos_previa_observados) = rodar(Some("aluno2"), respostas_previa("aluno20"));
    let previa = previa.unwrap();
    assert_eq!(previa.estado, EstadoDaAtribuicao::Previa);
    let json_previa = crate::processo_atribuir_cli::json_da_previa(&previa);
    assert_eq!(json_previa["confirmado"], false);
    assert_eq!(json_previa["alterado"], false);
    assert_eq!(pedidos_previa_observados, pedidos_previa());

    let mut respostas = respostas_previa("aluno20");
    respostas.push(retorno_arvore_com_identidade_observado());
    respostas.push(ok(arvore_atual("aluno2")));
    let (confirmacao, pedidos_confirmacao_observados) =
        rodar_confirmacao(Some("aluno2"), respostas);
    let confirmacao = confirmacao.unwrap();
    assert_eq!(confirmacao.estado, EstadoDaAtribuicao::AlteracaoConfirmada);
    let json_confirmacao = crate::processo_atribuir_cli::json_da_previa(&confirmacao);
    assert_eq!(json_confirmacao["confirmado"], true);
    assert_eq!(json_confirmacao["alterado"], true);
    assert_eq!(
        pedidos_confirmacao_observados,
        pedidos_confirmacao("100000035")
    );
}

#[test]
fn remocao_exige_atribuicao_observada_sem_enviar_post() {
    let (resultado, pedidos) = rodar(None, respostas_previa("aluno20"));
    let feito = resultado.unwrap();
    assert_eq!(feito.valor_da_opcao, "null");
    assert_eq!(feito.requisicoes, 4);
    assert_eq!(pedidos, pedidos_previa());
}

// O estado já está publicado na árvore: não é necessário abrir o formulário.
// A recusa evita um GET e não pode enviar POST, inclusive com --confirmar.
#[test]
fn atribuicao_ja_aplicada_recusa_antes_da_tela_sem_post() {
    for confirmar in [false, true] {
        let respostas = abertura(arvore_atual("aluno2"));
        let (_resultado, pedidos) = if confirmar {
            rodar_confirmacao(Some("aluno2"), respostas)
        } else {
            rodar(Some("aluno2"), respostas)
        };
        assert_eq!(pedidos, pedidos_da_abertura());
        assert_eq!(pedidos.len(), 3);
        assert!(pedidos.iter().all(|p| p.metodo == "GET"));
    }
}

#[test]
fn remocao_ja_aplicada_recusa_antes_da_tela_sem_post() {
    for confirmar in [false, true] {
        let respostas = abertura(arvore_atual(""));
        let (_resultado, pedidos) = if confirmar {
            rodar_confirmacao(None, respostas)
        } else {
            rodar(None, respostas)
        };
        assert_eq!(pedidos, pedidos_da_abertura());
        assert_eq!(pedidos.len(), 3);
        assert!(pedidos.iter().all(|p| p.metodo == "GET"));
    }
}

// Login vazio é inválido, mesmo que a árvore informe ausência de atribuição.
// Preservar a causa já publicada: não confundir pedido inválido com no-op.
#[test]
fn login_vazio_nao_se_torna_atribuicao_ja_aplicada() {
    for confirmar in [false, true] {
        let respostas = respostas_previa("");
        let (_resultado, pedidos) = if confirmar {
            rodar_confirmacao(Some(""), respostas)
        } else {
            rodar(Some(""), respostas)
        };
        assert_eq!(pedidos, pedidos_previa());
        assert!(pedidos.iter().all(|p| p.metodo == "GET"));
    }
}

#[test]
fn barra_ausente_recusa_com_somente_tres_gets() {
    let (resultado, pedidos) = rodar(Some("aluno2"), abertura(arvore(&[UNIDADE], &[])));
    assert_eq!(
        resultado,
        Err(ErroProcessoAtribuir::AtribuicaoNaoOferecidaNaBarra)
    );
    assert_eq!(pedidos, pedidos_da_abertura());
}

#[test]
fn tela_recusada_preserva_o_status_http_e_nao_faz_post() {
    let mut respostas = abertura(arvore_atual("aluno20"));
    respostas.push(Err(FalhaTransporte::Rede));
    let (resultado, pedidos) = rodar(Some("aluno2"), respostas);
    let erro = resultado.unwrap_err();
    assert_eq!(erro, ErroProcessoAtribuir::Tela(FalhaTransporte::Rede));
    assert_eq!(pedidos, pedidos_previa());
}

#[test]
fn http_403_sobe_status_e_mensagem_sem_repetir_requisicao() {
    let mut respostas = abertura(arvore_atual("aluno20"));
    respostas.push(Ok((
        403,
        "<html><body><div id=\"divInfraExcecao\">Acesso negado.</div></body></html>".to_owned(),
        None,
    )));
    let (resultado, pedidos) = rodar(Some("aluno2"), respostas);
    let erro = resultado.unwrap_err();
    assert_eq!(
        erro,
        ErroProcessoAtribuir::Tela(FalhaTransporte::Status(
            crate::transporte::StatusHttpRecusado {
                status: 403,
                mensagem_do_servidor: Some("Acesso negado.".into())
            }
        ))
    );
    assert_eq!(pedidos, pedidos_previa());
}

#[test]
fn confirmar_login_inexistente_falha_antes_do_post() {
    let (_resultado, pedidos) =
        rodar_confirmacao(Some("login_ausente"), respostas_previa("aluno20"));
    assert_eq!(pedidos, pedidos_previa());
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 0);
}

fn rodar_confirmacao(
    usuario: Option<&str>,
    respostas: Vec<Resposta>,
) -> (
    Result<super::fluxo::ProcessoAtribuido, ErroProcessoAtribuir>,
    Vec<Pedido>,
) {
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

// O 302 aceito contém id_procedimento na observação da UnB registrada
// no oráculo Todos @323eb01 (ACEITE), não apenas a ação genérica.
fn retorno_arvore_com_identidade_observado() -> Resposta {
    r302(&format!(
        "controlador.php?acao=arvore_visualizar&acao_origem=procedimento_atribuicao_cadastrar&id_procedimento={ID}&atualizar_arvore=1"
    ))
}

// Caso de Todos tests/test_ciclo_atribuir_010.py @323eb01: escrita única.
#[test]
fn confirmar_atribuicao_faz_um_post_e_verifica_por_arvore_nova() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(retorno_arvore_com_identidade_observado());
    respostas.push(ok(arvore_atual("aluno2")));
    let (resultado, pedidos) = rodar_confirmacao(Some("aluno2"), respostas);
    let feito = resultado.unwrap();
    assert_eq!(feito.estado, EstadoDaAtribuicao::AlteracaoConfirmada);
    assert_eq!(feito.requisicoes, 6);
    assert_eq!(pedidos, pedidos_confirmacao("100000035"));
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

// Caso de Todos tests/test_ciclo_atribuir_010.py @323eb01: remover = null.
#[test]
fn confirmar_remocao_posta_null_e_verifica_ausencia_de_atribuicao() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(retorno_arvore_com_identidade_observado());
    respostas.push(ok(arvore_atual("")));
    let (resultado, pedidos) = rodar_confirmacao(None, respostas);
    let feito = resultado.unwrap();
    assert_eq!(feito.estado, EstadoDaAtribuicao::AlteracaoConfirmada);
    assert_eq!(feito.requisicoes, 6);
    assert_eq!(pedidos, pedidos_confirmacao("null"));
}

#[test]
fn falha_do_post_e_releitura_sem_prova_sai_tres_sem_retry() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(Err(FalhaTransporte::Rede));
    respostas.push(ok(arvore_atual("aluno20")));
    let (resultado, pedidos) = rodar_confirmacao(Some("aluno2"), respostas);
    let _erro = resultado.unwrap_err();
    assert_eq!(pedidos, pedidos_confirmacao("100000035"));
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

// Depois de POST sem resposta, a árvore pode refletir alteração concorrente.
// Mesmo coincidente, a leitura não identifica qual agente atribuiu o processo.
#[test]
fn post_sem_resposta_e_releitura_coincidente_nao_comprovam_autoria() {
    let mut respostas = respostas_previa("aluno20");
    respostas.push(Err(FalhaTransporte::Rede));
    respostas.push(ok(arvore_atual("aluno2")));
    let (resultado, pedidos) = rodar_confirmacao(Some("aluno2"), respostas);
    let erro = resultado.unwrap_err();
    assert!(matches!(
        &erro,
        ErroProcessoAtribuir::NaoConfirmadoNaAtribuicao {
            falha_do_envio: Some(falha),
            leitura: None,
            atribuicao_relida: Some(atual),
            requisicoes: 6,
        } if matches!(
            falha.as_ref(),
            crate::formulario_web::ErroSubmissaoFormulario::Transporte(FalhaTransporte::Rede)
        ) && atual == "aluno2"
    ));
    assert_eq!(pedidos, pedidos_confirmacao("100000035"));
    assert_eq!(pedidos.len(), 6);
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

/// O fieldset desabilita o botão mesmo sem atributo disabled no botão.
/// Não se tenta o POST: a leitura de prévia custa exatamente quatro GETs.
#[test]
fn fieldset_desabilitado_recusa_confirmacao_sem_post() {
    let mut respostas = abertura(arvore_atual("aluno20"));
    respostas.push(ok(formulario().replace(
        "<button type=\"submit\" name=\"sbmSalvar\" value=\"Salvar\">Salvar</button>",
        "<fieldset disabled><button type=\"submit\" name=\"sbmSalvar\" value=\"Salvar\">Salvar</button></fieldset>",
    )));
    let (_resultado, pedidos) = rodar_confirmacao(Some("aluno2"), respostas);
    assert_eq!(pedidos, pedidos_previa());
    assert_eq!(pedidos.len(), 4);
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 0);
}

#[path = "tests_fluxo/adversariais.rs"]
mod adversariais;

#[path = "tests_fluxo/saida.rs"]
mod saida;
