//! Falhas de `ies documento DOC` pela Web (Refs #60): as de abertura do
//! processo (reaproveitadas de `processo NUP`) e uma variante por fato
//! verificável do documento (contracts/cli/documento-obter-web.md).
use super::super::ErroProcessoObter;
use crate::arvore_processo::ErroConteudoNaArvore;
use crate::documento_web::{ErroCorpoDocumento, ErroTituloDocumento};
use crate::transporte::StatusHttpRecusado;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ErroDocumentoObter {
    /// Sessão, Controle, pesquisa rápida, 302, tela ou árvore.
    Processo(ErroProcessoObter),
    /// A pesquisa rápida respondeu sem o 302 para a tela de trabalho.
    PesquisaRapidaNaoAbriuODocumento(StatusHttpRecusado),
    /// O nó do `id_documento` aberto traz outro número SEI (a15).
    NumeroSeiDaArvoreDiferenteDoPedido,
    /// Mais de um `DOCUMENTO` da árvore com o número pedido.
    NumeroSeiRepetidoNaArvore,
    /// `--processo NUP`: a árvore do processo, com as pastas abertas, não
    /// tem `DOCUMENTO` com o número pedido.
    NumeroSeiAusenteDoProcesso,
    /// O `.src` do nó (ausente, repetido, fora da forma observada).
    Conteudo(ErroConteudoNaArvore),
    /// `.src` vazio num nó com ícone e dica de cancelado.
    DocumentoCancelado,
    /// `.src` vazio: a árvore não publica o conteúdo para esta sessão.
    ConteudoNaoPublicadoNaArvore,
    /// `.src` para `documento_download_anexo`: externo, fora desta fatia.
    DocumentoExterno,
    /// O título do HTML servido.
    Titulo(ErroTituloDocumento),
    /// O título servido é de outro número SEI.
    TituloDeOutroDocumento,
    /// O corpo do HTML servido.
    Corpo(ErroCorpoDocumento),
}

impl ErroDocumentoObter {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Processo(erro) => erro.subcodigo(),
            Self::PesquisaRapidaNaoAbriuODocumento(_) => "pesquisa_rapida_nao_abriu_o_documento",
            Self::NumeroSeiDaArvoreDiferenteDoPedido => "numero_sei_da_arvore_diferente_do_pedido",
            Self::NumeroSeiRepetidoNaArvore => "numero_sei_repetido_na_arvore",
            Self::NumeroSeiAusenteDoProcesso => "numero_sei_ausente_da_arvore_do_processo",
            Self::Conteudo(erro) => erro.subcodigo(),
            Self::DocumentoCancelado => "documento_cancelado_sem_conteudo_na_arvore",
            Self::ConteudoNaoPublicadoNaArvore => "conteudo_nao_publicado_na_arvore",
            Self::DocumentoExterno => "documento_externo_por_download_de_anexo",
            Self::Titulo(erro) => erro.subcodigo(),
            Self::TituloDeOutroDocumento => "titulo_do_conteudo_de_outro_documento",
            Self::Corpo(erro) => erro.subcodigo(),
        }
    }

    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::Processo(erro) => erro.mensagem(),
            Self::PesquisaRapidaNaoAbriuODocumento(_) => {
                "a pesquisa rápida não abriu o documento (resultado não é lido)"
            }
            Self::NumeroSeiDaArvoreDiferenteDoPedido => {
                "o documento aberto não tem o número SEI pedido"
            }
            Self::NumeroSeiRepetidoNaArvore => "o número SEI aparece mais de uma vez na árvore",
            Self::NumeroSeiAusenteDoProcesso => {
                "o número SEI não está na árvore do processo informado"
            }
            Self::Conteudo(_) => "o link do conteúdo na árvore não foi reconhecido",
            Self::DocumentoCancelado => {
                "o documento está cancelado e a árvore não publica conteúdo"
            }
            Self::ConteudoNaoPublicadoNaArvore => {
                "a árvore não publica o conteúdo deste documento para a sessão"
            }
            Self::DocumentoExterno => {
                "documento externo (download de anexo) ainda não é lido por esta fatia"
            }
            Self::Titulo(_) => "o título do conteúdo não foi reconhecido",
            Self::TituloDeOutroDocumento => "o conteúdo servido é de outro número SEI",
            Self::Corpo(_) => "o corpo do conteúdo não foi reconhecido",
        }
    }

    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Processo(erro) => erro.status_http(),
            Self::PesquisaRapidaNaoAbriuODocumento(status) => Some(status),
            _ => None,
        }
    }
}
