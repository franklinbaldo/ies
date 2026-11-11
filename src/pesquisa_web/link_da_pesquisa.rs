//! Link do menu "Pesquisa" publicado pelo Controle (`acao=protocolo_pesquisar`).
//! Forma observada (`UnB` 4.0.9, PEN1 5.0.4, GO 5.0.3; capturas privadas):
//! `<a link="protocolo_pesquisar" href="controlador.php?acao=protocolo_pesquisar&…&infra_hash=<64 hex>">`,
//! exatamente um por Controle. Só esse link é seguido, da mesma resposta.
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::valor_do_atributo;
use crate::instalacao::Instalacao;
use crate::transporte::{conferir_chave_de_consulta, conferir_valor_de_consulta};

/// um erro por fato verificável do link no Controle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroLinkDaPesquisa {
    /// O tokenizador de atributos recusou a tag.
    Atributo(#[source] crate::html_atributos::ErroAtributoHtml),
    /// `https://`, `http://` ou `//`: o menu observado é relativo.
    LinkDaPesquisaComEnderecoAbsoluto,
    LinkDaPesquisaAusenteNoControle,
    LinkDaPesquisaNaoReconhecido,
    LinksDaPesquisaRepetidosNoControle,
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
    let nao = ErroLinkDaPesquisa::LinkDaPesquisaNaoReconhecido;
    let consulta = href.strip_prefix("controlador.php?").ok_or(nao)?;
    if !href.bytes().all(|b| (33..=126).contains(&b)) || href.contains(['#', '\\', '\'']) {
        return Err(nao);
    }
    let (mut acoes, mut hashes) = (0, 0);
    for par in consulta.split('&') {
        let (nome, valor) = par.split_once('=').ok_or(nao)?;
        conferir_chave_de_consulta(nome).map_err(|_| nao)?;
        conferir_valor_de_consulta(valor).map_err(|_| nao)?;
        match nome {
            "acao" if valor == ACAO_DA_PESQUISA => acoes += 1,
            "infra_hash" if valor.len() == 64 && valor.bytes().all(|b| b.is_ascii_hexdigit()) => {
                hashes += 1;
            }
            "acao" | "infra_hash" | "acao_ajax" => return Err(nao),
            _ => {}
        }
    }
    if acoes != 1 || hashes != 1 {
        return Err(nao);
    }
    let base = url_do_controle.split('?').next().unwrap_or_default();
    let (pasta, _) = base.rsplit_once('/').ok_or(nao)?;
    let url = format!("{pasta}/{href}");
    let (host, caminho) = url
        .strip_prefix("https://")
        .and_then(|r| r.split_once('?'))
        .and_then(|(antes, _)| antes.split_once('/'))
        .ok_or(nao)?;
    if !instalacao.autoriza_controlador(host, caminho) {
        return Err(nao);
    }
    Ok(url)
}

const ACAO_DA_PESQUISA: &str = "protocolo_pesquisar";

/// O único `href` (decodificado) de `<a>` cuja consulta tem
/// `acao=protocolo_pesquisar`; `protocolo_pesquisa_rapida` não conta.
fn href_publicado(html: &str) -> Result<String, ErroLinkDaPesquisa> {
    let nao = ErroLinkDaPesquisa::LinkDaPesquisaNaoReconhecido;
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| nao)?;
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
            if bruto.contains(ACAO_DA_PESQUISA) {
                return Err(nao);
            }
            continue;
        };
        let publica = href.split_once('?').is_some_and(|(_, consulta)| {
            consulta
                .split('&')
                .any(|par| par.strip_prefix("acao=") == Some(ACAO_DA_PESQUISA))
        });
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
