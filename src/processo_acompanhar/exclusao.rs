//! Oferta de retirada: reconhece somente os literais publicados na tela.
//! Não executa nem interpreta JavaScript e não inventa action assinada.
use super::{ErroAcompanhar, campo};
use crate::formulario_web::decodificar_atributo;
use crate::instalacao::Instalacao;

/// Confere formulário, linha e literal do script observado para o processo.
pub(super) fn destino_oferecido(
    instalacao: &Instalacao,
    html: &str,
    id_procedimento: &str,
    id_alvo: &str,
) -> Result<String, ErroAcompanhar> {
    let recusada = || ErroAcompanhar::ExclusaoNaoOferecida;
    if !id_alvo.bytes().all(|b| b.is_ascii_digit()) || id_alvo.is_empty() {
        return Err(recusada());
    }
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| recusada())?;
    let parser = dom.parser();
    let form = crate::formulario_web::localizar_formulario(&dom, "frmGerenciarAcompanhamento")
        .map_err(|_| recusada())?;
    let action = campo(form, "action")?.ok_or_else(recusada)?;
    let action = decodificar_atributo(&action).ok_or_else(recusada)?;
    crate::arvore_processo::conferir_endereco_relativo(
        instalacao, &action, "acompanhamento_gerenciar", id_procedimento,
    )
    .map_err(|_| recusada())?;

    // Mesmo alvo na checkbox, no link da linha e no campo enviado.
    let mut campos = 0;
    let mut links = 0;
    let assinatura = format!("acaoExcluir('{id_alvo}',");
    for no in form.children().all(parser) {
        let Some(tag) = no.as_tag() else { continue };
        if tag.name() == "input"
            && campo(tag, "name")?.as_deref() == Some("hdnInfraItemId")
            && campo(tag, "id")?.as_deref() == Some("hdnInfraItemId")
            && campo(tag, "type")?.as_deref() == Some("hidden")
        {
            campos += 1;
        }
        if tag.name() == "a" {
            let onclick = campo(tag, "onclick")?.unwrap_or_default();
            let onclick = decodificar_atributo(&onclick).ok_or_else(recusada)?;
            if onclick.trim().starts_with(&assinatura) && onclick.trim().ends_with(");") {
                links += 1;
            }
        }
    }
    if campos != 1 || links != 1 {
        return Err(recusada());
    }

    let prefixo = "document.getElementById('frmGerenciarAcompanhamento').action='";
    let gravar = "document.getElementById('hdnInfraItemId').value=id;";
    let enviar = "document.getElementById('frmGerenciarAcompanhamento').submit();";
    let mut destinos = Vec::new();
    for no in dom.query_selector("script").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else { continue };
        let codigo = tag.inner_text(parser);
        let codigo = codigo.as_ref();
        if !codigo.contains("function acaoExcluir(") {
            continue;
        }
        let (antes, resto) = codigo.split_once(prefixo).ok_or_else(recusada)?;
        let (destino, depois) = resto.split_once('\'').ok_or_else(recusada)?;
        if !antes.contains(gravar) || !depois.contains(enviar)
            || codigo.matches(prefixo).count() != 1
        {
            return Err(recusada());
        }
        let destino = decodificar_atributo(destino).ok_or_else(recusada)?;
        crate::arvore_processo::conferir_endereco_relativo(
            instalacao, &destino, "acompanhamento_excluir", id_procedimento,
        )
        .map_err(|_| recusada())?;
        destinos.push(destino);
    }
    match destinos.as_slice() {
        [destino] => Ok(destino.clone()),
        _ => Err(recusada()),
    }
}
