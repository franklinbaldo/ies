//! Escolha do documento externo na lista e do tipo no cadastro inicial (#332).
//! Os links e opções provêm das telas do Todos (`UnB` SEI 4/PEN3 SEI 5).
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::ConsultaDoControlador;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum EscolhaExterna {
    Url(String),
    Formulario,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroEscolhaExterna {
    NaoOferecido,
    Ambiguo,
    ForaDaForma,
}

/// Não executa JS: SEI 4 publica link para `documento_receber`;
/// SEI 4.1/5 publicam escolher(-1) na lista.
pub(crate) fn escolher_externo(
    html: &crate::tela_html::HtmlServido,
) -> Result<EscolhaExterna, ErroEscolhaExterna> {
    let dom = crate::tela_html::arvore(html);
    let mut escolhas = Vec::new();
    for no in dom.elementos("a") {
        let Some(href) = no.atributo("href").map(str::to_owned) else {
            continue;
        };
        let onclick = no.atributo("onclick");
        // A classificação vem da ação, nunca do texto da âncora.
        let pelo_link = href
            .strip_prefix("controlador.php?")
            .and_then(|q| ConsultaDoControlador::ler(q).ok())
            .is_some_and(|q| q.acao() == Ok(AcaoDoSei::DocumentoReceber.como_na_url()));
        let pelo_script =
            onclick.is_some_and(|v| v.trim().trim_end_matches(';').trim() == "escolher(-1)");
        if !pelo_link && !pelo_script {
            continue;
        }
        if pelo_link && onclick.is_none_or(str::is_empty) {
            escolhas.push(EscolhaExterna::Url(href));
        } else if (href == "#" || href == "javascript:void(0);") && pelo_script {
            let formularios = dom.com_id("frmDocumentoEscolherTipo").collect::<Vec<_>>();
            let [no] = formularios.as_slice() else {
                return Err(ErroEscolhaExterna::ForaDaForma);
            };
            let form = *no;
            if !form.nome().eq_ignore_ascii_case("form") {
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
    let dom = crate::tela_html::arvore(html);
    let selects = dom.com_id("selSerie").collect::<Vec<_>>();
    let [no] = selects.as_slice() else {
        return Err(ErroEscolhaExterna::ForaDaForma);
    };
    let select = *no;
    if !select.nome().eq_ignore_ascii_case("select")
        || select.presente("disabled")
        || select.atributo("name") != Some("selSerie")
    {
        return Err(ErroEscolhaExterna::ForaDaForma);
    }
    // A recarga do tipo usa flag 1; flag 2 é cadastro definitivo.
    // Conferi-la ANTES de enviar qualquer POST evita escrita acidental.
    let flags = dom.com_id("hdnFlagDocumentoCadastro").collect::<Vec<_>>();
    let [no] = flags.as_slice() else {
        return Err(ErroEscolhaExterna::ForaDaForma);
    };
    let flag = *no;
    if !flag.nome().eq_ignore_ascii_case("input")
        || flag.atributo("type") != Some("hidden")
        || flag.atributo("name") != Some("hdnFlagDocumentoCadastro")
        || flag.atributo("value") != Some("1")
    {
        return Err(ErroEscolhaExterna::ForaDaForma);
    }
    let mut ids = Vec::new();
    for tag in select.descendentes() {
        if !tag.nome().eq_ignore_ascii_case("option") || tag.presente("disabled") {
            continue;
        }
        // Texto de <option>, já decodificado (a UnB serve `&nbsp;` na
        // opção vazia).
        let texto = tag.texto().split_whitespace().collect::<Vec<_>>().join(" ");
        if texto != tipo {
            continue;
        }
        let id = tag
            .atributo("value")
            .map(str::to_owned)
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
