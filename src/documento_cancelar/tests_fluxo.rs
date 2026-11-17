//! Testes sintéticos integrados de cancelamento (#273): os cenários vêm
//! de Todos `tests/test_documento_mover_cancelar_010.py` @323eb01.
//! A bancada registra POST antes de devolver rede, sem retry.
use super::{DocumentoCancelado, ErroCancelar, PedidoDoCancelamento, cancelar_documento};
use crate::arvore_processo::tests_assinatura::arvore_assinavel;
use crate::documento::NumeroDocumento;
use crate::processo_concluir::tests_apoio::*;
use crate::transporte::FalhaTransporte;

const DOC: &str = "0000101";
const ID_DOC: &str = "21";
const MOTIVO: &str = "Teste fictício";

fn href() -> String {
    format!(
        "controlador.php?acao=documento_cancelar&acao_origem=arvore_visualizar&id_procedimento={ID}&id_documento={ID_DOC}&infra_hash={}",
        "a".repeat(64)
    )
}
fn action() -> String {
    format!(
        "controlador.php?acao=documento_cancelar&acao_origem=documento_cancelar&arvore=1&id_procedimento={ID}&id_documento={ID_DOC}&infra_hash={}",
        "b".repeat(64)
    )
}
fn arvore_ativa() -> String {
    let acao = format!("<a href=\"{}\">Cancelar Documento</a>", href());
    arvore_assinavel(ID, ID_DOC, DOC, &acao, &[])
}
fn arvore_cancelada() -> String {
    arvore_ativa()
        .replace(
            r"De: pessoa@exemplo.gov.br\nPara: outra@exemplo.gov.br",
            &format!("Documento Cancelado: {MOTIVO}"),
        )
        .replace("svg/documento_interno.svg", "svg/documento_cancelado.svg")
}
fn tela(doc: &str) -> String {
    format!(
        "<html><body><form id=\"frmDocumentoCancelar\" method=\"post\" onsubmit=\"return OnSubmitForm();\" action=\"{action}\">\
         <input type=\"hidden\" id=\"hdnInfraTipoPagina\" name=\"hdnInfraTipoPagina\" value=\"2\"/>\
         <button type=\"submit\" name=\"sbmSalvar\" id=\"sbmSalvar\" value=\"Salvar\">Salvar</button>\
         <textarea id=\"txaMotivo\" name=\"txaMotivo\"></textarea></form></body></html>",
        action = action().replace(
            &format!("id_documento={ID_DOC}"),
            &format!("id_documento={doc}")
        )
    )
}
fn location() -> String {
    format!(
        "controlador.php?acao=arvore_visualizar&acao_origem=documento_cancelar&arvore=1&id_procedimento={ID}&id_documento={ID_DOC}&atualizar_arvore=1&infra_hash={}",
        "c".repeat(64)
    )
}
fn ate_tela(arvore: String) -> Vec<Resposta> {
    let mut respostas = abertura(arvore);
    respostas.push(ok(tela(ID_DOC)));
    respostas
}
fn pedidos_previa() -> Vec<Pedido> {
    let mut pedidos = pedidos_da_abertura();
    pedidos.push(get(&absoluta(&href())));
    pedidos
}
fn pedido_post() -> Pedido {
    post(
        &absoluta(&action()),
        "hdnInfraTipoPagina=2&sbmSalvar=Salvar&txaMotivo=Teste+fict%EDcio",
    )
}
fn executar(
    confirmar: bool,
    respostas: Vec<Resposta>,
) -> (Result<DocumentoCancelado, ErroCancelar>, Vec<Pedido>) {
    executar_com_motivo(confirmar, respostas, MOTIVO)
}

fn executar_com_motivo(
    confirmar: bool,
    respostas: Vec<Resposta>,
    motivo: &str,
) -> (Result<DocumentoCancelado, ErroCancelar>, Vec<Pedido>) {
    let ambiente = ambiente();
    rodar_com(&ambiente, respostas, |sessao, nup, indice, envio| {
        let numero = NumeroDocumento::novo(DOC).unwrap();
        let pedido = PedidoDoCancelamento {
            numero: &numero,
            processo: Some(nup),
            unidade: None,
            motivo,
            confirmar,
        };
        cancelar_documento(sessao, &pedido, indice, envio)
    })
}

#[test]
fn previa_preserva_o_texto_do_motivo_sem_trim_ou_recusa_de_quebra_de_linha() {
    let motivo = "  Revisão\nnecessária  ";
    let (resultado, pedidos) = executar_com_motivo(false, ate_tela(arvore_ativa()), motivo);
    assert_eq!(resultado.unwrap().motivo, motivo);
    assert_eq!(pedidos, pedidos_previa());
}

