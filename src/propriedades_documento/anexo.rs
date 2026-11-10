//! Download de anexo externo publicado no cadastro do SEI (#60).
//! Fonte: Todos `tests/paginas_reais/documento_alterar_recebido.html`.
use super::{campo, ler_formulario, valor};
use crate::arvore_processo::EnderecoDoConteudo;
use crate::erro_html::ErroHtml;
use crate::html_atributos::nos_com_id;
use crate::instalacao::Instalacao;

/// Somente um vínculo inequívoco entre o `id_anexo` listado e o href servido.
pub(crate) fn ler_anexo_publicado(
    instalacao: &Instalacao,
    html: &str,
    documento: &str,
    processo: &str,
) -> Result<Option<EnderecoDoConteudo>, ErroHtml> {
    let _ = ler_formulario(html, documento, processo)?;
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroHtml::html("cadastro_documento_ilegivel"))?;
    let parser = dom.parser();
    let nos = nos_com_id(&dom, "frmDocumentoCadastro")
        .map_err(|e| ErroHtml::atributo("cadastro_documento_ilegivel", e))?;
    let Some(form) = nos
        .first()
        .and_then(|no| no.get(parser))
        .and_then(tl::Node::as_tag)
    else {
        return Err(ErroHtml::ausente("formulario_cadastro_documento_ausente"));
    };
    let lista = valor(campo(form, parser, "hdnAnexos")?)?;
    let mut encontrado = None;
    for linha in html.lines() {
        let Some((_, restante)) = linha.split_once("objTabelaAnexos.adicionarAcoes('") else {
            continue;
        };
        let Some((id_anexo, restante)) = restante.split_once("','") else {
            continue;
        };
        let Some((_, restante)) = restante.split_once("href=\"") else {
            continue;
        };
        let Some((href, _)) = restante.split_once('"') else {
            continue;
        };
        if !href.contains("acao=documento_download_anexo") {
            continue;
        }
        if !lista.starts_with(&format!("{id_anexo}±")) {
            return Err(ErroHtml::valor("anexo_do_cadastro_divergente"));
        }
        let link = EnderecoDoConteudo::do_cadastro(instalacao, href, id_anexo)
            .map_err(|_| ErroHtml::valor("link_anexo_cadastro_invalido"))?;
        if encontrado.replace(link).is_some() {
            return Err(ErroHtml::repetido("links_anexo_cadastro_repetidos"));
        }
    }
    Ok(encontrado)
}
