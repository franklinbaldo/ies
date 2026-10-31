//! Consulta offline de respostas POST de pesquisa verificadas por hashes.
//! Não chama SEI, não executa links e não confunde ausência na captura com inexistência.
use crate::{cassette, pesquisa_web};
use serde_json::json;
use std::path::Path;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErroReplayPesquisa {
    PassoDePesquisaNaoInformado,
    PesquisaExigeUmUnicoPasso,
    RespostaDePesquisaNaoVerificada,
    Pagina(pesquisa_web::ErroResultadoPesquisa),
}
impl ErroReplayPesquisa {
    fn subcodigo(self) -> &'static str {
        match self {
            Self::PassoDePesquisaNaoInformado => "passo_da_pesquisa_nao_informado",
            Self::PesquisaExigeUmUnicoPasso => "pesquisa_exige_um_unico_passo",
            Self::RespostaDePesquisaNaoVerificada => "resposta_de_pesquisa_nao_verificada",
            Self::Pagina(erro) => erro.subcodigo(),
        }
    }
    fn codigo(self) -> (&'static str, u8) {
        match self {
            Self::PassoDePesquisaNaoInformado | Self::PesquisaExigeUmUnicoPasso => {
                ("entrada_invalida", 2)
            }
            Self::RespostaDePesquisaNaoVerificada | Self::Pagina(_) => ("fonte_invalida", 1),
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
pub(super) fn mostrar(card: &Path, zip: &Path, passos: &[usize], json_mode: bool) -> ExitCode {
    let resultado = selecionar_passo(passos).and_then(|passo| {
        cassette::read_cassette_search_result(card, zip, passo)
            .map_err(|_| ErroReplayPesquisa::RespostaDePesquisaNaoVerificada)
            .and_then(|(html, instalacao)| {
                pesquisa_web::ler_resultados(&html)
                    .map(|processos| (processos, instalacao, passo))
                    .map_err(ErroReplayPesquisa::Pagina)
            })
    });
    match resultado {
        Ok((processos, instalacao, passo)) => {
            if json_mode {
                println!(
                    "{}",
                    json!({"processos": processos,"instalacao":instalacao,"passo":passo,
                    "total_observado":processos.len(),"escopo":"resultado_pesquisa_cassette_offline"})
                );
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
