//! Propriedades de `ies processo NUP` (contracts/cli/processo-obter-propriedades.md):
//! o link `procedimento_alterar` (ou `procedimento_consultar`) da barra do
//! nó do processo, lido por [`crate::arvore_processo::ler_link_das_propriedades`],
//! e o formulário `form#frmProcedimentoCadastro`, lido sem nunca ser
//! enviado. Estrutura em docs/como-o-sei-funciona/propriedades-do-processo.md.
mod tela;

use crate::acao_do_sei::AcaoDoSei;

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
    pub(crate) const fn acao(self) -> AcaoDoSei {
        match self {
            Self::Alterar => AcaoDoSei::ProcedimentoAlterar,
            Self::Consultar => AcaoDoSei::ProcedimentoConsultar,
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

pub(crate) use crate::dominio::NivelDeAcesso;

/// Uma `option` de `selAssuntos`: `<código> - <descrição>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Assunto {
    pub(crate) codigo: String,
    pub(crate) descricao: String,
}

/// O que o formulário publicou. `hipotese_legal` é `None` quando nenhuma
/// hipótese está marcada, o único campo cuja ausência é o domínio.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PropriedadesLidas {
    pub(crate) tipo: String,
    pub(crate) especificacao: String,
    pub(crate) interessados: Vec<String>,
    pub(crate) assuntos: Vec<Assunto>,
    pub(crate) observacoes: String,
    pub(crate) nivel_de_acesso: NivelDeAcesso,
    pub(crate) hipotese_legal: Option<String>,
    pub(crate) data_de_geracao: String,
    pub(crate) fonte: FonteDasPropriedades,
}

/// Falhas do link na barra e da tela: um erro por fato.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroPropriedades {
    Barra(#[from] crate::arvore_processo::ErroBarraDoProcesso),
    LinksDasPropriedadesRepetidosNaBarra,
    EnderecoDasPropriedadesNaoReconhecido,
    EnderecoDasPropriedadesDeOutroProcesso,
    FormularioDasPropriedadesAusente,
    NumeroDasPropriedadesDiferenteDoSolicitado,
    IdDasPropriedadesDiferenteDoAberto,
    TipoDoProcessoInconsistente,
    /// `hdnAssuntos` ou `hdnInteressadosProcedimento` com conteúdo: a lista
    /// visível pode estar incompleta, então nada se afirma.
    ListaOcultaDasPropriedadesPreenchida,
    TipoDoProcessoNaoMarcado,
    CodigoDoAssuntoNaoInterpretado,
    DataDeGeracaoAusente,
    NivelDeAcessoNaoMarcado,
    AssuntoForaDaFormaObservada,
    CampoDasPropriedadesAusente,
    CampoDasPropriedadesRepetido,
    DataDeGeracaoForaDaFormaObservada,
    FormulariosDasPropriedadesRepetidos,
    InteressadoForaDaFormaObservada,
    NivelDeAcessoForaDaFormaObservada,
    OpcoesMarcadasRepetidas,
}

impl ErroPropriedades {
    /// A página não é o formulário deste processo: um link guardado errou.
    pub(crate) fn link_envelhecido(&self) -> bool {
        matches!(
            self,
            Self::FormularioDasPropriedadesAusente
                | Self::NumeroDasPropriedadesDiferenteDoSolicitado
                | Self::IdDasPropriedadesDiferenteDoAberto
        )
    }
}
