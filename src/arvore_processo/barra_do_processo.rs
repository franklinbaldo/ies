//! Os links das propriedades e dos marcadores na barra do nó `PROCESSO` (Refs #59;
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
use crate::marcadores_processo::{self, EnderecoDosMarcadores, ErroMarcadores};
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
    let falha = |f: FalhaDaBarra| match f {
        FalhaDaBarra::Fora => E::BarraDoProcessoForaDaFormaObservada,
        FalhaDaBarra::Repetidas => E::BarrasDoProcessoRepetidasNaArvore,
    };
    let (mut alterar, mut consultar) = (Vec::new(), Vec::new());
    for href in hrefs_da_barra(html).map_err(falha)? {
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

/// O link da tela "Marcadores do Processo" (`andamento_marcador_gerenciar`)
/// publicado na barra do processo `id`; `Ok(None)` se a barra não o publica
/// (contracts/cli/processo-obter-marcadores.md).
pub(crate) fn ler_link_dos_marcadores(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDosMarcadores>, ErroMarcadores> {
    let hrefs = hrefs_da_barra(html).map_err(|f| match f {
        FalhaDaBarra::Fora => ErroMarcadores::BarraDoProcessoForaDaFormaObservada,
        FalhaDaBarra::Repetidas => ErroMarcadores::BarrasDoProcessoRepetidasNaArvore,
    })?;
    let achados: Vec<_> = hrefs
        .iter()
        .filter(|href| acao_do_href(href) == Some(marcadores_processo::ACAO))
        .collect();
    let href = match achados.as_slice() {
        [] => return Ok(None),
        [href] => href,
        _ => return Err(ErroMarcadores::LinksDosMarcadoresRepetidosNaBarra),
    };
    conferir_endereco_relativo(instalacao, href, marcadores_processo::ACAO, id)
        .map(|url| Some(EnderecoDosMarcadores::novo(url)))
        .map_err(|recusa| match recusa {
            EnderecoRecusado::NaoReconhecido => ErroMarcadores::EnderecoDosMarcadoresNaoReconhecido,
            EnderecoRecusado::DeOutroProcesso => {
                ErroMarcadores::EnderecoDosMarcadoresDeOutroProcesso
            }
        })
}

/// Barra fora da forma observada, ou duas barras para o nó `PROCESSO`.
pub(super) enum FalhaDaBarra {
    Fora,
    Repetidas,
}

/// Os `href` (entidades decodificadas) das tags `<a>` da barra do nó
/// `PROCESSO`, na ordem servida; sem barra, nenhum.
fn hrefs_da_barra(html: &str) -> Result<Vec<String>, FalhaDaBarra> {
    Ok(ancoras_da_barra(html)?
        .into_iter()
        .filter_map(|a| a.href)
        .collect())
}

/// Uma `<a>` da barra: `href` e `onclick`, entidades decodificadas.
pub(super) struct AncoraDaBarra {
    pub(super) href: Option<String>,
    pub(super) onclick: Option<String>,
}

/// As tags `<a>` da barra do nó `PROCESSO`, na ordem servida; sem barra,
/// nenhuma.
pub(super) fn ancoras_da_barra(html: &str) -> Result<Vec<AncoraDaBarra>, FalhaDaBarra> {
    let Some(texto) = barra_da_raiz(html)? else {
        return Ok(Vec::new());
    };
    let dom = tl::parse(&texto, tl::ParserOptions::default()).map_err(|_| FalhaDaBarra::Fora)?;
    let parser = dom.parser();
    let mut ancoras = Vec::new();
    for no in dom.query_selector("a").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let lidos = atributos(tag).map_err(|_| FalhaDaBarra::Fora)?;
        let decodificado = |nome: &'static str| -> Result<Option<String>, FalhaDaBarra> {
            match lidos.valor(nome).map_err(|_| FalhaDaBarra::Fora)? {
                None => Ok(None),
                Some(bruto) => decodificar_atributo(bruto)
                    .map(Some)
                    .ok_or(FalhaDaBarra::Fora),
            }
        };
        ancoras.push(AncoraDaBarra {
            href: decodificado("href")?,
            onclick: decodificado("onclick")?,
        });
    }
    Ok(ancoras)
}
/// O primeiro `acao=` da consulta do `href` (a conferência recusa dois).
pub(super) fn acao_do_href(href: &str) -> Option<&str> {
    let (_, consulta) = href.split_once('?')?;
    consulta
        .split('&')
        .find_map(|par| par.strip_prefix("acao="))
}

/// O texto da barra do nó `PROCESSO`; duas barras para ele são recusa.
fn barra_da_raiz(html: &str) -> Result<Option<String>, FalhaDaBarra> {
    let mut raiz = None;
    for linha in html.split('\n') {
        if let Some((i, argumentos)) = ler_declaracao(linha).map_err(|_| FalhaDaBarra::Fora)?
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
        let Some((_, lido)) = ler_barra(linha).map_err(|_| FalhaDaBarra::Fora)? else {
            return Err(FalhaDaBarra::Fora);
        };
        if texto.is_some() {
            return Err(FalhaDaBarra::Repetidas);
        }
        texto = Some(lido);
    }
    Ok(texto)
}
