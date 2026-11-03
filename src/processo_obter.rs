//! `ies processo NUP` sem `--resumo` (contracts/cli/processo-obter-arvore.md):
//! abre o processo pelo caminho mais barato que o índice local permite e lê
//! o primeiro nível da árvore. Ordem aprovada: link da árvore guardado →
//! entrada por id → pesquisa rápida guardada → Controle (linha do NUP ou
//! pesquisa rápida do formulário). Sem retry, sem redirect seguido; cada
//! palpite errado do índice é esquecido e o comando segue.
mod abertura;
mod andamentos;
mod controle;
pub(crate) mod erro;
mod fluxo;
mod login;

#[cfg(test)]
mod tests_andamentos;
#[cfg(test)]
mod tests_andamentos_indice;
#[cfg(test)]
mod tests_apoio;
#[cfg(test)]
mod tests_controle;
#[cfg(test)]
mod tests_fluxo;
#[cfg(test)]
mod tests_indice;
#[cfg(test)]
mod tests_login;

use crate::arvore_processo::ArvoreLida;
use crate::formulario_web::RespostaDoPost;
use crate::historico_processo::AndamentosLidos;
use crate::processo::NumeroProcesso;
use crate::transporte::FalhaTransporte;
use crate::visao_do_controle::{EnvioDoControle, EnvioHttps};
pub(crate) use erro::ErroProcessoObter;
pub(crate) use fluxo::obter_processo;
use std::path::Path;

/// GET que devolve também o `Location` (302 como dado, nunca seguido).
pub(crate) trait EnvioDoProcesso: EnvioDoControle {
    fn get_sem_seguir(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte>;
}

impl EnvioDoProcesso for EnvioHttps {
    fn get_sem_seguir(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        crate::transporte::envio_post::get_com_localizacao(url, cookie)
    }
}

/// Arquivos privados da sessão já lidos.
pub(crate) struct SessaoDoProcesso<'a> {
    pub(crate) url_do_controle: &'a str,
    pub(crate) origem: &'a str,
    pub(crate) cookie_file: &'a Path,
}

/// O NUP pedido, a `--unidade` (conferida na primeira tela que a publica)
/// e o que ler do histórico.
pub(crate) struct PedidoDoProcesso<'a> {
    pub(crate) nup: &'a NumeroProcesso,
    pub(crate) unidade: Option<&'a str>,
    pub(crate) andamentos: PedidoDeAndamentos,
}

/// `--sem-andamentos`, padrão (a visão que o GET servir) ou
/// `--andamentos-completos` (a visão `P`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PedidoDeAndamentos {
    Omitidos,
    VisaoServida,
    Completos,
}

/// Qual passo da resolução abriu o processo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ResolvidoPor {
    IndiceLink,
    IndiceEntrada,
    PesquisaRapida,
    Caixa,
}

impl ResolvidoPor {
    pub(crate) fn como_texto(self) -> &'static str {
        match self {
            Self::IndiceLink => "indice_link",
            Self::IndiceEntrada => "indice_entrada",
            Self::PesquisaRapida => "pesquisa_rapida",
            Self::Caixa => "caixa",
        }
    }
}

/// O processo aberto: árvore conferida, unidade publicada pela primeira
/// tela que a mostrou (`None` no link da árvore: ela não a publica),
/// passo, andamentos (`None` com `--sem-andamentos`) e requisições feitas
/// (inclusive as dos palpites que falharam).
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ProcessoAberto {
    pub(crate) arvore: ArvoreLida,
    pub(crate) unidade: Option<String>,
    pub(crate) resolvido_por: ResolvidoPor,
    pub(crate) andamentos: Andamentos,
    pub(crate) requisicoes: u8,
}

/// O que se sabe dos andamentos depois do comando.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Andamentos {
    Lidos(AndamentosLidos),
    /// `--sem-andamentos`: nenhum pedido ao histórico.
    Omitidos,
    /// A árvore não publicou o link em nenhuma das formas observadas
    /// (visto em GO); não é recusa: a árvore já provou a identidade.
    LinkNaoPublicadoNaArvore,
}

impl Andamentos {
    pub(crate) fn lidos(&self) -> Option<&AndamentosLidos> {
        match self {
            Self::Lidos(lidos) => Some(lidos),
            _ => None,
        }
    }

    /// `andamentos_nao_lidos_motivo` publicado quando não foram lidos.
    pub(crate) fn motivo(&self) -> Option<&'static str> {
        match self {
            Self::Lidos(_) => None,
            Self::Omitidos => Some("omitidos_por_sem_andamentos"),
            Self::LinkNaoPublicadoNaArvore => Some("link_do_historico_nao_publicado_na_arvore"),
        }
    }
}
