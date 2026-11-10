//! Falhas da leitura da árvore: uma variante por fato verificável, com
//! subcódigo estável (contracts/cli/processo-obter-arvore.md). Nenhum valor
//! servido é guardado.

use crate::erro_html::ErroHtml;

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroArvore {
    ArgumentoDeNoNaoLiteral,
    TextoDeNoSemFechamento,
    QuantidadeDeArgumentosDoNoDiferenteDaObservada,
    ArgumentoDeNoComTipoDiferenteDoObservado,
    TipoDeNoDaArvoreNaoSuportado,
    NoDoProcessoAusenteNaArvore,
    NoDoProcessoComPai,
    NupDoNoDoProcessoInconsistente,
    IdDeNoNaoDecimal,
    DocumentoComPaiDiferenteDoProcesso,
    IdentidadeDoProcessoDiferenteDaPedida,
    IdDoProcessoNaArvoreDiferenteDoAberto,
    PastaComPaiDiferenteDoProcesso,
    AguardeComPaiDiferenteDePasta,
    PastaSemProtocolosDeclarados,
    IdentidadesDivergentesNaBarraDoDocumento,
    DocumentoDaPastaSemIdentidadeNaBarra,
    DocumentoDaPastaNaoDeclarado,
    PastaCarregadaPelaMetade,
    PastasNaoCarregadasSemAcaoDeAbrir,
    PastasNaoCarregadasAposAbrirPastas,
    ContagemDaPastaDiferenteDaDeclarada,
    NupDoNoDoProcessoAnexadoInconsistente,
    ProcessoAnexadoComPaiDiferenteDoProcesso,
    ProcessoAnexadoIgualAoProcesso,
    NoComPaiNoProcessoAnexado,
    DocumentoMovidoComPaiDiferenteDoProcesso,
    Html(#[from] ErroHtml),
}

impl ErroArvore {
    /// A árvore é de outro processo: um palpite do índice errou.
    pub(crate) fn diverge_do_pedido(self) -> bool {
        matches!(
            self,
            Self::IdentidadeDoProcessoDiferenteDaPedida
                | Self::IdDoProcessoNaArvoreDiferenteDoAberto
        )
    }

    /// Apenas lacunas de documentos toleram consulta informativa.
    pub(crate) fn admite_leitura_parcial(self) -> bool {
        matches!(
            self,
            Self::PastaCarregadaPelaMetade
                | Self::PastasNaoCarregadasSemAcaoDeAbrir
                | Self::PastasNaoCarregadasAposAbrirPastas
                | Self::ContagemDaPastaDiferenteDaDeclarada
        ) || matches!(
            self,
            Self::Html(erro) if erro == ErroHtml::ausente("protocolo_declarado_ausente_da_pasta")
        )
    }
}
