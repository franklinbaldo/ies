//! `ies login [--forcar]`: borda da CLI. Credenciais só pelo arquivo
//! privado de `IES_CREDENCIAIS_FILE`; a saída nunca publica credencial,
//! cookie ou URL assinada.
use crate::configuracao;
use crate::indice_local::{self, IndiceLocal};
use crate::sessao_login::http::HttpsDoLogin;
use crate::sessao_login::{Conexao, ErroLogin, PedidoDeLogin, ResultadoDoLogin, entrar_colhendo};
use serde_json::{Value, json};
use std::path::PathBuf;

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
        let mut valor = json_do_resultado(&resultado);
        indice_local::anexar_avisos(&mut valor, &indice);
        println!("{valor}");
    } else {
        indice_local::avisar_no_texto(&indice);
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
}
