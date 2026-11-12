//! A unidade publicada na tela do formulário, conferida com a solicitada.
use super::ErroSubmissaoFormulario;

pub(super) fn conferir_unidade(
    dom: &crate::tela_html::Documento,
    solicitada: Option<&str>,
) -> Result<(), ErroSubmissaoFormulario> {
    // `unidade_html` só falha por identificações repetidas fora do par
    // móvel/desktop.
    let publicada = crate::controle_contexto::unidade_html(dom)
        .map_err(ErroSubmissaoFormulario::IdentificacoesDaUnidadeRepetidasNaTela)?
        .ok_or(ErroSubmissaoFormulario::IdentificacaoDaUnidadeAusenteNaTela)?;
    let Some(solicitada) = solicitada else {
        return Err(ErroSubmissaoFormulario::UnidadeNaoInformada);
    };
    let solicitada = solicitada.split_whitespace().collect::<Vec<_>>().join(" ");
    if publicada != solicitada {
        return Err(ErroSubmissaoFormulario::UnidadeDaTelaDiferenteDaSolicitada);
    }
    Ok(())
}
