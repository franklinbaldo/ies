//! Controles não ocultos de `frmProcedimentoControlar`, lidos como o
//! navegador os enviaria num `submit()` por script. Inventário estrutural de
//! 02/11/2026 (PEN1, PEN3, PEN4, Goiás, UnB): `txtBloco`, caixas de seleção
//! `chk{T}Item*` desmarcadas e, quando há várias páginas, `sel{T}Paginacao*`.
//! Nada de JavaScript é executado ou interpretado aqui.
use crate::html_atributos::ErroAtributoHtml;

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
    AtributoHtml(#[from] ErroAtributoHtml),
}

/// Valor que o navegador envia para um `<select>` simples: a única opção
/// marcada `selected`. Nenhuma, várias ou sem `value` são recusas.
fn opcao_selecionada(
    select: crate::tela_html::Elemento<'_>,
    campo: &str,
) -> Result<String, ErroControleDoFormulario> {
    let mut escolhidas = Vec::new();
    for opcao in select
        .descendentes()
        .filter(|t| t.nome().eq_ignore_ascii_case("option"))
    {
        if opcao.presente("selected") {
            escolhidas.push(opcao.atributo("value").map(str::to_owned));
        }
    }
    let campo_owned = || campo.to_owned();
    match escolhidas.as_slice() {
        [] => Err(ErroControleDoFormulario::PaginadorSemOpcaoSelecionada {
            campo: campo_owned(),
        }),
        [Some(valor)] => Ok(valor.clone()),
        [None] => Err(
            ErroControleDoFormulario::OpcaoSelecionadaDoPaginadorSemValor {
                campo: campo_owned(),
            },
        ),
        _ => Err(
            ErroControleDoFormulario::PaginadorComMaisDeUmaOpcaoSelecionada {
                campo: campo_owned(),
            },
        ),
    }
}

/// Pares (nome, valor) dos controles de usuário, na ordem do documento.
/// Ocultos ficam com o primitivo; botões não participam de `submit()`;
/// caixas desmarcadas não são enviadas; qualquer outro controle recusa.
/// `permitidos` são os `select` que a operação repete (seus `CAMPOS_USUARIO`).
pub(crate) fn campos_do_usuario(
    form: crate::tela_html::Elemento<'_>,
    permitidos: &[&str],
) -> Result<Vec<(String, String)>, ErroControleDoFormulario> {
    let mut campos: Vec<(String, String)> = Vec::new();
    for tag in form.descendentes() {
        let elemento = tag.nome().to_ascii_lowercase();
        if !matches!(
            elemento.as_str(),
            "input" | "select" | "textarea" | "button"
        ) || tag.presente("disabled")
        {
            continue;
        }
        let Some(bruto) = tag
            .atributo("name")
            .map(str::to_owned)
            .filter(|n| !n.is_empty())
        else {
            continue;
        };
        let nome = bruto;
        let tipo = tag
            .atributo("type")
            .map(str::to_owned)
            .map_or_else(|| "text".to_owned(), |t| t.to_ascii_lowercase());
        let valor = match (elemento.as_str(), tipo.as_str()) {
            ("input", "hidden" | "submit" | "button" | "reset" | "image") | ("button", _) => {
                continue;
            }
            ("input", "checkbox" | "radio") => {
                if tag.presente("checked") {
                    return Err(
                        ErroControleDoFormulario::CaixaDeSelecaoMarcadaNoFormularioDoControle {
                            campo: nome,
                        },
                    );
                }
                continue;
            }
            ("input", "text") if nome == "txtBloco" => {
                tag.atributo("value").unwrap_or_default().to_owned()
            }
            ("select", _) if permitidos.contains(&nome.as_str()) && !tag.presente("multiple") => {
                opcao_selecionada(tag, &nome)?
            }
            _ => {
                return Err(
                    ErroControleDoFormulario::ControleNaoPrevistoNoFormularioDoControle {
                        campo: nome,
                    },
                );
            }
        };
        if campos.iter().any(|(anterior, _)| *anterior == nome) {
            return Err(
                ErroControleDoFormulario::ControleRepetidoNoFormularioDoControle { campo: nome },
            );
        }
        campos.push((nome, valor));
    }
    Ok(campos)
}
