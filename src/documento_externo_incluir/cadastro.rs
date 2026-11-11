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
    /// `hdnIdSerie` ausente ou repetido: o tipo recarregado não é lido.
    TipoRecarregadoAusenteOuRepetido,
    /// `hdnIdSerie` sem `value`.
    ValorDoTipoRecarregadoAusente,
    FormularioDeCadastroAusenteOuRepetido,
    SelectDeSerieAusenteOuRepetido,
    OpcaoDoTipoSemValor,
    IdDoTipoInvalido,
    /// A declaração `arrExt[..] = {nome:..,tamanho:..};` não traz nome e tamanho.
    LinhaDoLimiteIlegivel,
    LimiteDoPdfEstourou,
    LimitePdfRepetido,
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

/// O `value` de `hdnIdSerie`: o tipo que a tela recarregou. A forma dos
/// campos ocultos é conferida pela derivação do formulário na escrita.
fn tipo_recarregado(dom: &tl::VDom<'_>) -> Result<String, FalhaCadastroExterno> {
    let nos = nos_com_id(dom, "hdnIdSerie").map_err(FalhaCadastroExterno::Atributo)?;
    let tags: Vec<_> = nos
        .iter()
        .filter_map(|no| no.get(dom.parser()).and_then(tl::Node::as_tag))
        .collect();
    let [tag] = tags.as_slice() else {
        return Err(FalhaCadastroExterno::TipoRecarregadoAusenteOuRepetido);
    };
    valor_do_atributo(tag, "value")
        .map_err(FalhaCadastroExterno::Atributo)?
        .ok_or(FalhaCadastroExterno::ValorDoTipoRecarregadoAusente)
}

/// Corpo de um `<script>` inline a partir da posição da tag.
fn corpo_do_script(html: &str, inicio: usize) -> Option<&str> {
    let restante = html.get(inicio..)?;
    let corpo = restante.find('>')? + 1;
    let fim = corpo + restante.get(corpo..)?.find("</script>")?;
    restante.get(corpo..fim)
}

/// `nome` e `tamanho` (MB) de `arrExt[i] = {nome:"pdf", tamanho:N};`.
fn limite_da_linha(linha: &str) -> Result<(&str, u64), FalhaCadastroExterno> {
    let ilegivel = || FalhaCadastroExterno::LinhaDoLimiteIlegivel;
    let (_, objeto) = linha.split_once("= {").ok_or_else(ilegivel)?;
    let (objeto, _) = objeto.split_once('}').ok_or_else(ilegivel)?;
    let mut nome = None;
    let mut tamanho = None;
    for par in objeto.split(',') {
        let (chave, valor) = par.split_once(':').ok_or_else(ilegivel)?;
        match chave.trim() {
            "nome" => nome = Some(valor.trim().trim_matches('"')),
            "tamanho" => tamanho = Some(valor.trim()),
            _ => {}
        }
    }
    let (Some(nome), Some(tamanho)) = (nome, tamanho) else {
        return Err(ilegivel());
    };
    let tamanho = tamanho
        .parse::<u64>()
        .map_err(FalhaCadastroExterno::TamanhoDeclaradoIlegivel)?;
    Ok((nome, tamanho))
}

/// Lê a declaração literal da extensão PDF no script do upload da tela.
/// Nenhum código JavaScript é executado.
fn limite_pdf(html: &str, dom: &tl::VDom<'_>) -> Result<u64, FalhaCadastroExterno> {
    let mut limites = Vec::new();
    for tag in dom
        .query_selector("script")
        .into_iter()
        .flatten()
        .filter_map(|no| no.get(dom.parser()).and_then(tl::Node::as_tag))
    {
        if atributo_presente(tag, "src").map_err(FalhaCadastroExterno::Atributo)? {
            continue;
        }
        let Some(fonte) = corpo_do_script(html, tag.boundaries(dom.parser()).0) else {
            continue;
        };
        for linha in fonte
            .lines()
            .map(str::trim)
            .filter(|linha| linha.starts_with("arrExt[") && linha.contains("= {"))
        {
            let (extensao, tamanho_mb) = limite_da_linha(linha)?;
            if extensao == "pdf" {
                limites.push(
                    tamanho_mb
                        .checked_mul(1024 * 1024)
                        .ok_or(FalhaCadastroExterno::LimiteDoPdfEstourou)?,
                );
            }
        }
    }
    match limites.as_slice() {
        [unico] => Ok(*unico),
        [] => Err(FalhaCadastroExterno::PdfNaoOferecido),
        _ => Err(FalhaCadastroExterno::LimitePdfRepetido),
    }
}

/// Controles de usuário realmente publicados dentro do cadastro ativo.
/// Não expõe valores, campos ocultos nem controles desabilitados.
fn campos_oferecidos(
    dom: &tl::VDom<'_>,
    formulario: &tl::HTMLTag<'_>,
) -> Result<Vec<String>, FalhaCadastroExterno> {
    let (inicio_form, fim_form) = formulario.boundaries(dom.parser());
    let mut encontrados = Vec::new();
    for elemento in ["input", "select", "textarea"] {
        for tag in dom
            .query_selector(elemento)
            .into_iter()
            .flatten()
            .filter_map(|no| no.get(dom.parser()).and_then(tl::Node::as_tag))
        {
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
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(FalhaCadastroExterno::HtmlNaoAnalisavel)?;
    let formularios =
        nos_com_id(&dom, "frmDocumentoCadastro").map_err(FalhaCadastroExterno::Atributo)?;
    let forms: Vec<_> = formularios
        .iter()
        .filter_map(|no| no.get(dom.parser()).and_then(tl::Node::as_tag))
        .collect();
    let [form] = forms.as_slice() else {
        return Err(FalhaCadastroExterno::FormularioDeCadastroAusenteOuRepetido);
    };
    let selects = nos_com_id(&dom, "selSerie").map_err(FalhaCadastroExterno::Atributo)?;
    let selects: Vec<_> = selects
        .iter()
        .filter_map(|no| no.get(dom.parser()).and_then(tl::Node::as_tag))
        .collect();
    let [select] = selects.as_slice() else {
        return Err(FalhaCadastroExterno::SelectDeSerieAusenteOuRepetido);
    };
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
            .ok_or(FalhaCadastroExterno::OpcaoDoTipoSemValor)?;
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
        return Err(FalhaCadastroExterno::IdDoTipoInvalido);
    }
    if tipo_recarregado(&dom)? != *id_tipo || !selecionado {
        return Err(FalhaCadastroExterno::TipoNaoRecarregado);
    }
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
