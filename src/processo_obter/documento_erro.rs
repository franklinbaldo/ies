//! Falhas de `ies documento DOC` pela Web (Refs #60): as de abertura do
//! processo (reaproveitadas de `processo NUP`) e uma variante por fato
//! verificável do documento (contracts/cli/documento-obter-web.md).
use super::super::ErroProcessoObter;
use crate::anexo_documento::ErroAnexo;
use crate::arvore_processo::{ErroBarraDoDocumento, ErroConteudoNaArvore};
use crate::documento_versoes::ErroVersao;
use crate::documento_web::{ErroCorpoDocumento, ErroTituloDocumento};
use crate::erro_html::ErroHtml;
use crate::transporte::StatusHttpRecusado;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroDocumentoObter {
    /// Sessão, Controle, pesquisa rápida, 302, tela ou árvore.
    Processo(#[from] ErroProcessoObter),
    /// A pesquisa rápida respondeu sem o 302 para a tela de trabalho.
    PesquisaRapidaNaoAbriuODocumento(StatusHttpRecusado),
    /// O nó do `id_documento` aberto traz outro número SEI (a15).
    NumeroSeiDaArvoreDiferenteDoPedido,
    /// O `.src` do nó (ausente, repetido, fora da forma observada).
    Conteudo(#[from] ErroConteudoNaArvore),
    /// `.src` vazio num nó com ícone e dica de cancelado.
    DocumentoCancelado,
    /// `.src` vazio: a árvore não publica o conteúdo para esta sessão.
    ConteudoNaoPublicadoNaArvore {
        requisicoes: u8,
    },
    /// O arquivo do documento externo (`documento_download_anexo`, fatia 3).
    Anexo(#[from] ErroAnexo),
    /// O título do HTML servido.
    Titulo(#[from] ErroTituloDocumento),
    /// O título servido é de outro número SEI.
    TituloDeOutroDocumento,
    /// O corpo do HTML servido.
    Corpo(#[from] ErroCorpoDocumento),
    /// A barra do `DOCUMENTO` fora da forma observada (Refs #88).
    BarraDoDocumento(#[from] ErroBarraDoDocumento),
    /// `--versao N` e a barra do documento não publica a lista de versões
    /// (inclusive no externo, recusado antes do GET do anexo).
    ListaDeVersoesNaoPublicadaNaArvore,
    /// A lista de versões servida, ou N fora dela.
    Versoes(#[from] ErroVersao),
    /// O rodapé da versão servida declara outra versão.
    VersaoServidaDiferenteDaPedida,
    /// `--saida`: o HTML servido traz `infra_hash`; a foto não é gravada (#207).
    ConteudoComLinkAssinado,
    Html(#[from] ErroHtml),
}

impl ErroDocumentoObter {
    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::Html(e) => e.mensagem,
            Self::Processo(erro) => erro.mensagem(),
            Self::PesquisaRapidaNaoAbriuODocumento(_) => {
                "a pesquisa rápida não abriu o documento (resultado não é lido)"
            }
            Self::NumeroSeiDaArvoreDiferenteDoPedido => {
                "o documento aberto não tem o número SEI pedido"
            }
            Self::Conteudo(_) => "o link do conteúdo na árvore não foi reconhecido",
            Self::DocumentoCancelado => {
                "o documento está cancelado e a árvore não publica conteúdo"
            }
            Self::ConteudoNaoPublicadoNaArvore { .. } => {
                "a árvore não publica o conteúdo deste documento para a sessão"
            }
            Self::Anexo(erro) => erro.mensagem(),
            Self::Titulo(_) => "o título do conteúdo não foi reconhecido",
            Self::TituloDeOutroDocumento => "o conteúdo servido é de outro número SEI",
            Self::Corpo(_) => "o corpo do conteúdo não foi reconhecido",
            Self::BarraDoDocumento(_) => "a barra do documento na árvore não foi reconhecida",
            Self::ListaDeVersoesNaoPublicadaNaArvore => {
                "a árvore não publica a lista de versões deste documento para a sessão"
            }
            Self::Versoes(ErroVersao::VersaoNaoObservada) => {
                "a versão pedida não está na lista de versões do documento"
            }
            Self::Versoes(_) => "a lista de versões não foi reconhecida",
            Self::VersaoServidaDiferenteDaPedida => {
                "o rodapé do conteúdo servido declara outra versão"
            }
            Self::ConteudoComLinkAssinado => {
                "o conteúdo servido traz um link assinado e não é gravado"
            }
        }
    }

    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Processo(erro) => erro.status_http(),
            Self::PesquisaRapidaNaoAbriuODocumento(status) => Some(status),
            Self::Anexo(erro) => erro.status_http(),
            _ => None,
        }
    }
}
