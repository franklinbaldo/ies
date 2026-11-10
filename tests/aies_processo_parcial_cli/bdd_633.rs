//! BDD-633-02 com a CLI real contra anti-IES e SQLite.
use super::*;

/// BDD-633-02: CLI real, TLS local, modal do anti-IES e SQLite.
/// Caso sintetico, sem corpo da modal observado em cassette.
#[test]
fn bdd_633_02_cli_real_recusa_modal_sem_credenciais_e_sem_post() {
    let (saida, pedidos, (controle, arvore)) = executar(Cena::ValidacaoExigida);
    assert_eq!((controle.as_str(), arvore.as_str()), (PEDIDO, PEDIDO));
    assert_eq!(
        saida.status.code(),
        Some(1),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert!(
        json["erro"]
            .as_str()
            .is_some_and(|e| e.contains("validar o acesso"))
    );
    assert!(
        json["erro"]
            .as_str()
            .is_some_and(|e| e.contains("arquivo de credenciais")),
        "a CLI deve explicar a ausência do arquivo de credenciais: {json}"
    );
    assert!(json.get("documentos").is_none());
    assert_eq!(
        pedidos,
        ["GET procedimento_controlar", "GET procedimento_trabalhar"]
    );
}

/// BDD-633-01: a CLI autentica a abertura de um processo após a modal.
/// A senha é exclusivamente fictícia, lida de arquivo privado 0600; o
/// formulário da modal é hipótese sintética, não replay do SEI GO.
#[test]
fn bdd_633_01_cli_real_valida_senha_e_abre_processo_sem_vazar_segredo() {
    let (saida, pedidos, (controle, arvore)) = executar(Cena::ValidacaoComCredenciais);
    assert_eq!((controle.as_str(), arvore.as_str()), (PEDIDO, PEDIDO));
    assert_eq!(
        saida.status.code(),
        Some(0),
        "status={:?} pedidos={pedidos:?} stdout={} stderr={}",
        saida.status.code(),
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr),
    );
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["numero"], PEDIDO);
    assert_eq!(json["id_procedimento"], "10");
    assert_eq!(
        pedidos,
        [
            "GET procedimento_controlar",
            "GET procedimento_trabalhar",
            "GET usuario_validar_acesso",
            "POST usuario_validar_acesso",
            "GET procedimento_trabalhar",
            "GET procedimento_visualizar",
        ]
    );
    assert!(!String::from_utf8_lossy(&saida.stdout).contains("SENHA_APENAS_SINTETICA"));
    assert!(!String::from_utf8_lossy(&saida.stderr).contains("SENHA_APENAS_SINTETICA"));
}

/// BDD-633-03: a recusa do servidor nao repete a senha nem libera a arvore.
#[test]
fn bdd_633_03_cli_real_recusa_validacao_sem_repetir_post() {
    let (saida, pedidos, (controle, arvore)) = executar(Cena::ValidacaoRecusada);
    assert_eq!((controle.as_str(), arvore.as_str()), (PEDIDO, PEDIDO));
    assert_eq!(
        saida.status.code(),
        Some(1),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr),
    );
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert!(
        json["erro"]
            .as_str()
            .is_some_and(|mensagem| mensagem.contains("recusou a validação")),
        "mensagem de recusa: {json}"
    );
    assert_eq!(
        pedidos,
        [
            "GET procedimento_controlar",
            "GET procedimento_trabalhar",
            "GET usuario_validar_acesso",
            "POST usuario_validar_acesso",
        ]
    );
    assert!(json.get("documentos").is_none());
    assert!(!String::from_utf8_lossy(&saida.stdout).contains("SENHA_APENAS_SINTETICA"));
    assert!(!String::from_utf8_lossy(&saida.stderr).contains("SENHA_APENAS_SINTETICA"));
}

/// BDD-633-01: POST com 302 não prova sucesso nem autoriza repetição.
/// A confirmação vem do GET independente do trabalho e da árvore.
/// 302 após validação é hipótese sintética, ausente das cassettes GO.
#[test]
fn bdd_633_01_cli_real_aceita_302_apenas_apos_releitura_do_processo() {
    let (saida, pedidos, (controle, arvore)) = executar(Cena::ValidacaoRedirecionada);
    assert_eq!((controle.as_str(), arvore.as_str()), (PEDIDO, PEDIDO));
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr),
    );
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["numero"], PEDIDO);
    assert_eq!(json["id_procedimento"], "10");
    assert_eq!(
        pedidos,
        [
            "GET procedimento_controlar",
            "GET procedimento_trabalhar",
            "GET usuario_validar_acesso",
            "POST usuario_validar_acesso",
            "GET procedimento_trabalhar",
            "GET procedimento_visualizar",
        ]
    );
    assert!(!String::from_utf8_lossy(&saida.stdout).contains("SENHA_APENAS_SINTETICA"));
    assert!(!String::from_utf8_lossy(&saida.stderr).contains("SENHA_APENAS_SINTETICA"));
}
