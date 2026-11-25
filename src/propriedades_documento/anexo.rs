//! Download de anexo externo publicado no cadastro do SEI (#60).
//! Fonte: Todos `tests/paginas_reais/documento_alterar_recebido.html`.
use super::{ErroPropriedadesDocumento, cadastro_do_documento, campo, valor};
use crate::arvore_processo::EnderecoDoConteudo;
use crate::instalacao::Instalacao;

/// Somente um vínculo inequívoco entre o `id_anexo` listado e o href servido,
/// no cadastro conferido como o do documento e processo pedidos.
pub(crate) fn ler_anexo_publicado(
    instalacao: &Instalacao,
    html: &crate::tela_html::HtmlServido,
    documento: &str,
    processo: &str,
) -> Result<Option<EnderecoDoConteudo>, ErroPropriedadesDocumento> {
    let dom = crate::tela_html::arvore(html);
    let form = cadastro_do_documento(&dom, documento, processo)?;
    let lista = valor(campo(form, "hdnAnexos")?)?;
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
            .map_err(ErroPropriedadesDocumento::LinkAnexoCadastroInvalido)?;
        if encontrado.replace(link).is_some() {
            return Err(ErroPropriedadesDocumento::LinksAnexoCadastroRepetidos);
        }
    }
    Ok(encontrado)
}
