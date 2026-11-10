//! Relogin único depois de sessão vencida, só nos comandos de leitura
//! (contracts/cli/sessao-login.md, "Relogin único nas leituras"; decisão
//! em docs/decisoes/sessao-login/relogin-de-leitura.md). Só a falha
//! tipada de sessão encerrada dispara; antes de logar, sob o lock do
//! `ies login`, a geração gravada é comparada com a do início do comando:
//! se mudou, outro processo já relogou e a leitura usa a sessão nova sem
//! login. No máximo um relogin; a leitura recomeça do início uma vez.
//! Escrita e prévia de escrita nunca chamam este módulo.
use super::{ErroLogin, PedidoDeLogin, TransporteDeLogin, fluxo, gravacao};
use crate::configuracao::ArquivosDeAcesso;
use crate::indice_local::IndiceLocal;
use std::path::Path;
use std::time::Duration;

/// A leitura recomeçou: `relogin` diz se ESTE processo logou (com a
/// geração já trocada por outro, `false`); `antes` conta os pedidos da
/// primeira tentativa e do login; `releitura`, os da segunda tentativa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NotaDoRelogin {
    pub(crate) relogin: bool,
    pub(crate) antes: usize,
    pub(crate) releitura: usize,
}

impl NotaDoRelogin {
    /// Todos os pedidos do comando.
    pub(crate) fn total(self) -> usize {
        self.antes.saturating_add(self.releitura)
    }
}

/// A falha da leitura (com a nota, se ela recomeçou) ou a do relogin.
#[derive(Debug)]
pub(crate) enum FalhaDaLeitura<E> {
    Leitura {
        erro: E,
        nota: Option<NotaDoRelogin>,
    },
    Relogin(ErroLogin),
}

/// O que o relogin precisa: arquivos da sessão, o caminho das credenciais
/// (`None` ou arquivo ausente: comportamento sem relogin), a unidade
/// pedida, a espera do lock e o índice para colher o Controle do login.
pub(crate) struct ContextoDoRelogin<'a> {
    pub(crate) arquivos: &'a ArquivosDeAcesso,
    pub(crate) credenciais: Option<&'a Path>,
    pub(crate) unidade_esperada: Option<&'a str>,
    pub(crate) espera_do_lock: Duration,
    pub(crate) indice: &'a IndiceLocal,
}

/// Credenciais presentes: o caminho existe (link simbólico e permissões
/// ficam com a leitura do login, que os recusa tipados).
fn credenciais_presentes(caminho: Option<&Path>) -> Option<&Path> {
    caminho.filter(|c| std::fs::symlink_metadata(c).is_ok())
}

/// Lê; se a falha é a de sessão encerrada e há credenciais, reloga (ou
/// aproveita a geração nova) e lê de novo, uma única vez. `medidor` é o
/// total de pedidos já feitos pelo processo (todos os transportes).
pub(crate) fn ler_com_relogin<T, E, L: TransporteDeLogin>(
    contexto: &ContextoDoRelogin<'_>,
    transporte: &mut L,
    medidor: &dyn Fn() -> usize,
    sessao_encerrada: impl Fn(&E) -> bool,
    mut ler: impl FnMut() -> Result<T, E>,
) -> Result<(T, Option<NotaDoRelogin>), FalhaDaLeitura<E>> {
    let inicio = medidor();
    let inicial = gravacao::geracao_gravada(contexto.arquivos);
    let erro = match ler() {
        Ok(valor) => return Ok((valor, None)),
        Err(erro) => erro,
    };
    let credenciais = credenciais_presentes(contexto.credenciais);
    // Só o fato tipado `erro_publico::SessaoEncerrada` na cadeia `source()` dispara; nenhum outro
    // (403, WAF, 3xx, transporte, tela inesperada) reloga.
    if !sessao_encerrada(&erro) || credenciais.is_none() {
        return Err(FalhaDaLeitura::Leitura { erro, nota: None });
    }
    let pedido = PedidoDeLogin {
        arquivos: contexto.arquivos,
        credenciais,
        forcar: true,
        unidade_esperada: contexto.unidade_esperada,
        espera_do_lock: contexto.espera_do_lock,
    };
    let relogado = fluxo::relogar(&pedido, inicial.as_deref(), contexto.indice, transporte)
        .map_err(FalhaDaLeitura::Relogin)?;
    let meio = medidor();
    let mut nota = NotaDoRelogin {
        relogin: relogado == fluxo::Relogado::LoginRealizado,
        antes: meio.saturating_sub(inicio),
        releitura: 0,
    };
    let segunda = ler();
    nota.releitura = medidor().saturating_sub(meio);
    match segunda {
        Ok(valor) => Ok((valor, Some(nota))),
        Err(erro) => Err(FalhaDaLeitura::Leitura {
            erro,
            nota: Some(nota),
        }),
    }
}
