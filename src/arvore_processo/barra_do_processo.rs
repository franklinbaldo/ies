//! O link das propriedades na barra do nó `PROCESSO` (Refs #59;
//! contracts/cli/processo-obter-propriedades.md). Forma observada na `UnB`
//! 4.0.9 (03 e 04/11/2026): a linha `Nos[i].acoes = '<literal>';` do nó
//! raiz, com `<a href="controlador.php?acao=procedimento_alterar&…">` (ou
//! `procedimento_consultar`, com o processo aberto só noutra unidade). As
//! tags são lidas pelo tokenizador de atributos; o ícone e o `title` não
//! classificam nada.
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use super::literais::{Literal, indice_da_barra, ler_barra, ler_declaracao};
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::atributos;
use crate::instalacao::Instalacao;
use crate::propriedades_processo::{
    EnderecoDasPropriedades, ErroPropriedades as E, FonteDasPropriedades,
};

/// O link da tela de propriedades publicado na barra do processo `id`;
/// `Ok(None)` se a barra não publica `procedimento_alterar` nem
/// `procedimento_consultar`. Os dois juntos: vale o de alterar.
pub(crate) fn ler_link_das_propriedades(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDasPropriedades>, E> {
    let Some(texto) = barra_da_raiz(html)? else {
        return Ok(None);
    };
    let fora = E::BarraDoProcessoForaDaFormaObservada;
    let dom = tl::parse(&texto, tl::ParserOptions::default()).map_err(|_| fora)?;
    let parser = dom.parser();
    let (mut alterar, mut consultar) = (Vec::new(), Vec::new());
    for no in dom.query_selector("a").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let lidos = atributos(tag).map_err(|_| fora)?;
        let Some(bruto) = lidos.valor("href").map_err(|_| fora)? else {
            continue;
        };
        let href = decodificar_atributo(bruto).ok_or(fora)?;
        match acao_do_href(&href) {
            Some(acao) if acao == FonteDasPropriedades::Alterar.acao() => alterar.push(href),
            Some(acao) if acao == FonteDasPropriedades::Consultar.acao() => consultar.push(href),
            _ => {}
        }
    }
    let (fonte, achados) = if alterar.is_empty() {
        (FonteDasPropriedades::Consultar, consultar)
    } else {
        (FonteDasPropriedades::Alterar, alterar)
    };
    let href = match achados.as_slice() {
        [] => return Ok(None),
        [href] => href,
        _ => return Err(E::LinksDasPropriedadesRepetidosNaBarra),
    };
    conferir_endereco_relativo(instalacao, href, fonte.acao(), id)
        .map(|url| Some(EnderecoDasPropriedades::novo(fonte, url)))
        .map_err(|recusa| match recusa {
            EnderecoRecusado::NaoReconhecido => E::EnderecoDasPropriedadesNaoReconhecido,
            EnderecoRecusado::DeOutroProcesso => E::EnderecoDasPropriedadesDeOutroProcesso,
        })
}

/// O primeiro `acao=` da consulta do `href` (a conferência recusa dois).
fn acao_do_href(href: &str) -> Option<&str> {
    let (_, consulta) = href.split_once('?')?;
    consulta
        .split('&')
        .find_map(|par| par.strip_prefix("acao="))
}

/// O texto da barra do nó `PROCESSO`; duas barras para ele são recusa.
fn barra_da_raiz(html: &str) -> Result<Option<String>, E> {
    let fora = E::BarraDoProcessoForaDaFormaObservada;
    let mut raiz = None;
    for linha in html.split('\n') {
        if let Some((i, argumentos)) = ler_declaracao(linha).map_err(|_| fora)?
            && argumentos.first() == Some(&Literal::Texto("PROCESSO".into()))
        {
            raiz = Some(i);
            break;
        }
    }
    let Some(raiz) = raiz else {
        return Ok(None);
    };
    let mut texto = None;
    for linha in html.split('\n') {
        if indice_da_barra(linha) != Some(raiz) {
            continue;
        }
        let Some((_, lido)) = ler_barra(linha).map_err(|_| fora)? else {
            return Err(fora);
        };
        if texto.is_some() {
            return Err(E::BarrasDoProcessoRepetidasNaArvore);
        }
        texto = Some(lido);
    }
    Ok(texto)
}
