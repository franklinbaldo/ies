//! Texto humano de uma tela de recusa do SEI, com `tl` para a árvore e
//! `html_atributos` para `id`/classe (sem regex). Evidência estrutural: as
//! telas reais do SEI testam a exceção por `getElementById('divInfraExcecao')`
//! e o Todos lê `divInfraExcecao`, `infraExcecao` e `infraMensagemErro`.
use crate::html_atributos::{nos_com_id, nos_por_elemento_e_classe};

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
/// texto visível do `<body>`; entidades básicas decodificadas.
pub(crate) fn texto_do_html(html: &str) -> Option<String> {
    let dom = tl::parse(html, tl::ParserOptions::default()).ok()?;
    let parser = dom.parser();
    // A validação da Infra (txaInfraValidacao) vem antes de tudo.
    if let Some(texto) = super::validacao_da_infra::texto_da_validacao(&dom) {
        return Some(texto);
    }
    // Tag ilegível: não se afirma o contêiner; vale o texto geral da tela.
    if let Some(handle) = conteiner_de_erro(&dom) {
        let texto = texto_visivel(handle, parser);
        if !texto.trim().is_empty() {
            return Some(html_escape::decode_html_entities(&texto).into_owned());
        }
    }
    let titulo = primeiro(&dom, "title").map(|h| texto_visivel(h, parser));
    let corpo = if let Some(body) = primeiro(&dom, "body") {
        texto_visivel(body, parser)
    } else {
        // Sem `<body>`: os nós do topo, fora os invisíveis e o `<title>`.
        let mut texto = String::new();
        for handle in dom.children() {
            let invisivel = handle
                .get(parser)
                .and_then(tl::Node::as_tag)
                .is_some_and(|tag| nome_em(tag, &INVISIVEIS));
            if !invisivel {
                texto.push(' ');
                texto.push_str(&texto_visivel(*handle, parser));
            }
        }
        texto
    };
    let partes: Vec<&str> = [titulo.as_deref().unwrap_or(""), corpo.as_str()]
        .into_iter()
        .map(str::trim)
        .filter(|parte| !parte.is_empty())
        .collect();
    if partes.is_empty() {
        return None;
    }
    Some(html_escape::decode_html_entities(&partes.join(" — ")).into_owned())
}

/// A página traz o contêiner de exceção do SEI (`divInfraExcecao`,
/// `div.infraExcecao` ou `div.infraMensagemErro`): é a tela de recusa,
/// não um conteúdo (Refs #60, fatia 3: o HTML recebido de um anexo).
pub(crate) fn e_excecao_do_sei(html: &str) -> bool {
    tl::parse(html, tl::ParserOptions::default()).is_ok_and(|dom| conteiner_de_erro(&dom).is_some())
}

fn conteiner_de_erro(dom: &tl::VDom<'_>) -> Option<tl::NodeHandle> {
    if let Some(handle) = nos_com_id(dom, "divInfraExcecao").ok()?.first() {
        return Some(*handle);
    }
    for classe in CLASSES_DE_ERRO {
        if let Some(handle) = nos_por_elemento_e_classe(dom, None, "div", classe)
            .ok()?
            .first()
        {
            return Some(*handle);
        }
    }
    None
}

fn primeiro(dom: &tl::VDom<'_>, elemento: &str) -> Option<tl::NodeHandle> {
    dom.query_selector(elemento)?.next()
}

fn nome_em(tag: &tl::HTMLTag<'_>, nomes: &[&str]) -> bool {
    let nome = tag.name().as_utf8_str();
    nomes.iter().any(|n| nome.eq_ignore_ascii_case(n))
}

enum Passo {
    No(tl::NodeHandle, usize),
    Espaco,
}

/// Texto dos nós de texto sob `raiz` (que é sempre lida), sem subárvores
/// invisíveis; elementos de bloco separam palavras. Percurso iterativo.
pub(crate) fn texto_visivel(raiz: tl::NodeHandle, parser: &tl::Parser<'_>) -> String {
    let mut saida = String::new();
    let mut pilha = vec![Passo::No(raiz, 0)];
    while let Some(passo) = pilha.pop() {
        let (handle, profundidade) = match passo {
            Passo::Espaco => {
                saida.push(' ');
                continue;
            }
            Passo::No(handle, profundidade) => (handle, profundidade),
        };
        match handle.get(parser) {
            Some(tl::Node::Raw(texto)) => saida.push_str(&texto.as_utf8_str()),
            Some(tl::Node::Tag(tag))
                if profundidade == 0
                    || (profundidade < MAX_PROFUNDIDADE && !nome_em(tag, &INVISIVEIS)) =>
            {
                let bloco = !nome_em(tag, &DE_LINHA);
                if bloco {
                    saida.push(' ');
                    pilha.push(Passo::Espaco);
                }
                let filhos: Vec<tl::NodeHandle> = tag.children().top().iter().copied().collect();
                for filho in filhos.into_iter().rev() {
                    pilha.push(Passo::No(filho, profundidade + 1));
                }
            }
            _ => {}
        }
    }
    saida
}

#[cfg(test)]
#[path = "tests_texto_de_recusa.rs"]
mod tests_texto_de_recusa;
