//! Falhas da leitura da árvore: uma variante por fato verificável, com
//! subcódigo estável (contracts/cli/processo-obter-arvore.md). Nenhum valor
//! servido é guardado.

#[derive(Clone, Copy, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
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
    #[causa(interna)]
    Html(crate::erro_html::ErroHtml),
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
}
