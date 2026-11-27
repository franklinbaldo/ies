//! Prévia da oferta de documento modelo manual, sem resolver ou abrir a fonte.
use crate::documento::NumeroDocumento;
use crate::formulario_web::localizar_formulario;
use crate::tela_html::HtmlServido;

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroModelo {
    Formulario(#[from] crate::formulario_web::ErroSubmissaoFormulario),
    ControlesNaoOferecidos,
    NumeroExcedeLimite,
}

/// Perfil histórico Todos caabe699, documento_gerar.html, L838–866.
/// O número lexical não é o ID oculto retornado pela lupa de favoritos.
pub(super) fn conferir(
    html: &HtmlServido,
    numero: &NumeroDocumento,
) -> Result<String, ErroModelo> {
    let dom = crate::tela_html::arvore(html);
    let form = localizar_formulario(&dom, "frmDocumentoCadastro")?;
    let radios: Vec<_> = form
        .descendentes()
        .filter(|tag| tag.atributo("name") == Some("rdoTextoInicial"))
        .collect();
    if radios.len() != 3
        || !["N", "D", "T"].iter().all(|valor| {
            radios
                .iter()
                .filter(|tag| {
                    tag.nome() == "input"
                        && tag.atributo("type") == Some("radio")
                        && tag.atributo("value") == Some(*valor)
                        && !tag.presente("disabled")
                })
                .count()
                == 1
        })
    {
        return Err(ErroModelo::ControlesNaoOferecidos);
    }
    let mut limite = None;
    for (nome, tipo) in [
        ("txtProtocoloDocumentoTextoBase", "text"),
        ("txtTextoPadrao", "text"),
        ("hdnIdDocumentoTextoBase", "hidden"),
        ("hdnIdTextoPadrao", "hidden"),
    ] {
        let campos: Vec<_> = form
            .descendentes()
            .filter(|tag| tag.atributo("name") == Some(nome))
            .collect();
        let [campo] = campos.as_slice() else {
            return Err(ErroModelo::ControlesNaoOferecidos);
        };
        if campo.nome() != "input"
            || campo.atributo("type") != Some(tipo)
            || campo.presente("disabled")
            || campo.atributo("value").is_some_and(|v| !v.is_empty())
        {
            return Err(ErroModelo::ControlesNaoOferecidos);
        }
        if nome == "txtProtocoloDocumentoTextoBase" {
            limite = campo
                .atributo("maxlength")
                .and_then(|v| v.parse::<usize>().ok());
        }
    }
    let limite = limite.ok_or(ErroModelo::ControlesNaoOferecidos)?;
    if numero.as_str().len() > limite {
        return Err(ErroModelo::NumeroExcedeLimite);
    }
    Ok(numero.as_str().to_owned())
}
