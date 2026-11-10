//! Borda do relogin único nas leituras (contracts/cli/sessao-login.md,
//! "Relogin único nas leituras"): o transporte real do login, as
//! credenciais de `IES_CREDENCIAIS_FILE`, a contagem de pedidos do
//! processo e os campos `relogin` e `requisicoes` da saída. Só os
//! comandos de leitura chamam [`ler`]; escrita e prévia nunca.
use crate::configuracao::ArquivosDeAcesso;
use crate::indice_local::IndiceLocal;
use crate::saida_erro;
use crate::sessao_login::http::HttpsDoLogin;
use crate::sessao_login::relogin::{ContextoDoRelogin, ler_com_relogin};
use crate::transporte::{FalhaTransporte, StatusHttpRecusado, cliente_https};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::process::ExitCode;

pub(crate) use crate::sessao_login::relogin::{FalhaDaLeitura, NotaDoRelogin};

/// Falha de leitura publicável como `fonte_invalida`.
pub(crate) trait FalhaPublicavel {
    fn mensagem(&self) -> &'static str;
    fn status_http(&self) -> Option<&StatusHttpRecusado>;
    /// O fato tipado que dispara o relogin único.
    fn sessao_encerrada(&self) -> bool;
}

macro_rules! publicavel {
    ($($tipo:ty),*) => {$(
        impl FalhaPublicavel for $tipo {
            fn mensagem(&self) -> &'static str {
                <$tipo>::mensagem(self)
            }
            fn status_http(&self) -> Option<&StatusHttpRecusado> {
                <$tipo>::status_http(self)
            }
            fn sessao_encerrada(&self) -> bool {
                crate::erro_publico::Causa::sessao_encerrada(self)
            }
        }
    )*};
}

publicavel!(
    FalhaTransporte,
    crate::processo_obter::ErroProcessoObter,
    crate::processo_obter::ErroDocumentoObter,
    crate::pesquisa_web::erro_busca::ErroBuscaDeProcesso,
    crate::visao_do_controle::ErroVisaoDoControle
);

/// A leitura com o relogin único: credenciais só de `IES_CREDENCIAIS_FILE`
/// (ausente: comportamento sem relogin), lock do `ies login` e a contagem
/// de todos os pedidos do processo.
pub(crate) fn ler<T, E: FalhaPublicavel>(
    arquivos: &ArquivosDeAcesso,
    unidade: Option<&str>,
    ler: impl FnMut() -> Result<T, E>,
) -> Result<(T, Option<NotaDoRelogin>), FalhaDaLeitura<E>> {
    let credenciais = std::env::var_os("IES_CREDENCIAIS_FILE")
        .filter(|valor| !valor.is_empty())
        .map(PathBuf::from);
    let indice = IndiceLocal::da_configuracao(arquivos);
    let contexto = ContextoDoRelogin {
        arquivos,
        credenciais: credenciais.as_deref(),
        unidade_esperada: unidade,
        espera_do_lock: crate::sessao_login_cli::ESPERA_DO_LOCK,
        indice: &indice,
    };
    ler_com_relogin(
        &contexto,
        &mut HttpsDoLogin,
        &cliente_https::pedidos_do_processo,
        E::sessao_encerrada,
        ler,
    )
}

/// `requisicoes` próprias da releitura mais as da primeira tentativa e do
/// login; saturam em 255 como as próprias.
pub(crate) fn somar(proprias: u8, nota: Option<&NotaDoRelogin>) -> u8 {
    let antes = nota.map_or(0, |n| n.antes);
    u8::try_from(usize::from(proprias).saturating_add(antes)).unwrap_or(u8::MAX)
}

/// `relogin: true` só quando este processo logou (senão, ausente).
pub(crate) fn anotar(valor: &mut Value, nota: Option<&NotaDoRelogin>) {
    if nota.is_some_and(|n| n.relogin) {
        em_cada_objeto(valor, |objeto| {
            objeto.insert("relogin".to_owned(), json!(true));
        });
    }
}

/// [`anotar`] e, se a leitura recomeçou, `requisicoes` com o total do
/// comando: para as saídas que não publicam a própria contagem.
pub(crate) fn anotar_com_total(valor: &mut Value, nota: Option<&NotaDoRelogin>) {
    anotar(valor, nota);
    if let Some(nota) = nota {
        em_cada_objeto(valor, |objeto| {
            objeto.insert("requisicoes".to_owned(), json!(nota.total()));
        });
    }
}

/// Sem `--json`, a linha de `stderr` que diz que a leitura recomeçou.
pub(crate) fn avisar(json_mode: bool, nota: Option<&NotaDoRelogin>) {
    let (false, Some(nota)) = (json_mode, nota) else {
        return;
    };
    if nota.relogin {
        eprintln!(
            "IES: a sessão tinha vencido; relogin feito e leitura refeita do início ({} requisições no total).",
            nota.total()
        );
    } else {
        eprintln!(
            "IES: a sessão tinha vencido e outro processo já tinha relogado; leitura refeita com a sessão nova ({} requisições no total).",
            nota.total()
        );
    }
}

/// O JSON da falha de leitura, com `relogin` quando houve relogin.
pub(crate) fn json_da_falha(
    mensagem: &str,
    http: Option<&StatusHttpRecusado>,
    nota: Option<&NotaDoRelogin>,
) -> Value {
    let mut valor = saida_erro::json_de_erro_fonte(mensagem, http);
    anotar(&mut valor, nota);
    valor
}

/// Imprime a falha (da leitura ou do relogin) e devolve a saída 1.
pub(crate) fn publicar_falha<E: FalhaPublicavel>(
    json_mode: bool,
    falha: &FalhaDaLeitura<E>,
) -> ExitCode {
    match falha {
        FalhaDaLeitura::Relogin(erro) => crate::sessao_login_cli::falhar(json_mode, erro),
        FalhaDaLeitura::Leitura { erro, nota } => {
            let (msg, http) = (erro.mensagem(), erro.status_http());
            if json_mode {
                println!("{}", json_da_falha(msg, http, nota.as_ref()));
            } else {
                avisar(json_mode, nota.as_ref());
                eprintln!("{}", saida_erro::linha_humana(msg, http));
            }
            ExitCode::from(1)
        }
    }
}

/// `relogin` no objeto (ou em cada objeto do array da lista D).
fn em_cada_objeto(valor: &mut Value, mut f: impl FnMut(&mut serde_json::Map<String, Value>)) {
    match valor {
        Value::Object(objeto) => f(objeto),
        Value::Array(itens) => {
            for item in itens {
                if let Value::Object(objeto) = item {
                    f(objeto);
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
#[path = "relogin_cli/tests.rs"]
mod tests;
