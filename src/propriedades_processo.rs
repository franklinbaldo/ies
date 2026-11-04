//! Propriedades de `ies processo NUP` (contracts/cli/processo-obter-propriedades.md):
//! o link `procedimento_alterar` (ou `procedimento_consultar`) da barra do
//! nó do processo, lido por [`crate::arvore_processo::ler_link_das_propriedades`],
//! e o formulário `form#frmProcedimentoCadastro`, lido sem nunca ser
//! enviado. Estrutura em docs/como-o-sei-funciona/propriedades-do-processo.md.
mod tela;

#[cfg(test)]
pub(crate) mod tests_apoio;
#[cfg(test)]
mod tests_tela;

pub(crate) use tela::ler_propriedades;

/// Qual tela a barra publicou (e o `acao` do link guardado no índice).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FonteDasPropriedades {
    /// "Alterar Processo": o processo aberto na unidade.
    Alterar,
    /// "Consultar Processo": a barra sem `procedimento_alterar`.
    Consultar,
}

impl FonteDasPropriedades {
    pub(crate) fn acao(self) -> &'static str {
        match self {
            Self::Alterar => "procedimento_alterar",
            Self::Consultar => "procedimento_consultar",
        }
    }
}

/// URL absoluta e assinada da tela: `Debug` redigido, sem `Display`.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct EnderecoDasPropriedades {
    pub(crate) fonte: FonteDasPropriedades,
    url: String,
}

impl EnderecoDasPropriedades {
    pub(crate) fn novo(fonte: FonteDasPropriedades, url: String) -> Self {
        Self { fonte, url }
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.url
    }
}

impl std::fmt::Debug for EnderecoDasPropriedades {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EnderecoDasPropriedades({:?}, <redigido>)", self.fonte)
    }
}

/// `rdoNivelAcesso` marcado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NivelDeAcesso {
    Publico,
    Restrito,
    Sigiloso,
}

impl NivelDeAcesso {
    pub(crate) fn como_texto(self) -> &'static str {
        match self {
            Self::Publico => "publico",
            Self::Restrito => "restrito",
            Self::Sigiloso => "sigiloso",
        }
    }
}

/// Uma `option` de `selAssuntos`: `<código> - <descrição>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Assunto {
    pub(crate) codigo: Option<String>,
    pub(crate) descricao: String,
}

/// O que o formulário publicou; `None` é "não observado".
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PropriedadesLidas {
    pub(crate) tipo: Option<String>,
    pub(crate) especificacao: String,
    pub(crate) interessados: Vec<String>,
    pub(crate) assuntos: Vec<Assunto>,
    pub(crate) observacoes: String,
    pub(crate) nivel_de_acesso: NivelDeAcesso,
    pub(crate) hipotese_legal: Option<String>,
    pub(crate) data_de_geracao: Option<String>,
    pub(crate) fonte: FonteDasPropriedades,
}

/// Falhas do link na barra e da tela: uma variante por fato.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroPropriedades {
    BarraDoProcessoForaDaFormaObservada,
    BarrasDoProcessoRepetidasNaArvore,
    LinksDasPropriedadesRepetidosNaBarra,
    EnderecoDasPropriedadesNaoReconhecido,
    EnderecoDasPropriedadesDeOutroProcesso,
    HtmlDasPropriedadesNaoInterpretado,
    FormularioDasPropriedadesAusente,
    FormulariosDasPropriedadesRepetidos,
    CampoDasPropriedadesAusente,
    CampoDasPropriedadesRepetido,
    NumeroDasPropriedadesDiferenteDoSolicitado,
    IdDasPropriedadesDiferenteDoAberto,
    TipoDoProcessoInconsistente,
    OpcoesMarcadasRepetidas,
    AssuntoForaDaFormaObservada,
    InteressadoForaDaFormaObservada,
    ListaOcultaDasPropriedadesNaoObservada,
    NivelDeAcessoNaoMarcado,
    NivelDeAcessoForaDaFormaObservada,
    DataDeGeracaoForaDaFormaObservada,
    TextoDasPropriedadesNaoDecodificado,
}

impl ErroPropriedades {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::BarraDoProcessoForaDaFormaObservada => {
                "barra_do_processo_fora_da_forma_observada"
            }
            Self::BarrasDoProcessoRepetidasNaArvore => "barras_do_processo_repetidas_na_arvore",
            Self::LinksDasPropriedadesRepetidosNaBarra => {
                "links_das_propriedades_repetidos_na_barra"
            }
            Self::EnderecoDasPropriedadesNaoReconhecido => {
                "endereco_das_propriedades_nao_reconhecido"
            }
            Self::EnderecoDasPropriedadesDeOutroProcesso => {
                "endereco_das_propriedades_de_outro_processo"
            }
            Self::HtmlDasPropriedadesNaoInterpretado => "html_das_propriedades_nao_interpretado",
            Self::FormularioDasPropriedadesAusente => "formulario_das_propriedades_ausente",
            Self::FormulariosDasPropriedadesRepetidos => "formularios_das_propriedades_repetidos",
            Self::CampoDasPropriedadesAusente => "campo_das_propriedades_ausente",
            Self::CampoDasPropriedadesRepetido => "campo_das_propriedades_repetido",
            Self::NumeroDasPropriedadesDiferenteDoSolicitado => {
                "numero_das_propriedades_diferente_do_solicitado"
            }
            Self::IdDasPropriedadesDiferenteDoAberto => "id_das_propriedades_diferente_do_aberto",
            Self::TipoDoProcessoInconsistente => "tipo_do_processo_inconsistente",
            Self::OpcoesMarcadasRepetidas => "opcoes_marcadas_repetidas",
            Self::AssuntoForaDaFormaObservada => "assunto_fora_da_forma_observada",
            Self::InteressadoForaDaFormaObservada => "interessado_fora_da_forma_observada",
            Self::ListaOcultaDasPropriedadesNaoObservada => {
                "lista_oculta_das_propriedades_nao_observada"
            }
            Self::NivelDeAcessoNaoMarcado => "nivel_de_acesso_nao_marcado",
            Self::NivelDeAcessoForaDaFormaObservada => "nivel_de_acesso_fora_da_forma_observada",
            Self::DataDeGeracaoForaDaFormaObservada => "data_de_geracao_fora_da_forma_observada",
            Self::TextoDasPropriedadesNaoDecodificado => "texto_das_propriedades_nao_decodificado",
        }
    }

    /// Recusas do link na barra (o resto é da tela).
    pub(crate) fn do_link(self) -> bool {
        matches!(
            self,
            Self::BarraDoProcessoForaDaFormaObservada
                | Self::BarrasDoProcessoRepetidasNaArvore
                | Self::LinksDasPropriedadesRepetidosNaBarra
                | Self::EnderecoDasPropriedadesNaoReconhecido
                | Self::EnderecoDasPropriedadesDeOutroProcesso
        )
    }

    /// A página não é o formulário deste processo: um link guardado errou.
    pub(crate) fn link_envelhecido(self) -> bool {
        matches!(
            self,
            Self::FormularioDasPropriedadesAusente
                | Self::NumeroDasPropriedadesDiferenteDoSolicitado
                | Self::IdDasPropriedadesDiferenteDoAberto
        )
    }
}
