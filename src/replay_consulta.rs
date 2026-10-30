//! Consulta de identidade em cassettes da caixa, sem rede nem inferência de inexistência.
use crate::erro_web::ErroCaixa;
use crate::paginas;
use crate::processo::{NumeroProcesso, NumeroProcessoInvalido};
use crate::resumo_processo;
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
}

impl ErroBusca {
    fn subcodigo(&self) -> &'static str {
        match self {
            Self::Numero(erro) => erro.subcodigo(),
            Self::Caixa(erro) => erro.subcodigo(),
            Self::PaginasNaoObtidas => "paginas_do_controle_nao_obtidas",
            Self::NumeroNaoObservado => "numero_nao_observado_na_caixa",
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
    htmls: &[String],
    instalacao: &str,
    numero: &str,
    unidade: Option<&str>,
) -> Result<Consulta, ErroBusca> {
    let numero = NumeroProcesso::novo(numero).map_err(ErroBusca::Numero)?;
    let leitura = paginas::juntar(htmls, unidade).map_err(ErroBusca::Caixa)?;
    if !leitura.recebidos.completa(&leitura.caixa.recebidos)
        || !leitura.gerados.completa(&leitura.caixa.gerados)
    {
        return Err(ErroBusca::PaginasNaoObtidas);
    }
    let ocorrencias = resumo_processo::ocorrencias(&leitura.caixa, numero.as_str());
    if ocorrencias.is_empty() {
        return Err(ErroBusca::NumeroNaoObservado);
    }
    Ok(Consulta {
        escopo: "resumo_cassette_caixa",
        numero: numero.as_str().to_owned(),
        instalacao: instalacao.to_owned(),
        unidade: leitura.unidade,
        ocorrencias,
    })
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
                println!(
                    "{}",
                    serde_json::to_string_pretty(&consulta).expect("resultado serializável")
                );
            } else {
                println!(
                    "IES: processo {} observado em {} ocorrência(s) da cassette; consulta offline.",
                    consulta.numero,
                    consulta.ocorrencias.len()
                );
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
                println!(
                    "{}",
                    json!({
                        "codigo": codigo,
                        "subcodigo": erro.subcodigo(),
                        "retentavel": false,
                        "erro": "consulta offline de processo não concluída"
                    })
                );
            } else {
                eprintln!("IES: consulta offline não concluída [{}]", erro.subcodigo());
            }
            ExitCode::from(exit)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numero_com_controle_falha_por_tipo_antes_de_ler_html() {
        let erro = consultar(&[], "unb", "0001\nSECRETO", None).unwrap_err();
        assert_eq!(erro, ErroBusca::Numero(NumeroProcessoInvalido::Controle));
        assert_eq!(erro.subcodigo(), "numero_processo_controle");
    }

    #[test]
    fn caixa_ausente_tem_erro_diferente_de_numero_ausente() {
        let erro = consultar(&[], "unb", "0001", None).unwrap_err();
        assert_eq!(erro, ErroBusca::Caixa(ErroCaixa::SemPaginas));
        assert_ne!(erro, ErroBusca::NumeroNaoObservado);
    }
}
