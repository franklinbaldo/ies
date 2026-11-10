//! Vínculo do processo aberto com o pai, quando ele mesmo é um anexado
//! (Refs #59; contracts/cli/processo-obter-arvore.md). Forma observada na
//! `UnB` 4.0.9 (03/11/2026): o `.html` do nó raiz é um literal JS com um
//! único `<a>` cujo `href` é o `procedimento_trabalhar` assinado do pai
//! (`id_procedimento` do pai, `id_procedimento_anexado` igual à raiz) e
//! cujo texto é o NUP do pai. Sem regex; a frase em volta e o ícone não
//! classificam nada.
use super::endereco::conferir_endereco_relativo;
use super::literais::{Literal, indice_do_html, ler_declaracao, ler_html};
use super::{EnderecoDaArvore, ErroArvore, VinculoDeAnexacao};
use crate::html_atributos::atributos;
use crate::instalacao::Instalacao;

const ACAO: &str = "procedimento_trabalhar";
const CHAVE_DO_ANEXADO: &str = "id_procedimento_anexado";

/// O pai do processo `id`, se o `.html` da raiz publica o vínculo.
pub(crate) fn ler_vinculo_de_anexacao(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<VinculoDeAnexacao>, ErroArvore> {
    let Some(texto) = html_da_raiz(html)? else {
        return Ok(None);
    };
    let fora = ErroArvore::HtmlDoNoDoProcessoForaDaFormaObservada;
    let dom = tl::parse(&texto, tl::ParserOptions::default()).map_err(|_| fora)?;
    let parser = dom.parser();
    let mut achado = None;
    for no in dom.query_selector("a").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let lidos = atributos(tag).map_err(ErroArvore::Atributo)?;
        let Some(href) = lidos.valor("href").map_err(ErroArvore::Atributo)? else {
            continue;
        };
        if !pares(href).any(|(chave, _)| chave == CHAVE_DO_ANEXADO) {
            continue;
        }
        if achado.is_some() {
            return Err(ErroArvore::VinculosDeAnexacaoRepetidos);
        }
        achado = Some(conferir(instalacao, href, &texto_unico(tag, parser), id)?);
    }
    Ok(achado)
}

/// O texto do link quando ele é o único filho (nó de texto cru); senão
/// vazio, que a conferência recusa.
pub(super) fn texto_unico(tag: &tl::HTMLTag<'_>, parser: &tl::Parser<'_>) -> String {
    let filhos = tag.children();
    match filhos.top().as_slice() {
        [filho] => match filho.get(parser) {
            Some(tl::Node::Raw(bytes)) => bytes.as_utf8_str().into_owned(),
            _ => String::new(),
        },
        _ => String::new(),
    }
}

/// O texto do `.html` do nó `PROCESSO`; dois `.html` para ele são recusa.
pub(super) fn html_da_raiz(html: &str) -> Result<Option<String>, ErroArvore> {
    let mut raiz = None;
    let mut texto = None;
    for linha in html.split('\n') {
        if raiz.is_none()
            && let Some((i, argumentos)) = ler_declaracao(linha)?
            && argumentos.first() == Some(&Literal::Texto("PROCESSO".into()))
        {
            raiz = Some(i);
        }
        // Só o `.html` da raiz é lido (e tem de estar na forma observada).
        if raiz.is_some() && indice_do_html(linha) == raiz {
            let Some((_, lido)) = ler_html(linha)? else {
                continue;
            };
            if texto.is_some() {
                return Err(ErroArvore::VinculosDeAnexacaoRepetidos);
            }
            texto = Some(lido);
        }
    }
    Ok(texto)
}

/// Os pares `chave=valor` da consulta de `href` (sem `?`, nenhum).
pub(super) fn pares(href: &str) -> impl Iterator<Item = (&str, &str)> {
    href.split_once('?')
        .map_or("", |(_, consulta)| consulta)
        .split('&')
        .filter(|par| !par.is_empty())
        .map(|par| par.split_once('=').unwrap_or((par, "")))
}

fn conferir(
    instalacao: &Instalacao,
    href: &str,
    texto: &str,
    id: &str,
) -> Result<VinculoDeAnexacao, ErroArvore> {
    let fora = ErroArvore::VinculoDeAnexacaoForaDaFormaObservada;
    let valores = |nome: &str| -> Vec<&str> {
        pares(href)
            .filter(|(chave, _)| *chave == nome)
            .map(|(_, valor)| valor)
            .collect()
    };
    let (anexados, pais) = (valores(CHAVE_DO_ANEXADO), valores("id_procedimento"));
    let ([anexado], [pai]) = (anexados.as_slice(), pais.as_slice()) else {
        return Err(fora);
    };
    if *anexado != id || *pai == id || !super::decimal(pai) {
        return Err(fora);
    }
    let absoluta = conferir_endereco_relativo(instalacao, href, ACAO, pai).map_err(|_| fora)?;
    if texto.is_empty() || texto.trim() != texto || texto.contains(['<', '>', '&']) {
        return Err(fora);
    }
    Ok(VinculoDeAnexacao {
        numero: texto.to_owned(),
        id_procedimento: (*pai).to_owned(),
        trabalhar: EnderecoDaArvore::conferido(absoluta),
    })
}
