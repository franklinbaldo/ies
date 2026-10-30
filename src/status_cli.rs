//! Relato local da configuração, sem abrir credenciais nem consultar o SEI.
use crate::configuracao;
use serde_json::json;
use std::process::ExitCode;

pub(super) fn mostrar_status(json_mode: bool) -> ExitCode {
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
