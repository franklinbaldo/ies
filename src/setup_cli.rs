//! Grava os caminhos privados de acesso, sem verificar a conexão.
use crate::{SetupArgs, configuracao};
use serde_json::json;

pub(crate) fn executar_setup(setup: &SetupArgs, json_mode: bool) -> anyhow::Result<()> {
    let arquivos = configuracao::ArquivosDeAcesso {
        url_controle: setup.url_controle.clone(),
        cookie: setup.cookie.clone(),
        origem: setup.origem.clone(),
    };
    configuracao::salvar_arquivos_de_acesso(&setup.destino, &arquivos)?;
    if json_mode {
        println!(
            "{}",
            json!({"configuracao": "gravada", "conexao": "nao_verificada"})
        );
    } else {
        println!("IES: caminhos privados configurados; conexão ainda não verificada.");
    }
    Ok(())
}
