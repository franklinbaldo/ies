//! Equivalência do JSON público sem construir Value na produção.
use super::*;
use crate::arvore_processo::ProcessoNaArvore;

fn exemplo() -> DocumentoCancelado {
    DocumentoCancelado {
        numero: "0012345".into(),
        id_documento: "42".into(),
        processo: ProcessoNaArvore {
            id: "10".into(),
            nup: "00001.000001/2026-01".into(),
            tipo: "Processo público".into(),
        },
        unidade: Some("TESTE".into()),
        motivo: "Motivo observado".into(),
        confirmado: false,
        alterado: false,
        reconciliado_por: None,
        requisicoes: 4,
    }
}

fn conferir_mesmos_bytes(feito: &DocumentoCancelado) {
    let mut bytes = Vec::new();
    serde_json::to_writer_pretty(&mut bytes, &resultado_cancelamento(feito)).unwrap();
    let esperado = serde_json::to_string_pretty(&json_do_sucesso(feito)).unwrap();
    assert_eq!(std::str::from_utf8(&bytes).unwrap(), esperado);
    let texto = std::str::from_utf8(&bytes).unwrap();
    assert!(!texto.contains("infra_hash"));
    assert!(
        !texto.contains("Processo público"),
        "tipo interno não pode ser exposto"
    );
    assert!(!texto.contains('\u{1b}'));
}

#[test]
fn previa_igual_ao_contrato_legado() {
    conferir_mesmos_bytes(&exemplo());
}

#[test]
fn reconciliacao_mantem_causa_http_e_status() {
    let mut feito = exemplo();
    feito.confirmado = true;
    feito.alterado = true;
    feito.reconciliado_por = Some(FalhaDoCancelamento::RedirecionamentoNaoReconhecido);
    conferir_mesmos_bytes(&feito);
    let json = serde_json::to_value(resultado_cancelamento(&feito)).unwrap();
    assert_eq!(json["falha_do_envio"]["status_http"], 302);
    assert_eq!(json["reconciliado_por_leitura"], true);
}

#[test]
fn falha_de_cancelamento_apresentada_diretamente_do_enum() {
    let erro = ErroCancelar::CancelamentoNaoAplicado {
        falha_do_envio: Some(FalhaDoCancelamento::RedirecionamentoNaoReconhecido),
    };
    assert_eq!(
        super::linhas_da_falha(&erro),
        vec![
            format!("IES: {} [{}]", erro.mensagem(), erro.subcodigo()),
            "IES: resposta da escrita: redirecionamento_do_cancelamento_nao_reconhecido".to_owned(),
        ]
    );
}

#[test]
fn falha_sem_detalhes_nao_cria_linhas_artificiais() {
    let erro = ErroCancelar::DocumentoJaCancelado;
    assert_eq!(
        super::linhas_da_falha(&erro),
        vec![format!("IES: {} [{}]", erro.mensagem(), erro.subcodigo())]
    );
}

#[test]
fn bdd_273_02_cancelamento_conferido_sem_falha_do_post_informa_releitura() {
    let mut feito = exemplo();
    feito.confirmado = true;
    feito.alterado = true;
    // A releitura da árvore confirmou o efeito; nenhum erro de envio ocorreu.
    feito.reconciliado_por = None;
    let json = serde_json::to_value(resultado_cancelamento(&feito)).unwrap();
    assert_eq!(json["confirmado"], true);
    assert_eq!(json["alterado"], true);
    assert_eq!(json["reconciliado_por_leitura"], true);
    assert!(json.get("falha_do_envio").is_none());
}
