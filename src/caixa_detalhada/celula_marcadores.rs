//! Célula Marcadores da tabela D: um `a.ancMarcador` por marcador (forma
//! da `UnB` 4.0.9). Lista sempre; nomes nunca concatenados. Leitura pela
//! árvore do `tl` e pelo tokenizador de atributos, sem regex nem JavaScript.
use super::ErroDetalhada;
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::atributos;
use crate::marcadores_processo::rotulo_do_marcador;
use crate::nivel_de_detalhe::texto_colapsado;
use serde::Serialize;

/// Um marcador da célula, na ordem servida.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct MarcadorDaCelula {
    pub(crate) nome: String,
    pub(crate) cor: String,
    /// Texto do tooltip; `None` quando vazio ou sem `onmouseover`.
    pub(crate) texto: Option<String>,
    /// `<n>` de `#ID-<n>`; `None` sem fragmento.
    pub(crate) id: Option<String>,
    /// `<id>#<nome>`; `None` sem id (o nome sozinho seria ambíguo).
    pub(crate) rotulo: Option<String>,
}

const FORA: ErroDetalhada = ErroDetalhada::MarcadorDaCelulaDetalhadaForaDaFormaObservada;
const SOLTO: ErroDetalhada = ErroDetalhada::TextoForaDosMarcadoresNaCelula;

/// Filhos diretos de `tag` que são elementos; texto só de espaços (inclusive
/// `&nbsp;`) e comentários são ignorados, outro texto é `erro`.
fn filhos<'d>(
    tag: &tl::HTMLTag<'_>,
    parser: &'d tl::Parser<'d>,
    erro: ErroDetalhada,
) -> Result<Vec<&'d tl::HTMLTag<'d>>, ErroDetalhada> {
    let mut elementos = Vec::new();
    for no in tag.children().top().iter().filter_map(|h| h.get(parser)) {
        match no {
            tl::Node::Tag(filho) => elementos.push(filho),
            tl::Node::Raw(_) => {
                if !texto_colapsado(&no.inner_text(parser)).is_empty() {
                    return Err(erro);
                }
            }
            tl::Node::Comment(_) => {}
        }
    }
    Ok(elementos)
}

fn atributo(tag: &tl::HTMLTag<'_>, nome: &'static str) -> Result<Option<String>, ErroDetalhada> {
    let bruto = atributos(tag)
        .map_err(ErroDetalhada::AtributoDaDetalhadaIlegivel)?
        .valor(nome)
        .map_err(ErroDetalhada::AtributoDaDetalhadaIlegivel)?
        .map(str::to_owned);
    bruto
        .map(|v| decodificar_atributo(&v).ok_or(FORA))
        .transpose()
}

/// `controlador.php?` com uma `acao=andamento_marcador_gerenciar` e um
/// `id_procedimento` igual ao da linha; devolve o id de `#ID-<n>`.
fn id_do_href(href: &str, id_procedimento: &str) -> Result<Option<String>, ErroDetalhada> {
    let (endereco, fragmento) = match href.split_once('#') {
        Some((endereco, fragmento)) => (endereco, Some(fragmento)),
        None => (href, None),
    };
    let query = endereco.strip_prefix("controlador.php?").ok_or(FORA)?;
    let campos: Vec<_> = query.split('&').collect();
    let valores = |chave: &str| -> Vec<&str> {
        campos
            .iter()
            .filter_map(|c| c.strip_prefix(chave)?.strip_prefix('='))
            .collect()
    };
    if valores("acao") != ["andamento_marcador_gerenciar"]
        || valores("id_procedimento") != [id_procedimento]
    {
        return Err(FORA);
    }
    let Some(fragmento) = fragmento else {
        return Ok(None);
    };
    let id = fragmento.strip_prefix("ID-").ok_or(FORA)?;
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
        return Err(FORA);
    }
    Ok(Some(id.to_owned()))
}

/// O literal de `return infraTooltipMostrar('<literal>');`, com os escapes
/// de um literal JS simples; qualquer outra forma é recusa.
fn texto_do_tooltip(onmouseover: &str) -> Result<String, ErroDetalhada> {
    let resto = onmouseover
        .strip_prefix("return infraTooltipMostrar('")
        .ok_or(FORA)?;
    let mut texto = String::new();
    let mut caracteres = resto.char_indices();
    while let Some((posicao, caractere)) = caracteres.next() {
        match caractere {
            '\'' => {
                return if &resto[posicao + 1..] == ");" {
                    Ok(texto)
                } else {
                    Err(FORA)
                };
            }
            '\\' => texto.push(match caracteres.next().ok_or(FORA)?.1 {
                c @ ('"' | '\'' | '\\' | '/') => c,
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                _ => return Err(FORA),
            }),
            c => texto.push(c),
        }
    }
    Err(FORA)
}

/// `<cor>` de `svg/marcador_<cor>.svg` (com `?…` opcional).
fn cor(src: &str) -> Result<String, ErroDetalhada> {
    let resto = src.strip_prefix("svg/marcador_").ok_or(FORA)?;
    let (cor, depois) = resto.split_once(".svg").ok_or(FORA)?;
    if cor.is_empty()
        || !cor.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')
        || !(depois.is_empty() || depois.starts_with('?'))
    {
        return Err(FORA);
    }
    Ok(cor.to_owned())
}

fn marcador(
    ancora: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    id_procedimento: &str,
) -> Result<MarcadorDaCelula, ErroDetalhada> {
    if atributo(ancora, "class")?.as_deref() != Some("ancMarcador") {
        return Err(FORA);
    }
    let id = id_do_href(&atributo(ancora, "href")?.ok_or(FORA)?, id_procedimento)?;
    let texto = atributo(ancora, "onmouseover")?
        .map(|v| texto_do_tooltip(&v))
        .transpose()?
        .map(|t| texto_colapsado(&t))
        .filter(|t| !t.is_empty());
    let [icone, rotulo] = filhos(ancora, parser, FORA)?[..] else {
        return Err(FORA);
    };
    if icone.name() != "div" || rotulo.name() != "div" {
        return Err(FORA);
    }
    let [img] = filhos(icone, parser, FORA)?[..] else {
        return Err(FORA);
    };
    if img.name() != "img" {
        return Err(FORA);
    }
    let cor = cor(&atributo(img, "src")?.ok_or(FORA)?)?;
    let so_texto = rotulo
        .children()
        .top()
        .iter()
        .filter_map(|h| h.get(parser))
        .all(|no| matches!(no, tl::Node::Raw(_)));
    if !so_texto {
        return Err(FORA);
    }
    let bruto = rotulo.inner_text(parser);
    let nome = texto_colapsado(&decodificar_atributo(&bruto).ok_or(FORA)?);
    if nome.is_empty() {
        return Err(FORA);
    }
    let rotulo = id.as_deref().map(|id| rotulo_do_marcador(id, &nome));
    Ok(MarcadorDaCelula {
        nome,
        cor,
        texto,
        id,
        rotulo,
    })
}

/// Um objeto por `a` da célula; texto ou tag fora das âncoras é recusa.
pub(super) fn marcadores_da_celula(
    celula: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    id_procedimento: &str,
) -> Result<Vec<MarcadorDaCelula>, ErroDetalhada> {
    let mut marcadores = Vec::new();
    for filho in filhos(celula, parser, SOLTO)? {
        if filho.name() != "a" {
            return Err(SOLTO);
        }
        marcadores.push(marcador(filho, parser, id_procedimento)?);
    }
    Ok(marcadores)
}
