//! Diagnóstico local por padrão, com consulta Web somente por opt-in explícito.
use crate::{configuracao, transporte};
use serde_json::json;

/// Abre só arquivos privados locais; nunca testa autorização no servidor.
fn mostrar_sessao_local(json_mode: bool) -> anyhow::Result<()> {
    let arquivos = configuracao::arquivos_de_acesso()?;
    transporte::verificar_sessao_local_de_arquivos(
        &arquivos.url_controle,
        &arquivos.cookie,
        &arquivos.origem,
    )?;
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
        println!("IES: formato e origem da sessão local conferidos; conexão SEI não verificada.");
    }
    Ok(())
}

fn mostrar_verificacao(json_mode: bool) -> anyhow::Result<()> {
    let arquivos = configuracao::arquivos_de_acesso()?;
    configuracao::verificar_arquivos_privados(&arquivos)?;
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
        println!("IES: metadados dos arquivos privados verificados; conexão SEI não verificada.");
    }
    Ok(())
}

pub(super) fn mostrar_status(
    json_mode: bool,
    args: &crate::StatusArgs,
    unidade_esperada: Option<&str>,
) -> anyhow::Result<()> {
    if args.verificar_controle_web {
        return crate::status_web::mostrar(json_mode, unidade_esperada);
    }
    if args.verificar_sessao_local {
        return mostrar_sessao_local(json_mode);
    }
    if args.verificar_arquivos {
        return mostrar_verificacao(json_mode);
    }
    // Diagnóstico local somente: não abre arquivos privados nem faz rede.
    let configuracao = configuracao::arquivos_de_acesso();
    let estado = if configuracao.is_ok() {
        "caminhos_informados"
    } else {
        "caminhos_pendentes"
    };
    if json_mode {
        println!(
            "{}",
            json!({
                "configuracao": estado,
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
    Ok(())
}
