//! Consulta offline de respostas POST de pesquisa verificadas por hashes.
//! Não chama SEI, não executa links e não confunde ausência na captura com inexistência.
use crate::{
    cassette, pesquisa_web,
    processo::{NumeroProcesso, NumeroProcessoInvalido},
};
use serde_json::json;
use std::collections::HashSet;
use std::path::Path;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErroReplayPesquisa {
    PassoDePesquisaNaoInformado,
    PesquisaExigeUmUnicoPasso,
    RespostaDePesquisaNaoVerificada,
    Pagina(pesquisa_web::ErroResultadoPesquisa),
    NumeroSolicitado(NumeroProcessoInvalido),
    NumeroNaoObservadoNaRespostaDePesquisa,
    RespostasParaConsolidarInsuficientes,
    NumeroDePesquisaRepetidoEntreRespostas,
    ConsolidacaoPendente,
}
impl ErroReplayPesquisa {
    fn subcodigo(self) -> &'static str {
        match self {
            Self::PassoDePesquisaNaoInformado => "passo_da_pesquisa_nao_informado",
            Self::PesquisaExigeUmUnicoPasso => "pesquisa_exige_um_unico_passo",
            Self::RespostaDePesquisaNaoVerificada => "resposta_de_pesquisa_nao_verificada",
            Self::Pagina(erro) => erro.subcodigo(),
            Self::NumeroSolicitado(erro) => erro.subcodigo(),
            Self::NumeroNaoObservadoNaRespostaDePesquisa => {
                "numero_nao_observado_na_resposta_de_pesquisa"
            }
            Self::RespostasParaConsolidarInsuficientes => {
                "respostas_para_consolidar_insuficientes"
            }
            Self::NumeroDePesquisaRepetidoEntreRespostas => {
                "numero_de_pesquisa_repetido_entre_respostas"
            }
            Self::ConsolidacaoPendente => "consolidacao_de_pesquisa_nao_implementada",
        }
    }
    fn codigo(self) -> (&'static str, u8) {
        match self {
            Self::PassoDePesquisaNaoInformado
            | Self::PesquisaExigeUmUnicoPasso
            | Self::NumeroSolicitado(_)
            | Self::RespostasParaConsolidarInsuficientes => ("entrada_invalida", 2),
            Self::NumeroNaoObservadoNaRespostaDePesquisa => ("numero_nao_observado", 1),
            Self::RespostaDePesquisaNaoVerificada
            | Self::Pagina(_)
            | Self::NumeroDePesquisaRepetidoEntreRespostas
            | Self::ConsolidacaoPendente => ("fonte_invalida", 1),
        }
    }
}
fn selecionar_passo(passos: &[usize]) -> Result<usize, ErroReplayPesquisa> {
    match passos {
        [] => Err(ErroReplayPesquisa::PassoDePesquisaNaoInformado),
        [passo] => Ok(*passo),
        _ => Err(ErroReplayPesquisa::PesquisaExigeUmUnicoPasso),
    }
}
fn selecionar_numero(
    processos: Vec<pesquisa_web::ResultadoPesquisa>,
    solicitado: Option<&str>,
) -> Result<Vec<pesquisa_web::ResultadoPesquisa>, ErroReplayPesquisa> {
    let Some(numero) = solicitado else {
        return Ok(processos);
    };
    let numero = NumeroProcesso::novo(numero).map_err(ErroReplayPesquisa::NumeroSolicitado)?;
    processos
        .into_iter()
        .find(|processo| processo.numero == numero)
        .map(|processo| vec![processo])
        .ok_or(ErroReplayPesquisa::NumeroNaoObservadoNaRespostaDePesquisa)
}

/// Consolida somente as respostas explicitamente selecionadas, sem afirmar paginação completa.
fn juntar_resultados(
    paginas: Vec<Vec<pesquisa_web::ResultadoPesquisa>>,
) -> Result<Vec<pesquisa_web::ResultadoPesquisa>, ErroReplayPesquisa> {
    let _ = paginas;
    Err(ErroReplayPesquisa::ConsolidacaoPendente)
}

