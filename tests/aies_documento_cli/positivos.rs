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
    let texto = documento["texto"]
        .as_str()
        .expect("texto da versão solicitada");
    assert!(texto.contains("Primeira redação"), "versão 1 ausente");
    assert!(!texto.contains("Terceira redação"), "retornou versão atual");
    assert!(documento.get("arquivo").is_none(), "não pediu --saida");
    assert!(!pasta.0.join("0011519.v1.html").exists());
}

// BDD-88-02: uma foto editada, mas ainda pertencente ao mesmo documento
// e à mesma versão, deve ser comparada com o HTML remoto. Não basta
// rejeitar um arquivo local sem título/rodapé antes de usar o anti-IES.
#[test]
fn bdd_88_02_cli_preserva_foto_local_coerente_mas_editada() {
    let pasta = Pasta::nova();
    let destino = pasta.0.join("saida");
    fs::create_dir(&destino).unwrap();
    let args = [
        "--json",
        "documento",
        DOC,
        "--versao",
        "1",
        "--saida",
        destino.to_str().unwrap(),
        "--sem-cache",
    ];

    let primeira = executar(&pasta, &args, 6);
    assert_eq!(primeira.status.code(), Some(0));
    let foto = destino.join("0011519.v1.html");
    let html_servido = fs::read_to_string(&foto).unwrap();
    let html_editado = html_servido.replacen(
        "<p>Primeira redação</p>",
        "<p>Primeira redação editada localmente</p>",
        1,
    );
    assert_ne!(html_editado, html_servido, "fixture não contém o corpo esperado");
    fs::write(&foto, &html_editado).unwrap();

    // O título e o rodapé permanecem intactos: a divergência só fica
    // provada ao confrontar o conteúdo com a versão no anti-IES.
    let segunda = executar(&pasta, &args, 6);
    assert_eq!(segunda.status.code(), Some(1));
    assert_eq!(json(&segunda)["subcodigo"], "foto_local_divergente");
    assert_eq!(fs::read_to_string(&foto).unwrap(), html_editado);
}
