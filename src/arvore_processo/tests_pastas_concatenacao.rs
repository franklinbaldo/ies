//! RED/GREEN da barra concatenada (Refs #59, replay do acervo
//! docs/validacao-replay/2026-11-04-processo-obter-acervo.md): na UFRJ e em
//! GO cada documento traz, depois de `.acoes`, `.src` e `.html`, a linha
//! `Nos[i].acoes = Nos[i].acoes.concat('');` (1.104 vezes no acervo, sempre
//! com o literal vazio). Num documento de pasta a árvore inteira era
//! recusada (`barra_de_no_fora_da_forma_observada`). HTML sintético.
use super::tests::NUP;
use super::tests_pastas::{ID, arvore_fechada};
use super::*;
use crate::erro_html::ErroHtml;
use std::fmt::Write as _;

/// A linha observada logo depois do `.html` de cada documento.
fn com_concatenacao(html: &str, literal: &str) -> String {
    let mut saida = String::new();
    for linha in html.split('\n') {
        saida.push_str(linha);
        saida.push('\n');
        let indice = linha
            .strip_prefix("Nos[")
            .and_then(|resto| resto.strip_suffix("].html = '';"));
        if let Some(i) = indice {
            let _ = writeln!(
                saida,
                "Nos[{i}].acoes = Nos[{i}].acoes.concat('{literal}');"
            );
        }
    }
    saida
}

#[test]
fn concatenacao_vazia_da_barra_nao_muda_a_leitura_das_pastas() {
    let sem = ler_arvore(&arvore_fechada(), NUP, ID).unwrap();
    let html = com_concatenacao(&arvore_fechada(), "");
    assert!(html.contains("Nos[4].acoes = Nos[4].acoes.concat('');"));
    assert_eq!(ler_arvore(&html, NUP, ID), Ok(sem));
}

#[test]
fn concatenacao_com_conteudo_continua_fora_da_forma_observada() {
    let html = com_concatenacao(&arvore_fechada(), "<a href=\"#\"></a>");
    assert_eq!(
        ler_arvore(&html, NUP, ID),
        Err(super::ErroArvore::Html(ErroHtml::valor(
            "barra_de_no_fora_da_forma_observada"
        )))
    );
}
