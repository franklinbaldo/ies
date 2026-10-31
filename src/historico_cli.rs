//! Consulta offline de histórico reproduzido a partir de evidência verificada.
use crate::{erro_fonte, historico, processo};
use serde_json::json;
use std::process::ExitCode;

pub(super) fn mostrar_historico(
    html: &str,
    instalacao: &str,
    numero_solicitado: Option<&str>,
    json_mode: bool,
) -> ExitCode {
    let numero = if let Some(informado) = numero_solicitado {
        match processo::NumeroProcesso::novo(informado) {
            Ok(numero) => numero.as_str().to_owned(),
            Err(erro) => {
                if json_mode {
                    println!(
                        "{}",
                        json!({
                            "codigo": "entrada_invalida",
                            "subcodigo": erro.subcodigo(),
                            "erro": "número de processo não aceito",
                            "retentavel": false
                        })
                    );
                } else {
                    eprintln!("IES: número de processo não aceito [{}]", erro.subcodigo());
                }
                return ExitCode::from(2);
            }
        }
    } else {
        match historico::numero_do_historico(html) {
            Ok(numero) => numero,
            Err(motivo) => {
                return erro_fonte(json_mode, motivo.subcodigo(), motivo.mensagem());
            }
        }
    };
    let detalhe = match historico::ler_historico(html, &numero) {
        Ok(valor) => valor,
        Err(motivo) => {
            return erro_fonte(json_mode, motivo.subcodigo(), motivo.mensagem());
        }
    };
    if json_mode {
        println!(
            "{}",
            json!({"processo": detalhe, "instalacao": instalacao, "escopo": "historico_offline", "parcial": true})
        );
    } else {
        println!(
            "Histórico offline do processo {}: {} andamento(s), dados parciais.",
            detalhe.numero.as_str(),
            detalhe.andamentos.len()
        );
    }
    ExitCode::SUCCESS
}
