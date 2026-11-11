//! Falhas da leitura da árvore: um erro por fato verificável
//! (contracts/cli/processo-obter-arvore.md). Nenhum valor
//! servido é guardado.

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroArvore {
    /// O tokenizador de atributos recusou a tag.
    Atributo(#[source] crate::html_atributos::ErroAtributoHtml),
    ParseDoHtmlDoNoDoProcessoFalhou(#[source] tl::ParseError),
    EnderecoDoVinculoDeAnexacaoRecusado(
        #[source] crate::arvore_processo::endereco::EnderecoRecusado,
    ),
    ParseDoDestinoMovidoFalhou(#[source] tl::ParseError),
    EnderecoDoDestinoMovidoRecusado(#[source] crate::arvore_processo::endereco::EnderecoRecusado),
    LinhaDoDestinoMovidoIlegivel(#[source] Box<ErroArvore>),
    EnderecoDeAbrirPastasRecusado(#[source] crate::arvore_processo::endereco::EnderecoRecusado),
    BarraDeNoIlegivel(#[source] Box<ErroArvore>),
    SrcDeNoIlegivel(#[source] Box<ErroArvore>),
    HtmlDoNoDoProcessoIlegivel(#[source] Box<ErroArvore>),
    ProtocolosDaPastaIlegiveis(#[source] Box<ErroArvore>),
    ArgumentoDeNoNaoLiteral,
    TextoDeNoSemFechamento,
    QuantidadeDeArgumentosDoNoDiferenteDaObservada,
    ArgumentoDeNoComTipoDiferenteDoObservado,
    TipoDeNoDaArvoreNaoSuportado,
    NoDoProcessoAusenteNaArvore,
    NoDoProcessoComPai,
    NupDoNoDoProcessoInconsistente,
    IconeDeCancelamentoSemMotivoNaDica,
    IconeDoDocumentoForaDaFormaObservada,
    IdDeNoNaoDecimal,
    DocumentoComPaiDiferenteDoProcesso,
    IdentidadeDoProcessoDiferenteDaPedida,
    IdDoProcessoNaArvoreDiferenteDoAberto,
    PastaComPaiDiferenteDoProcesso,
    AguardeComPaiDiferenteDePasta,
    PastaSemProtocolosDeclarados,
    IdentidadesDivergentesNaBarraDoDocumento,
    DicaDeCancelamentoSemIconeDeCancelado,
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
    AcaoDeAbrirPastasForaDaFormaObservada,
    AcoesDeAbrirPastasRepetidas,
    AguardeForaDaFormaObservada,
    BarraDeNoForaDaFormaObservada,
    DeclaracaoDeNoForaDaFormaObservada,
    DestinoDoDocumentoMovidoForaDaFormaObservada,
    DestinosDoDocumentoMovidoRepetidos,
    DocumentoMovidoForaDaFormaObservada,
    DocumentoMovidoRepetidoNaArvore,
    DocumentoRepetidoNaArvore,
    EnderecoDeAbrirPastasNaoReconhecido,
    EscapeDeNoNaoReconhecido,
    HtmlDoNoDoProcessoForaDaFormaObservada,
    IndiceDeNoForaDaSequencia,
    NosDoProcessoRepetidosNaArvore,
    PastaForaDaFormaObservada,
    PastaRepetidaNaArvore,
    ProcessoAnexadoForaDaFormaObservada,
    ProcessoAnexadoRepetidoNaArvore,
    ProtocoloDeclaradoAusenteDaPasta,
    ProtocoloRepetidoNaPasta,
    ProtocolosDaPastaDeclaradosMaisDeUmaVez,
    ProtocolosDaPastaForaDaFormaObservada,
    ProtocolosDeclaradosParaPastaAusente,
    SrcDeNoForaDaFormaObservada,
    VinculoDeAnexacaoForaDaFormaObservada,
    VinculosDeAnexacaoRepetidos,
}

impl ErroArvore {
    /// A árvore é de outro processo: um palpite do índice errou.
    pub(crate) fn diverge_do_pedido(&self) -> bool {
        matches!(
            self,
            Self::IdentidadeDoProcessoDiferenteDaPedida
                | Self::IdDoProcessoNaArvoreDiferenteDoAberto
        )
    }

    /// Apenas lacunas de documentos toleram consulta informativa.
    pub(crate) fn admite_leitura_parcial(&self) -> bool {
        matches!(
            self,
            Self::PastaCarregadaPelaMetade
                | Self::PastasNaoCarregadasSemAcaoDeAbrir
                | Self::PastasNaoCarregadasAposAbrirPastas
                | Self::ContagemDaPastaDiferenteDaDeclarada
                | Self::ProtocoloDeclaradoAusenteDaPasta
        )
    }
}
