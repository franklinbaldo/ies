//! A unidade publicada na tela do formulário, conferida com a solicitada.
use super::ErroSubmissaoFormulario;
use crate::html_atributos::nos_com_id;

pub(super) fn conferir_unidade(
    dom: &tl::VDom<'_>,
    solicitada: &str,
) -> Result<(), ErroSubmissaoFormulario> {
    // Tag ilegível sai com o erro do tokenizador; depois disso `unidade_html`
    // só falha por identificações repetidas fora do par móvel/desktop.
    nos_com_id(dom, "lnkInfraUnidade")?;
    let publicada = crate::controle_contexto::unidade_html(dom)
        .map_err(|_| ErroSubmissaoFormulario::IdentificacoesDaUnidadeRepetidasNaTela)?
        .ok_or(ErroSubmissaoFormulario::IdentificacaoDaUnidadeAusenteNaTela)?;
    let solicitada = solicitada.split_whitespace().collect::<Vec<_>>().join(" ");
    if publicada != solicitada {
        return Err(ErroSubmissaoFormulario::UnidadeDaTelaDiferenteDaSolicitada);
    }
    Ok(())
}
