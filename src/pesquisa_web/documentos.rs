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

/// Uma variante por fato verificável da linha de documento.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroDocumentoDaPesquisa {
    /// Recusa da leitura comum do link do processo e da árvore.
    Linha(ErroResultadoPesquisa),
    AtributoHtml(ErroAtributoHtml),
    CelulaDoTituloDoDocumentoAusenteOuRepetida,
    LinhaDeDocumentoSemIdDocumentoNoLinkDoProcesso,
    LinkDoDocumentoAusenteNaLinha,
    LinksDoDocumentoRepetidosNaLinha,
    LinkDaLinhaDeDocumentoNaoReconhecido,
    LinkDoDocumentoDiferenteDoProcesso,
    NumeroSeiComLinkDiferenteDoDocumento,
    NumeroSeiAusenteNaLinha,
    RotuloDoDocumentoAusente,
    MetadadoDoDocumentoRepetido,
    DataDeInclusaoForaDaFormaObservada,
    LinhaDaPesquisaForaDeUmRegistro,
    NumeroSeiRepetidoNaPesquisa,
    NumeroDoProcesso(NumeroProcessoInvalido),
}

impl ErroDocumentoDaPesquisa {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::Linha(erro) => erro.subcodigo(),
            Self::AtributoHtml(erro) => erro.subcodigo(),
            Self::CelulaDoTituloDoDocumentoAusenteOuRepetida => {
                "celula_do_titulo_do_documento_ausente_ou_repetida"
            }
            Self::LinhaDeDocumentoSemIdDocumentoNoLinkDoProcesso => {
                "linha_de_documento_sem_id_documento_no_link_do_processo"
            }
            Self::LinkDoDocumentoAusenteNaLinha => "link_do_documento_ausente_na_linha",
            Self::LinksDoDocumentoRepetidosNaLinha => "links_do_documento_repetidos_na_linha",
            Self::LinkDaLinhaDeDocumentoNaoReconhecido => {
                "link_da_linha_de_documento_nao_reconhecido"
            }
            Self::LinkDoDocumentoDiferenteDoProcesso => "link_do_documento_diferente_do_processo",
            Self::NumeroSeiComLinkDiferenteDoDocumento => {
                "numero_sei_com_link_diferente_do_documento"
            }
            Self::NumeroSeiAusenteNaLinha => "numero_sei_ausente_na_linha",
            Self::RotuloDoDocumentoAusente => "rotulo_do_documento_ausente",
            Self::MetadadoDoDocumentoRepetido => "metadado_do_documento_repetido",
            Self::DataDeInclusaoForaDaFormaObservada => "data_de_inclusao_fora_da_forma_observada",
            Self::LinhaDaPesquisaForaDeUmRegistro => "linha_da_pesquisa_fora_de_um_registro",
            Self::NumeroSeiRepetidoNaPesquisa => "numero_sei_repetido_na_pesquisa",
            Self::NumeroDoProcesso(erro) => erro.subcodigo(),
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_fluxo;

#[cfg(test)]
pub(crate) mod tests_fixtures;
