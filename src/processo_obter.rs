//! `ies processo NUP` (contracts/cli/processo-obter-arvore.md):
//! abre o processo pelo caminho mais barato que o índice local permite e lê
//! a árvore, com as pastas. Ordem aprovada: link da árvore guardado →
//! link `procedimento_trabalhar` guardado (colhido do Controle) → entrada
//! por id → pesquisa rápida guardada → Controle (linha do NUP ou pesquisa
//! rápida do formulário). Pasta fechada soma o GET de "Abrir
//! todas as Pastas". Sem retry, sem redirect seguido; cada palpite errado
//! do índice é esquecido e o comando segue.
mod abertura;
#[cfg(test)]
mod acervo;
#[cfg(test)]
mod acervo_documento;
#[cfg(test)]
mod acervo_versoes;
mod acoes;
mod andamentos;
pub(crate) mod consulta_parcial;
mod controle;
mod documento;
mod documento_indice;
pub(crate) mod erro;
mod fluxo;
pub(crate) mod login;
mod marcadores;
mod pastas;
mod propriedades;
mod validacao_acesso;

#[cfg(test)]
mod tests_consulta_parcial;

#[cfg(test)]
mod tests_andamentos;
#[cfg(test)]
mod tests_andamentos_indice;
#[cfg(test)]
mod tests_anexado_a;
#[cfg(test)]
mod tests_anexados;
#[cfg(test)]
mod tests_apoio;
#[cfg(test)]
mod tests_colheita;
#[cfg(test)]
mod tests_controle;
#[cfg(test)]
mod tests_documento;
#[cfg(test)]
mod tests_documento_antiies;
#[cfg(test)]
mod tests_documento_externo;
#[cfg(test)]
mod tests_documento_indice;
#[cfg(test)]
mod tests_documento_versoes;
#[cfg(test)]
mod tests_fluxo;

#[cfg(test)]
mod tests_indice;
#[cfg(test)]
mod tests_login;
#[cfg(test)]
mod tests_marcadores;
#[cfg(test)]
mod tests_movidos;
#[cfg(test)]
mod tests_pastas;
#[cfg(test)]
mod tests_propriedades;
#[cfg(test)]
mod tests_sessao;

use crate::arvore_processo::ArvoreLida;
use crate::formulario_web::RespostaDoPost;
use crate::historico_processo::AndamentosLidos;
use crate::marcadores_processo::MarcadoresLidos;
use crate::processo::NumeroProcesso;
use crate::propriedades_processo::PropriedadesLidas;
use crate::transporte::FalhaTransporte;
use crate::transporte::resposta_binaria::RespostaBinaria;
use crate::transporte::sessao::CookieDaSessao;
use crate::visao_do_controle::{EnvioDoControle, EnvioHttps};
pub(crate) use abertura::{CAMPO as CAMPO_DA_PESQUISA_RAPIDA, PESQUISA_RAPIDA};
#[cfg(test)]
pub(crate) use documento::obter_documento_com_versoes;
pub(crate) use documento::{
    DocumentoNaArvoreAberto, DocumentoObtido, ErroDocumentoObter, PedidoDeVersoes, PedidoDoAnexo,
    PedidoDoDocumento, abrir_documento_na_arvore, obter_documento_com_propriedades,
};
#[cfg(test)]
pub(crate) use documento::{FotoDaVersao, SemVersaoAtual, VersoesDoDocumento, obter_documento};
pub(crate) use erro::ErroProcessoObter;
pub(crate) use fluxo::{abrir_para_escrever, obter_processo};

/// GET que devolve também o `Location` (302 como dado, nunca seguido).
pub(crate) trait EnvioDoProcesso: EnvioDoControle {
    fn get_sem_seguir(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte>;

    /// GET binário do link do anexo (Refs #60, fatia 3): `limite` é o do
    /// total em `chunked`; 3xx volta como dado, nunca seguido.
    fn get_binario(
        &mut self,
        url: &str,
        cookie: &str,
        limite: usize,
    ) -> Result<RespostaBinaria<Self::Corpo>, FalhaTransporte>;
}

impl EnvioDoProcesso for EnvioHttps {
    fn get_sem_seguir(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        crate::transporte::envio_post::get_com_localizacao(url, cookie)
    }

    fn get_binario(
        &mut self,
        url: &str,
        cookie: &str,
        limite: usize,
    ) -> Result<RespostaBinaria<Self::Corpo>, FalhaTransporte> {
        crate::transporte::resposta_binaria::get_binario(url, cookie, limite)
    }
}

/// Arquivos privados da sessão já lidos.
pub(crate) struct SessaoDoProcesso<'a> {
    pub(crate) url_do_controle: &'a str,
    pub(crate) origem: &'a str,
    pub(crate) cookie: &'a CookieDaSessao,
}

