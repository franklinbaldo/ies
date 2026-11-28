//! Inventário de candidatos no texto extraído; não resolve nem reescreve HTML.
use crate::tela_html::{HtmlServido, No, fragmento};
use crate::{Cli, DocumentoAcao, Domain};
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Tipo {
    Documento,
    Processo,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct Candidato<'a> {
    tipo: Tipo,
    texto: String,
    ocorrencias: usize,
    estado: &'a str,
}

fn separador(segmentos: &mut Vec<String>) {
    if segmentos.last().is_some_and(|s| !s.is_empty()) {
        segmentos.push(String::new());
    }
}

fn segmentos(html: &HtmlServido) -> Vec<String> {
    enum Visita<'a> {
        No(No<'a>),
        Limite,
    }
    let documento = fragmento(html);
    let mut segmentos = vec![String::new()];
    let mut pilha: Vec<_> = documento.nos().map(Visita::No).collect();
    pilha.reverse();
    while let Some(visita) = pilha.pop() {
        match visita {
            Visita::Limite => separador(&mut segmentos),
            Visita::No(No::Texto(texto)) => segmentos.last_mut().unwrap().push_str(texto),
            Visita::No(No::Comentario) => {}
            Visita::No(No::Elemento(e)) => {
                if e.presente("hidden")
                    || matches!(
                        e.nome(),
                        "a" | "script" | "style" | "head" | "title" | "template"
                            | "textarea" | "select" | "option" | "optgroup" | "input"
                            | "button" | "datalist" | "output" | "noscript"
                    )
                {
                    separador(&mut segmentos);
                    continue;
                }
                if matches!(
                    e.nome(),
                    "p" | "div" | "li" | "td" | "th" | "tr" | "table"
                        | "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
                        | "blockquote" | "pre" | "br" | "hr" | "section" | "article"
                ) {
                    separador(&mut segmentos);
                    pilha.push(Visita::Limite);
                }
                let filhos: Vec<_> = e.nos().map(Visita::No).collect();
                pilha.extend(filhos.into_iter().rev());
            }
        }
    }
    segmentos
}

fn inventariar(html: &HtmlServido, proprio: &str, processo: &str) -> Vec<Candidato<'static>> {
    let numeros = Regex::new(
        r"[0-9]{4,5}\.[0-9]{6}/[0-9]{4}-[0-9]{2}|00[0-9]{8}|[0-9]{8}",
    )
    .expect("expressão literal");
    let anterior = Regex::new(r"[\p{L}\p{N}\p{M}_./#&-]$").expect("expressão literal");
    let posterior =
        Regex::new(r"^(?:[\p{L}\p{N}\p{M}_/-]|\.\p{Nd})").expect("expressão literal");
    let contexto = Regex::new(
        r"(?i)\b(?:ids?|PJe|guias?|OB|CPF|RG|CEP|CNPJ|OAB|NIT|PIS|PASEP|matr[íi]cula|mat|conta|ag[êe]ncia|banco|empenho|NE)\s*[.:]?\s*(?:n(?:[º°o]|\.?º)?\.?\s*)?(?:[0-9][0-9.\-/]*\s*(?:,|;|/|\be\b|\ba\b)\s*)*[0-9][0-9.\-/]*",
    )
    .expect("expressão literal");
    let mut indices = HashMap::new();
    let mut candidatos: Vec<Candidato<'static>> = Vec::new();
    for segmento in segmentos(html) {
        let mut contextos = contexto.find_iter(&segmento).peekable();
        for numero in numeros.find_iter(&segmento) {
            let texto = numero.as_str();
            while contextos.peek().is_some_and(|c| c.end() <= numero.start()) {
                contextos.next();
            }
            if texto == proprio
                || texto == processo
                || anterior.is_match(&segmento[..numero.start()])
                || posterior.is_match(&segmento[numero.end()..])
                || contextos.peek().is_some_and(|c| {
                    c.start() <= numero.start() && numero.end() <= c.end()
                })
            {
                continue;
            }
            let tipo = if texto.contains('/') {
                Tipo::Processo
            } else {
                Tipo::Documento
            };
            let chave = (tipo, texto.to_owned());
            if let Some(&i) = indices.get(&chave) {
                let candidato: &mut Candidato<'_> = &mut candidatos[i];
                candidato.ocorrencias += 1;
            } else {
                indices.insert(chave, candidatos.len());
                candidatos.push(Candidato {
                    tipo,
                    texto: texto.to_owned(),
                    ocorrencias: 1,
                    estado: "candidato",
                });
            }
        }
    }
    candidatos
}

pub(crate) fn tentar_inventariar(cli: &Cli) -> Option<anyhow::Result<()>> {
    let Domain::Documento(doc) = &cli.command else {
        return None;
    };
    let Some(DocumentoAcao::Referencias { arquivo, proprio, nup }) = &doc.acao else {
        return None;
    };
    if cli.unidade.is_some()
        || !doc.numeros.is_empty()
        || doc.processo.is_some()
        || doc.versao.is_some()
        || doc.html_privado.is_some()
        || doc.saida.is_some()
        || doc.sem_cache
        || doc.ocultar_inacessiveis
        || doc.com_propriedades
    {
        crate::uso_invalido::recusar(
            crate::uso_invalido::UsoInvalido::OpcoesDeDocumentoSemEvidencia,
        );
    }
    Some((|| {
        let html = HtmlServido::novo(crate::arquivo_privado::ler(
            arquivo,
            crate::MAX_HTML_BYTES,
        )?)?;
        let candidatos = inventariar(
            &html,
            proprio.as_deref().unwrap_or(""),
            nup.as_deref().unwrap_or(""),
        );
        if cli.json {
            println!("{}", serde_json::json!({
                "escopo": "texto_extraido_offline",
                "candidatos": candidatos
            }));
        } else {
            println!("Candidatos no texto extraído (sem resolução SEI):");
            for c in candidatos {
                println!("{:?} {} ({} ocorrências)", c.tipo, c.texto, c.ocorrencias);
            }
        }
        Ok(())
    })())
}

#[cfg(test)]
mod testes;
