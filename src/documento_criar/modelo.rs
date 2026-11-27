//! Prévia da oferta de documento modelo manual, sem resolver ou abrir a fonte.
use crate::documento::NumeroDocumento;
use crate::formulario_web::localizar_formulario;
use crate::tela_html::{Documento, Elemento, HtmlServido};

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroModelo {
    Formulario(#[from] crate::formulario_web::ErroSubmissaoFormulario),
    ControlesNaoOferecidos,
    NumeroExcedeLimite,
}

/// Perfil histórico Todos caabe699, documento_gerar.html, L838–866.
/// O número lexical não é o ID oculto retornado pela lupa de favoritos.
pub(super) fn conferir(html: &HtmlServido, numero: &NumeroDocumento) -> Result<String, ErroModelo> {
    let dom = crate::tela_html::arvore(html);
    let form = localizar_formulario(&dom, "frmDocumentoCadastro")?;
    // O perfil observado usa controles internos, sem associação explícita.
    // Um controle externo associado ao cadastro também participa do envio.
    if let Some(id) = form.atributo("id")
        && ["input", "select", "textarea", "button"].iter().any(|tipo| {
            dom.elementos(tipo).any(|tag| {
                tag.atributo("form") == Some(id)
                    && matches!(
                        tag.atributo("name"),
                        Some(
                            "rdoTextoInicial"
                                | "txtProtocoloDocumentoTextoBase"
                                | "txtTextoPadrao"
                                | "hdnIdDocumentoTextoBase"
                                | "hdnIdTextoPadrao"
                        )
                    )
            })
        })
    {
        return Err(ErroModelo::ControlesNaoOferecidos);
    }
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
                        && controle_disponivel(&dom, **tag)
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
            || !controle_disponivel(&dom, *campo)
            || (tipo == "text" && campo.presente("readonly"))
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

// O primeiro legend de cada fieldset disabled é exceção no HTML.
fn controle_disponivel(dom: &Documento, tag: Elemento<'_>) -> bool {
    !tag.presente("disabled")
        && !tag.presente("form")
        && !dom.elementos("fieldset").any(|fieldset| {
            fieldset.presente("disabled")
                && fieldset.descendentes().any(|filho| filho == tag)
                && !fieldset
                    .filhos()
                    .find(|filho| filho.nome() == "legend")
                    .is_some_and(|legend| legend.descendentes().any(|filho| filho == tag))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const PERFIL: &str = r#"<form id="frmDocumentoCadastro">
      <input type="radio" name="rdoTextoInicial" value="D">
      <input type="radio" name="rdoTextoInicial" value="T">
      <input type="radio" name="rdoTextoInicial" value="N">
      <input type="text" name="txtProtocoloDocumentoTextoBase" maxlength="7">
      <input type="text" name="txtTextoPadrao">
      <input type="hidden" name="hdnIdDocumentoTextoBase" value="">
      <input type="hidden" name="hdnIdTextoPadrao" value="">
    </form>"#;

    #[test]
    fn perfil_manual_preserva_numero_e_recusa_limite() {
        let html = HtmlServido::de_teste(PERFIL);
        let numero = NumeroDocumento::novo("0011526").unwrap();
        assert_eq!(conferir(&html, &numero).unwrap(), "0011526");
        let longo = NumeroDocumento::novo("00011526").unwrap();
        assert!(matches!(
            conferir(&html, &longo),
            Err(ErroModelo::NumeroExcedeLimite)
        ));
    }

    #[test]
    fn perfil_manual_recusa_residuo_e_radio_ambiguo() {
        let numero = NumeroDocumento::novo("0011526").unwrap();
        for html in [
            PERFIL.replace(
                "name=\"hdnIdDocumentoTextoBase\" value=\"\"",
                "name=\"hdnIdDocumentoTextoBase\" value=\"9\"",
            ),
            PERFIL.replace("value=\"D\"", "value=\"D\" disabled"),
            PERFIL.replace("value=\"N\"", "value=\"D\""),
        ] {
            assert!(matches!(
                conferir(&HtmlServido::de_teste(&html), &numero),
                Err(ErroModelo::ControlesNaoOferecidos)
            ));
        }
    }

    #[test]
    fn perfil_manual_recusa_controles_sem_disponibilidade_ou_associacao_implicita() {
        let numero = NumeroDocumento::novo("0011526").unwrap();
        let radio = r#"<input type="radio" name="rdoTextoInicial" value="D">"#;
        let texto = r#"<input type="text" name="txtProtocoloDocumentoTextoBase" maxlength="7">"#;
        let oculto = r#"<input type="hidden" name="hdnIdDocumentoTextoBase" value="">"#;
        for campo in [radio, texto, oculto] {
            let html = PERFIL.replace(campo, &format!("<fieldset disabled>{campo}</fieldset>"));
            assert!(matches!(
                conferir(&HtmlServido::de_teste(&html), &numero),
                Err(ErroModelo::ControlesNaoOferecidos)
            ));
        }
        for html in [
            PERFIL.replace(texto, &texto.replace("maxlength", "readonly maxlength")),
            format!(
                "{PERFIL}<input type='hidden' form='frmDocumentoCadastro' name='hdnIdDocumentoTextoBase' value='9'>"
            ),
            format!(
                "{}<form id='outro'></form>",
                PERFIL.replace(texto, &texto.replace("maxlength", "form='outro' maxlength"))
            ),
        ] {
            assert!(matches!(
                conferir(&HtmlServido::de_teste(&html), &numero),
                Err(ErroModelo::ControlesNaoOferecidos)
            ));
        }
        let html = PERFIL.replace(
            radio,
            &format!("<fieldset disabled><legend>{radio}</legend></fieldset>"),
        );
        assert_eq!(
            conferir(&HtmlServido::de_teste(&html), &numero).unwrap(),
            "0011526"
        );
    }
}
