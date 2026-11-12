//! Falhas da leitura da árvore: um erro por fato verificável
//! (contracts/cli/processo-obter-arvore.md). Nenhum valor
//! servido é guardado.

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroArvore {
    EnderecoDoVinculoDeAnexacaoRecusado(
        #[source] crate::arvore_processo::endereco::EnderecoRecusado,
    ),
    EnderecoDoDestinoMovidoRecusado(#[source] crate::arvore_processo::endereco::EnderecoRecusado),
    LinhaDoDestinoMovidoIlegivel(#[source] Box<ErroArvore>),
    EnderecoDeAbrirPastasRecusado(#[source] crate::arvore_processo::endereco::EnderecoRecusado),
    /// A consulta do link de abrir pastas não se deixa ler em pares.
    ConsultaDeAbrirPastasIlegivel(
        #[source] crate::consulta_do_controlador::ErroConsultaDoControlador,
    ),
    BarraDeNoIlegivel(#[source] Box<ErroArvore>),
    SrcDeNoIlegivel(#[source] Box<ErroArvore>),
    HtmlDoNoDoProcessoIlegivel(#[source] Box<ErroArvore>),
    /// O HTML de um literal (`.acoes`, `.html`) não cabe no limite do
    /// HTML interpretado.
    HtmlDoLiteralAcimaDoLimite(#[from] crate::tela_html::HtmlAcimaDoLimite),
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
    /// A consulta do link do pai não se deixa ler em pares.
    ConsultaDoVinculoIlegivel(#[source] crate::consulta_do_controlador::ErroConsultaDoControlador),
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
