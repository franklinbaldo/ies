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
#[derive(Clone, Debug, PartialEq, Eq)]
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
    Formulario(ErroControleDoFormulario),
    Submissao(ErroSubmissaoFormulario),
    Caixa(ErroCaixa),
}

impl ErroPaginaDoControle {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::PaginaExigeVisaoResumida => "pagina_do_controle_exige_visao_resumida",
            Self::CharsetDaTelaNaoSuportado => "charset_da_tela_do_controle_nao_suportado",
            Self::PaginaAtualDaTabelaIlegivel { .. } => "pagina_atual_da_tabela_ilegivel",
            Self::CamposDePaginaAtualDaTabelaRepetidos { .. } => {
                "campos_de_pagina_atual_da_tabela_repetidos"
            }
            Self::PaginaAtualDaTabelaIncompativelComAFaixa { .. } => {
                "pagina_atual_da_tabela_incompativel_com_a_faixa"
            }
            Self::PaginaForaDoIntervalo { .. } => "pagina_fora_do_intervalo",
            Self::PaginadorDaTabelaNaoPublicado { .. } => "paginador_da_tabela_nao_publicado",
            Self::UnidadeAlteradaAposPaginar => "unidade_alterada_apos_paginar",
            Self::VisualizacaoAlteradaAposPaginar => "visualizacao_alterada_apos_paginar",
            Self::CriteriosRecebidosAlteradosAposPaginar => {
                "criterios_recebidos_alterados_apos_paginar"
            }
            Self::CriteriosGeradosAlteradosAposPaginar => {
                "criterios_gerados_alterados_apos_paginar"
            }
            Self::TotalDeclaradoAlteradoAposPaginar { .. } => {
                "total_declarado_alterado_apos_paginar"
            }
            Self::FaixaDaRespostaDiferenteDaPaginaSolicitada { .. } => {
                "faixa_da_resposta_diferente_da_pagina_solicitada"
            }
            Self::RedirecionamentoAposPaginar => "redirecionamento_apos_paginar_controle",
            Self::Formulario(erro) => erro.subcodigo(),
            Self::Submissao(erro) => erro.subcodigo(),
            Self::Caixa(erro) => erro.subcodigo(),
        }
    }

    /// Status e mensagem do SEI quando o POST foi recusado pelo status.
    pub(crate) fn status_http(&self) -> Option<&crate::transporte::StatusHttpRecusado> {
        match self {
            Self::Submissao(erro) => erro.status_http(),
            _ => None,
        }
    }
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