fn mostrar_consolidado(
    card: &Path,
    zip: &Path,
    passos: &[usize],
    numero: Option<&str>,
    json_mode: bool,
) -> ExitCode {
    let resultado = (|| {
        if passos.len() < 2 {
            return Err(ErroReplayPesquisa::RespostasParaConsolidarInsuficientes);
        }
        let (htmls, instalacao) = cassette::read_cassette_search_results(card, zip, passos)
            .map_err(|_| ErroReplayPesquisa::RespostaDePesquisaNaoVerificada)?;
        let resultados = htmls
            .iter()
            .map(|html| pesquisa_web::ler_resultados(html).map_err(ErroReplayPesquisa::Pagina))
            .collect::<Result<Vec<_>, _>>()?;
        let reunidos = juntar_resultados(resultados)?;
        let total = reunidos.len();
        let escolhidos = selecionar_numero(reunidos, numero)?;
        Ok((escolhidos, instalacao, total))
    })();
    match resultado {
        Ok((processos, instalacao, total_observado)) => {
            if json_mode {
                println!("{}", json!({
                    "processos":processos, "instalacao":instalacao,
                    "passos":passos,"total_observado":total_observado,
                    "escopo":"respostas_pesquisa_cassette_offline",
                    "cobertura":"somente_passos_selecionados"
                }));
            } else {
                println!("IES: {} processo(s) nas respostas escolhidas offline.", processos.len());
                for processo in processos {
                    println!("Processo {}", processo.numero.as_str());
                }
            }
            ExitCode::SUCCESS
        }
        Err(erro) => {
            let (codigo, saida) = erro.codigo();
            if json_mode {
                println!("{}", json!({"codigo":codigo,"subcodigo":erro.subcodigo(),"retentavel":false}));
            } else {
                eprintln!("IES: consolidação offline recusada [{}]", erro.subcodigo());
            }
            ExitCode::from(saida)
        }
    }
}

