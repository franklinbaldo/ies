//! Leitura das ações de sobrestar e retomar da barra do processo (#269).
//! A forma da URL é publicada pelo SEI. Este shim será completado após o RED.
use super::concluir::EnderecoDeEscrita;
use crate::instalacao::Instalacao;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroAcaoSobrestar {
    BarraInvalida,
    AcoesRepetidas,
    EnderecoNaoReconhecido,
    OutroProcesso,
    TelaDoNoNaoReconhecida,
}

pub(crate) fn ler_sobrestar_da_barra(
    _instalacao: &Instalacao,
    _html: &str,
    _id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroAcaoSobrestar> {
    Ok(None)
}

pub(crate) fn ler_retomar_da_barra(
    _instalacao: &Instalacao,
    _html: &str,
    _id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroAcaoSobrestar> {
    Ok(None)
}
