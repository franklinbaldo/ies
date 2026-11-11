//! Link do menu "Pesquisa" publicado pelo Controle (`acao=protocolo_pesquisar`).
//! Forma observada (`UnB` 4.0.9, PEN1 5.0.4, GO 5.0.3; capturas privadas):
//! `<a link="protocolo_pesquisar" href="controlador.php?acao=protocolo_pesquisar&…&infra_hash=<64 hex>">`,
//! exatamente um por Controle. Só esse link é seguido, da mesma resposta.
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::{
    ConsultaDoControlador, ErroConsultaDoControlador, menciona_acao,
};
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::valor_do_atributo;
use crate::instalacao::Instalacao;
use crate::transporte::FalhaTransporte;

/// um erro por fato verificável do link no Controle.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroLinkDaPesquisa {
    /// O tokenizador de atributos recusou a tag.
    Atributo(#[source] crate::html_atributos::ErroAtributoHtml),
    /// `https://`, `http://` ou `//`: o menu observado é relativo.
    LinkDaPesquisaComEnderecoAbsoluto,
    LinkDaPesquisaAusenteNoControle,
    LinkDaPesquisaNaoReconhecido,
    LinksDaPesquisaRepetidosNoControle,
    /// Chave ou valor da consulta do link recusado pelo transporte.
    ConsultaDoLinkRecusadaPeloTransporte(#[source] FalhaTransporte),
    /// O parser HTML recusou a página do menu.
    HtmlDoLinkNaoInterpretado(#[source] tl::ParseError),
}

/// URL absoluta do link de pesquisa do Controle, resolvida contra a URL
/// da tela servida e conferida na instalação. Nenhuma requisição.
pub(crate) fn link_da_pesquisa(
    instalacao: &Instalacao,
    url_do_controle: &str,
    html: &str,
) -> Result<String, ErroLinkDaPesquisa> {
    let href = href_publicado(html)?;
    let minusculo = href.to_ascii_lowercase();
    if ["https://", "http://", "//"]
        .iter()
        .any(|prefixo| minusculo.starts_with(prefixo))
    {
        return Err(ErroLinkDaPesquisa::LinkDaPesquisaComEnderecoAbsoluto);
    }
    let nao = || ErroLinkDaPesquisa::LinkDaPesquisaNaoReconhecido;
    let consulta = href.strip_prefix("controlador.php?").ok_or_else(nao)?;
    if !href.bytes().all(|b| (33..=126).contains(&b)) || href.contains(['#', '\\', '\'']) {
        return Err(nao());
    }
    let consulta = ConsultaDoControlador::ler(consulta).map_err(|erro| match erro {
        ErroConsultaDoControlador::RecusadaPeloTransporte(falha) => {
            ErroLinkDaPesquisa::ConsultaDoLinkRecusadaPeloTransporte(falha)
        }
        ErroConsultaDoControlador::ParSemIgual => nao(),
    })?;
    if consulta.acao() != Ok(AcaoDoSei::ProtocoloPesquisar)
        || consulta.valores("acao_ajax").next().is_some()
        || !consulta
            .unico("infra_hash")
            .is_ok_and(|hash| hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Err(nao());
    }
    let base = url_do_controle.split('?').next().unwrap_or_default();
    let (pasta, _) = base.rsplit_once('/').ok_or_else(nao)?;
    let url = format!("{pasta}/{href}");
    let (host, caminho) = url
        .strip_prefix("https://")
        .and_then(|r| r.split_once('?'))
        .and_then(|(antes, _)| antes.split_once('/'))
        .ok_or_else(nao)?;
    if !instalacao.autoriza_controlador(host, caminho) {
        return Err(nao());
    }
    Ok(url)
}

/// O único `href` (decodificado) de `<a>` cuja consulta tem
/// `acao=protocolo_pesquisar`; `protocolo_pesquisa_rapida` não conta.
fn href_publicado(html: &str) -> Result<String, ErroLinkDaPesquisa> {
    let nao = || ErroLinkDaPesquisa::LinkDaPesquisaNaoReconhecido;
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(ErroLinkDaPesquisa::HtmlDoLinkNaoInterpretado)?;
    let parser = dom.parser();
    let mut achados = Vec::new();
    for no in dom.query_selector("a").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let Some(bruto) = valor_do_atributo(tag, "href").map_err(ErroLinkDaPesquisa::Atributo)?
        else {
            continue;
        };
        let Some(href) = decodificar_atributo(&bruto) else {
            // Entidade desconhecida só importa se puder ser o menu da pesquisa.
            if bruto.contains(AcaoDoSei::ProtocoloPesquisar.como_na_url()) {
                return Err(nao());
            }
            continue;
        };
        let publica = menciona_acao(&href, AcaoDoSei::ProtocoloPesquisar);
        if publica {
            achados.push(href);
        }
    }
    match achados.len() {
        0 => Err(ErroLinkDaPesquisa::LinkDaPesquisaAusenteNoControle),
        1 => Ok(achados.remove(0)),
        _ => Err(ErroLinkDaPesquisa::LinksDaPesquisaRepetidosNoControle),
    }
}

#[cfg(test)]
#[path = "tests_link.rs"]
mod tests;