pub(super) fn mostrar(
    card: &Path,
    zip: &Path,
    passos: &[usize],
    numero: Option<&str>,
    unir: bool,
    json_mode: bool,
) -> ExitCode {
    if unir {
        return mostrar_consolidado(card, zip, passos, numero, json_mode);
    }
    let resultado = selecionar_passo(passos).and_then(|passo| {
        cassette::read_cassette_search_result(card, zip, passo)
            .map_err(|_| ErroReplayPesquisa::RespostaDePesquisaNaoVerificada)
            .and_then(|(html, instalacao)| {
                pesquisa_web::ler_resultados(&html)
                    .map_err(ErroReplayPesquisa::Pagina)
                    .and_then(|processos| {
                        let total = processos.len();
                        selecionar_numero(processos, numero)
                            .map(|selecionados| (selecionados, instalacao, passo, total))
                    })
            })
    });
    match resultado {
        Ok((processos, instalacao, passo, total_observado)) => {
            if json_mode {
                let mut saida = json!({"processos": processos,"instalacao":instalacao,"passo":passo,
                    "total_observado":total_observado,"escopo":"resultado_pesquisa_cassette_offline"});
                if let Some(numero) = numero {
                    saida["numero_solicitado"] = json!(numero);
                }
                println!("{saida}");
            } else {
                println!(
                    "IES: {} processo(s) observados na resposta da pesquisa offline.",
                    processos.len()
                );
                for processo in processos {
                    println!("Processo {}", processo.numero.as_str());
                }
            }
            ExitCode::SUCCESS
        }
        Err(erro) => {
            let (codigo, exit) = erro.codigo();
            if json_mode {
                println!(
                    "{}",
                    json!({"codigo":codigo,"subcodigo":erro.subcodigo(),"retentavel":false})
                );
            } else {
                eprintln!(
                    "IES: resposta da pesquisa offline não concluída [{}]",
                    erro.subcodigo()
                );
            }
            ExitCode::from(exit)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn processos_observados() -> Vec<pesquisa_web::ResultadoPesquisa> {
        ["00001.123456/2026-07", "00123.456789/2026-00"]
            .iter()
            .map(|n| pesquisa_web::ResultadoPesquisa {
                numero: NumeroProcesso::novo(n).unwrap(),
            })
            .collect()
    }

    #[test]
    fn sem_filtro_preserva_todos_os_resultados() {
        assert_eq!(
            selecionar_numero(processos_observados(), None)
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn nup_exato_preserva_zeros_e_pontuacao_sem_retornar_outro() {
        let saida =
            selecionar_numero(processos_observados(), Some("00001.123456/2026-07")).unwrap();
        assert_eq!(saida.len(), 1);
        assert_eq!(saida[0].numero.as_str(), "00001.123456/2026-07");
    }

    #[test]
    fn nup_sem_zeros_a_esquerda_nao_identifica_outro_processo() {
        assert_eq!(
            selecionar_numero(processos_observados(), Some("1.123456/2026-07")),
            Err(ErroReplayPesquisa::NumeroNaoObservadoNaRespostaDePesquisa)
        );
    }

    #[test]
    fn ausencia_na_resposta_nao_declara_inexistencia_no_sei() {
        let erro =
            selecionar_numero(processos_observados(), Some("00000.999999/2026-01")).unwrap_err();
        assert_eq!(
            erro,
            ErroReplayPesquisa::NumeroNaoObservadoNaRespostaDePesquisa
        );
        assert_eq!(
            erro.subcodigo(),
            "numero_nao_observado_na_resposta_de_pesquisa"
        );
    }

    #[test]
    fn numero_com_controle_tem_erro_tipado_de_entrada() {
        assert_eq!(
            selecionar_numero(processos_observados(), Some("123\nPRIVADO")),
            Err(ErroReplayPesquisa::NumeroSolicitado(
                NumeroProcessoInvalido::Controle
            ))
        );
    }

    #[test]
    fn selecao_de_passo_exige_exatamente_uma_resposta() {
        assert_eq!(
            selecionar_passo(&[]),
            Err(ErroReplayPesquisa::PassoDePesquisaNaoInformado)
        );
        assert_eq!(
            selecionar_passo(&[1, 2]),
            Err(ErroReplayPesquisa::PesquisaExigeUmUnicoPasso)
        );
        assert_eq!(selecionar_passo(&[2]), Ok(2));
    }
}


#[cfg(test)]
mod testes_consolidacao {
    use super::*;

    fn resultado(numero: &str) -> pesquisa_web::ResultadoPesquisa {
        pesquisa_web::ResultadoPesquisa {
            numero: NumeroProcesso::novo(numero).unwrap(),
        }
    }

    #[test]
    fn consolida_respostas_distintas_sem_perder_ordem_nem_zeros() {
        let resultado = juntar_resultados(vec![
            vec![resultado("000123/2026"), resultado("000124/2026")],
            vec![resultado("000125/2026")],
        ]).unwrap();
        assert_eq!(resultado.len(), 3);
        assert_eq!(resultado[0].numero.as_str(), "000123/2026");
        assert_eq!(resultado[2].numero.as_str(), "000125/2026");
    }

    #[test]
    fn numero_repetido_entre_paginas_falha_sem_inventario_parcial() {
        let erro = juntar_resultados(vec![
            vec![resultado("000123/2026")],
            vec![resultado("000123/2026")],
        ]).unwrap_err();
        assert_eq!(erro, ErroReplayPesquisa::NumeroDePesquisaRepetidoEntreRespostas);
        assert_eq!(erro.subcodigo(), "numero_de_pesquisa_repetido_entre_respostas");
    }
}
