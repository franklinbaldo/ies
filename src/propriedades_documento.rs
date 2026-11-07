//! Cadastro somente leitura a partir de documento_alterar publicado na barra
//! (Todos tests/paginas_reais/documento_alterar.html, SEI 4.0.9; Refs #60).
use crate::arvore_processo::{barra_do_no, endereco::conferir_endereco_relativo_por};
use crate::erro_html::ErroHtml;
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::{atributos, nos_com_id, nos_com_id_em};
use crate::instalacao::Instalacao;
use serde::Serialize;
type Tag<'p> = &'p tl::HTMLTag<'p>;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PropriedadesDocumento { NaoPedidas, NaoPublicadas, Lidas(PropriedadesLidas) }

#[derive(Debug, PartialEq, Eq, Serialize)]
pub(crate) struct PropriedadesLidas {
    pub(crate) fonte: &'static str,
    pub(crate) tipo: String,
    pub(crate) descricao: String,
    pub(crate) nome_na_arvore: String,
    pub(crate) observacoes: String,
    pub(crate) nivel_de_acesso: Option<Nivel>,
}
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Nivel { Publico, Restrito, Sigiloso }

fn atributo(tag: Tag<'_>, nome: &str) -> Result<Option<String>, ErroHtml> {
    let a = atributos(tag).map_err(|e| ErroHtml::atributo("cadastro_documento_ilegivel", e))?;
    a.valor(nome).map_err(|e| ErroHtml::atributo("cadastro_documento_ilegivel", e))?
        .map(|v| decodificar_atributo(v).ok_or(ErroHtml::valor("cadastro_documento_ilegivel")))
        .transpose()
}
fn campo<'p>(form: Tag<'p>, parser: &'p tl::Parser<'p>, id: &str) -> Result<Tag<'p>, ErroHtml> {
    let nos = nos_com_id_em(form, parser, id)
        .map_err(|e| ErroHtml::atributo("cadastro_documento_ilegivel", e))?;
    match nos.as_slice() {
        [unico] => unico.get(parser).and_then(tl::Node::as_tag)
            .ok_or(ErroHtml::valor("cadastro_documento_ilegivel")),
        [] => Err(ErroHtml::ausente("campo_cadastro_documento_ausente")),
        _ => Err(ErroHtml::repetido("campo_cadastro_documento_repetido")),
    }
}
fn valor(tag: Tag<'_>) -> Result<String, ErroHtml> {
    atributo(tag, "value")?.ok_or(ErroHtml::ausente("valor_cadastro_documento_ausente"))
}
fn texto(tag: Tag<'_>, parser: &tl::Parser<'_>) -> Result<String, ErroHtml> {
    decodificar_atributo(&tag.inner_text(parser))
        .ok_or(ErroHtml::valor("texto_cadastro_documento_ilegivel"))
}
fn nivel(form: Tag<'_>, parser: &tl::Parser<'_>) -> Result<Option<Nivel>, ErroHtml> {
    let mut marcados = Vec::new();
    for no in form.query_selector(parser, "input").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else { continue; };
        let a = atributos(tag).map_err(|e| ErroHtml::atributo("nivel_cadastro_documento_ilegivel", e))?;
        if a.valor("name").map_err(|e| ErroHtml::atributo("nivel_cadastro_documento_ilegivel", e))?
            == Some("rdoNivelAcesso") && a.presente("checked") {
            marcados.push(atributo(tag, "value")?);
        }
    }
    match marcados.as_slice() {
        [] => Ok(None),
        [Some(x)] if x == "0" => Ok(Some(Nivel::Publico)),
        [Some(x)] if x == "1" => Ok(Some(Nivel::Restrito)),
        [Some(x)] if x == "2" => Ok(Some(Nivel::Sigiloso)),
        _ => Err(ErroHtml::valor("nivel_cadastro_documento_inesperado")),
    }
}
/// Usa somente o link oferecido pelo SEI, sem inventar URLs assinadas.
pub(crate) fn ler_link(
    instalacao: &Instalacao, arvore: &str, documento: &str, processo: &str,
) -> Result<Option<String>, ErroHtml> {
    let Some(barra) = barra_do_no(arvore, documento)
        .map_err(|_| ErroHtml::valor("barra_cadastro_documento_ilegivel"))?
    else { return Ok(None); };
    let dom = tl::parse(&barra, tl::ParserOptions::default())
        .map_err(|_| ErroHtml::html("barra_cadastro_documento_ilegivel"))?;
    let mut encontrado = None;
    for no in dom.query_selector("a").into_iter().flatten() {
        let Some(tag) = no.get(dom.parser()).and_then(tl::Node::as_tag) else { continue; };
        let Some(href) = atributo(tag, "href")? else { continue; };
        let acao = href.strip_prefix("controlador.php?")
            .and_then(|q| q.split('&').find_map(|p| p.strip_prefix("acao=")));
        if acao != Some("documento_alterar") { continue; }
        conferir_endereco_relativo_por(
            instalacao, &href, "documento_alterar", "id_procedimento", processo,
        ).map_err(|_| ErroHtml::valor("link_cadastro_documento_invalido"))?;
        let url = conferir_endereco_relativo_por(
            instalacao, &href, "documento_alterar", "id_documento", documento,
        ).map_err(|_| ErroHtml::valor("link_cadastro_documento_invalido"))?;
        if encontrado.replace(url).is_some() {
            return Err(ErroHtml::repetido("links_cadastro_documento_repetidos"));
        }
    }
    Ok(encontrado)
}
/// Lê o cadastro pelo formulário servido, nunca o submete.
pub(crate) fn ler_formulario(
    html: &str, documento: &str, processo: &str,
) -> Result<PropriedadesLidas, ErroHtml> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroHtml::html("cadastro_documento_ilegivel"))?;
    let parser = dom.parser();
    let nos = nos_com_id(&dom, "frmDocumentoCadastro")
        .map_err(|e| ErroHtml::atributo("cadastro_documento_ilegivel", e))?;
    let form = match nos.as_slice() {
        [unico] => unico.get(parser).and_then(tl::Node::as_tag)
            .ok_or(ErroHtml::html("cadastro_documento_ilegivel"))?,
        [] => return Err(ErroHtml::ausente("formulario_cadastro_documento_ausente")),
        _ => return Err(ErroHtml::repetido("formularios_cadastro_documento_repetidos")),
    };
    if valor(campo(form, parser, "hdnIdDocumento")?)? != documento
        || valor(campo(form, parser, "hdnIdProcedimento")?)? != processo {
        return Err(ErroHtml::valor("identidade_cadastro_documento_divergente"));
    }
    let tipo = texto(campo(form, parser, "lblSerieTitulo")?, parser)?;
    if tipo.trim().is_empty() { return Err(ErroHtml::valor("tipo_cadastro_documento_vazio")); }
    Ok(PropriedadesLidas {
        fonte: "documento_alterar", tipo,
        descricao: valor(campo(form, parser, "txtDescricao")?)?,
        nome_na_arvore: valor(campo(form, parser, "txtNomeArvore")?)?,
        observacoes: texto(campo(form, parser, "txaObservacoes")?, parser)?,
        nivel_de_acesso: nivel(form, parser)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arvore_processo::tests::{NUP, arvore, no_processo, no_documento};
    use crate::arvore_processo::tests_barra_documento::{com_barra, barra};
    use crate::arvore_processo::tests_tela::instalacao;
    #[test]
    fn dado_barra_servida_quando_lida_entao_confere_ambos_ids() {
        let html = arvore(&[no_processo(0, "10", NUP), no_documento(1, "21", "10", "0000101")]);
        let html = com_barra(&html, 1, &barra("10", "21", None));
        assert!(ler_link(&instalacao(), &html, "21", "10").unwrap().is_some());
        assert!(ler_link(&instalacao(), &html, "22", "10").unwrap().is_none());
        assert!(ler_link(&instalacao(), &html, "21", "11").is_err());
    }
    #[test]
    fn dado_formulario_observado_quando_lido_entao_preserva_campos() {
        // Caso Todos tests/paginas_reais/documento_alterar.html @323eb01.
        let html = r#"<form id="frmDocumentoCadastro" method="post">
            <input id="hdnIdDocumento" value="21"><input id="hdnIdProcedimento" value="10">
            <label id="lblSerieTitulo">Ofício</label>
            <input id="txtDescricao" value="Revisão">
            <input id="txtNomeArvore" value="Ofício 0000101">
            <textarea id="txaObservacoes">texto livre</textarea>
            <input name="rdoNivelAcesso" value="0" checked>
            <input name="rdoNivelAcesso" value="1"></form>"#;
        let lida = ler_formulario(html, "21", "10").unwrap();
        assert_eq!((lida.tipo.as_str(), lida.descricao.as_str()), ("Ofício", "Revisão"));
        assert_eq!(lida.nivel_de_acesso, Some(Nivel::Publico));
        assert!(ler_formulario(html, "21", "11").is_err());
    }
}
