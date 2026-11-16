//! Gerencia credenciais no cofre seguro do sistema operacional (Keyring).
//! Utiliza exclusivamente o serviço "ies".

use clap::{Args, Subcommand};
use std::process::ExitCode;

#[derive(Debug, Args)]
pub(crate) struct CredenciaisArgs {
    #[command(subcommand)]
    pub(crate) comando: ComandoCredenciais,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ComandoCredenciais {
    /// Salva a senha com segurança no cofre nativo do SO
    Salvar {
        #[arg(long, help = "Nome de usuário no SEI")]
        usuario: String,
        #[arg(long, help = "URL base da instalação (ex: https://sei.sistemas.ro.gov.br/sei)")]
        origem: String,
    },
    /// Verifica se a senha está gravada no cofre nativo
    Status {
        #[arg(long, help = "Nome de usuário no SEI")]
        usuario: String,
        #[arg(long, help = "URL base da instalação")]
        origem: String,
    },
    /// Remove a credencial do cofre nativo
    Excluir {
        #[arg(long, help = "Nome de usuário no SEI")]
        usuario: String,
        #[arg(long, help = "URL base da instalação")]
        origem: String,
    },
}

pub(crate) fn executar_credenciais(args: &CredenciaisArgs, json_mode: bool) -> ExitCode {
    match &args.comando {
        ComandoCredenciais::Salvar { usuario, origem } => {
            eprint!("Digite a senha para {}@{}: ", usuario, origem);
            let senha = match rpassword::read_password() {
                Ok(s) if !s.trim().is_empty() => s,
                _ => {
                    eprintln!("\nSenha não pode ser vazia.");
                    return ExitCode::from(1);
                }
            };
            match crate::cofre::salvar_senha(usuario, origem, &senha) {
                Ok(()) => {
                    if json_mode {
                        println!(r#"{{"cofre":"ies","status":"gravado"}}"#);
                    } else {
                        println!("✓ Senha gravada com sucesso no cofre seguro (Keyring: serviço 'ies').");
                    }
                    ExitCode::SUCCESS
                }
                Err(err) => {
                    eprintln!("Erro ao gravar no cofre: {err}");
                    ExitCode::from(1)
                }
            }
        }
        ComandoCredenciais::Status { usuario, origem } => {
            let tem = crate::cofre::obter_senha(usuario, origem).is_some();
            if json_mode {
                println!(r#"{{"cofre":"ies","presente":{}}}"#, tem);
            } else if tem {
                println!("✓ Senha presente no cofre seguro (Keyring: serviço 'ies').");
            } else {
                println!("✗ Nenhuma senha encontrada no cofre para este usuário/instalação.");
            }
            if tem { ExitCode::SUCCESS } else { ExitCode::from(1) }
        }
        ComandoCredenciais::Excluir { usuario, origem } => {
            match crate::cofre::excluir_senha(usuario, origem) {
                Ok(()) => {
                    if json_mode {
                        println!(r#"{{"cofre":"ies","status":"excluido"}}"#);
                    } else {
                        println!("✓ Credencial removida do cofre.");
                    }
                    ExitCode::SUCCESS
                }
                Err(err) => {
                    eprintln!("Erro ao excluir do cofre: {err}");
                    ExitCode::from(1)
                }
            }
        }
    }
}
