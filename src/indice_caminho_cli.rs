//! Descoberta local do caminho do índice SQLite, sem rede nem sessão Web.
use crate::{configuracao, erro_fonte, indice_local};
use serde_json::json;
use std::process::ExitCode;

/// Não abre o SQLite nem os arquivos de sessão; apenas resolve os caminhos
/// fornecidos pela configuração, inclusive o opt-out `IES_SEM_INDICE`.
pub(super) fn mostrar(json_mode: bool) -> ExitCode {
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => return erro_fonte(json_mode, erro.mensagem()),
    };
    let sem_indice = std::env::var_os("IES_SEM_INDICE");
    let arquivo = std::env::var_os("IES_INDICE_FILE");
    let caminho = indice_local::caminho_do_indice(
        sem_indice.as_deref(),
        arquivo.as_deref(),
        &arquivos.url_controle,
    );
    if json_mode {
        println!(
            "{}",
            json!({
                "indice": if caminho.is_some() { "configurado" } else { "desligado" },
                "caminho": caminho.as_ref().map(|p| p.to_string_lossy().into_owned()),
                "acesso_remoto": false,
                "banco_aberto": false
            })
        );
    } else if let Some(caminho) = caminho {
        println!("{}", caminho.display());
    } else {
        println!("Índice local desligado (IES_SEM_INDICE).");
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    #[test]
    fn clap_aceita_o_comando_de_descoberta_sem_conectar_ao_sei() {
        let cli = crate::Cli::try_parse_from(["ies", "indice", "caminho"]).unwrap();
        assert!(matches!(
            cli.command,
            crate::Domain::Indice(crate::Indice {
                acao: Some(crate::IndiceAcao::Caminho)
            })
        ));
    }

    #[test]
    fn opcao_json_continua_global_no_novo_subcomando() {
        let cli = crate::Cli::try_parse_from(["ies", "indice", "caminho", "--json"]).unwrap();
        assert!(cli.json);
    }
}
