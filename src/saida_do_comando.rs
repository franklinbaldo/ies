//! O que um comando devolve e o único ponto que mostra o erro que subiu
//! (#695): o próprio tipo, sem texto à parte.
use serde_json::json;
use std::process::ExitCode;

/// O que um comando devolve: o código de saída, ou o erro que sobe até o
/// `main`, o único lugar que o mostra (#695).
pub(crate) type Saida = Result<ExitCode, Box<dyn std::error::Error>>;

/// Mostra o erro que subiu: o próprio tipo (`Debug`), sem texto à parte, e
/// o status e a mensagem do SEI se uma recusa HTTP estiver na cadeia.
pub(crate) fn mostrar(json_mode: bool, erro: &(dyn std::error::Error + 'static)) -> ExitCode {
    let http = std::iter::successors(Some(erro), |e| e.source())
        .find_map(|e| e.downcast_ref::<crate::transporte::StatusHttpRecusado>());
    if json_mode {
        let mut valor = json!({"erro": format!("{erro:?}")});
        if let Some(http) = http {
            if http.status != 200 {
                valor["status_http"] = json!(http.status);
            }
            if let Some(texto) = &http.mensagem_do_servidor {
                valor["mensagem_do_servidor"] = json!(texto);
            }
        }
        println!("{valor}");
    } else {
        match http {
            Some(h) => eprintln!(
                "IES: {erro:?} (HTTP {}{})",
                h.status,
                h.mensagem_do_servidor
                    .as_deref()
                    .map(|t| format!(": {t}"))
                    .unwrap_or_default()
            ),
            None => eprintln!("IES: {erro:?}"),
        }
    }
    ExitCode::from(1)
}
