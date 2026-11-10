//! Troca entre as visões resumida (`R`) e detalhada (`D`) do Controle de
//! Processos e paginação da D. A sessão fica na visão e na página para as
//! quais o comando foi (decisão do proprietário, 02/11/2026): sem restaurar.
//! Contratos: contracts/cli/unidade-listar-processos-visao-detalhada.md,
//! contracts/cli/unidade-recebidos-gerados.md, contracts/cli/unidade-colunas.md e
//! contracts/cli/unidade-definir-colunas.md.
//!
//! Evidência (02/11/2026, treinamento; capturas privadas só estruturais):
//! `trocarVisualizacao(v)` grava `hdnTipoVisualizacao` e chama `submit()` de
//! `frmProcedimentoControlar` (`UnB`, PEN1, GO); a D publica
//! `hdnDetalhadoPaginaAtual` e `infraAcaoPaginar('=', i, 'Detalhado', null)`
//! (PEN1, GO); a R não publica `hdnDetalhadoPaginaAtual`.
mod definir;
mod definir_erro;
mod envio;
mod fluxo;
mod nivel;
mod passos;
mod plano;
mod resumida;

use crate::caixa_detalhada::ErroDetalhada;
use crate::erro_html::ErroHtml;
use crate::erro_web::ErroCaixa;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, ErroSubmissaoFormulario,
    EstadoDeSessao, MarcadorDeResultado, OperacaoDeFormulario, ResultadoDoEnvio,
};
use crate::nivel_de_detalhe::ErroNivelDeDetalhe;
use crate::pagina_do_controle::{ErroControleDoFormulario, ErroPaginaDoControle};
use crate::transporte::FalhaTransporte;

pub(crate) use definir::definir_colunas;
pub(crate) use definir_erro::ColunasDefinidas;
#[cfg(test)]
pub(crate) use definir_erro::{
    ErroDefinirColunas, EstadoDasColunas, FalhaDoSalvar, VerificacaoDasColunas,
};
pub(crate) use envio::{EnvioContado, EnvioDoControle, EnvioHttps, TelaAtual};
pub(crate) use fluxo::listar_na_visao_detalhada;
pub(crate) use nivel::ler_colunas;
pub(crate) use resumida::obter_na_visao_resumida;

/// Controles de usuário que um `submit()` de `frmProcedimentoControlar`
/// repete, nas duas visões (inventário de 02/11/2026).
const CAMPOS_USUARIO: &[&str] = &[
    "txtBloco",
    "selRecebidosPaginacaoSuperior",
    "selRecebidosPaginacaoInferior",
    "selGeradosPaginacaoSuperior",
    "selGeradosPaginacaoInferior",
    "selDetalhadoPaginacaoSuperior",
    "selDetalhadoPaginacaoInferior",
];

/// `trocarVisualizacao(v)`: só `hdnTipoVisualizacao` muda.
pub(crate) const TROCAR_VISAO_DO_CONTROLE: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "procedimento_controlar",
    formulario: "frmProcedimentoControlar",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &["hdnTipoVisualizacao"],
    campos_ocultos_reescritos: &["hdnTipoVisualizacao"],
    campos_usuario: CAMPOS_USUARIO,
    resultado: ResultadoDoEnvio::Marcador(MarcadorDeResultado {
        elemento: "table",
        classe: "tabelaControle",
    }),
    unidade: ConferenciaDaUnidade::NaTelaDoFormulario,
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::PreferenciaDeVisualizacao,
    },
};

/// `infraAcaoPaginar('=', i, 'Detalhado')` a partir de uma tela D.
pub(crate) const PAGINAR_VISAO_DETALHADA: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "procedimento_controlar",
    formulario: "frmProcedimentoControlar",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &["hdnTipoVisualizacao", "hdnDetalhadoPaginaAtual"],
    campos_ocultos_reescritos: &["hdnDetalhadoPaginaAtual"],
    campos_usuario: CAMPOS_USUARIO,
    resultado: ResultadoDoEnvio::Marcador(MarcadorDeResultado {
        elemento: "table",
        classe: "tabelaControle",
    }),
    unidade: ConferenciaDaUnidade::NaTelaDoFormulario,
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::PaginaOuFiltroLembrado,
    },
};

/// Valor de `hdnTipoVisualizacao`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Visao {
    Resumida,
    Detalhada,
}

impl Visao {
    pub(crate) fn valor(self) -> &'static str {
        match self {
            Self::Resumida => "R",
            Self::Detalhada => "D",
        }
    }
}

/// Uma variante por fato verificável; nenhum valor servido é guardado.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroVisaoDoControle {
    CharsetDaTelaNaoSuportado,
    UnidadeDiferenteDaSolicitada,
    RedirecionamentoAoTrocarVisao,
    VisaoNaoTrocadaAposEnvio { esperada: Visao },
    UnidadeAlteradaAoTrocarVisao,
    PaginaAtualDaVisaoDetalhadaIlegivel,
    PaginaAtualDaVisaoDetalhadaIncompativelComAFaixa,
    PaginaForaDoIntervalo { solicitada: usize },
    PaginadorDaVisaoDetalhadaNaoPublicado,
    CriteriosDaVisaoDetalhadaAlteradosAposPaginar,
    TotalDeclaradoAlteradoAposPaginar,
    FaixaDaRespostaDiferenteDaPaginaSolicitada,
    RedirecionamentoAposPaginar,
    Caixa(#[from] ErroCaixa),
    Detalhada(#[from] ErroDetalhada),
    NivelDeDetalhe(#[from] ErroNivelDeDetalhe),
    Formulario(#[from] ErroControleDoFormulario),
    Submissao(#[from] ErroSubmissaoFormulario),
    Pagina(#[from] ErroPaginaDoControle),
    Transporte(#[from] FalhaTransporte),
    Html(#[from] ErroHtml),
}

impl ErroVisaoDoControle {
    /// A sessão do SEI acabou (decide o relogin único): por variante, sem
    /// percorrer a cadeia `source()`.
    pub(crate) fn sessao_encerrada(&self) -> bool {
        match self {
            Self::Submissao(erro) => erro.sessao_encerrada(),
            Self::Pagina(erro) => erro.sessao_encerrada(),
            Self::Transporte(erro) => erro.sessao_encerrada(),
            Self::CharsetDaTelaNaoSuportado
            | Self::UnidadeDiferenteDaSolicitada
            | Self::RedirecionamentoAoTrocarVisao
            | Self::VisaoNaoTrocadaAposEnvio { .. }
            | Self::UnidadeAlteradaAoTrocarVisao
            | Self::PaginaAtualDaVisaoDetalhadaIlegivel
            | Self::PaginaAtualDaVisaoDetalhadaIncompativelComAFaixa
            | Self::PaginaForaDoIntervalo { .. }
            | Self::PaginadorDaVisaoDetalhadaNaoPublicado
            | Self::CriteriosDaVisaoDetalhadaAlteradosAposPaginar
            | Self::TotalDeclaradoAlteradoAposPaginar
            | Self::FaixaDaRespostaDiferenteDaPaginaSolicitada
            | Self::RedirecionamentoAposPaginar
            | Self::Caixa(_)
            | Self::Detalhada(_)
            | Self::NivelDeDetalhe(_)
            | Self::Formulario(_)
            | Self::Html(_) => false,
        }
    }
}

#[cfg(test)]
mod acervo;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_fluxo;

#[cfg(test)]
mod tests_resumida;

#[cfg(test)]
mod tests_definir;

#[cfg(test)]
mod tests_colheita;

#[cfg(test)]
mod tests_contagem;