#[test]
fn charset_da_tela_recusa_valor_nao_representavel_antes_do_post() {
    let (resultado, pedidos) = executar_com_motivo(true, ate_tela(arvore_ativa()), "Cópia 🚀");
    assert!(matches!(
        resultado,
        Err(ErroCancelar::Formulario(
            crate::formulario_web::ErroSubmissaoFormulario::ValorNaoRepresentavelNaCodificacao {
                campo
            }
        )) if campo == "txaMotivo"
    ));
    assert_eq!(pedidos, pedidos_previa());
}

#[test]
fn post_preserva_texto_e_motivo_remoto_diferente_nao_comprova_cancelamento() {
    let motivo = "  Revisão\nnecessária  ";
    let mut respostas = ate_tela(arvore_ativa());
    respostas.extend([r302(&location()), ok(arvore_cancelada())]);
    let (resultado, pedidos) = executar_com_motivo(true, respostas, motivo);
    assert!(matches!(resultado, Err(ErroCancelar::NaoConfirmado { .. })));
    let mut esperados = pedidos_previa();
    esperados.push(post(
        &absoluta(&action()),
        "hdnInfraTipoPagina=2&sbmSalvar=Salvar&txaMotivo=++Revis%E3o%0Anecess%E1ria++",
    ));
    esperados.push(get(&url_arvore()));
    assert_eq!(pedidos, esperados);
}

#[test]
fn previa_abre_arvore_e_tela_em_quatro_gets_sem_post() {
    let (resultado, pedidos) = executar(false, ate_tela(arvore_ativa()));
    let feito = resultado.unwrap();
    assert_eq!(pedidos, pedidos_previa());
    assert_eq!(feito.requisicoes, 4);
    assert!(matches!(
        feito.resultado,
        super::ResultadoDaVerificacao::Previa
    ));
    assert_eq!(feito.motivo, MOTIVO);
}

#[test]
fn confirmar_envia_um_post_e_prova_cancelamento_pela_arvore_nova() {
    let mut respostas = ate_tela(arvore_ativa());
    respostas.extend([r302(&location()), ok(arvore_cancelada())]);
    let (resultado, pedidos) = executar(true, respostas);
    let feito = resultado.unwrap();
    let mut esperados = pedidos_previa();
    esperados.push(pedido_post());
    esperados.push(get(&url_arvore()));
    assert_eq!(pedidos, esperados);
    assert_eq!(feito.requisicoes, 6);
    assert!(feito.resultado.cancelado() && feito.resultado.falha_do_envio().is_none());
}

#[test]
fn servidor_falha_depois_do_post_mas_arvore_confirma_sem_retry() {
    let mut respostas = ate_tela(arvore_ativa());
    respostas.extend([Err(FalhaTransporte::Rede), ok(arvore_cancelada())]);
    let (resultado, pedidos) = executar(true, respostas);
    let feito = resultado.unwrap();
    assert!(feito.resultado.cancelado() && feito.resultado.falha_do_envio().is_some());
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
    assert_eq!(pedidos.len(), 6);
}

#[test]
fn servidor_falha_e_releitura_falha_e_nao_confirmado_saida_tres() {
    let mut respostas = ate_tela(arvore_ativa());
    respostas.extend([Err(FalhaTransporte::Rede), Err(FalhaTransporte::Rede)]);
    let (resultado, pedidos) = executar(true, respostas);
    let erro = resultado.unwrap_err();
    assert!(matches!(erro, ErroCancelar::NaoConfirmado { .. }));
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
    assert_eq!(pedidos.len(), 6);
}

#[test]
fn arvore_reaberta_sem_icone_de_cancelamento_nao_comprova() {
    let mut respostas = ate_tela(arvore_ativa());
    respostas.extend([r302(&location()), ok(arvore_ativa())]);
    let (resultado, pedidos) = executar(true, respostas);
    assert!(matches!(
        resultado,
        Err(ErroCancelar::CancelamentoNaoAplicado { .. })
    ));
    assert_eq!(pedidos.iter().filter(|p| p.metodo == "POST").count(), 1);
}

#[test]
fn formulario_assinado_de_outro_documento_recusa_antes_do_post() {
    let mut respostas = abertura(arvore_ativa());
    respostas.push(ok(tela("99")));
    let (resultado, pedidos) = executar(true, respostas);
    assert!(matches!(
        resultado,
        Err(ErroCancelar::EnderecoDoDocumentoRecusado(_))
    ));
    assert_eq!(pedidos, pedidos_previa());
}

#[test]
fn documento_ja_cancelado_e_recusado_antes_da_tela_ou_post() {
    let (resultado, pedidos) = executar(false, abertura(arvore_cancelada()));
    assert_eq!(resultado, Err(ErroCancelar::DocumentoJaCancelado));
    assert_eq!(pedidos, pedidos_da_abertura());
}
