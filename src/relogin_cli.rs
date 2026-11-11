//! Borda do relogin único nas leituras (contracts/cli/sessao-login.md,
//! "Relogin único nas leituras"): o transporte real do login, as
//! credenciais de `IES_CREDENCIAIS_FILE`, a contagem de pedidos do
//! processo e os campos `relogin` e `requisicoes` da saída. Só os
//! comandos de leitura chamam [`ler`]; escrita e prévia nunca.
use crate::configuracao::ArquivosDeAcesso;
use crate::indice_local::IndiceLocal;
use crate::sessao_login::http::HttpsDoLogin;
use crate::sessao_login::relogin::{ContextoDoRelogin, ler_com_relogin};
use crate::transporte::cliente_https;
use serde_json::{Value, json};
use std::path::PathBuf;

pub(crate) use crate::sessao_login::relogin::{FalhaDaLeitura, NotaDoRelogin};

/// A leitura com o relogin único: credenciais só de `IES_CREDENCIAIS_FILE`
/// (ausente: comportamento sem relogin), lock do `ies login` e a contagem
/// de todos os pedidos do processo.
pub(crate) fn ler<T, E>(
    arquivos: &ArquivosDeAcesso,
    unidade: Option<&str>,
    sessao_encerrada: fn(&E) -> bool,
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
        sessao_encerrada,
        ler,
    )
}

/// A leitura falhou depois de recomeçar com a sessão nova: o fato da nota
/// (relogin feito aqui ou por outro processo e os pedidos já feitos) sobe
/// junto com o erro da releitura.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) struct LeituraRefeitaAposRelogin(pub(crate) NotaDoRelogin);

/// A falha que sobe: a do relogin, ou a da leitura com a nota como contexto
/// quando a leitura recomeçou.
pub(crate) fn subir<E>(falha: FalhaDaLeitura<E>) -> anyhow::Error
where
    E: std::error::Error + Send + Sync + 'static,
{
    match falha {
        FalhaDaLeitura::Relogin(erro) => erro.into(),
        FalhaDaLeitura::Leitura { erro, nota: None } => erro.into(),
        FalhaDaLeitura::Leitura {
            erro,
            nota: Some(nota),
        } => anyhow::Error::from(erro).context(LeituraRefeitaAposRelogin(nota)),
    }
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

/// `relogin` no objeto (ou em cada objeto do array da lista D).
fn em_cada_objeto(valor: &mut Value, mut f: impl FnMut(&mut serde_json::Map<String, Value>)) {
    if let Some(objeto) = valor.as_object_mut() {
        f(objeto);
    } else if let Some(itens) = valor.as_array_mut() {
        for item in itens {
            if let Some(objeto) = item.as_object_mut() {
                f(objeto);
            }
        }
    }
}

#[cfg(test)]
#[path = "relogin_cli/tests.rs"]
mod tests;
