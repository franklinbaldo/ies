//! Testes de transporte roteirizado: uma escrita e uma leitura verificadora.
//! Casos de Todos `tests/test_ciclo_sobrestar_010.py` @323eb01.
use super::tests_tela::formulario;
use super::{Acao, ErroSobrestamento as E, PedidoDaMudanca, executar};
use crate::arvore_processo::tests_reabrir::tela_do_no_pela_variavel;
use crate::processo_concluir::tests_apoio::*;
use crate::propriedades_processo::tests_apoio::acao_da_barra;
use crate::transporte::FalhaTransporte;

fn sobrestar() -> String {
    acao_da_barra(&format!(
        "controlador.php?acao=procedimento_sobrestar&id_procedimento={ID}&infra_hash={}",
        "e".repeat(64)
    ))
}
fn barra_retomar() -> String {
    "<a href=\"#\" onclick=\"removerSobrestamentoProcesso();\">Retomar</a>".into()
}
fn sobrestado() -> String {
    arvore_para_concluir(
        ID,
        NUP,
        &format!("Processo sobrestado na unidade {}.<br />", ancora(UNIDADE)),
        &[barra_retomar()],
    )
}
fn url_tela_sobrestar() -> String {
    absoluta(&format!(
        "controlador.php?acao=procedimento_sobrestar&id_procedimento={ID}&infra_hash={}",
        "e".repeat(64)
    ))
}
fn url_post() -> String {
    absoluta(&format!(
        "controlador.php?acao=procedimento_sobrestar&acao_origem=procedimento_sobrestar&id_procedimento={ID}&infra_hash={}",
        "f".repeat(64)
    ))
}
fn volta_a_arvore() -> String {
    format!(
        "controlador.php?acao=arvore_visualizar&acao_origem=procedimento_sobrestar&id_procedimento={ID}&infra_hash={}",
        "b".repeat(64)
    )
}
fn rodar(
    acao: Acao,
    motivo: Option<&str>,
    confirmar: bool,
    respostas: Vec<Resposta>,
) -> (Result<super::Previa, E>, Vec<Pedido>) {
    let amb = ambiente();
    rodar_com(&amb, respostas, |sessao, nup, indice, envio| {
        let pedido = PedidoDaMudanca {
            nup,
            unidade: None,
            acao,
            motivo,
            confirmar,
        };
        executar(sessao, &pedido, indice, envio)
    })
}
fn respostas_sobrestar(resposta: Resposta, verificar: Resposta) -> Vec<Resposta> {
    let mut respostas = abertura(arvore(&[UNIDADE], &[sobrestar()]));
    respostas.extend([ok(formulario()), resposta, verificar]);
    respostas
}
fn corpo_esperado() -> String {
    "hdnInfraTipoPagina=2&sbmSalvar=Salvar&rdoSobrestar=S&txtProcedimentoDestino=&txtIdentificacaoProcedimentoDestino=&txaMotivo=Aguarda+decisao&hdnIdProtocolos=10&hdnIdProcedimentoDestino=".into()
}
fn pedidos_sobrestar() -> Vec<Pedido> {
    let mut pedidos = pedidos_da_abertura();
    pedidos.extend([
        get(&url_tela_sobrestar()),
        post(&url_post(), &corpo_esperado()),
        get(&url_arvore()),
    ]);
    pedidos
}
#[test]
fn sobrestar_um_post_verifica_sobrestamento_por_leitura_nova() {
    let (r, pedidos) = rodar(
        Acao::Sobrestar,
        Some("Aguarda decisao"),
        true,
        respostas_sobrestar(r302(&volta_a_arvore()), ok(sobrestado())),
    );
    let feito = r.unwrap();
    assert!(feito.estado.aplicada() && feito.estado.aplicada() && feito.estado.aplicada());
    assert_eq!(feito.requisicoes, 6);
    assert_eq!(pedidos, pedidos_sobrestar());
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}
#[test]
fn sobrestar_preserva_espacos_e_quebra_de_linha_no_post_unico() {
    let (r, pedidos) = rodar(
        Acao::Sobrestar,
        Some("  Aguarda\ndecisao  "),
        true,
        respostas_sobrestar(r302(&volta_a_arvore()), ok(sobrestado())),
    );
    let feito = r.unwrap();
    assert!(feito.estado.aplicada() && feito.estado.aplicada() && feito.estado.aplicada());
    assert_eq!(feito.requisicoes, 6);
    let mut esperados = pedidos_da_abertura();
    esperados.extend([
        get(&url_tela_sobrestar()),
        post(
            &url_post(),
            "hdnInfraTipoPagina=2&sbmSalvar=Salvar&rdoSobrestar=S&txtProcedimentoDestino=&txtIdentificacaoProcedimentoDestino=&txaMotivo=++Aguarda%0Adecisao++&hdnIdProtocolos=10&hdnIdProcedimentoDestino=",
        ),
        get(&url_arvore()),
    ]);
    assert_eq!(pedidos, esperados);
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

#[test]
fn sobrestar_nao_aplicado_na_releitura_nao_declara_sucesso() {
    let (r, pedidos) = rodar(
        Acao::Sobrestar,
        Some("Aguarda decisao"),
        true,
        respostas_sobrestar(
            r302(&volta_a_arvore()),
            ok(arvore(&[UNIDADE], &[sobrestar()])),
        ),
    );
    assert_eq!(r, Err(E::EscritaNaoAplicada));
    assert_eq!(pedidos, pedidos_sobrestar());
}
#[test]
fn sobrestar_sem_resposta_exige_releitura_sem_retry() {
    let (r, pedidos) = rodar(
        Acao::Sobrestar,
        Some("Aguarda decisao"),
        true,
        respostas_sobrestar(Err(FalhaTransporte::Rede), Err(FalhaTransporte::Rede)),
    );
    assert_eq!(r, Err(E::EfeitoIncerto));
    assert_eq!(pedidos, pedidos_sobrestar());
}
#[test]
fn formulario_de_outro_processo_recusa_antes_de_enviar() {
    let mut respostas = abertura(arvore(&[UNIDADE], &[sobrestar()]));
    respostas.push(ok(formulario().replace(
        "name=\"hdnIdProtocolos\" value=\"10\"",
        "name=\"hdnIdProtocolos\" value=\"11\"",
    )));
    let (r, pedidos) = rodar(Acao::Sobrestar, Some("Aguarda decisao"), true, respostas);
    assert_eq!(
        r,
        Err(E::TelaSobrestar(super::tela::ErroTela::OutroProcesso))
    );
    let mut esperados = pedidos_da_abertura();
    esperados.push(get(&url_tela_sobrestar()));
    assert_eq!(pedidos, esperados);
}
fn tela_de_retomar() -> String {
    // A tela real da família 4.1/5.0 já contém esta variável, além da
    // de reabrir. Não duplicar um campo que o SEI publica apenas uma vez.
    tela_do_no_pela_variavel(ID)
}
fn url_retomar() -> String {
    absoluta(&format!(
        "controlador.php?acao=procedimento_remover_sobrestamento&acao_origem=arvore_visualizar&id_procedimento={ID}&atualizar_arvore=1&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={}",
        "7".repeat(64),
    ))
}
#[test]
fn retomar_uma_escrita_get_sem_retry_e_verificacao() {
    let mut respostas = abertura(sobrestado());
    respostas.extend([
        ok(tela_de_retomar()),
        ok("<html></html>".into()),
        ok(arvore(&[UNIDADE], &[sobrestar()])),
    ]);
    let (r, pedidos) = rodar(Acao::Retomar, None, true, respostas);
    let feito = r.unwrap();
    assert!(feito.estado.aplicada() && feito.estado.aplicada() && feito.estado.aplicada());
    assert_eq!(feito.requisicoes, 6);
    let mut esperados = pedidos_da_abertura();
    esperados.extend([get(&links().1), get(&url_retomar()), get(&url_arvore())]);
    assert_eq!(pedidos, esperados);
}

#[test]
fn retomar_get_mutador_recusado_nao_atribui_mudanca_concorrente() {
    let mut respostas = abertura(sobrestado());
    respostas.extend([
        ok(tela_de_retomar()),
        r302("controlador.php?acao=login"),
        // Outra sessão pode ter alterado o processo antes da releitura.
        // A ausência de sobrestamento não prova que este GET o retirou.
        ok(arvore(&[UNIDADE], &[sobrestar()])),
    ]);
    let (resultado, pedidos) = rodar(Acao::Retomar, None, true, respostas);
    assert_eq!(resultado, Err(E::EfeitoIncerto));
    assert_eq!(pedidos.len(), 6);
    assert_eq!(
        pedidos.iter().filter(|p| p.metodo == "GET").count(),
        6
    );
}
