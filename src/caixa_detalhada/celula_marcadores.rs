//! Célula Marcadores da tabela D: um `a.ancMarcador` por marcador (forma
//! da `UnB` 4.0.9). Lista sempre; nomes nunca concatenados. Leitura pela
//! árvore `tela_html`, sem regex nem JavaScript.
use super::ErroDetalhada;
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::marcadores_processo::rotulo_do_marcador;
use crate::nivel_de_detalhe::texto_colapsado;
use serde::Serialize;

/// Um marcador da célula, na ordem servida.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct MarcadorDaCelula {
    pub(crate) nome: String,
    pub(crate) cor: String,
    /// Texto do tooltip; `None` quando vazio ou sem `onmouseover`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) texto: Option<String>,
    /// `<n>` de `#ID-<n>`; `None` sem fragmento.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<String>,
    /// `<id>#<nome>`; `None` sem id (o nome sozinho seria ambíguo).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) rotulo: Option<String>,
}

const FORA: ErroDetalhada = ErroDetalhada::MarcadorDaCelulaDetalhadaForaDaFormaObservada;
const SOLTO: ErroDetalhada = ErroDetalhada::TextoForaDosMarcadoresNaCelula;

/// Filhos diretos de `tag` que são elementos; texto só de espaços (inclusive
/// `&nbsp;`) e comentários são ignorados, outro texto é `erro`.
fn filhos(
    tag: crate::tela_html::Elemento<'_>,
    erro: ErroDetalhada,
) -> Result<Vec<crate::tela_html::Elemento<'_>>, ErroDetalhada> {
    let mut elementos = Vec::new();
    for no in tag.nos() {
        match no {
            crate::tela_html::No::Elemento(filho) => elementos.push(filho),
            crate::tela_html::No::Texto(texto) => {
                if !texto_colapsado(texto).is_empty() {
                    return Err(erro);
                }
            }
            crate::tela_html::No::Comentario => {}
        }
    }
    Ok(elementos)
}

/// `controlador.php?` com uma `acao=andamento_marcador_gerenciar` e um
/// `id_procedimento` igual ao da linha; devolve o id de `#ID-<n>`.
fn id_do_href(href: &str, id_procedimento: &str) -> Result<Option<String>, ErroDetalhada> {
    let (endereco, fragmento) = match href.split_once('#') {
        Some((endereco, fragmento)) => (endereco, Some(fragmento)),
        None => (href, None),
    };
    let query = endereco.strip_prefix("controlador.php?").ok_or(FORA)?;
    let consulta =
        ConsultaDoControlador::ler(query).map_err(ErroDetalhada::ConsultaDoMarcadorIlegivel)?;
    let acao = consulta
        .acao()
        .map_err(ErroDetalhada::AcaoDoMarcadorDetalhado)?;
    let id_lido = consulta
        .unico("id_procedimento")
        .map_err(ErroDetalhada::ChaveDoMarcadorDetalhado)?;
    if acao != AcaoDoSei::AndamentoMarcadorGerenciar.como_na_url() || id_lido != id_procedimento {
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
    while let Some((_, caractere)) = caracteres.next() {
        match caractere {
            '\'' => {
                return if caracteres.as_str() == ");" {
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
    ancora: crate::tela_html::Elemento<'_>,
    id_procedimento: &str,
) -> Result<MarcadorDaCelula, ErroDetalhada> {
    if ancora.atributo("class") != Some("ancMarcador") {
        return Err(FORA);
    }
    let id = id_do_href(ancora.atributo("href").ok_or(FORA)?, id_procedimento)?;
    let texto = ancora
        .atributo("onmouseover")
        .map(texto_do_tooltip)
        .transpose()?
        .map(|t| texto_colapsado(&t))
        .filter(|t| !t.is_empty());
    let [icone, rotulo] = filhos(ancora, FORA)?[..] else {
        return Err(FORA);
    };
    if icone.nome() != "div" || rotulo.nome() != "div" {
        return Err(FORA);
    }
    let [img] = filhos(icone, FORA)?[..] else {
        return Err(FORA);
    };
    if img.nome() != "img" {
        return Err(FORA);
    }
    let cor = cor(img.atributo("src").ok_or(FORA)?)?;
    let so_texto = rotulo
        .nos()
        .all(|no| matches!(no, crate::tela_html::No::Texto(_)));
    if !so_texto {
        return Err(FORA);
    }
    let bruto = rotulo.texto();
    let nome = texto_colapsado(&bruto);
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
    celula: crate::tela_html::Elemento<'_>,
    id_procedimento: &str,
) -> Result<Vec<MarcadorDaCelula>, ErroDetalhada> {
    let mut marcadores = Vec::new();
    for filho in filhos(celula, SOLTO)? {
        if filho.nome() != "a" {
            return Err(SOLTO);
        }
        marcadores.push(marcador(filho, id_procedimento)?);
    }
    Ok(marcadores)
}
