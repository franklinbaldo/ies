//! Relato local da configuração, sem abrir credenciais nem consultar o SEI.
use crate::{configuracao, transporte};
use serde_json::json;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FalhaSessaoLocal {
    Configuracao(configuracao::ErroConfiguracao),
    Transporte(transporte::FalhaTransporte),
}
impl FalhaSessaoLocal {
    fn subcodigo(self) -> &'static str {
        match self {
            Self::Configuracao(erro) => erro.subcodigo(),
            Self::Transporte(erro) => erro.subcodigo(),
        }
    }
    fn codigo(self) -> &'static str {
        match self {
            Self::Configuracao(_) => "configuracao_invalida",
            Self::Transporte(_) => "sessao_local_invalida",
        }
    }
}

/// Abre só arquivos privados locais; nunca testa autorização no servidor.
fn mostrar_sessao_local(json_mode: bool) -> ExitCode {
    let resultado = configuracao::arquivos_de_acesso()
        .map_err(FalhaSessaoLocal::Configuracao)
        .and_then(|arquivos| {
            transporte::verificar_sessao_local_de_arquivos(
                &arquivos.url_controle,
                &arquivos.cookie,
                &arquivos.origem,
            )
            .map_err(FalhaSessaoLocal::Transporte)
        });
    match resultado {
        Ok(()) => {
            if json_mode {
                println!(
                    "{}",
                    json!({
                        "sessao_local": "formato_e_origem_verificados",
                        "conexao": "nao_verificada",
                        "acesso_remoto": false,
                        "autenticacao_remota": false
                    })
                );
            } else {
                println!(
                    "IES: formato e origem da sessão local conferidos; conexão SEI não verificada."
                );
            }
            ExitCode::SUCCESS
        }
        Err(erro) => {
            if json_mode {
                println!(
                    "{}",
                    json!({
                        "codigo": erro.codigo(),
                        "subcodigo": erro.subcodigo(),
                        "conexao": "nao_verificada",
                        "acesso_remoto": false,
                        "retentavel": false
                    })
                );
            } else {
                eprintln!("IES: sessão local recusada [{}]", erro.subcodigo());
            }
            ExitCode::from(1)
        }
    }
}

fn mostrar_verificacao(json_mode: bool) -> ExitCode {
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => {
            if json_mode {
                println!(
                    "{}",
                    json!({
                        "codigo": "configuracao_invalida",
                        "subcodigo": erro.subcodigo(),
                        "conexao": "nao_verificada",
                        "retentavel": false
                    })
                );
            } else {
                eprintln!("IES: caminhos privados pendentes [{}]", erro.subcodigo());
            }
            return ExitCode::from(1);
        }
    };
    match configuracao::verificar_arquivos_privados(&arquivos) {
        Ok(()) => {
            if json_mode {
                println!(
                    "{}",
                    json!({
                        "configuracao": "caminhos_informados",
                        "arquivos_privados": "metadados_verificados",
                        "conexao": "nao_verificada",
                        "acesso_remoto": false
                    })
                );
            } else {
                println!(
                    "IES: metadados dos arquivos privados verificados; conexão SEI não verificada."
                );
            }
            ExitCode::SUCCESS
        }
        Err(erro) => {
            if json_mode {
                println!(
                    "{}",
                    json!({
                        "codigo": "configuracao_invalida",
                        "subcodigo": erro.subcodigo(),
                        "arquivo": erro.arquivo().rotulo(),
                        "conexao": "nao_verificada",
                        "retentavel": false
                    })
                );
            } else {
                eprintln!(
                    "IES: arquivo privado {} não validado [{}]",
                    erro.arquivo().rotulo(),
                    erro.subcodigo()
                );
            }
            ExitCode::from(1)
        }
    }
}

pub(super) fn mostrar_status(
    json_mode: bool,
    verificar_arquivos: bool,
    verificar_sessao_local: bool,
) -> ExitCode {
    if verificar_sessao_local {
        return mostrar_sessao_local(json_mode);
    }
    if verificar_arquivos {
        return mostrar_verificacao(json_mode);
    }
    // Diagnóstico local somente: não abre arquivos privados nem faz rede.
    let configuracao = configuracao::arquivos_de_acesso();
    let (estado, detalhe) = match &configuracao {
        Ok(_) => ("caminhos_informados", None),
        Err(erro) => ("caminhos_pendentes", Some(erro.subcodigo())),
    };
    if json_mode {
        println!(
            "{}",
            json!({
                "configuracao": estado,
                "subcodigo": detalhe,
                "conexao": "nao_verificada",
                "acesso_remoto": false,
                "proximo_passo": if configuracao.is_ok() {
                    "validar os arquivos privados e consultar a instalacao autorizada"
                } else {
                    "configurar os caminhos de acesso com ies setup"
                }
            })
        );
    } else {
        println!("IES: {estado}; conexão SEI não verificada (nenhuma requisição enviada).");
    }
    ExitCode::SUCCESS
}
