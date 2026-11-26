//! #876: hipótese sintética de prévia, não replay do POST original.
use super::{Command, executar, json};

#[test]
fn criar_bloco_previa_nao_grava_nem_adivinha_identidade() {
    // RED de gramática antes do TLS; não mascarar a lacuna com timeout.
    let ajuda = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["bloco", "criar", "--help"])
        .output()
        .unwrap();
    assert!(
        ajuda.status.success(),
        "gramática bloco criar ausente: {}",
        String::from_utf8_lossy(&ajuda.stderr)
    );
    // A descrição já existe no SQLite. A prévia não pode adotar seu ID.
    let (saida, pedidos) = executar(false, None, true);
    assert!(pedidos.iter().all(|p| p.starts_with("GET ")));
    let valor = json(&saida);
    assert_eq!(valor["unidade"], "TESTE");
    assert_eq!(valor["tipo_bloco"], "assinatura");
    assert_eq!(valor["descricao"], "Despachos de teste");
    assert_eq!(valor["grupo_padrao"], "null");
    assert_eq!(valor["confirmado"], false);
    assert_eq!(valor["alterado"], false);
    assert!(valor["bloco"].is_null());
    assert!(!valor.to_string().contains("infra_hash"));
    assert!(!valor.to_string().contains("PHPSESSID"));
}
