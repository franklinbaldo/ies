//! Controles não ocultos de `frmProcedimentoControlar`, lidos como o
//! navegador os enviaria num `submit()` por script. Inventário estrutural de
//! 02/11/2026 (PEN1, PEN3, PEN4, Goiás, UnB): `txtBloco`, caixas de seleção
//! `chk{T}Item*` desmarcadas e, quando há várias páginas, `sel{T}Paginacao*`.
//! Nada de JavaScript é executado ou interpretado aqui.
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::{ErroAtributoHtml, atributo_presente, valor_do_atributo};
use ErroControleDoFormulario as E;

/// Únicos controles de usuário que o POST do paginador repete.
pub(super) const CAMPOS_USUARIO: &[&str] = &[
    "txtBloco",
    "selRecebidosPaginacaoSuperior",
    "selRecebidosPaginacaoInferior",
    "selGeradosPaginacaoSuperior",
    "selGeradosPaginacaoInferior",
];

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroControleDoFormulario {
    ControleNaoPrevistoNoFormularioDoControle { campo: String },
    CaixaDeSelecaoMarcadaNoFormularioDoControle { campo: String },
    PaginadorSemOpcaoSelecionada { campo: String },
    PaginadorComMaisDeUmaOpcaoSelecionada { campo: String },
    OpcaoSelecionadaDoPaginadorSemValor { campo: String },
    ControleRepetidoNoFormularioDoControle { campo: String },
    EntidadeHtmlDesconhecidaNoControle { campo: String },
    AtributoHtml(#[from] ErroAtributoHtml),
}

fn atributo(tag: &tl::HTMLTag<'_>, nome: &'static str) -> Result<Option<String>, E> {
    valor_do_atributo(tag, nome).map_err(E::AtributoHtml)
}

fn decodificado(bruto: &str, campo: &str) -> Result<String, E> {
    decodificar_atributo(bruto).ok_or_else(|| E::EntidadeHtmlDesconhecidaNoControle {
        campo: campo.to_owned(),
    })
}

/// Valor que o navegador envia para um `<select>` simples: a única opção
/// marcada `selected`. Nenhuma, várias ou sem `value` são recusas.
fn opcao_selecionada(
    select: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    campo: &str,
) -> Result<String, E> {
    let mut escolhidas = Vec::new();
    for opcao in select
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
        .filter(|t| t.name().as_utf8_str().eq_ignore_ascii_case("option"))
    {
        if atributo_presente(opcao, "selected")? {
            escolhidas.push(atributo(opcao, "value")?);
        }
    }
    let campo_owned = || campo.to_owned();
    match escolhidas.as_slice() {
        [] => Err(E::PaginadorSemOpcaoSelecionada {
            campo: campo_owned(),
        }),
        [Some(valor)] => decodificado(valor, campo),
        [None] => Err(E::OpcaoSelecionadaDoPaginadorSemValor {
            campo: campo_owned(),
        }),
        _ => Err(E::PaginadorComMaisDeUmaOpcaoSelecionada {
            campo: campo_owned(),
        }),
    }
}

/// Pares (nome, valor) dos controles de usuário, na ordem do documento.
/// Ocultos ficam com o primitivo; botões não participam de `submit()`;
/// caixas desmarcadas não são enviadas; qualquer outro controle recusa.
/// `permitidos` são os `select` que a operação repete (seus `CAMPOS_USUARIO`).
pub(crate) fn campos_do_usuario(
    form: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    permitidos: &[&str],
) -> Result<Vec<(String, String)>, E> {
    let mut campos: Vec<(String, String)> = Vec::new();
    for tag in form
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        let elemento = tag.name().as_utf8_str().to_ascii_lowercase();
        if !matches!(
            elemento.as_str(),
            "input" | "select" | "textarea" | "button"
        ) || atributo_presente(tag, "disabled")?
        {
            continue;
        }
        let Some(bruto) = atributo(tag, "name")?.filter(|n| !n.is_empty()) else {
            continue;
        };
        let nome = decodificado(&bruto, &bruto)?;
        let tipo =
            atributo(tag, "type")?.map_or_else(|| "text".to_owned(), |t| t.to_ascii_lowercase());
        let valor = match (elemento.as_str(), tipo.as_str()) {
            ("input", "hidden" | "submit" | "button" | "reset" | "image") | ("button", _) => {
                continue;
            }
            ("input", "checkbox" | "radio") => {
                if atributo_presente(tag, "checked")? {
                    return Err(E::CaixaDeSelecaoMarcadaNoFormularioDoControle { campo: nome });
                }
                continue;
            }
            ("input", "text") if nome == "txtBloco" => {
                decodificado(&atributo(tag, "value")?.unwrap_or_default(), &nome)?
            }
            ("select", _)
                if permitidos.contains(&nome.as_str()) && !atributo_presente(tag, "multiple")? =>
            {
                opcao_selecionada(tag, parser, &nome)?
            }
            _ => return Err(E::ControleNaoPrevistoNoFormularioDoControle { campo: nome }),
        };
        if campos.iter().any(|(anterior, _)| *anterior == nome) {
            return Err(E::ControleRepetidoNoFormularioDoControle { campo: nome });
        }
        campos.push((nome, valor));
    }
    Ok(campos)
}
