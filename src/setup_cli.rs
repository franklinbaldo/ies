//! Grava os caminhos privados de acesso, sem verificar a conexão.
use crate::{SetupArgs, configuracao, erro_fonte};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn executar_setup(setup: &SetupArgs, json_mode: bool) -> ExitCode {
    let arquivos = configuracao::ArquivosDeAcesso {
        url_controle: setup.url_controle.clone(),
        cookie: setup.cookie.clone(),
        origem: setup.origem.clone(),
    };
    match configuracao::salvar_arquivos_de_acesso(&setup.destino, &arquivos) {
        Ok(()) => {
            if json_mode {
                println!(
                    "{}",
                    json!({"configuracao": "gravada", "conexao": "nao_verificada"})
                );
            } else {
                println!("IES: caminhos privados configurados; conexão ainda não verificada.");
            }
            ExitCode::SUCCESS
        }
        Err(_) => erro_fonte(json_mode, "não foi possível gravar a configuração local"),
    }
}
