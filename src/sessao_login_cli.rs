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