/// O NUP pedido, a `--unidade` (conferida na primeira tela que a publica)
/// e o que ler do histórico.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LeituraDasPastas {
    Estrita,
    Informativa,
}

#[derive(Clone, Copy)]
pub(crate) struct PedidoDoProcesso<'a> {
    pub(crate) nup: &'a NumeroProcesso,
    pub(crate) unidade: Option<&'a str>,
    pub(crate) andamentos: PedidoDeAndamentos,
    /// `false` com `--sem-propriedades`.
    pub(crate) propriedades: bool,
    /// `false` com `--sem-marcadores`.
    pub(crate) marcadores: bool,
    /// Pula o link guardado da árvore (passo 1), que não publica a unidade:
    /// quem escreve precisa da unidade da sessão (`processo concluir`).
    pub(crate) unidade_exigida: bool,
    /// Consulta informativa tolera somente lacunas de documentos comprovadas.
    pub(crate) leitura_das_pastas: LeituraDasPastas,
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
    IndiceTrabalhar,
    IndiceEntrada,
    /// A entrada por id com `id_documento` (`documento DOC`, fatia 2).
    IndiceEntradaDocumento,
    PesquisaRapida,
    Caixa,
}

impl ResolvidoPor {
    pub(crate) fn como_texto(self) -> &'static str {
        match self {
            Self::IndiceLink => "indice_link",
            Self::IndiceTrabalhar => "indice_trabalhar",
            Self::IndiceEntrada => "indice_entrada",
            Self::IndiceEntradaDocumento => "indice_entrada_documento",
            Self::PesquisaRapida => "pesquisa_rapida",
            Self::Caixa => "caixa",
        }
    }
}

/// O processo aberto: árvore conferida, unidade publicada pela primeira
/// tela que a mostrou (`None` no link da árvore: ela não a publica),
/// passo, andamentos, propriedades e marcadores (cada um lido ou omitido
/// por pedido do usuário) e requisições feitas (inclusive as dos palpites
/// que falharam).
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ProcessoAberto {
    pub(crate) arvore: ArvoreLida,
    pub(crate) unidade: Option<String>,
    pub(crate) resolvido_por: ResolvidoPor,
    pub(crate) andamentos: Andamentos,
    pub(crate) propriedades: Propriedades,
    pub(crate) marcadores: Marcadores,
    pub(crate) requisicoes: u8,
}

/// O que se sabe dos andamentos depois do comando: lidos, ou omitidos por
/// `--sem-andamentos`. Link do histórico ausente na árvore é erro tipado.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Andamentos {
    Lidos(AndamentosLidos),
    /// `--sem-andamentos`: nenhum pedido ao histórico.
    Omitidos,
}

impl Andamentos {
    #[cfg(test)]
    pub(crate) fn lidos(&self) -> Option<&AndamentosLidos> {
        match self {
            Self::Lidos(lidos) => Some(lidos),
            Self::Omitidos => None,
        }
    }

    /// `andamentos_nao_lidos_motivo` publicado quando foram omitidos.
    pub(crate) fn motivo(&self) -> Option<&'static str> {
        match self {
            Self::Lidos(_) => None,
            Self::Omitidos => Some("omitidos_por_sem_andamentos"),
        }
    }
}

/// O que se sabe das propriedades depois do comando: lidas, ou omitidas
/// por `--sem-propriedades`. Barra sem a tela é erro tipado.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Propriedades {
    Lidas(PropriedadesLidas),
    /// `--sem-propriedades`: nenhum pedido à tela.
    Omitidas,
}

impl Propriedades {
    /// `propriedades_nao_lidas_motivo` publicado quando foram omitidas.
    pub(crate) fn motivo(&self) -> Option<&'static str> {
        match self {
            Self::Lidas(_) => None,
            Self::Omitidas => Some("omitidas_por_sem_propriedades"),
        }
    }
}

/// O que se sabe dos marcadores depois do comando: lidos (sempre lista),
/// ou omitidos por `--sem-marcadores`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Marcadores {
    Lidos(MarcadoresLidos),
    /// `--sem-marcadores`: nenhum pedido à tela.
    Omitidos,
}

impl Marcadores {
    /// `marcadores_nao_lidos_motivo` publicado quando foram omitidos.
    pub(crate) fn motivo(&self) -> Option<&'static str> {
        match self {
            Self::Lidos(_) => None,
            Self::Omitidos => Some("omitidos_por_sem_marcadores"),
        }
    }
}
