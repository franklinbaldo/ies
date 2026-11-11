//! Consulta de link assinado na forma conferida (chaves canônicas, `acao`
//! única, `infra_hash` hexadecimal único, sem `acao_ajax`).
use crate::acao_do_sei::AcaoDoSei;

/// Confere a consulta de um link assinado: chaves canônicas, valores sem
/// `;`, `acao` única igual a `acao`, um `infra_hash` hexadecimal aceito por
/// `comprimento`, sem `acao_ajax`. Devolve o `infra_unidade_atual` (único),
/// se houver.
pub(crate) fn conferir_consulta(
    consulta: &str,
    acao: AcaoDoSei,
    comprimento: impl Fn(&str) -> bool,
) -> Result<Option<String>, ConsultaForaDaForma> {
    let mut unidades = conferir(consulta, acao, comprimento).ok_or(ConsultaForaDaForma)?;
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
) -> Option<Vec<String>> {
    use crate::transporte::{conferir_chave_de_consulta, conferir_valor_de_consulta};
    let (mut acoes, mut hashes) = (0, 0);
    let mut unidades = Vec::new();
    for par in consulta.split('&') {
        let (nome, valor) = par.split_once('=')?;
        conferir_chave_de_consulta(nome).ok()?;
        conferir_valor_de_consulta(valor).ok()?;
        match nome {
            "acao" if valor == acao.como_na_url() => acoes += 1,
            "infra_hash" if comprimento(valor) && valor.bytes().all(|b| b.is_ascii_hexdigit()) => {
                hashes += 1;
            }
            "infra_unidade_atual" => unidades.push(valor.to_owned()),
            "acao" | "infra_hash" | "acao_ajax" => return None,
            _ => {}
        }
    }
    (acoes == 1 && hashes == 1).then_some(unidades)
}
