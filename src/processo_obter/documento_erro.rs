//! Falhas de `ies documento DOC` pela Web (Refs #60): as de abertura do
//! processo (reaproveitadas de `processo NUP`) e um erro por fato
//! verificável do documento (contracts/cli/documento-obter-web.md).
use super::super::ErroProcessoObter;
use crate::anexo_documento::ErroAnexo;
use crate::arvore_processo::{ErroBarraDoDocumento, ErroConteudoNaArvore};
use crate::documento_versoes::ErroVersao;
use crate::documento_web::{ErroCorpoDocumento, ErroTituloDocumento};

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroDocumentoObter {
    /// Sessão, Controle, pesquisa rápida, 302, tela ou árvore.
    Processo(#[from] ErroProcessoObter),
    /// A pesquisa rápida respondeu sem o 302 para a tela de trabalho.
    PesquisaRapidaNaoAbriuODocumento,
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
    /// O link da versão não tem a forma observada.
    LinkDaVersaoRecusado(#[source] crate::arvore_processo::endereco::EnderecoRecusado),
    /// A raiz do cache de documentos não pôde ser resolvida.
    RaizDoCacheIlegivel(#[source] crate::cache_documento::FalhaCache),
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
    /// O cadastro (propriedades/anexo) publicado fora da forma observada.
    Cadastro(#[source] crate::propriedades_documento::ErroPropriedadesDocumento),
    /// `--com-propriedades` e a barra do documento não publica o cadastro.
    PropriedadesNaoPublicadasNaBarra,
    BarraDoDocumentoForaDaFormaObservada,
    DocumentoAusenteDaArvore,
    NumeroSeiAusenteDaArvoreDoProcesso,
    NumeroSeiRepetidoNaArvore,
}

impl ErroDocumentoObter {
    /// A sessão do SEI acabou (decide o relogin único): por `match` no tipo, sem
    /// percorrer a cadeia `source()`.
    pub(crate) fn sessao_encerrada(&self) -> bool {
        match self {
            Self::Processo(erro) => erro.sessao_encerrada(),
            Self::Anexo(erro) => erro.sessao_encerrada(),
            Self::PesquisaRapidaNaoAbriuODocumento
            | Self::NumeroSeiDaArvoreDiferenteDoPedido
            | Self::Conteudo(_)
            | Self::DocumentoCancelado
            | Self::ConteudoNaoPublicadoNaArvore { .. }
            | Self::Titulo(_)
            | Self::TituloDeOutroDocumento
            | Self::Corpo(_)
            | Self::BarraDoDocumento(_)
            | Self::ListaDeVersoesNaoPublicadaNaArvore
            | Self::Versoes(_)
            | Self::VersaoServidaDiferenteDaPedida
            | Self::ConteudoComLinkAssinado
            | Self::Cadastro(_)
            | Self::PropriedadesNaoPublicadasNaBarra
            | Self::NumeroSeiAusenteDaArvoreDoProcesso
            | Self::NumeroSeiRepetidoNaArvore
            | Self::DocumentoAusenteDaArvore
            | Self::BarraDoDocumentoForaDaFormaObservada
            | Self::LinkDaVersaoRecusado(_)
            | Self::RaizDoCacheIlegivel(_) => false,
        }
    }
}
