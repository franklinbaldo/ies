//! Falhas técnicas de interpretação e paginação Web; não são entidades do SEI.
use crate::dominio::Categoria;
use std::fmt;

/// Nunca inclui HTML, cookies, URLs ou conteúdo de processos nas mensagens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErroCaixa {
    HtmlInvalido,
    TabelaAusente { categoria: Categoria },
    TabelaInvalida { categoria: Categoria },
    LegendaAusente { categoria: Categoria },
    IdInvalido,
    IdRepetido { categoria: Categoria, id: String },
    NumeroInvalido { categoria: Categoria, id: String },
    MaisDeUmNumeroNaLinhaDoProcesso { categoria: Categoria, id: String },
    LinkDoProcessoNaoIdentificadoNaLinha { categoria: Categoria, id: String },
    TotalExcedido { categoria: Categoria },
    TotaisDeclaradosDiferentesEntrePaginas { categoria: Categoria },
    FaixaInvalida { categoria: Categoria },
    FaixaDesconhecida { categoria: Categoria },
    FaixaAlterada { categoria: Categoria },
    FaixaDescontinua { categoria: Categoria },
    LimitePaginas,
    UnidadeInesperada,
    IdentificacaoDaUnidadeAusenteNoControle,
    ContextoNaoVerificavel,
    ContextoDaPaginaDiferenteDoAnterior,
    SemPaginas,
}

impl ErroCaixa {
    pub fn subcodigo(&self) -> &'static str {
        match self {
            Self::HtmlInvalido => "html_invalido",
            Self::TabelaAusente { .. } => "tabela_ausente",
            Self::TabelaInvalida { .. } => "tabela_invalida",
            Self::LegendaAusente { .. } => "legenda_ausente",
            Self::IdInvalido => "id_invalido",
            Self::IdRepetido { .. } => "id_repetido",
            Self::NumeroInvalido { .. } => "numero_invalido",
            Self::MaisDeUmNumeroNaLinhaDoProcesso { .. } => "processo_ambiguo",
            Self::LinkDoProcessoNaoIdentificadoNaLinha { .. } => {
                "link_do_processo_nao_identificado_na_linha"
            }
            Self::TotalExcedido { .. } => "total_excedido",
            Self::TotaisDeclaradosDiferentesEntrePaginas { .. } => "total_divergente",
            Self::FaixaInvalida { .. } => "faixa_invalida",
            Self::FaixaDesconhecida { .. } => "faixa_desconhecida",
            Self::FaixaAlterada { .. } => "faixa_alterada",
            Self::FaixaDescontinua { .. } => "faixa_descontinua",
            Self::LimitePaginas => "limite_paginas",
            Self::UnidadeInesperada => "unidade_inesperada",
            Self::IdentificacaoDaUnidadeAusenteNoControle => {
                "identificacao_da_unidade_ausente_no_controle"
            }
            Self::ContextoNaoVerificavel => "contexto_nao_verificavel",
            Self::ContextoDaPaginaDiferenteDoAnterior => "contexto_divergente",
            Self::SemPaginas => "sem_paginas",
        }
    }
    pub fn categoria(&self) -> Option<Categoria> {
        match self {
            Self::TabelaAusente { categoria }
            | Self::TabelaInvalida { categoria }
            | Self::LegendaAusente { categoria }
            | Self::IdRepetido { categoria, .. }
            | Self::NumeroInvalido { categoria, .. }
            | Self::MaisDeUmNumeroNaLinhaDoProcesso { categoria, .. }
            | Self::LinkDoProcessoNaoIdentificadoNaLinha { categoria, .. }
            | Self::TotalExcedido { categoria }
            | Self::TotaisDeclaradosDiferentesEntrePaginas { categoria }
            | Self::FaixaInvalida { categoria }
            | Self::FaixaDesconhecida { categoria }
            | Self::FaixaAlterada { categoria }
            | Self::FaixaDescontinua { categoria } => Some(*categoria),
            _ => None,
        }
    }
    pub fn id(&self) -> Option<&str> {
        match self {
            Self::IdRepetido { id, .. }
            | Self::NumeroInvalido { id, .. }
            | Self::MaisDeUmNumeroNaLinhaDoProcesso { id, .. }
            | Self::LinkDoProcessoNaoIdentificadoNaLinha { id, .. } => Some(id),
            _ => None,
        }
    }
}

impl fmt::Display for ErroCaixa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "leitura da caixa: {}", self.subcodigo())?;
        if let Some(categoria) = self.categoria() {
            write!(f, " ({})", categoria.rotulo())?;
        }
        if let Some(id) = self.id() {
            write!(f, " (id {id})")?;
        }
        Ok(())
    }
}
impl std::error::Error for ErroCaixa {}
