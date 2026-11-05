//! A unidade publicada na tela do formulário, conferida com a solicitada.
use super::ErroSubmissaoFormulario;
use crate::html_atributos::nos_com_id;

pub(super) fn conferir_unidade(
    dom: &tl::VDom<'_>,
    solicitada: &str,
) -> Result<(), ErroSubmissaoFormulario> {
    use ErroSubmissaoFormulario as E;
    // Tag ilegível sai com o erro do tokenizador; depois disso `unidade_html`
    // só falha por identificações repetidas fora do par móvel/desktop.
    nos_com_id(dom, "lnkInfraUnidade")?;
    let publicada = crate::controle_contexto::unidade_html(dom)
        .map_err(|_| {
            E::Html(crate::erro_html::ErroHtml::repetido(
                "identificacoes_da_unidade_repetidas_na_tela",
                "o elemento aparece mais de uma vez",
            ))
        })?
        .ok_or(E::Html(crate::erro_html::ErroHtml::ausente(
            "identificacao_da_unidade_ausente_na_tela",
            "a tela não tem o elemento observado",
        )))?;
    let solicitada = solicitada.split_whitespace().collect::<Vec<_>>().join(" ");
    if publicada != solicitada {
        return Err(E::UnidadeDaTelaDiferenteDaSolicitada);
    }
    Ok(())
}
