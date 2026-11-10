//! Ajuda por substantivo e contrato de comandos derivados da superfície Clap.
use crate::Cli;
use clap::CommandFactory;
use serde_json::json;

pub(crate) fn show_noun_help(noun: &str) {
    let mut root = Cli::command();
    if let Some(command) = root.find_subcommand_mut(noun) {
        let _ = command.print_help();
        println!();
    }
}

pub(crate) fn mostrar_esquema(json_mode: bool) {
    let mut comandos: Vec<_> = Cli::command()
        .get_subcommands()
        .map(|c| c.get_name().to_owned())
        .collect();
    comandos.sort();
    if json_mode {
        println!(
            "{}",
            json!({"comandos": comandos, "versao": env!("CARGO_PKG_VERSION")})
        );
    } else {
        println!("Comandos do IES: {}", comandos.join(", "));
    }
}
