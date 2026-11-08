//! BDD-273-01/02 preexistentes: CLI real → TLS local → anti-IES → SQLite.
use super::*;

#[test]
fn bdd_273_01_cli_previa_sem_cancelar_documento() {
    let pasta = Pasta::nova();
    let saida = executar(
        &pasta,
        &["--json", "documento", "cancelar", DOC, "--motivo", "Teste ficticio"],
        4, // resolução, tela de trabalho, árvore e formulário; nenhum POST.
    );
    assert_eq!(saida.status.code(), Some(0), "stderr={}; stdout={}",
        String::from_utf8_lossy(&saida.stderr),
        String::from_utf8_lossy(&saida.stdout));
    let previa = json(&saida);
    assert_eq!(previa["numero"], DOC);
    assert_eq!(previa["motivo"], "Teste ficticio");
    assert_eq!(previa["confirmado"], false);
    assert_eq!(previa["alterado"], false);
    assert_eq!(previa["requisicoes"], 4);
}

#[test]
fn bdd_273_02_cli_um_cancelamento_com_releitura_e_sem_retry() {
    let pasta = Pasta::nova();
    let previa = executar(
        &pasta,
        &["--json", "documento", "cancelar", DOC, "--motivo", "Teste ficticio"],
        4,
    );
    assert_eq!(previa.status.code(), Some(0));
    let confirmacao = executar(
        &pasta,
        &[
            "--json", "documento", "cancelar", DOC, "--motivo", "Teste ficticio",
            "--confirmar",
        ],
        6, // 3 GET de abertura, GET formulário, 1 POST, 1 GET independente.
    );
    assert_eq!(
        confirmacao.status.code(),
        Some(0),
        "stderr={}; stdout={}",
        String::from_utf8_lossy(&confirmacao.stderr),
        String::from_utf8_lossy(&confirmacao.stdout),
    );
    let feito = json(&confirmacao);
    assert_eq!(feito["numero"], DOC);
    assert_eq!(feito["motivo"], "Teste ficticio");
    assert_eq!(feito["confirmado"], true);
    assert_eq!(feito["alterado"], true);
    assert_eq!(feito["reconciliado_por_leitura"], true);
    assert_eq!(feito["requisicoes"], 6);
    assert!(feito.get("falha_do_envio").is_none());

    // Reexecutar a prévia no mesmo SQLite não pode oferecer segundo cancelamento.
    let repetida = executar(
        &pasta,
        &["--json", "documento", "cancelar", DOC, "--motivo", "Novo motivo"],
        3,
    );
    assert_eq!(repetida.status.code(), Some(1));
    let recusa = json(&repetida);
    assert_eq!(recusa["subcodigo"], "documento_ja_cancelado");
}
