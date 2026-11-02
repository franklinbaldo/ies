//! `ies login [--forcar]`: borda da CLI. Credenciais só pelo arquivo
//! privado de `IES_CREDENCIAIS_FILE`; a saída nunca publica credencial,
//! cookie ou URL assinada.
use crate::sessao_login::{ErroLogin, ResultadoDoLogin};
use serde_json::Value;
use std::process::ExitCode;

/// Espera máxima pelo lock de outro `ies login`.
pub(crate) const ESPERA_DO_LOCK: std::time::Duration = std::time::Duration::from_secs(120);

/// JSON do sucesso.
pub(crate) fn json_do_resultado(resultado: &ResultadoDoLogin) -> Value {
    let _ = resultado;
    todo!("CONTRACT: JSON do login")
}

/// JSON da falha `fonte_invalida`, com `status_http` e
/// `mensagem_do_servidor` quando houver.
pub(crate) fn json_do_erro(erro: &ErroLogin) -> Value {
    let _ = erro;
    todo!("CONTRACT: JSON da falha do login")
}

pub(crate) fn executar_login(json_mode: bool, forcar: bool, unidade: Option<&str>) -> ExitCode {
    let _ = (json_mode, forcar, unidade, ESPERA_DO_LOCK);
    let _ = crate::sessao_login::entrar_via::<crate::sessao_login::http::HttpsDoLogin>;
    todo!("CONTRACT: ies login")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sessao_login::Conexao;
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
        assert_eq!(valor["subcodigo"], "credencial_recusada");
        assert_eq!(valor["mensagem_do_servidor"], "Senha inválida.");
        assert!(valor.get("status_http").is_none());
        let valor = json_do_erro(&ErroLogin::Transporte(
            FalhaTransporte::StatusHttpDeErroDoServidor(StatusHttpRecusado {
                status: 502,
                mensagem_do_servidor: Some("Bad Gateway".to_owned()),
            }),
        ));
        assert_eq!(valor["status_http"], 502);
        assert_eq!(valor["mensagem_do_servidor"], "Bad Gateway");
        let valor = json_do_erro(&ErroLogin::CaptchaExigido);
        assert_eq!(valor["subcodigo"], "captcha_exigido");
        assert!(valor.get("mensagem_do_servidor").is_none());
    }
}
