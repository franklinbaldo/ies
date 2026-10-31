//! Consulta offline de histórico reproduzido a partir de evidência verificada.
use crate::{erro_fonte, filtro_andamentos, historico, processo};
use serde_json::json;
use std::process::ExitCode;

pub(super) fn mostrar_historico(
    html: &str,
    instalacao: &str,
    numero_solicitado: Option<&str>,
    termo: Option<&str>,
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
    let mut detalhe = match historico::ler_historico(html, &numero) {
        Ok(valor) => valor,
        Err(motivo) => {
            return erro_fonte(json_mode, motivo.subcodigo(), motivo.mensagem());
        }
    };
    let total = detalhe.andamentos.len();
    if let Some(termo) = termo {
        if let Err(erro) = filtro_andamentos::filtrar(&mut detalhe, termo) {
            if json_mode {
                println!("{}", json!({"codigo":"entrada_invalida",
                    "subcodigo":erro.subcodigo(),"retentavel":false}));
            } else {
                eprintln!("IES: filtro de histórico não concluído [{}]", erro.subcodigo());
            }
            return ExitCode::from(2);
        }
    }
    if json_mode {
        if termo.is_some() {
            println!("{}", json!({"processo":detalhe,"instalacao":instalacao,
                "escopo":"historico_filtrado_offline","parcial":true,
                "total_observado":total, "total_filtrado":detalhe.andamentos.len()}));
            return ExitCode::SUCCESS;
        }
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


/// Um filtro de histórico nunca pode ser aplicado à caixa como se tivesse andamentos.
pub(super) fn erro_filtro_em_caixa(json_mode: bool) -> ExitCode {
    if json_mode {
        println!("{}", json!({"codigo":"entrada_invalida",
            "subcodigo":"termo_andamento_exige_historico","retentavel":false}));
    } else {
        eprintln!("IES: filtro exige resposta de histórico [termo_andamento_exige_historico]");
    }
    ExitCode::from(2)
}
