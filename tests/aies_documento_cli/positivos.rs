use super::*;

// BDD-88-01: JSON sem --saida tem de trazer a revisão N solicitada,
// nunca a última, mesmo quando o histórico anuncia uma versão mais recente.
#[test]
fn bdd_88_01_cli_json_ler_versao_historica_sem_criar_foto() {
    let pasta = Pasta::nova();
    let saida = executar(
        &pasta,
        &["--json", "documento", DOC, "--versao", "1", "--sem-cache"],
        6,
    );
    assert_eq!(
        saida.status.code(),
        Some(0),
        "CLI: {}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let valor = json(&saida);
    let documento = &valor["documentos"][0];
    assert_eq!(documento["numero"], DOC);
    assert_eq!(documento["versao"], 1);
    assert_eq!(documento["versao_atual"], 3);
    assert_eq!(valor["requisicoes"], 6);
    let texto = documento["texto"].as_str().expect("texto da versão solicitada");
    assert!(texto.contains("Primeira redação"), "versão 1 ausente");
    assert!(!texto.contains("Terceira redação"), "retornou versão atual");
    assert!(documento.get("arquivo").is_none(), "não pediu --saida");
    assert!(!pasta.0.join("0011519.v1.html").exists());
}
