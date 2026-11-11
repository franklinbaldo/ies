//! Falhas técnicas de interpretação e paginação Web; não são entidades do SEI.
use crate::dominio::Categoria;

/// Nunca inclui HTML, cookies, URLs ou conteúdo de processos nas mensagens.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{}", self.texto())]
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
        #[source]
        motivo: crate::paginas::ErroFaixaDaLegenda,
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
    HashesDeCriteriosGeradosRepetidosNoControle,
    HashesDeCriteriosRecebidosRepetidosNoControle,
    HtmlDaCaixaInvalido,
    /// `tl` recusou o HTML da caixa; a causa fica no `source`.
    HtmlDaCaixaRecusadoPeloParser(#[source] tl::ParseError),
    AtributoDaCaixaIlegivel(#[from] crate::html_atributos::ErroAtributoHtml),
    IdentificacoesDaUnidadeRepetidasNoControle,
    TiposDeVisualizacaoRepetidosNoControle,
    UnidadeInesperada,
}

impl ErroCaixa {
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
            | Self::FaixaInvalida { categoria, .. }
            | Self::FaixaDesconhecida { categoria }
            | Self::FaixaAlterada { categoria }
            | Self::FaixaDescontinua { categoria }
            | Self::TotalNaoDeclaradoNaLegenda { categoria }
            | Self::LegendaSemFaixaComTotalDiferenteDasLinhas { categoria }
            | Self::PaginadorDaTabelaSemFaixaNaLegenda { categoria } => Some(*categoria),
            Self::IdInvalido
            | Self::LimitePaginas
            | Self::VisaoResumidaComPaginacao
            | Self::FiltroDeMarcadorAtivoNoControle
            | Self::IdentificacaoDaUnidadeAusenteNoControle
            | Self::ContextoNaoVerificavel
            | Self::ContextoDaPaginaDiferenteDoAnterior
            | Self::SemPaginas
            | Self::HashesDeCriteriosGeradosRepetidosNoControle
            | Self::HashesDeCriteriosRecebidosRepetidosNoControle
            | Self::HtmlDaCaixaInvalido
            | Self::HtmlDaCaixaRecusadoPeloParser(_)
            | Self::AtributoDaCaixaIlegivel(_)
            | Self::IdentificacoesDaUnidadeRepetidasNoControle
            | Self::TiposDeVisualizacaoRepetidosNoControle
            | Self::UnidadeInesperada => None,
        }
    }
    pub fn id(&self) -> Option<&str> {
        match self {
            Self::IdRepetido { id, .. }
            | Self::NumeroInvalido { id, .. }
            | Self::MaisDeUmNumeroNaLinhaDoProcesso { id, .. }
            | Self::LinkDoProcessoNaoIdentificadoNaLinha { id, .. }
            | Self::ClassesDeEstadoDiferentesNoLinkDoProcesso { id, .. } => Some(id),
            Self::TabelaAusente { .. }
            | Self::TabelasDaCategoriaRepetidas { .. }
            | Self::TabelaInvalida { .. }
            | Self::LegendaAusente { .. }
            | Self::LegendasDaTabelaRepetidas { .. }
            | Self::IdInvalido
            | Self::TotalExcedido { .. }
            | Self::TotaisDeclaradosDiferentesEntrePaginas { .. }
            | Self::FaixaInvalida { .. }
            | Self::FaixaDesconhecida { .. }
            | Self::FaixaAlterada { .. }
            | Self::FaixaDescontinua { .. }
            | Self::TotalNaoDeclaradoNaLegenda { .. }
            | Self::LegendaSemFaixaComTotalDiferenteDasLinhas { .. }
            | Self::PaginadorDaTabelaSemFaixaNaLegenda { .. }
            | Self::LimitePaginas
            | Self::VisaoResumidaComPaginacao
            | Self::FiltroDeMarcadorAtivoNoControle
            | Self::IdentificacaoDaUnidadeAusenteNoControle
            | Self::ContextoNaoVerificavel
            | Self::ContextoDaPaginaDiferenteDoAnterior
            | Self::SemPaginas
            | Self::HashesDeCriteriosGeradosRepetidosNoControle
            | Self::HashesDeCriteriosRecebidosRepetidosNoControle
            | Self::HtmlDaCaixaInvalido
            | Self::HtmlDaCaixaRecusadoPeloParser(_)
            | Self::AtributoDaCaixaIlegivel(_)
            | Self::IdentificacoesDaUnidadeRepetidasNoControle
            | Self::TiposDeVisualizacaoRepetidosNoControle
            | Self::UnidadeInesperada => None,
        }
    }
}

impl ErroCaixa {
    fn texto(&self) -> String {
        let categoria = self
            .categoria()
            .map(|c| format!(" ({})", c.rotulo()))
            .unwrap_or_default();
        let id = self
            .id()
            .map(|id| format!(" (id {id})"))
            .unwrap_or_default();
        format!("leitura da caixa: {self:?}{categoria}{id}")
    }
}
