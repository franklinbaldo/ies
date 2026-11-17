//! BDD-61-02: páginas distintas através do binário real e anti-IES.
use super::*;

#[test]
fn bdd_61_02_cli_pagina_2_nao_mistura_faixas_nem_inventa_linhas() {
    let pasta = Pasta::nova();
    let saida = executar_com_estado(
        &pasta,
        &[
            "--json",
            "documento",
            "buscar",
            TERMO_PAGINADO,
            "--pagina",
            "2",
        ],
        EstadoDeBusca::Paginas,
    );
    assert_eq!(
        saida.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let json: Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["resultado"], "lista");
    assert_eq!(json["termo"], TERMO_PAGINADO);
    assert_eq!(json["unidade"], "TESTE");
    assert_eq!(json["origem"], "web");
    assert_eq!(json["requisicoes"], 4);
    // Goiás, pesquisa-paginas-go#3: o total da barra observada é 27.338.
    // As identidades da fixture continuam sintéticas, não são paridade do HTML.
    assert_eq!(json["total_declarado"], 27_338);
    assert_eq!(
        json["faixa_observada"],
        serde_json::json!({"inicio": 11, "fim": 20})
    );
    assert_eq!(json["linhas_ausentes_da_faixa"], 9);
    let documentos = json["documentos"].as_array().unwrap();
    assert_eq!(documentos.len(), 1);
    assert_eq!(documentos[0]["numero_sei"], "0005011");
    assert_eq!(documentos[0]["rotulo"], "Despacho 11");
    assert_eq!(documentos[0]["processo"], "00002.000011/2026-00");
    assert_eq!(documentos[0]["unidade"], "TESTE");
    assert_eq!(documentos[0]["inclusao"], "12/10/2026");
    assert_eq!(documentos[0]["anexo"], false);
    assert!(!saida.stdout.windows(7).any(|w| w == b"0005001"));
}

#[test]
fn bdd_61_02_cli_pagina_1_usa_mesmo_estado_sem_observar_a_segunda() {
    let pasta = Pasta::nova();
    let saida = executar_com_estado(
        &pasta,
        &["--json", "documento", "buscar", TERMO_PAGINADO],
        EstadoDeBusca::Paginas,
    );
    assert_eq!(
        saida.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let json: Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["resultado"], "lista");
    assert_eq!(json["requisicoes"], 3);
    assert_eq!(json["total_declarado"], 27_338);
    assert_eq!(
        json["faixa_observada"],
        serde_json::json!({"inicio": 1, "fim": 10})
    );
    assert_eq!(json["documentos"].as_array().unwrap().len(), 10);
    assert_eq!(json["linhas_ausentes_da_faixa"], 0);
    assert_eq!(json["documentos"][0]["numero_sei"], "0005001");
}
