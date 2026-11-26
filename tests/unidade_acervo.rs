//! Contrato da CLI de acervo histórico (#878), sem sessão ou rede.
use std::process::{Command, Output};

fn ies(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(args)
        .output()
        .expect("binário ies compilado no teste")
}

#[test]
fn ajuda_publica_acervo_e_data_obrigatoria() {
    let ajuda = ies(&["unidade", "acervo", "--help"]);
    assert!(ajuda.status.success());
    let texto = String::from_utf8(ajuda.stdout).unwrap();
    assert!(texto.contains("--em"), "{texto}");
    let ausente = ies(&["--json", "unidade", "acervo"]);
    assert_eq!(ausente.status.code(), Some(2));
    assert!(ausente.stdout.is_empty());
    assert!(String::from_utf8_lossy(&ausente.stderr).contains("--em"));
}

#[test]
fn datas_civis_invalidas_sao_erros_de_uso() {
    for data in [
        "2026-02-29",
        "2026-04-31",
        "2026-13-01",
        "2026-11-00",
        "20/11/2026",
        "2026-1-01",
        "2026-01-1",
        "0000-01-01",
    ] {
        let saida = ies(&["--json", "unidade", "acervo", "--em", data]);
        assert_eq!(saida.status.code(), Some(2), "{data}");
        assert!(saida.stdout.is_empty(), "{data}");
        let erro = String::from_utf8_lossy(&saida.stderr);
        assert!(erro.contains("--em"), "{data}: {erro}");
        assert!(erro.contains(data), "{data}: {erro}");
    }
}

// A data válida atravessa o parser. Enquanto o fluxo Web não existe, a
// CLI falha explicitamente em vez de publicar uma lista vazia.
#[test]
fn data_valida_alcanca_o_recorte_wip_sem_saida_de_acervo() {
    let saida = ies(&["--json", "unidade", "acervo", "--em", "2024-02-29"]);
    assert_eq!(saida.status.code(), Some(1));
    assert!(saida.stdout.is_empty());
    let erro = String::from_utf8_lossy(&saida.stderr);
    assert!(erro.contains("ComandoSemImplementacaoNoSei"), "{erro}");
    assert!(erro.contains("29/02/2024"), "{erro}");
}
