//! Busca de NUP em resposta HTML de pesquisa privada, sem rede ou links.
//! Não confunde uma captura selecionada com resultado completo/atual do SEI.
use crate::{Cli, Domain, Processo, pesquisa_web, processo, transporte};
use serde_json::json;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErroPesquisaPrivada {
    NumeroNaoInformado,
    Numero(processo::NumeroProcessoInvalido),
    OpcoesIncompativeisComPesquisaPrivada,
    Arquivo(transporte::FalhaTransporte),
    Resposta(pesquisa_web::ErroResultadoPesquisa),
    NumeroNaoObservadoNaResposta,
}

impl ErroPesquisaPrivada {
    fn subcodigo(self) -> &'static str {
        match self {
            Self::NumeroNaoInformado => "numero_de_processo_nao_informado",
            Self::Numero(erro) => erro.subcodigo(),
            Self::OpcoesIncompativeisComPesquisaPrivada => {
                "opcoes_incompativeis_com_pesquisa_privada"
            }
            Self::Arquivo(erro) => erro.subcodigo(),
            Self::Resposta(erro) => erro.subcodigo(),
            Self::NumeroNaoObservadoNaResposta => "numero_nao_observado_na_resposta_de_pesquisa",
        }
    }

    fn codigo(self) -> (&'static str, u8) {
        match self {
            Self::NumeroNaoInformado
            | Self::Numero(_)
            | Self::OpcoesIncompativeisComPesquisaPrivada => ("entrada_invalida", 2),
            Self::NumeroNaoObservadoNaResposta => ("numero_nao_observado", 1),
            Self::Arquivo(_) | Self::Resposta(_) => ("fonte_invalida", 1),
        }
    }
}

fn conferir(
    comando: &Processo,
    unidade: Option<&str>,
) -> Result<processo::NumeroProcesso, ErroPesquisaPrivada> {
    if comando.resumo
        || comando.historico_privado.is_some()
        || comando.saida.is_some()
        || comando.caixa_local.is_some()
        || comando.observado_na_pagina
        || !comando.paginas_capturadas.is_empty()
        || comando.sem_documentos
        || comando.sem_marcadores
        || comando.sem_andamentos
        || comando.sem_propriedades
        || comando.andamentos_completos
        || comando.acao.is_some()
        || unidade.is_some()
    {
        return Err(ErroPesquisaPrivada::OpcoesIncompativeisComPesquisaPrivada);
    }
    let nup = comando
        .numero
        .as_deref()
        .ok_or(ErroPesquisaPrivada::NumeroNaoInformado)?;
    processo::NumeroProcesso::novo(nup).map_err(ErroPesquisaPrivada::Numero)
}

/// Apenas uma resposta; o total declarado não é cobertura nacional nem
/// autorização para abrir o href do resultado.
fn selecionar(html: &str, numero: &processo::NumeroProcesso) -> Result<usize, ErroPesquisaPrivada> {
    let processos = pesquisa_web::ler_resultados(html).map_err(ErroPesquisaPrivada::Resposta)?;
    let total_observado = processos.len();
    processos
        .iter()
        .any(|processo| processo.numero.as_str() == numero.as_str())
        .then_some(total_observado)
        .ok_or(ErroPesquisaPrivada::NumeroNaoObservadoNaResposta)
}

pub(super) fn tentar_consultar(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(comando) = &cli.command else {
        return None;
    };
    let caminho = comando.resultado_pesquisa_file.as_deref()?;
    let resultado = conferir(comando, cli.unidade.as_deref()).and_then(|numero| {
        let html = transporte::arquivo_privado(caminho, crate::MAX_HTML_BYTES as u64)
            .map_err(ErroPesquisaPrivada::Arquivo)?;
        let total = selecionar(&html, &numero)?;
        Ok((numero, total))
    });
    Some(match resultado {
        Ok((numero, total_observado)) => {
            if cli.json {
                println!(
                    "{}",
                    json!({
                        "processo": {"numero": numero},
                        "total_observado":total_observado,
                        "escopo":"resposta_pesquisa_html_privado_offline",
                        "autenticacao_remota":false,
                        "requisicoes":0
                    })
                );
            } else {
                println!(
                    "IES: processo {} observado em resposta HTML privada (sem consulta ao SEI).",
                    numero.as_str()
                );
            }
            ExitCode::SUCCESS
        }
        Err(erro) => {
            let (codigo, saida) = erro.codigo();
            if cli.json {
                println!(
                    "{}",
                    json!({
                        "codigo":codigo,
                        "subcodigo":erro.subcodigo(),
                        "retentavel":false
                    })
                );
            } else {
                eprintln!(
                    "IES: busca em resposta HTML privada recusada [{}]",
                    erro.subcodigo()
                );
            }
            ExitCode::from(saida)
        }
    })
}

#[cfg(test)]
#[path = "processo_pesquisa_privada_cli/tests.rs"]
mod tests;
