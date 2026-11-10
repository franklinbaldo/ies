//! Download de anexo externo publicado no cadastro do SEI (#60).
//! Fonte: Todos `tests/paginas_reais/documento_alterar_recebido.html`.
use super::{ErroPropriedadesDocumento, campo, ler_formulario, valor};
use crate::arvore_processo::EnderecoDoConteudo;
use crate::html_atributos::nos_com_id;
use crate::instalacao::Instalacao;

/// Somente um vínculo inequívoco entre o `id_anexo` listado e o href servido.
pub(crate) fn ler_anexo_publicado(
    instalacao: &Instalacao,
    html: &str,
    documento: &str,
    processo: &str,
) -> Result<Option<EnderecoDoConteudo>, ErroPropriedadesDocumento> {
    let _ = ler_formulario(html, documento, processo)?;
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroPropriedadesDocumento::CadastroDocumentoIlegivel)?;
    let parser = dom.parser();
    let nos = nos_com_id(&dom, "frmDocumentoCadastro")
        .map_err(|_| ErroPropriedadesDocumento::CadastroDocumentoIlegivel)?;
    let Some(form) = nos
        .first()
        .and_then(|no| no.get(parser))
        .and_then(tl::Node::as_tag)
    else {
        return Err(ErroPropriedadesDocumento::FormularioCadastroDocumentoAusente);
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
            return Err(ErroPropriedadesDocumento::AnexoDoCadastroDivergente);
        }
        let link = EnderecoDoConteudo::do_cadastro(instalacao, href, id_anexo)
            .map_err(|_| ErroPropriedadesDocumento::LinkAnexoCadastroInvalido)?;
        if encontrado.replace(link).is_some() {
            return Err(ErroPropriedadesDocumento::LinksAnexoCadastroRepetidos);
        }
    }
    Ok(encontrado)
}
