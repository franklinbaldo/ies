//! `ies login [--forcar]`: borda da CLI. Credenciais só pelo arquivo
//! privado de `IES_CREDENCIAIS_FILE`; a saída nunca publica credencial,
//! cookie ou URL assinada.
use crate::configuracao;
use crate::indice_local::IndiceLocal;
use crate::saida_erro::{json_de_erro_fonte, linha_humana};
use crate::sessao_login::http::HttpsDoLogin;
use crate::sessao_login::{Conexao, ErroLogin, PedidoDeLogin, ResultadoDoLogin, entrar_colhendo};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::process::ExitCode;

/// Espera máxima pelo lock de outro `ies login`.
pub(crate) const ESPERA_DO_LOCK: std::time::Duration = std::time::Duration::from_secs(120);

/// JSON do sucesso.
pub(crate) fn json_do_resultado(resultado: &ResultadoDoLogin) -> Value {
    json!({
        "instalacao": resultado.instalacao,
        "unidade_exibida": resultado.unidade_exibida,
        "conexao": resultado.conexao.rotulo(),
        "requisicoes": resultado.requisicoes,
    })
}

/// JSON da falha `fonte_invalida`, com `status_http` e `mensagem_do_servidor`
/// quando houver.
pub(crate) fn json_do_erro(erro: &ErroLogin) -> Value {
    let mut valor = json_de_erro_fonte(erro.mensagem(), erro.status_http());
    if let (Some(texto), Some(objeto)) = (erro.mensagem_do_servidor(), valor.as_object_mut()) {
        objeto.insert("mensagem_do_servidor".to_owned(), json!(texto));
    }
    if let (Some(campo), Some(objeto)) = (erro.campo_das_credenciais(), valor.as_object_mut()) {
        objeto.insert("campo".to_owned(), json!(campo));
    }
    valor
}

/// A falha do relogin nos comandos de leitura ainda não convertidos (#695).
pub(crate) fn falhar(json_mode: bool, erro: &ErroLogin) -> ExitCode {
    if json_mode {
        println!("{}", json_do_erro(erro));
    } else {
        let linha = linha_humana(erro.mensagem(), erro.status_http());
        match (erro.status_http(), erro.mensagem_do_servidor()) {
            (None, Some(texto)) => eprintln!("{linha} ({texto})"),
            _ => eprintln!("{linha}"),
        }
    }
    ExitCode::from(1)
}

pub(crate) fn executar_login(
    json_mode: bool,
    forcar: bool,
    unidade: Option<&str>,
) -> anyhow::Result<()> {
    let arquivos = configuracao::arquivos_de_acesso().map_err(ErroLogin::Configuracao)?;
    let credenciais = std::env::var_os("IES_CREDENCIAIS_FILE")
        .filter(|valor| !valor.is_empty())
        .map(PathBuf::from);
    let pedido = PedidoDeLogin {
        arquivos: &arquivos,
        credenciais: credenciais.as_deref(),
        forcar,
        unidade_esperada: unidade,
        espera_do_lock: ESPERA_DO_LOCK,
    };
    let indice = IndiceLocal::da_configuracao(&arquivos);
    let resultado = entrar_colhendo(&pedido, &indice, &mut HttpsDoLogin)?;
    if json_mode {
        println!("{}", json_do_resultado(&resultado));
    } else {
        let como = match resultado.conexao {
            Conexao::LoginRealizado => "login realizado",
            Conexao::SessaoExistenteValida => "sessão gravada ainda válida",
        };
        println!(
            "IES: {como} na unidade {} ({} requisições).",
            resultado.unidade_exibida, resultado.requisicoes
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

    #[test]
    fn sucesso_publica_so_instalacao_unidade_conexao_e_requisicoes() {
        let valor = json_do_resultado(&ResultadoDoLogin {
            instalacao: "https://sei.exemplo.gov.br/sei".to_owned(),
            unidade_exibida: "UNIDADE-TESTE".to_owned(),
            conexao: Conexao::LoginRealizado,
            requisicoes: 5,
        });
        assert_eq!(
            valor,
            serde_json::json!({
                "instalacao": "https://sei.exemplo.gov.br/sei",
                "unidade_exibida": "UNIDADE-TESTE",
                "conexao": "login_realizado",
                "requisicoes": 5
            })
        );
    }

    #[test]
    fn recusa_de_credencial_publica_a_mensagem_sem_status_http() {
        let valor = json_do_erro(&ErroLogin::CredencialRecusada {
            mensagem_do_servidor: Some("Senha inválida.".to_owned()),
        });
        assert_eq!(valor["codigo"], "fonte_invalida");
        assert_eq!(valor["mensagem_do_servidor"], "Senha inválida.");
        assert!(valor.get("status_http").is_none());
        let valor = json_do_erro(&ErroLogin::Transporte(FalhaTransporte::Status(
            StatusHttpRecusado {
                status: 502,
                mensagem_do_servidor: Some("Bad Gateway".to_owned()),
            },
        )));
        assert_eq!(valor["status_http"], 502);
        assert_eq!(valor["mensagem_do_servidor"], "Bad Gateway");
        let valor = json_do_erro(&ErroLogin::CaptchaExigido);
        assert!(valor.get("mensagem_do_servidor").is_none());
    }
}
