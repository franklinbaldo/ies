//! Escolha do documento externo na lista e do tipo no cadastro inicial (#332).
//! Os links e opções provêm das telas do Todos (`UnB` SEI 4/PEN3 SEI 5).
use crate::entidades_html::decodificar_texto;
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::{atributo_presente, nos_com_id, valor_do_atributo};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum EscolhaExterna {
    Url(String),
    Formulario,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroEscolhaExterna {
    NaoOferecido,
    Ambiguo,
    ForaDaForma,
}

/// Não executa JS: SEI 4 publica link para `documento_receber`;
/// SEI 4.1/5 publicam escolher(-1) na lista.
pub(crate) fn escolher_externo(html: &str) -> Result<EscolhaExterna, ErroEscolhaExterna> {
    use ErroEscolhaExterna as E;
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| E::ForaDaForma)?;
    // Links em <template> não estão na interface ativa.
    let mut inertes = Vec::new();
    for no in dom.query_selector("template").into_iter().flatten() {
        let tag = no
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .ok_or(E::ForaDaForma)?;
        inertes.push(tag.boundaries(dom.parser()));
    }
    let mut escolhas = Vec::new();
    for no in dom.query_selector("a").into_iter().flatten() {
        let tag = no
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .ok_or(E::ForaDaForma)?;
        let inicio = tag.boundaries(dom.parser()).0;
        if inertes
            .iter()
            .any(|&(primeiro, ultimo)| (primeiro..=ultimo).contains(&inicio))
        {
            continue;
        }
        let Some(href) = valor_do_atributo(tag, "href").map_err(|_| E::ForaDaForma)? else {
            continue;
        };
        let href = decodificar_atributo(&href).ok_or(E::ForaDaForma)?;
        let onclick = valor_do_atributo(tag, "onclick").map_err(|_| E::ForaDaForma)?;
        // A classificação vem da ação, nunca do texto da âncora.
        let pelo_link = href.starts_with("controlador.php?")
            && href
                .split('?')
                .nth(1)
                .is_some_and(|q| q.split('&').any(|p| p == "acao=documento_receber"));
        let pelo_script = onclick
            .as_deref()
            .is_some_and(|v| v.trim().trim_end_matches(';').trim() == "escolher(-1)");
        if !pelo_link && !pelo_script {
            continue;
        }
        if pelo_link && onclick.as_deref().is_none_or(str::is_empty) {
            escolhas.push(EscolhaExterna::Url(href));
        } else if (href == "#" || href == "javascript:void(0);") && pelo_script {
            let formularios =
                nos_com_id(&dom, "frmDocumentoEscolherTipo").map_err(|_| E::ForaDaForma)?;
            let [no] = formularios.as_slice() else {
                return Err(E::ForaDaForma);
            };
            let form = no
                .get(dom.parser())
                .and_then(tl::Node::as_tag)
                .ok_or(E::ForaDaForma)?;
            if !form.name().as_utf8_str().eq_ignore_ascii_case("form") {
                return Err(E::ForaDaForma);
            }
            escolhas.push(EscolhaExterna::Formulario);
        } else {
            return Err(E::ForaDaForma);
        }
    }
    match escolhas.len() {
        0 => Err(E::NaoOferecido),
        1 => Ok(escolhas.pop().expect("comprimento conferido")),
        _ => Err(E::Ambiguo),
    }
}

/// Na tela inicial o tipo ainda não está marcado: extrair somente o valor
/// da opção oferecida. A confirmação da seleção ocorrerá na tela recarregada.
pub(crate) fn identificar_tipo(html: &str, tipo: &str) -> Result<String, ErroEscolhaExterna> {
    use ErroEscolhaExterna as E;
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| E::ForaDaForma)?;
    let selects = nos_com_id(&dom, "selSerie").map_err(|_| E::ForaDaForma)?;
    let [no] = selects.as_slice() else {
        return Err(E::ForaDaForma);
    };
    let select = no
        .get(dom.parser())
        .and_then(tl::Node::as_tag)
        .ok_or(E::ForaDaForma)?;
    if !select.name().as_utf8_str().eq_ignore_ascii_case("select")
        || atributo_presente(select, "disabled").map_err(|_| E::ForaDaForma)?
        || valor_do_atributo(select, "name")
            .map_err(|_| E::ForaDaForma)?
            .as_deref()
            != Some("selSerie")
    {
        return Err(E::ForaDaForma);
    }
    // A recarga do tipo usa flag 1; flag 2 é cadastro definitivo.
    // Conferi-la ANTES de enviar qualquer POST evita escrita acidental.
    let flags = nos_com_id(&dom, "hdnFlagDocumentoCadastro").map_err(|_| E::ForaDaForma)?;
    let [no] = flags.as_slice() else {
        return Err(E::ForaDaForma);
    };
    let flag = no
        .get(dom.parser())
        .and_then(tl::Node::as_tag)
        .ok_or(E::ForaDaForma)?;
    if !flag.name().as_utf8_str().eq_ignore_ascii_case("input")
        || valor_do_atributo(flag, "type")
            .map_err(|_| E::ForaDaForma)?
            .as_deref()
            != Some("hidden")
        || valor_do_atributo(flag, "name")
            .map_err(|_| E::ForaDaForma)?
            .as_deref()
            != Some("hdnFlagDocumentoCadastro")
        || valor_do_atributo(flag, "value")
            .map_err(|_| E::ForaDaForma)?
            .as_deref()
            != Some("1")
    {
        return Err(E::ForaDaForma);
    }
    let mut ids = Vec::new();
    for tag in select
        .children()
        .all(dom.parser())
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        if !tag.name().as_utf8_str().eq_ignore_ascii_case("option")
            || atributo_presente(tag, "disabled").map_err(|_| E::ForaDaForma)?
        {
            continue;
        }
        // Texto de <option> é texto, não atributo: a UnB serve `&nbsp;`
        // na opção vazia, que o decodificador de atributo não conhece.
        let texto = decodificar_texto(&tag.inner_text(dom.parser()))
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if texto != tipo {
            continue;
        }
        let id = valor_do_atributo(tag, "value")
            .map_err(|_| E::ForaDaForma)?
            .ok_or(E::ForaDaForma)?;
        if id.is_empty() || id == "null" {
            return Err(E::ForaDaForma);
        }
        ids.push(id);
    }
    match ids.as_slice() {
        [] => Err(E::NaoOferecido),
        [id] => Ok(id.clone()),
        _ => Err(E::Ambiguo),
    }
}

#[cfg(test)]
#[path = "tests_escolha.rs"]
mod tests;
