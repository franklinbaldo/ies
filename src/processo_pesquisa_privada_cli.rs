//! Busca de NUP em resposta de pesquisa HTML privada; fase CONTRACT.
//! Não realiza rede nem aceita links da página como autorização de acesso.
use crate::{Cli, Processo, pesquisa_web, processo, transporte};
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErroPesquisaPrivada {
    NumeroNaoInformado,
    Numero(processo::NumeroProcessoInvalido),
    OpcoesIncompativeisComPesquisaPrivada,
    Arquivo(transporte::FalhaTransporte),
    Resposta(pesquisa_web::ErroResultadoPesquisa),
    NumeroNaoObservadoNaResposta,
    ImplementacaoPendente,
}

fn conferir(
    _comando: &Processo,
    _unidade: Option<&str>,
) -> Result<processo::NumeroProcesso, ErroPesquisaPrivada> {
    Err(ErroPesquisaPrivada::ImplementacaoPendente)
}

fn selecionar(
    _html: &str,
    _numero: &processo::NumeroProcesso,
) -> Result<usize, ErroPesquisaPrivada> {
    Err(ErroPesquisaPrivada::ImplementacaoPendente)
}

pub(super) fn tentar_consultar(_cli: &Cli) -> Option<ExitCode> {
    None
}

#[cfg(test)]
#[path = "processo_pesquisa_privada_cli/tests.rs"]
mod tests;
