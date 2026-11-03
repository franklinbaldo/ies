//! Falhas da leitura da árvore: uma variante por fato verificável, com
//! subcódigo estável (contracts/cli/processo-obter-arvore.md). Nenhum valor
//! servido é guardado.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
}

impl ErroArvore {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::DeclaracaoDeNoForaDaFormaObservada => "declaracao_de_no_fora_da_forma_observada",
            Self::ArgumentoDeNoNaoLiteral => "argumento_de_no_nao_literal",
            Self::TextoDeNoSemFechamento => "texto_de_no_sem_fechamento",
            Self::EscapeDeNoNaoReconhecido => "escape_de_no_nao_reconhecido",
            Self::QuantidadeDeArgumentosDoNoDiferenteDaObservada => {
                "quantidade_de_argumentos_do_no_diferente_da_observada"
            }
            Self::ArgumentoDeNoComTipoDiferenteDoObservado => {
                "argumento_de_no_com_tipo_diferente_do_observado"
            }
            Self::IndiceDeNoForaDaSequencia => "indice_de_no_fora_da_sequencia",
            Self::TipoDeNoDaArvoreNaoSuportado => "tipo_de_no_da_arvore_nao_suportado",
            Self::NoDoProcessoAusenteNaArvore => "no_do_processo_ausente_na_arvore",
            Self::NosDoProcessoRepetidosNaArvore => "nos_do_processo_repetidos_na_arvore",
            Self::NoDoProcessoComPai => "no_do_processo_com_pai",
            Self::NupDoNoDoProcessoInconsistente => "nup_do_no_do_processo_inconsistente",
            Self::IdDeNoNaoDecimal => "id_de_no_nao_decimal",
            Self::DocumentoComPaiDiferenteDoProcesso => "documento_com_pai_diferente_do_processo",
            Self::DocumentoRepetidoNaArvore => "documento_repetido_na_arvore",
            Self::IdentidadeDoProcessoDiferenteDaPedida => {
                "identidade_do_processo_diferente_da_pedida"
            }
            Self::IdDoProcessoNaArvoreDiferenteDoAberto => {
                "id_do_processo_na_arvore_diferente_do_aberto"
            }
            outro => outro.subcodigo_das_pastas(),
        }
    }

    fn subcodigo_das_pastas(self) -> &'static str {
        match self {
            Self::PastaForaDaFormaObservada => "pasta_fora_da_forma_observada",
            Self::PastaRepetidaNaArvore => "pasta_repetida_na_arvore",
            Self::PastaComPaiDiferenteDoProcesso => "pasta_com_pai_diferente_do_processo",
            Self::AguardeForaDaFormaObservada => "aguarde_fora_da_forma_observada",
            Self::AguardeComPaiDiferenteDePasta => "aguarde_com_pai_diferente_de_pasta",
            Self::ProtocolosDaPastaForaDaFormaObservada => {
                "protocolos_da_pasta_fora_da_forma_observada"
            }
            Self::ProtocolosDaPastaDeclaradosMaisDeUmaVez => {
                "protocolos_da_pasta_declarados_mais_de_uma_vez"
            }
            Self::ProtocolosDeclaradosParaPastaAusente => {
                "protocolos_declarados_para_pasta_ausente"
            }
            Self::PastaSemProtocolosDeclarados => "pasta_sem_protocolos_declarados",
            Self::BarraDeNoForaDaFormaObservada => "barra_de_no_fora_da_forma_observada",
            Self::IdentidadesDivergentesNaBarraDoDocumento => {
                "identidades_divergentes_na_barra_do_documento"
            }
            Self::DocumentoDaPastaSemIdentidadeNaBarra => {
                "documento_da_pasta_sem_identidade_na_barra"
            }
            Self::DocumentoDaPastaNaoDeclarado => "documento_da_pasta_nao_declarado",
            Self::ProtocoloRepetidoNaPasta => "protocolo_repetido_na_pasta",
            Self::ProtocoloDeclaradoAusenteDaPasta => "protocolo_declarado_ausente_da_pasta",
            Self::PastaCarregadaPelaMetade => "pasta_carregada_pela_metade",
            Self::PastasNaoCarregadasSemAcaoDeAbrir => "pastas_nao_carregadas_sem_acao_de_abrir",
            Self::AcaoDeAbrirPastasForaDaFormaObservada => {
                "acao_de_abrir_pastas_fora_da_forma_observada"
            }
            Self::AcoesDeAbrirPastasRepetidas => "acoes_de_abrir_pastas_repetidas",
            Self::EnderecoDeAbrirPastasNaoReconhecido => "endereco_de_abrir_pastas_nao_reconhecido",
            Self::PastasNaoCarregadasAposAbrirPastas => "pastas_nao_carregadas_apos_abrir_pastas",
            Self::ContagemDaPastaDiferenteDaDeclarada => "contagem_da_pasta_diferente_da_declarada",
            outro => outro.subcodigo_dos_anexados(),
        }
    }

    fn subcodigo_dos_anexados(self) -> &'static str {
        match self {
            Self::ProcessoAnexadoForaDaFormaObservada => "processo_anexado_fora_da_forma_observada",
            Self::NupDoNoDoProcessoAnexadoInconsistente => {
                "nup_do_no_do_processo_anexado_inconsistente"
            }
            Self::ProcessoAnexadoComPaiDiferenteDoProcesso => {
                "processo_anexado_com_pai_diferente_do_processo"
            }
            Self::ProcessoAnexadoRepetidoNaArvore => "processo_anexado_repetido_na_arvore",
            Self::ProcessoAnexadoIgualAoProcesso => "processo_anexado_igual_ao_processo",
            Self::NoComPaiNoProcessoAnexado => "no_com_pai_no_processo_anexado",
            _ => "arvore_nao_reconhecida",
        }
    }

    /// A árvore é de outro processo: um palpite do índice errou.
    pub(crate) fn diverge_do_pedido(self) -> bool {
        matches!(
            self,
            Self::IdentidadeDoProcessoDiferenteDaPedida
                | Self::IdDoProcessoNaArvoreDiferenteDoAberto
        )
    }
}
