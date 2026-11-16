//! Consulta de link assinado na forma conferida (chaves canônicas, `acao`
//! única, `infra_hash` hexadecimal único, sem `acao_ajax`).
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::ConsultaDoControlador;

/// Confere a consulta de um link assinado: chaves canônicas, valores sem
/// `;`, `acao` única igual a `acao`, um `infra_hash` hexadecimal aceito por
/// `comprimento`, sem `acao_ajax`. Devolve o `infra_unidade_atual` (único),
/// se houver.
pub(crate) fn conferir_consulta(
    consulta: &str,
    acao: AcaoDoSei,
    comprimento: impl Fn(&str) -> bool,
) -> Result<Option<String>, ConsultaForaDaForma> {
    let mut unidades = conferir(consulta, acao, comprimento)?;
    if unidades.len() > 1 {
        return Err(ConsultaForaDaForma);
    }
    Ok(unidades.pop())
}

/// A consulta não tem a forma conferida por [`conferir_consulta`].
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) struct ConsultaForaDaForma;

fn conferir(
    consulta: &str,
    acao: AcaoDoSei,
    comprimento: impl Fn(&str) -> bool,
) -> Result<Vec<String>, ConsultaForaDaForma> {
    let consulta = ConsultaDoControlador::ler(consulta).map_err(|_| ConsultaForaDaForma)?;
    let hash = consulta
        .unico("infra_hash")
        .map_err(|_| ConsultaForaDaForma)?;
    if consulta.acao() != Ok(acao.como_na_url())
        || consulta.valores("acao_ajax").next().is_some()
        || !comprimento(hash)
        || !hash.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(ConsultaForaDaForma);
    }
    Ok(consulta
        .valores("infra_unidade_atual")
        .map(str::to_owned)
        .collect())
}
