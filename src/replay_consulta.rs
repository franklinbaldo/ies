//! Consulta de identidade em cassettes da caixa, sem rede nem inferência de inexistência.
use crate::erro_web::ErroCaixa;
use crate::processo::NumeroProcessoInvalido;
use crate::resumo_processo::Ocorrencia;
use serde::Serialize;
use serde_json::json;
use std::process::ExitCode;

#[derive(Debug, PartialEq, Eq)]
enum ErroBusca {
    Numero(NumeroProcessoInvalido),
    Caixa(ErroCaixa),
    PaginasNaoObtidas,
    NumeroNaoObservado,
    AindaNaoImplementada,
}

impl ErroBusca {
    fn subcodigo(&self) -> &'static str {
        match self {
            Self::Numero(erro) => erro.subcodigo(),
            Self::Caixa(erro) => erro.subcodigo(),
            Self::PaginasNaoObtidas => "paginas_do_controle_nao_obtidas",
            Self::NumeroNaoObservado => "numero_nao_observado_na_caixa",
            Self::AindaNaoImplementada => "consulta_no_replay_nao_implementada",
        }
    }
}

#[derive(Debug, Serialize)]
struct Consulta {
    escopo: &'static str,
    numero: String,
    instalacao: String,
    unidade: Option<String>,
    ocorrencias: Vec<Ocorrencia>,
}

fn consultar(
    _htmls: &[String],
    _instalacao: &str,
    _numero: &str,
    _unidade: Option<&str>,
) -> Result<Consulta, ErroBusca> {
    Err(ErroBusca::AindaNaoImplementada)
}

pub(super) fn mostrar(
    htmls: &[String],
    instalacao: &str,
    numero: &str,
    unidade: Option<&str>,
    json_mode: bool,
) -> ExitCode {
    match consultar(htmls, instalacao, numero, unidade) {
        Ok(consulta) => {
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&consulta).expect("resultado serializável"));
            } else {
                println!("IES: processo {} observado em {} ocorrência(s) da cassette; consulta offline.",
                    consulta.numero, consulta.ocorrencias.len());
            }
            ExitCode::SUCCESS
        }
        Err(erro) => {
            let codigo = if matches!(&erro, ErroBusca::Numero(_)) {
                "entrada_invalida"
            } else {
                "fonte_invalida"
            };
            let exit = if codigo == "entrada_invalida" { 2 } else { 1 };
            if json_mode {
                println!("{}", json!({
                    "codigo": codigo,
                    "subcodigo": erro.subcodigo(),
                    "retentavel": false,
                    "erro": "consulta offline de processo não concluída"
                }));
            } else {
                eprintln!("IES: consulta offline não concluída [{}]", erro.subcodigo());
            }
            ExitCode::from(exit)
        }
    }
}
