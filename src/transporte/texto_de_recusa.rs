//! Texto humano de uma tela de recusa do SEI, lido na árvore html5ever
//! (`tela_html`), sem regex. Evidência estrutural: as telas reais do SEI
//! testam a exceção por `getElementById('divInfraExcecao')` e o Todos lê
//! `divInfraExcecao`, `infraExcecao` e `infraMensagemErro`.
use crate::tela_html::{Documento, Elemento, No};

/// Classes de erro do SEI, na ordem de preferência, quando falta o `id`.
const CLASSES_DE_ERRO: [&str; 2] = ["infraExcecao", "infraMensagemErro"];

/// Subárvores que não são texto visível do corpo (`title` é lido à parte).
const INVISIVEIS: [&str; 7] = [
    "script", "style", "noscript", "template", "head", "svg", "title",
];

/// Elementos de linha: não separam palavras.
const DE_LINHA: [&str; 13] = [
    "a", "abbr", "b", "code", "em", "font", "i", "label", "small", "span", "strong", "sub", "u",
];

/// Profundidade além da qual a árvore não é percorrida (a resposta já
/// está limitada a 64 KiB).
const MAX_PROFUNDIDADE: usize = 256;

/// Texto bruto (ainda não saneado) da exceção do SEI, ou `<title>` e o
/// texto visível do `<body>`; entidades decodificadas pela árvore.
pub(crate) fn texto_do_html(html: &crate::tela_html::HtmlServido) -> Option<String> {
    let dom = crate::tela_html::arvore(html);
    // A validação da Infra (txaInfraValidacao) vem antes de tudo.
    if let Some(texto) = super::validacao_da_infra::texto_da_validacao(&dom) {
        return Some(texto);
    }
    if let Some(conteiner) = conteiner_de_erro(&dom) {
        let texto = texto_visivel(conteiner);
        if !texto.trim().is_empty() {
            return Some(texto);
        }
    }
    // A árvore tem `<body>` mesmo sem a tag (o html5ever o implica); o que
    // vai ao `<head>` (script, style, title) não é texto do corpo.
    let titulo = primeiro(&dom, "title").map(texto_visivel);
    let corpo = primeiro(&dom, "body").map(texto_visivel);
    let partes: Vec<&str> = [titulo.as_deref(), corpo.as_deref()]
        .into_iter()
        .flatten()
        .map(str::trim)
        .filter(|parte| !parte.is_empty())
        .collect();
    if partes.is_empty() {
        return None;
    }
    Some(partes.join(" — "))
}

/// A página traz o contêiner de exceção do SEI (`divInfraExcecao`,
/// `div.infraExcecao` ou `div.infraMensagemErro`): é a tela de recusa,
/// não um conteúdo (Refs #60, fatia 3: o HTML recebido de um anexo).
pub(crate) fn e_excecao_do_sei(html: &crate::tela_html::HtmlServido) -> bool {
    let dom = crate::tela_html::arvore(html);
    conteiner_de_erro(&dom).is_some()
}

fn conteiner_de_erro(dom: &Documento) -> Option<Elemento<'_>> {
    dom.com_id("divInfraExcecao").next().or_else(|| {
        CLASSES_DE_ERRO
            .iter()
            .find_map(|classe| dom.elementos("div").find(|div| div.tem_classe(classe)))
    })
}

fn primeiro<'d>(dom: &'d Documento, elemento: &str) -> Option<Elemento<'d>> {
    dom.elementos(elemento).next()
}

fn nome_em(tag: Elemento<'_>, nomes: &[&str]) -> bool {
    let nome = tag.nome();
    nomes.iter().any(|n| nome.eq_ignore_ascii_case(n))
}

enum Passo<'d> {
    No(No<'d>, usize),
    Espaco,
}

/// Texto dos nós de texto sob `raiz` (que é sempre lida), sem subárvores
/// invisíveis; elementos de bloco separam palavras. Percurso iterativo.
/// O texto já vem decodificado pela árvore.
pub(crate) fn texto_visivel(raiz: Elemento<'_>) -> String {
    let mut saida = String::new();
    let mut pilha = vec![Passo::No(No::Elemento(raiz), 0)];
    while let Some(passo) = pilha.pop() {
        let (no, profundidade) = match passo {
            Passo::Espaco => {
                saida.push(' ');
                continue;
            }
            Passo::No(no, profundidade) => (no, profundidade),
        };
        match no {
            No::Texto(texto) => saida.push_str(texto),
            No::Elemento(tag)
                if profundidade == 0
                    || (profundidade < MAX_PROFUNDIDADE && !nome_em(tag, &INVISIVEIS)) =>
            {
                let bloco = !nome_em(tag, &DE_LINHA);
                if bloco {
                    saida.push(' ');
                    pilha.push(Passo::Espaco);
                }
                let filhos: Vec<No<'_>> = tag.nos().collect();
                for filho in filhos.into_iter().rev() {
                    pilha.push(Passo::No(filho, profundidade + 1));
                }
            }
            No::Elemento(_) | No::Comentario => {}
        }
    }
    saida
}

#[cfg(test)]
#[path = "tests_texto_de_recusa.rs"]
mod tests_texto_de_recusa;
