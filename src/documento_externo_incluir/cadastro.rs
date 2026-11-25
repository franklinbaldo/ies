//! Cadastro externo recarregado por tipo — observação estrutural (#332).
//! Não envia upload nem cadastra documentos.
use super::anexo::{FalhaAnexo, PlanoAnexo};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaCadastroExterno {
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
fn tipo_recarregado(dom: &crate::tela_html::Documento) -> Result<String, FalhaCadastroExterno> {
    let tags = dom.com_id("hdnIdSerie").collect::<Vec<_>>();
    let [tag] = tags.as_slice() else {
        return Err(FalhaCadastroExterno::TipoRecarregadoAusenteOuRepetido);
    };
    tag.atributo("value")
        .map(str::to_owned)
        .ok_or(FalhaCadastroExterno::ValorDoTipoRecarregadoAusente)
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
fn limite_pdf(dom: &crate::tela_html::Documento) -> Result<u64, FalhaCadastroExterno> {
    let mut limites = Vec::new();
    for tag in dom.elementos("script") {
        if tag.presente("src") {
            continue;
        }
        let fonte = tag.texto();
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

/// Controles de usuário realmente publicados dentro do cadastro ativo,
/// na ordem do documento. Não expõe valores, campos ocultos nem controles
/// desabilitados.
fn campos_oferecidos(formulario: crate::tela_html::Elemento<'_>) -> Vec<String> {
    let mut nomes: Vec<String> = Vec::new();
    for tag in formulario.descendentes() {
        let elemento = tag.nome();
        if !matches!(elemento, "input" | "select" | "textarea") || tag.presente("disabled") {
            continue;
        }
        let Some(nome) = tag.atributo("name").filter(|nome| !nome.is_empty()) else {
            continue;
        };
        if elemento == "input"
            && matches!(
                tag.atributo("type"),
                Some("hidden" | "file" | "submit" | "button" | "reset" | "image")
            )
        {
            continue;
        }
        if !nomes.iter().any(|anterior| anterior == nome) {
            nomes.push(nome.to_owned());
        }
    }
    nomes
}

/// Inspeciona o cadastro servido depois da escolha do tipo (sem escrita).
/// A saída mantém o destino de upload somente em memória.
pub(crate) fn observar_cadastro(
    html: &crate::tela_html::HtmlServido,
    tipo: &str,
    tamanho_pdf: u64,
) -> Result<CadastroExterno, FalhaCadastroExterno> {
    let dom = crate::tela_html::arvore(html);
    let forms = dom.com_id("frmDocumentoCadastro").collect::<Vec<_>>();
    let [form] = forms.as_slice() else {
        return Err(FalhaCadastroExterno::FormularioDeCadastroAusenteOuRepetido);
    };
    let selects = dom.com_id("selSerie").collect::<Vec<_>>();
    let [select] = selects.as_slice() else {
        return Err(FalhaCadastroExterno::SelectDeSerieAusenteOuRepetido);
    };
    let mut candidatos = Vec::new();
    for tag in select.descendentes() {
        if !tag.nome().eq_ignore_ascii_case("option") {
            continue;
        }
        // Texto de <option>: `&nbsp;` e entidades Latin-1 servidas pela UnB.
        let nome = tag.texto().split_whitespace().collect::<Vec<_>>().join(" ");
        if nome != tipo || tag.presente("disabled") {
            continue;
        }
        let id = tag
            .atributo("value")
            .map(str::to_owned)
            .ok_or(FalhaCadastroExterno::OpcaoDoTipoSemValor)?;
        let selecionado = tag.presente("selected");
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
    let limite_pdf_bytes = limite_pdf(&dom)?;
    if tamanho_pdf > limite_pdf_bytes {
        return Err(FalhaCadastroExterno::PdfAcimaDoLimite);
    }
    let plano = super::anexo::ler_plano(html).map_err(FalhaCadastroExterno::PlanoAnexo)?;
    let campos_oferecidos = campos_oferecidos(*form);
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
