//! BDD-273-01/02 preexistentes: CLI real → TLS local → anti-IES → SQLite.
use super::*;

#[test]
fn bdd_273_01_cli_previa_sem_cancelar_documento() {
    let pasta = Pasta::nova();
    let saida = executar(
        &pasta,
        &[
            "--json",
            "documento",
            "cancelar",
            DOC,
            "--motivo",
            "Teste ficticio",
        ],
        5, // pesquisa, tela, árvore, formulário; nenhum POST.
    );
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stderr={}; stdout={}",
        String::from_utf8_lossy(&saida.stderr),
        String::from_utf8_lossy(&saida.stdout)
    );
    let previa = json(&saida);
    assert_eq!(previa["numero"], DOC);
    assert_eq!(previa["motivo"], "Teste ficticio");
    assert_eq!(previa["confirmado"], false);
    assert_eq!(previa["alterado"], false);
    assert_eq!(previa["requisicoes"], 5);
}

#[test]
fn bdd_273_02_cli_um_cancelamento_com_releitura_e_sem_retry() {
    let pasta = Pasta::nova();
    let confirmacao = executar(
        &pasta,
        &[
            "--json",
            "documento",
            "cancelar",
            DOC,
            "--motivo",
            "Teste ficticio",
            "--confirmar",
        ],
        9, // consulta inicial, formulário, POST único e nova abertura independente.
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
    assert_eq!(feito["requisicoes"], 9);
    assert!(feito.get("falha_do_envio").is_none());
}
