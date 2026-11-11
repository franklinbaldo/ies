//! Cadastro externo recarregado por tipo — observação estrutural (#332).
//! Não envia upload nem cadastra documentos.
use super::anexo::{FalhaAnexo, PlanoAnexo};
use crate::html_atributos::{atributo_presente, nos_com_id, valor_do_atributo};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaCadastroExterno {
    /// O tokenizador de atributos recusou a tag.
    Atributo(#[source] crate::html_atributos::ErroAtributoHtml),
    /// O tokenizador HTML recusou a tela inteira.
    HtmlNaoAnalisavel(#[source] tl::ParseError),
    FormaNaoObservada,
    TipoNaoOferecido,
    TipoAmbiguo,
    TipoNaoRecarregado,
    PdfNaoOferecido,
    PdfAcimaDoLimite,
    /// O tamanho máximo declarado no script não é um número.
    TamanhoDeclaradoIlegivel(#[source] std::num::ParseIntError),
    PlanoAnexo(#[source] FalhaAnexo),
}

pub(crate) struct CadastroExterno {
    pub(crate) id_tipo: String,
    pub(crate) limite_pdf_bytes: u64,
    pub(crate) plano: PlanoAnexo,
    pub(crate) campos_oferecidos: Vec<String>,
}

/// Um controle só vale se pertence ao formulário observado e não está desabilitado.
fn valor_input(
    dom: &tl::VDom<'_>,
    formulario: &tl::HTMLTag<'_>,
    id: &str,
) -> Result<String, FalhaCadastroExterno> {
    let erro = || FalhaCadastroExterno::FormaNaoObservada;
    let nos = nos_com_id(dom, id).map_err(FalhaCadastroExterno::Atributo)?;
    let [no] = nos.as_slice() else {
        return Err(erro());
    };
    let tag = no
        .get(dom.parser())
        .and_then(tl::Node::as_tag)
        .ok_or_else(erro)?;
    let (ini, fim) = formulario.boundaries(dom.parser());
    let (start, end) = tag.boundaries(dom.parser());
    if start <= ini
        || end > fim
        || !tag.name().as_utf8_str().eq_ignore_ascii_case("input")
        || atributo_presente(tag, "disabled").map_err(FalhaCadastroExterno::Atributo)?
        || valor_do_atributo(tag, "name")
            .map_err(FalhaCadastroExterno::Atributo)?
            .as_deref()
            != Some(id)
        || valor_do_atributo(tag, "type")
            .map_err(FalhaCadastroExterno::Atributo)?
            .as_deref()
            != Some("hidden")
    {
        return Err(erro());
    }
    Ok(valor_do_atributo(tag, "value")
        .map_err(FalhaCadastroExterno::Atributo)?
        .unwrap_or_default())
}

/// Lê a declaração literal da extensão PDF no script do upload da tela.
/// Nenhum código JavaScript é executado.
fn limite_pdf(html: &str, dom: &tl::VDom<'_>) -> Result<u64, FalhaCadastroExterno> {
    let erro = || FalhaCadastroExterno::FormaNaoObservada;
    let mut limites = Vec::new();
    let mut declarou_lista = false;
    for no in dom.query_selector("script").into_iter().flatten() {
        let tag = no
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .ok_or_else(erro)?;
        if atributo_presente(tag, "src").map_err(FalhaCadastroExterno::Atributo)? {
            continue;
        }
        let inicio = tag.boundaries(dom.parser()).0;
        let restante = html.get(inicio..).ok_or_else(erro)?;
        let inicio_corpo = restante.find('>').ok_or_else(erro)? + 1;
        let fim_corpo = inicio_corpo
            + restante
                .get(inicio_corpo..)
                .ok_or_else(erro)?
                .find("</script>")
                .ok_or_else(erro)?;
        let fonte = restante.get(inicio_corpo..fim_corpo).ok_or_else(erro)?;
        if fonte.contains("var arrExt = [];") {
            declarou_lista = true;
        }
        for linha in fonte
            .lines()
            .map(str::trim)
            .filter(|linha| linha.starts_with("arrExt[") && linha.contains("= {"))
        {
            let (_, objeto) = linha.split_once("= {").ok_or_else(erro)?;
            let (objeto, fim) = objeto.split_once("};").ok_or_else(erro)?;
            if !fim.trim().is_empty() {
                return Err(erro());
            }
            let mut atributos = objeto.split(',').map(str::trim);
            let nome = atributos.next().ok_or_else(erro)?;
            let tamanho = atributos.next().ok_or_else(erro)?;
            if atributos.next().is_some() {
                return Err(erro());
            }
            let (chave, valor) = nome.split_once(':').ok_or_else(erro)?;
            if chave.trim() != "nome" {
                return Err(erro());
            }
            let extensao = valor.trim().trim_matches('"');
            let (chave, valor) = tamanho.split_once(':').ok_or_else(erro)?;
            if chave.trim() != "tamanho" {
                return Err(erro());
            }
            let tamanho_mb = valor
                .trim()
                .parse::<u64>()
                .map_err(FalhaCadastroExterno::TamanhoDeclaradoIlegivel)?;
            if extensao == "pdf" {
                limites.push(tamanho_mb.checked_mul(1024 * 1024).ok_or_else(erro)?);
            }
        }
    }
    if !declarou_lista {
        return Err(erro());
    }
    match limites.as_slice() {
        [unico] => Ok(*unico),
        [] => Err(FalhaCadastroExterno::PdfNaoOferecido),
        _ => Err(erro()),
    }
}

/// Controles de usuário realmente publicados dentro do cadastro ativo.
/// Não expõe valores, campos ocultos nem controles desabilitados.
fn campos_oferecidos(
    dom: &tl::VDom<'_>,
    formulario: &tl::HTMLTag<'_>,
) -> Result<Vec<String>, FalhaCadastroExterno> {
    let erro = || FalhaCadastroExterno::FormaNaoObservada;
    let (inicio_form, fim_form) = formulario.boundaries(dom.parser());
    let mut encontrados = Vec::new();
    for elemento in ["input", "select", "textarea"] {
        for no in dom.query_selector(elemento).into_iter().flatten() {
            let tag = no
                .get(dom.parser())
                .and_then(tl::Node::as_tag)
                .ok_or_else(erro)?;
            let (inicio, fim) = tag.boundaries(dom.parser());
            if inicio <= inicio_form
                || fim > fim_form
                || atributo_presente(tag, "disabled").map_err(FalhaCadastroExterno::Atributo)?
            {
                continue;
            }
            let Some(nome) =
                valor_do_atributo(tag, "name").map_err(FalhaCadastroExterno::Atributo)?
            else {
                continue;
            };
            if nome.is_empty() {
                continue;
            }
            if elemento == "input"
                && matches!(
                    valor_do_atributo(tag, "type")
                        .map_err(FalhaCadastroExterno::Atributo)?
                        .as_deref(),
                    Some("hidden" | "file" | "submit" | "button" | "reset" | "image")
                )
            {
                continue;
            }
            encontrados.push((inicio, nome));
        }
    }
    encontrados.sort_by_key(|(inicio, _)| *inicio);
    let mut nomes = Vec::new();
    for (_, nome) in encontrados {
        if !nomes.contains(&nome) {
            nomes.push(nome);
        }
    }
    Ok(nomes)
}

/// Inspeciona o cadastro servido depois da escolha do tipo (sem escrita).
/// A saída mantém o destino de upload somente em memória.
pub(crate) fn observar_cadastro(
    html: &str,
    tipo: &str,
    tamanho_pdf: u64,
) -> Result<CadastroExterno, FalhaCadastroExterno> {
    let erro = || FalhaCadastroExterno::FormaNaoObservada;
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(FalhaCadastroExterno::HtmlNaoAnalisavel)?;
    let formularios =
        nos_com_id(&dom, "frmDocumentoCadastro").map_err(FalhaCadastroExterno::Atributo)?;
    let [no] = formularios.as_slice() else {
        return Err(erro());
    };
    let form = no
        .get(dom.parser())
        .and_then(tl::Node::as_tag)
        .ok_or_else(erro)?;
    if !form.name().as_utf8_str().eq_ignore_ascii_case("form")
        || valor_do_atributo(form, "method")
            .map_err(FalhaCadastroExterno::Atributo)?
            .as_deref()
            != Some("post")
    {
        return Err(erro());
    }
    let selects = nos_com_id(&dom, "selSerie").map_err(FalhaCadastroExterno::Atributo)?;
    let [no] = selects.as_slice() else {
        return Err(erro());
    };
    let select = no
        .get(dom.parser())
        .and_then(tl::Node::as_tag)
        .ok_or_else(erro)?;
    let (ini, fim) = form.boundaries(dom.parser());
    let (start, end) = select.boundaries(dom.parser());
    if !select.name().as_utf8_str().eq_ignore_ascii_case("select")
        || start <= ini
        || end > fim
        || atributo_presente(select, "disabled").map_err(FalhaCadastroExterno::Atributo)?
        || valor_do_atributo(select, "name")
            .map_err(FalhaCadastroExterno::Atributo)?
            .as_deref()
            != Some("selSerie")
    {
        return Err(erro());
    }
    let mut candidatos = Vec::new();
    for tag in select
        .children()
        .all(dom.parser())
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        if !tag.name().as_utf8_str().eq_ignore_ascii_case("option") {
            continue;
        }
        // Texto de <option>: `&nbsp;` e entidades Latin-1 servidas pela UnB.
        let nome = html_escape::decode_html_entities(&tag.inner_text(dom.parser()))
            .into_owned()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if nome != tipo
            || atributo_presente(tag, "disabled").map_err(FalhaCadastroExterno::Atributo)?
        {
            continue;
        }
        let id = valor_do_atributo(tag, "value")
            .map_err(FalhaCadastroExterno::Atributo)?
            .ok_or_else(erro)?;
        let selecionado =
            atributo_presente(tag, "selected").map_err(FalhaCadastroExterno::Atributo)?;
        candidatos.push((id, selecionado));
    }
    let (id_tipo, selecionado) = match candidatos.as_slice() {
        [] => return Err(FalhaCadastroExterno::TipoNaoOferecido),
        [tipo] => tipo,
        _ => return Err(FalhaCadastroExterno::TipoAmbiguo),
    };
    if id_tipo == "null" || id_tipo.is_empty() {
        return Err(erro());
    }
    if valor_input(&dom, form, "hdnIdSerie")? != *id_tipo || !selecionado {
        return Err(FalhaCadastroExterno::TipoNaoRecarregado);
    }
    valor_input(&dom, form, "hdnAnexos")?;
    valor_input(&dom, form, "hdnFlagDocumentoCadastro")?;
    let limite_pdf_bytes = limite_pdf(html, &dom)?;
    if tamanho_pdf > limite_pdf_bytes {
        return Err(FalhaCadastroExterno::PdfAcimaDoLimite);
    }
    let plano = super::anexo::ler_plano(html).map_err(FalhaCadastroExterno::PlanoAnexo)?;
    let campos_oferecidos = campos_oferecidos(&dom, form)?;
    Ok(CadastroExterno {
        id_tipo: id_tipo.clone(),
        limite_pdf_bytes,
        plano,
        campos_oferecidos,
    })
}

#[cfg(test)]
#[path = "tests_cadastro.rs"]
mod tests;
