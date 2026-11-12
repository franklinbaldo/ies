//! O parser HTML (html5ever) só é chamado sob `src/tela_html` (#751, #676):
//! trocá-lo mexe num ponto só. Busca por substring, sem regex.
use std::fs;
use std::path::{Path, PathBuf};

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

/// Código antes do primeiro módulo de testes embutido.
fn codigo_de_producao(fonte: &str) -> &str {
    let fim = ["#[cfg(test)]\nmod ", "#[cfg(test)]\npub(crate) mod "]
        .iter()
        .filter_map(|marca| fonte.find(marca))
        .min()
        .unwrap_or(fonte.len());
    &fonte[..fim]
}

/// `src/tela_html.rs` e o módulo da árvore, tudo sob `src/tela_html/`.
fn e_tela_html(caminho: &Path) -> bool {
    caminho.ends_with("src/tela_html.rs")
        || caminho
            .ancestors()
            .any(|pasta| pasta.ends_with("src/tela_html"))
}

#[test]
fn so_tela_html_chama_o_parser() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut arquivos = Vec::new();
    arquivos_rs(&raiz.join("src"), &mut arquivos);
    let violacoes: Vec<_> = arquivos
        .iter()
        .filter(|c| !e_tela_html(c))
        .filter(|c| codigo_de_producao(&fs::read_to_string(c).unwrap()).contains("html5ever::"))
        .map(|c| c.strip_prefix(raiz).unwrap_or(c).display().to_string())
        .collect();
    assert!(violacoes.is_empty(), "{violacoes:#?}");
}

#[test]
fn guarda_ignora_o_modulo_de_testes_embutido() {
    let fonte = "fn a() {}\n#[cfg(test)]\nmod tests { html5ever::x() }";
    assert!(!codigo_de_producao(fonte).contains("html5ever::"));
}
