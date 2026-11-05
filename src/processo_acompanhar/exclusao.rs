//! Validação pura da oferta de exclusão do Acompanhamento Especial (#254).
//! A função `acaoExcluir` é examinada como texto; nunca é executada.
//! Estrutura derivada da `UnB` em Todos `tests/test_processo_acompanhar_010.py` @323eb01.

use super::{ErroAcompanhar, TABELA, campo};
use crate::html_atributos::atributo_presente;
use crate::instalacao::Instalacao;

const FORMULARIO: &str = "frmGerenciarAcompanhamento";
const OCULTO: &str = "hdnInfraItemId";

/// Comprova que a linha escolhida oferece exclusão pelo formulário servido.
/// Não emite requisição nem autoriza `--confirmar`; esta última exige um
/// derivador de POST e verificação por leitura numa fatia posterior.
pub(super) fn conferir_oferta_exclusao(
    instalacao: &Instalacao,
    html: &str,
    id_procedimento: &str,
    id_alvo: &str,
) -> Result<(), ErroAcompanhar> {
    use ErroAcompanhar as E;
    let sem_oferta = E::ExclusaoNaoOferecida;
    if html.len() > crate::MAX_HTML_BYTES {
        return Err(E::HtmlInvalido);
    }
    if id_alvo.is_empty() || !id_alvo.chars().all(|c| c.is_ascii_digit()) {
        return Err(sem_oferta);
    }
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| E::HtmlInvalido)?;
    let parser = dom.parser();
    let mut form = None;
    for no in dom.query_selector("form").into_iter().flatten() {
        let tag = no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(E::HtmlInvalido)?;
        if campo(tag, "id")?.as_deref() == Some(FORMULARIO) && form.replace(tag).is_some() {
            return Err(sem_oferta);
        }
    }
    let form = form.ok_or(sem_oferta)?;
    // getElementById consulta TODO o DOM. Um elemento fora do form
    // com o mesmo ID pode receber o valor em vez do oculto enviável.
    let mut ids_do_oculto = 0;
    for no in dom.query_selector("[id]").into_iter().flatten() {
        let tag = no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(E::HtmlInvalido)?;
        if campo(tag, "id")?.as_deref() == Some(OCULTO) {
            ids_do_oculto += 1;
        }
    }
    if ids_do_oculto != 1 {
        return Err(sem_oferta);
    }
    if campo(form, "method")?.as_deref() != Some("post") {
        return Err(sem_oferta);
    }
    let action = campo(form, "action")?.ok_or(sem_oferta)?;
    let action = crate::formulario_web::decodificar_atributo(&action).ok_or(sem_oferta)?;
    crate::arvore_processo::conferir_endereco_relativo(
        instalacao,
        &action,
        "acompanhamento_gerenciar",
        id_procedimento,
    )
    .map_err(|_| sem_oferta)?;

    let mut ocultos = 0;
    let mut tabelas = Vec::new();
    for no in form.children().all(parser) {
        let Some(tag) = no.as_tag() else {
            continue;
        };
        if tag.name() == "input" && campo(tag, "name")?.as_deref() == Some(OCULTO) {
            if campo(tag, "type")?.as_deref() != Some("hidden")
                || campo(tag, "id")?.as_deref() != Some(OCULTO)
                || campo(tag, "value")?.as_deref() != Some("")
                || atributo_presente(tag, "disabled").map_err(|_| E::HtmlInvalido)?
            {
                return Err(sem_oferta);
            }
            ocultos += 1;
        }
        if tag.name() == "table" && campo(tag, "summary")?.as_deref() == Some(TABELA) {
            tabelas.push(tag);
        }
    }
    if ocultos != 1 || tabelas.len() != 1 {
        return Err(sem_oferta);
    }
    conferir_linha(tabelas[0], parser, id_alvo)?;
    conferir_script(instalacao, &dom, id_procedimento)
}

fn conferir_linha(
    tabela: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    id_alvo: &str,
) -> Result<(), ErroAcompanhar> {
    let sem_oferta = ErroAcompanhar::ExclusaoNaoOferecida;
    let prefixo = format!("acaoExcluir('{id_alvo}',");
    let mut linhas_do_alvo = 0;
    for no in tabela.children().all(parser) {
        let Some(tr) = no.as_tag() else {
            continue;
        };
        if tr.name() != "tr" {
            continue;
        }
        let mut alvo = false;
        let mut links = 0;
        for descendente in tr.children().all(parser) {
            let Some(tag) = descendente.as_tag() else {
                continue;
            };
            if tag.name() == "input"
                && campo(tag, "type")?.as_deref() == Some("checkbox")
                && campo(tag, "value")?.as_deref() == Some(id_alvo)
            {
                alvo = true;
            }
            if tag.name() == "a" && let Some(onclick) = campo(tag, "onclick")? {
                let onclick =
                    crate::formulario_web::decodificar_atributo(&onclick).ok_or(sem_oferta)?;
                let onclick = onclick.trim();
                if onclick.starts_with(&prefixo) && onclick.ends_with(");") {
                    links += 1;
                }
            }
        }
        if alvo {
            linhas_do_alvo += 1;
            if links != 1 {
                return Err(sem_oferta);
            }
        }
    }
    if linhas_do_alvo != 1 {
        return Err(sem_oferta);
    }

    Ok(())
}

fn conferir_script(
    instalacao: &Instalacao,
    dom: &tl::VDom<'_>,
    id_procedimento: &str,
) -> Result<(), ErroAcompanhar> {
    use ErroAcompanhar as E;
    let sem_oferta = E::ExclusaoNaoOferecida;
    let parser = dom.parser();
    let mut scripts = Vec::new();
    for no in dom.query_selector("script").into_iter().flatten() {
        let tag = no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(E::HtmlInvalido)?;
        let codigo = tag.inner_text(parser);
        if codigo.contains("function acaoExcluir(") {
            scripts.push(codigo.to_string());
        }
    }
    let codigo = match scripts.as_slice() {
        [codigo] if codigo.matches("function acaoExcluir(").count() == 1 => codigo,
        _ => return Err(sem_oferta),
    };
    let gravar = "document.getElementById('hdnInfraItemId').value=id;";
    let atribuir = "document.getElementById('frmGerenciarAcompanhamento').action='";
    let enviar = "document.getElementById('frmGerenciarAcompanhamento').submit();";
    if codigo.matches(atribuir).count() != 1 {
        return Err(sem_oferta);
    }
    let i_gravar = codigo.find(gravar).ok_or(sem_oferta)?;
    let i_atribuir = codigo.find(atribuir).ok_or(sem_oferta)?;
    let i_enviar = codigo.find(enviar).ok_or(sem_oferta)?;
    if !(i_gravar < i_atribuir && i_atribuir < i_enviar) {
        return Err(sem_oferta);
    }
    let depois_do_atribuir = &codigo[i_atribuir + atribuir.len()..];
    let (destino, _) = depois_do_atribuir.split_once('\'').ok_or(sem_oferta)?;
    let destino = crate::formulario_web::decodificar_atributo(destino).ok_or(sem_oferta)?;
    crate::arvore_processo::conferir_endereco_relativo(
        instalacao,
        &destino,
        "acompanhamento_excluir",
        id_procedimento,
    )
    .map_err(|_| sem_oferta)?;
    Ok(())
}

#[cfg(test)]
#[path = "tests_exclusao.rs"]
mod tests;
