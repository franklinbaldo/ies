//! Escolha do documento externo na lista e do tipo no cadastro inicial (#332).
//! Os links e opções provêm das telas do Todos (`UnB` SEI 4/PEN3 SEI 5).
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::{atributo_presente, nos_com_id, valor_do_atributo};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum EscolhaExterna {
    Url(String),
    Formulario,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroEscolhaExterna {
    /// O tokenizador de atributos recusou a tag.
    Atributo(#[source] crate::html_atributos::ErroAtributoHtml),
    NaoOferecido,
    Ambiguo,
    ForaDaForma,
}

/// Não executa JS: SEI 4 publica link para `documento_receber`;
/// SEI 4.1/5 publicam escolher(-1) na lista.
pub(crate) fn escolher_externo(
    html: &crate::tela_html::HtmlServido,
) -> Result<EscolhaExterna, ErroEscolhaExterna> {
    let dom = crate::tela_html::interpretar(html);
    // Links em <template> não estão na interface ativa.
    let mut inertes = Vec::new();
    for no in dom.query_selector("template").into_iter().flatten() {
        let tag = no
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .ok_or(ErroEscolhaExterna::ForaDaForma)?;
        inertes.push(tag.boundaries(dom.parser()));
    }
    let mut escolhas = Vec::new();
    for no in dom.query_selector("a").into_iter().flatten() {
        let tag = no
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .ok_or(ErroEscolhaExterna::ForaDaForma)?;
        let inicio = tag.boundaries(dom.parser()).0;
        if inertes
            .iter()
            .any(|&(primeiro, ultimo)| (primeiro..=ultimo).contains(&inicio))
        {
            continue;
        }
        let Some(href) = valor_do_atributo(tag, "href").map_err(ErroEscolhaExterna::Atributo)?
        else {
            continue;
        };
        let href = decodificar_atributo(&href);
        let onclick = valor_do_atributo(tag, "onclick").map_err(ErroEscolhaExterna::Atributo)?;
        // A classificação vem da ação, nunca do texto da âncora.
        let pelo_link = href
            .strip_prefix("controlador.php?")
            .and_then(|q| ConsultaDoControlador::ler(q).ok())
            .is_some_and(|q| q.acao() == Ok(AcaoDoSei::DocumentoReceber));
        let pelo_script = onclick
            .as_deref()
            .is_some_and(|v| v.trim().trim_end_matches(';').trim() == "escolher(-1)");
        if !pelo_link && !pelo_script {
            continue;
        }
        if pelo_link && onclick.as_deref().is_none_or(str::is_empty) {
            escolhas.push(EscolhaExterna::Url(href));
        } else if (href == "#" || href == "javascript:void(0);") && pelo_script {
            let formularios = nos_com_id(&dom, "frmDocumentoEscolherTipo")
                .map_err(ErroEscolhaExterna::Atributo)?;
            let [no] = formularios.as_slice() else {
                return Err(ErroEscolhaExterna::ForaDaForma);
            };
            let form = no
                .get(dom.parser())
                .and_then(tl::Node::as_tag)
                .ok_or(ErroEscolhaExterna::ForaDaForma)?;
            if !form.name().as_utf8_str().eq_ignore_ascii_case("form") {
                return Err(ErroEscolhaExterna::ForaDaForma);
            }
            escolhas.push(EscolhaExterna::Formulario);
        } else {
            return Err(ErroEscolhaExterna::ForaDaForma);
        }
    }
    let mut escolhas = escolhas.into_iter();
    match (escolhas.next(), escolhas.next()) {
        (None, _) => Err(ErroEscolhaExterna::NaoOferecido),
        (Some(escolha), None) => Ok(escolha),
        (Some(_), Some(_)) => Err(ErroEscolhaExterna::Ambiguo),
    }
}

/// Na tela inicial o tipo ainda não está marcado: extrair somente o valor
/// da opção oferecida. A confirmação da seleção ocorrerá na tela recarregada.
pub(crate) fn identificar_tipo(
    html: &crate::tela_html::HtmlServido,
    tipo: &str,
) -> Result<String, ErroEscolhaExterna> {
    let dom = crate::tela_html::interpretar(html);
    let selects = nos_com_id(&dom, "selSerie").map_err(ErroEscolhaExterna::Atributo)?;
    let [no] = selects.as_slice() else {
        return Err(ErroEscolhaExterna::ForaDaForma);
    };
    let select = no
        .get(dom.parser())
        .and_then(tl::Node::as_tag)
        .ok_or(ErroEscolhaExterna::ForaDaForma)?;
    if !select.name().as_utf8_str().eq_ignore_ascii_case("select")
        || atributo_presente(select, "disabled").map_err(ErroEscolhaExterna::Atributo)?
        || valor_do_atributo(select, "name")
            .map_err(ErroEscolhaExterna::Atributo)?
            .as_deref()
            != Some("selSerie")
    {
        return Err(ErroEscolhaExterna::ForaDaForma);
    }
    // A recarga do tipo usa flag 1; flag 2 é cadastro definitivo.
    // Conferi-la ANTES de enviar qualquer POST evita escrita acidental.
    let flags =
        nos_com_id(&dom, "hdnFlagDocumentoCadastro").map_err(ErroEscolhaExterna::Atributo)?;
    let [no] = flags.as_slice() else {
        return Err(ErroEscolhaExterna::ForaDaForma);
    };
    let flag = no
        .get(dom.parser())
        .and_then(tl::Node::as_tag)
        .ok_or(ErroEscolhaExterna::ForaDaForma)?;
    if !flag.name().as_utf8_str().eq_ignore_ascii_case("input")
        || valor_do_atributo(flag, "type")
            .map_err(ErroEscolhaExterna::Atributo)?
            .as_deref()
            != Some("hidden")
        || valor_do_atributo(flag, "name")
            .map_err(ErroEscolhaExterna::Atributo)?
            .as_deref()
            != Some("hdnFlagDocumentoCadastro")
        || valor_do_atributo(flag, "value")
            .map_err(ErroEscolhaExterna::Atributo)?
            .as_deref()
            != Some("1")
    {
        return Err(ErroEscolhaExterna::ForaDaForma);
    }
    let mut ids = Vec::new();
    for tag in select
        .children()
        .all(dom.parser())
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        if !tag.name().as_utf8_str().eq_ignore_ascii_case("option")
            || atributo_presente(tag, "disabled").map_err(ErroEscolhaExterna::Atributo)?
        {
            continue;
        }
        // Texto de <option> é texto, não atributo: a UnB serve `&nbsp;`
        // na opção vazia, que o decodificador de atributo não conhece.
        let texto = html_escape::decode_html_entities(&tag.inner_text(dom.parser()))
            .into_owned()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if texto != tipo {
            continue;
        }
        let id = valor_do_atributo(tag, "value")
            .map_err(ErroEscolhaExterna::Atributo)?
            .ok_or(ErroEscolhaExterna::ForaDaForma)?;
        if id.is_empty() || id == "null" {
            return Err(ErroEscolhaExterna::ForaDaForma);
        }
        ids.push(id);
    }
    match ids.as_slice() {
        [] => Err(ErroEscolhaExterna::NaoOferecido),
        [id] => Ok(id.clone()),
        _ => Err(ErroEscolhaExterna::Ambiguo),
    }
}

#[cfg(test)]
#[path = "tests_escolha.rs"]
mod tests;
