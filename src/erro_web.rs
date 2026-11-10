//! Falhas técnicas de interpretação e paginação Web; não são entidades do SEI.
use crate::dominio::Categoria;
use crate::erro_html::ErroHtml;
use std::fmt;

/// Nunca inclui HTML, cookies, URLs ou conteúdo de processos nas mensagens.
#[derive(Debug, Clone, PartialEq, Eq, crate::erro_publico::Causa)]
pub enum ErroCaixa {
    TabelaAusente {
        categoria: Categoria,
    },
    TabelasDaCategoriaRepetidas {
        categoria: Categoria,
    },
    TabelaInvalida {
        categoria: Categoria,
    },
    LegendaAusente {
        categoria: Categoria,
    },
    LegendasDaTabelaRepetidas {
        categoria: Categoria,
    },
    IdInvalido,
    IdRepetido {
        categoria: Categoria,
        id: String,
    },
    NumeroInvalido {
        categoria: Categoria,
        id: String,
    },
    MaisDeUmNumeroNaLinhaDoProcesso {
        categoria: Categoria,
        id: String,
    },
    LinkDoProcessoNaoIdentificadoNaLinha {
        categoria: Categoria,
        id: String,
    },
    ClassesDeEstadoDiferentesNoLinkDoProcesso {
        categoria: Categoria,
        id: String,
    },
    TotalExcedido {
        categoria: Categoria,
    },
    TotaisDeclaradosDiferentesEntrePaginas {
        categoria: Categoria,
    },
    FaixaInvalida {
        categoria: Categoria,
    },
    FaixaDesconhecida {
        categoria: Categoria,
    },
    FaixaAlterada {
        categoria: Categoria,
    },
    FaixaDescontinua {
        categoria: Categoria,
    },
    /// Legenda sem `registros`: a página não declara o total da tabela.
    TotalNaoDeclaradoNaLegenda {
        categoria: Categoria,
    },
    /// Legenda sem intervalo e total diferente do número de linhas.
    LegendaSemFaixaComTotalDiferenteDasLinhas {
        categoria: Categoria,
    },
    /// Paginador próprio da tabela publicado com legenda sem intervalo.
    PaginadorDaTabelaSemFaixaNaLegenda {
        categoria: Categoria,
    },
    LimitePaginas,
    VisaoResumidaComPaginacao,
    /// `hdnIdMarcador{unidade}` preenchido: a lista é só a do marcador.
    FiltroDeMarcadorAtivoNoControle,
    IdentificacaoDaUnidadeAusenteNoControle,
    ContextoNaoVerificavel,
    ContextoDaPaginaDiferenteDoAnterior,
    SemPaginas,
    #[causa(transparente)]
    Html(ErroHtml),
}

impl ErroCaixa {
    pub fn subcodigo(&self) -> &'static str {
        crate::erro_publico::folha(self)
    }
    pub fn categoria(&self) -> Option<Categoria> {
        match self {
            Self::TabelaAusente { categoria }
            | Self::TabelasDaCategoriaRepetidas { categoria }
            | Self::TabelaInvalida { categoria }
            | Self::LegendaAusente { categoria }
            | Self::LegendasDaTabelaRepetidas { categoria }
            | Self::IdRepetido { categoria, .. }
            | Self::NumeroInvalido { categoria, .. }
            | Self::MaisDeUmNumeroNaLinhaDoProcesso { categoria, .. }
            | Self::LinkDoProcessoNaoIdentificadoNaLinha { categoria, .. }
            | Self::ClassesDeEstadoDiferentesNoLinkDoProcesso { categoria, .. }
            | Self::TotalExcedido { categoria }
            | Self::TotaisDeclaradosDiferentesEntrePaginas { categoria }
            | Self::FaixaInvalida { categoria }
            | Self::FaixaDesconhecida { categoria }
            | Self::FaixaAlterada { categoria }
            | Self::FaixaDescontinua { categoria }
            | Self::TotalNaoDeclaradoNaLegenda { categoria }
            | Self::LegendaSemFaixaComTotalDiferenteDasLinhas { categoria }
            | Self::PaginadorDaTabelaSemFaixaNaLegenda { categoria } => Some(*categoria),
            _ => None,
        }
    }
    pub fn id(&self) -> Option<&str> {
        match self {
            Self::IdRepetido { id, .. }
            | Self::NumeroInvalido { id, .. }
            | Self::MaisDeUmNumeroNaLinhaDoProcesso { id, .. }
            | Self::LinkDoProcessoNaoIdentificadoNaLinha { id, .. }
            | Self::ClassesDeEstadoDiferentesNoLinkDoProcesso { id, .. } => Some(id),
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
