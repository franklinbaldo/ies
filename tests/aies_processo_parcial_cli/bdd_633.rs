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


/// BDD-633-01: a CLI valida uma única vez, relê e publica a identidade.

/// A modal e o POST são hipótese local, não observação de cassette.
#[test]
fn bdd_633_01_cli_real_valida_acesso_privado_sem_vazar_senha() {
    let (saida, pedidos, (controle, arvore)) = executar(Cena::ValidacaoAutorizada);
    assert_eq!((controle.as_str(), arvore.as_str()), (PEDIDO, PEDIDO));
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr)
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
    assert!(!String::from_utf8_lossy(&saida.stdout).contains("senha="));
}
