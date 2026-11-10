//! `--categoria T --pagina N` da caixa: no máximo um POST do paginador de
//! uma tabela, derivado de `frmProcedimentoControlar` servido.
//! Contrato: contracts/cli/unidade-listar-processos-paginas.md.
//! Evidência estrutural (02/11/2026, treinamento): PEN1, PEN3, PEN4 (SEI
//! 5.0.4) e Goiás (5.0.3) publicam `infraAcaoPaginar('+'|'=', i, 'T', null)`
//! em `lnk{T}ProximaPagina*`, `lnk{T}UltimaPagina*` e `sel{T}Paginacao*`,
//! com índice a partir de 0 gravado em `hdn{T}PaginaAtual`.
mod envio;
mod plano;
mod regras;

use crate::dominio::Categoria;
use crate::erro_web::ErroCaixa;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, ErroSubmissaoFormulario,
    EstadoDeSessao, MarcadorDeResultado, OperacaoDeFormulario, ResultadoDoEnvio,
};
use crate::paginas::FaixaNaPagina;
#[cfg(test)]
use crate::{
    formulario_web::{RespostaDoPost, SubmissaoDerivada, TelaServida},
    instalacao::Instalacao,
    transporte::FalhaTransporte,
};
#[cfg(test)]
pub(crate) use envio::derivar_pagina;
pub(crate) use envio::obter_pagina_via;
pub(crate) use plano::{codificacao_da_tela, planejar, tamanho_da_pagina};
pub(crate) use regras::{ErroControleDoFormulario, campos_do_usuario};

/// O servidor lembra a página de cada tabela na sessão.
pub(crate) const OBTER_PAGINA_DO_CONTROLE: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "procedimento_controlar",
    formulario: "frmProcedimentoControlar",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &[
        "hdnRecebidosPaginaAtual",
        "hdnGeradosPaginaAtual",
        "hdnRecebidosHashCriterios",
        "hdnGeradosHashCriterios",
        "hdnTipoVisualizacao",
    ],
    campos_ocultos_reescritos: &["hdnRecebidosPaginaAtual", "hdnGeradosPaginaAtual"],
    campos_usuario: regras::CAMPOS_USUARIO,
    resultado: ResultadoDoEnvio::Marcador(MarcadorDeResultado {
        elemento: "table",
        classe: "tabelaControle",
    }),
    unidade: ConferenciaDaUnidade::NaTelaDoFormulario,
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::PaginaOuFiltroLembrado,
    },
};

/// Uma variante por fato verificável; nenhum valor servido é guardado.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroPaginaDoControle {
    PaginaExigeVisaoResumida,
    CharsetDaTelaNaoSuportado,
    PaginaAtualDaTabelaIlegivel {
        categoria: Categoria,
    },
    CamposDePaginaAtualDaTabelaRepetidos {
        categoria: Categoria,
    },
    PaginaAtualDaTabelaIncompativelComAFaixa {
        categoria: Categoria,
    },
    PaginaForaDoIntervalo {
        categoria: Categoria,
        solicitada: usize,
        paginas: usize,
    },
    PaginadorDaTabelaNaoPublicado {
        categoria: Categoria,
    },
    UnidadeAlteradaAposPaginar,
    VisualizacaoAlteradaAposPaginar,
    CriteriosRecebidosAlteradosAposPaginar,
    CriteriosGeradosAlteradosAposPaginar,
    TotalDeclaradoAlteradoAposPaginar {
        categoria: Categoria,
    },
    FaixaDaRespostaDiferenteDaPaginaSolicitada {
        categoria: Categoria,
    },
    RedirecionamentoAposPaginar,
    Formulario(#[from] ErroControleDoFormulario),
    Submissao(#[from] ErroSubmissaoFormulario),
    Caixa(#[from] ErroCaixa),
}

/// O que a tela de origem exige para mostrar a página N.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlanoDaPagina {
    /// A tela já mostra a página N: nenhum POST.
    JaExibida,
    /// Um POST com `hdn{T}PaginaAtual = indice`; a resposta deve trazer `esperada`.
    Pedir {
        indice: usize,
        esperada: FaixaNaPagina,
    },
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_envio;
