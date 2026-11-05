//! Falhas da leitura da árvore: uma variante por fato verificável, com
//! subcódigo estável (contracts/cli/processo-obter-arvore.md). Nenhum valor
//! servido é guardado.

#[derive(Clone, Copy, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroArvore {
    DeclaracaoDeNoForaDaFormaObservada,
    ArgumentoDeNoNaoLiteral,
    TextoDeNoSemFechamento,
    EscapeDeNoNaoReconhecido,
    QuantidadeDeArgumentosDoNoDiferenteDaObservada,
    ArgumentoDeNoComTipoDiferenteDoObservado,
    IndiceDeNoForaDaSequencia,
    TipoDeNoDaArvoreNaoSuportado,
    NoDoProcessoAusenteNaArvore,
    NosDoProcessoRepetidosNaArvore,
    NoDoProcessoComPai,
    NupDoNoDoProcessoInconsistente,
    IdDeNoNaoDecimal,
    DocumentoComPaiDiferenteDoProcesso,
    DocumentoRepetidoNaArvore,
    IdentidadeDoProcessoDiferenteDaPedida,
    IdDoProcessoNaArvoreDiferenteDoAberto,
    // Pastas (fatia 3).
    PastaForaDaFormaObservada,
    PastaRepetidaNaArvore,
    PastaComPaiDiferenteDoProcesso,
    AguardeForaDaFormaObservada,
    AguardeComPaiDiferenteDePasta,
    ProtocolosDaPastaForaDaFormaObservada,
    ProtocolosDaPastaDeclaradosMaisDeUmaVez,
    ProtocolosDeclaradosParaPastaAusente,
    PastaSemProtocolosDeclarados,
    BarraDeNoForaDaFormaObservada,
    /// `Nos[i].src = …` fora de um literal de texto único (Refs #60).
    SrcDeNoForaDaFormaObservada,
    IdentidadesDivergentesNaBarraDoDocumento,
    DocumentoDaPastaSemIdentidadeNaBarra,
    DocumentoDaPastaNaoDeclarado,
    ProtocoloRepetidoNaPasta,
    ProtocoloDeclaradoAusenteDaPasta,
    PastaCarregadaPelaMetade,
    PastasNaoCarregadasSemAcaoDeAbrir,
    AcaoDeAbrirPastasForaDaFormaObservada,
    AcoesDeAbrirPastasRepetidas,
    EnderecoDeAbrirPastasNaoReconhecido,
    PastasNaoCarregadasAposAbrirPastas,
    ContagemDaPastaDiferenteDaDeclarada,
    // Processos anexados.
    ProcessoAnexadoForaDaFormaObservada,
    NupDoNoDoProcessoAnexadoInconsistente,
    ProcessoAnexadoComPaiDiferenteDoProcesso,
    ProcessoAnexadoRepetidoNaArvore,
    ProcessoAnexadoIgualAoProcesso,
    NoComPaiNoProcessoAnexado,
    // Vínculo do processo aberto com o pai (ele mesmo anexado).
    VinculoDeAnexacaoForaDaFormaObservada,
    VinculosDeAnexacaoRepetidos,
    HtmlDoNoDoProcessoForaDaFormaObservada,
    // Documentos movidos para outro processo.
    DocumentoMovidoForaDaFormaObservada,
    DocumentoMovidoComPaiDiferenteDoProcesso,
    DocumentoMovidoRepetidoNaArvore,
    DestinoDoDocumentoMovidoForaDaFormaObservada,
    DestinosDoDocumentoMovidoRepetidos,
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
