//! Guarda do defeito do `tl` 0.7.8: fora de `src/html_atributos` e dos
//! testes, nenhum código lê o mapa de atributos do `tl` nem os índices de
//! `id`/classe construídos a partir dele. Busca por substring, sem regex.
use std::fs;
use std::path::{Path, PathBuf};

const PROIBIDOS: [&str; 5] = [
    ".attributes()",
    ".attributes_mut()",
    "get_element_by_id(",
    "get_elements_by_class_name(",
    "is_class_member(",
];

fn arquivos_rs(pasta: &Path, saida: &mut Vec<PathBuf>) {
    for entrada in fs::read_dir(pasta).unwrap() {
        let caminho = entrada.unwrap().path();
        if caminho.is_dir() {
            arquivos_rs(&caminho, saida);
        } else if caminho.extension().is_some_and(|e| e == "rs") {
            saida.push(caminho);
        }
    }
}

fn eh_teste_ou_tokenizador(caminho: &Path) -> bool {
    let nome = caminho.file_name().unwrap().to_string_lossy();
    nome == "tests.rs"
        || nome.starts_with("tests_")
        || caminho
            .components()
            .any(|c| c.as_os_str() == "html_atributos")
        || nome == "html_atributos.rs"
}

/// Código antes do primeiro módulo de testes embutido.
fn codigo_de_producao(fonte: &str) -> &str {
    let fim = ["#[cfg(test)]\nmod ", "#[cfg(test)]\npub(crate) mod "]
        .iter()
        .filter_map(|marca| fonte.find(marca))
        .min()
        .unwrap_or(fonte.len());
    &fonte[..fim]
}

/// Seletor literal com `#` ou `.` depende do índice corrompido do `tl`.
fn seletor_por_id_ou_classe(codigo: &str) -> bool {
    codigo.match_indices("query_selector(").any(|(inicio, _)| {
        let resto = &codigo[inicio..];
        let argumentos = &resto[..resto.find(')').unwrap_or(resto.len())];
        argumentos
            .split('"')
            .skip(1)
            .step_by(2)
            .any(|literal| literal.contains(['#', '.']))
    })
}

#[test]
fn nenhum_codigo_de_producao_le_atributos_pelo_tl() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut arquivos = Vec::new();
    arquivos_rs(&raiz.join("src"), &mut arquivos);
    assert!(arquivos.len() > 10);
    let mut violacoes = Vec::new();
    for caminho in arquivos.iter().filter(|c| !eh_teste_ou_tokenizador(c)) {
        let fonte = fs::read_to_string(caminho).unwrap();
        let codigo = codigo_de_producao(&fonte);
        let caminho = caminho.strip_prefix(raiz).unwrap_or(caminho);
        for proibido in PROIBIDOS {
            if codigo.contains(proibido) {
                violacoes.push(format!("{}: {proibido}", caminho.display()));
            }
        }
        if seletor_por_id_ou_classe(codigo) {
            violacoes.push(format!("{}: seletor por id/classe", caminho.display()));
        }
    }
    assert!(violacoes.is_empty(), "{violacoes:#?}");
}

#[test]
fn guarda_reconhece_as_formas_proibidas() {
    assert!(seletor_por_id_ou_classe(
        "dom.query_selector(\"#tblHistorico\")"
    ));
    assert!(seletor_por_id_ou_classe(
        "t.query_selector(parser, \"a.protocoloNormal\")"
    ));
    assert!(seletor_por_id_ou_classe(
        "dom.query_selector(&format!(\"#{id}\"))"
    ));
    assert!(!seletor_por_id_ou_classe(
        "t.query_selector(parser, \"caption\")"
    ));
    let fonte = "fn a() {}\n#[cfg(test)]\nmod tests { x.attributes() }";
    assert!(!codigo_de_producao(fonte).contains(".attributes()"));
}
