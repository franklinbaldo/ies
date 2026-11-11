//! Pesquisa em documentos (`ies documento buscar`, Refs #61): a mesma tela
//! e o mesmo fluxo de `processo buscar`, com `#optDocumentos`. Formas
//! observadas nas 26 pesquisas em documentos do acervo público e nas
//! páginas de `pesquisa-paginas-*` (sete instalações): linhas com o link do
//! processo (com `id_documento`), o do documento (`documento_visualizar` ou
//! `documento_download_anexo`) e o número SEI à direita; o número SEI exato
//! responde 302 para `procedimento_trabalhar` com `id_documento`.
use super::ErroResultadoPesquisa;
use super::resposta::FaixaDaPesquisa;
use crate::dominio::IdProcedimento;
use crate::html_atributos::ErroAtributoHtml;
use crate::processo::{NumeroProcesso, NumeroProcessoInvalido};

mod envio;
mod linhas;

pub(crate) use envio::EmDocumentos;
#[cfg(test)]
pub(crate) use envio::ler_resposta_de_documentos;
pub(crate) use linhas::ler_documentos;

/// Um documento da página observada: só o que a linha mostra.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DocumentoEncontrado {
    /// Texto do número SEI à direita, opaco.
    pub(crate) numero_sei: String,
    /// Texto do link do documento (tipo e número na árvore).
    pub(crate) rotulo: String,
    /// NUP do processo, opaco.
    pub(crate) processo: NumeroProcesso,
    /// Sigla de "Unidade:"; `None` quando a linha não mostra.
    pub(crate) unidade: Option<String>,
    /// "Inclusão:" como mostrada (`dd/mm/aaaa`); `None` quando ausente.
    pub(crate) inclusao: Option<String>,
    /// O link do documento é `documento_download_anexo`.
    pub(crate) anexo: bool,
}

/// Uma página da lista de documentos, com total e faixa da barra.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ListaDeDocumentos {
    pub(crate) documentos: Vec<DocumentoEncontrado>,
    pub(crate) total_declarado: u64,
    pub(crate) faixa: FaixaDaPesquisa,
    /// Posições da faixa da barra sem linha na tabela (Goiás 5.0.3).
    pub(crate) linhas_ausentes_da_faixa: u64,
}

/// As três respostas observadas da pesquisa em documentos.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RespostaDeDocumentos {
    /// 302 interpretado (nunca seguido): só os ids internos.
    DocumentoUnico {
        documento: String,
        processo: IdProcedimento,
    },
    Lista(ListaDeDocumentos),
    /// Nada para esta conta/unidade; não prova inexistência.
    SemResultado,
}

/// um erro por fato verificável da linha de documento.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroDocumentoDaPesquisa {
    /// Recusa da leitura comum do link do processo e da árvore.
    Linha(#[from] ErroResultadoPesquisa),
    AtributoHtml(#[from] ErroAtributoHtml),
    LinhaDeDocumentoSemIdDocumentoNoLinkDoProcesso,
    LinkDoDocumentoDiferenteDoProcesso,
    NumeroSeiComLinkDiferenteDoDocumento,
    LinhaDaPesquisaForaDeUmRegistro,
    NumeroDoProcesso(#[from] NumeroProcessoInvalido),
    CelulaDoTituloDoDocumentoAusenteOuRepetida,
    DataDeInclusaoForaDaFormaObservada,
    LinkDaLinhaDeDocumentoNaoReconhecido,
    LinkDoDocumentoAusenteNaLinha,
    LinksDoDocumentoRepetidosNaLinha,
    MetadadoDoDocumentoRepetido,
    NumeroSeiAusenteNaLinha,
    NumeroSeiRepetidoNaPesquisa,
    RotuloDoDocumentoAusente,
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_fluxo;

#[cfg(test)]
pub(crate) mod tests_fixtures;
