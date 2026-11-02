//! Troca entre as visões resumida (`R`) e detalhada (`D`) do Controle de
//! Processos e paginação da D, deixando a tela **como foi encontrada**.
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
use crate::dominio::Categoria;
use crate::erro_web::ErroCaixa;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, ErroSubmissaoFormulario,
    EstadoDeSessao, MarcadorDeResultado, OperacaoDeFormulario, ResultadoDoEnvio,
};
use crate::nivel_de_detalhe::ErroNivelDeDetalhe;
use crate::pagina_do_controle::{ErroControleDoFormulario, ErroPaginaDoControle};
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

pub(crate) use definir::definir_colunas;
pub(crate) use definir_erro::{
    ColunasDefinidas, DesfechoDasColunas, ErroDefinirColunas, FalhaDoSalvar, VerificacaoDasColunas,
};
pub(crate) use envio::{EnvioHttps, TelaAtual};
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ErroVisaoDoControle {
    VisaoDoControleNaoReconhecida,
    CharsetDaTelaNaoSuportado,
    IdentificacaoDaUnidadeAusente,
    UnidadeDiferenteDaSolicitada,
    RedirecionamentoAoTrocarVisao,
    VisaoNaoTrocadaAposEnvio {
        esperada: Visao,
    },
    UnidadeAlteradaAoTrocarVisao,
    PaginaAtualDaVisaoDetalhadaIlegivel,
    PaginaAtualDaVisaoDetalhadaIncompativelComAFaixa,
    PaginaForaDoIntervalo {
        solicitada: usize,
    },
    PaginadorDaVisaoDetalhadaNaoPublicado,
    CriteriosDaVisaoDetalhadaAlteradosAposPaginar,
    TotalDeclaradoAlteradoAposPaginar,
    FaixaDaRespostaDiferenteDaPaginaSolicitada,
    RedirecionamentoAposPaginar,
    PaginasDaVisaoResumidaAlteradasAposRestaurar,
    CriteriosDaVisaoResumidaAlteradosAposRestaurar,
    PaginaDaVisaoDetalhadaAlteradaAposRestaurar,
    LinkDoNivelDeDetalheAusente,
    LinksDoNivelDeDetalheRepetidos,
    LinkDoNivelDeDetalheNaoReconhecido,
    /// A tela ficou em `visao_deixada`; a causa é a falha da restauração.
    VisaoDoControleNaoRestaurada {
        visao_deixada: Visao,
        causa: Box<ErroVisaoDoControle>,
    },
    /// A D lembra a página `pagina_deixada` (≥ 1); a tela ficou em `visao_deixada`.
    PaginaDaVisaoDetalhadaNaoRestaurada {
        visao_deixada: Visao,
        pagina_deixada: usize,
        causa: Box<ErroVisaoDoControle>,
    },
    /// A tabela `categoria` da R lembra a página `pagina_deixada` (≥ 1);
    /// a tela ficou na D.
    PaginaDaVisaoResumidaNaoRestaurada {
        categoria: Categoria,
        pagina_deixada: usize,
        causa: Box<ErroVisaoDoControle>,
    },
    Caixa(ErroCaixa),
    Detalhada(ErroDetalhada),
    NivelDeDetalhe(ErroNivelDeDetalhe),
    Formulario(ErroControleDoFormulario),
    Submissao(ErroSubmissaoFormulario),
    Pagina(ErroPaginaDoControle),
    Transporte(FalhaTransporte),
}

impl ErroVisaoDoControle {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::VisaoDoControleNaoReconhecida => "visao_do_controle_nao_reconhecida",
            Self::CharsetDaTelaNaoSuportado => "charset_da_tela_do_controle_nao_suportado",
            Self::IdentificacaoDaUnidadeAusente => "identificacao_da_unidade_ausente_no_controle",
            Self::UnidadeDiferenteDaSolicitada => "unidade_da_tela_diferente_da_solicitada",
            Self::RedirecionamentoAoTrocarVisao => "redirecionamento_ao_trocar_visao_do_controle",
            Self::VisaoNaoTrocadaAposEnvio { .. } => "visao_do_controle_nao_trocada_apos_envio",
            Self::UnidadeAlteradaAoTrocarVisao => "unidade_alterada_ao_trocar_visao_do_controle",
            Self::PaginaAtualDaVisaoDetalhadaIlegivel => "pagina_atual_da_visao_detalhada_ilegivel",
            Self::PaginaAtualDaVisaoDetalhadaIncompativelComAFaixa => {
                "pagina_atual_da_visao_detalhada_incompativel_com_a_faixa"
            }
            Self::PaginaForaDoIntervalo { .. } => "pagina_fora_do_intervalo",
            Self::PaginadorDaVisaoDetalhadaNaoPublicado => {
                "paginador_da_visao_detalhada_nao_publicado"
            }
            Self::CriteriosDaVisaoDetalhadaAlteradosAposPaginar => {
                "criterios_da_visao_detalhada_alterados_apos_paginar"
            }
            Self::TotalDeclaradoAlteradoAposPaginar => "total_declarado_alterado_apos_paginar",
            Self::FaixaDaRespostaDiferenteDaPaginaSolicitada => {
                "faixa_da_resposta_diferente_da_pagina_solicitada"
            }
            Self::RedirecionamentoAposPaginar => "redirecionamento_apos_paginar_controle",
            Self::PaginasDaVisaoResumidaAlteradasAposRestaurar => {
                "paginas_da_visao_resumida_alteradas_apos_restaurar"
            }
            Self::CriteriosDaVisaoResumidaAlteradosAposRestaurar => {
                "criterios_da_visao_resumida_alterados_apos_restaurar"
            }
            Self::PaginaDaVisaoDetalhadaAlteradaAposRestaurar => {
                "pagina_da_visao_detalhada_alterada_apos_restaurar"
            }
            Self::LinkDoNivelDeDetalheAusente => "link_do_nivel_de_detalhe_ausente",
            Self::LinksDoNivelDeDetalheRepetidos => "links_do_nivel_de_detalhe_repetidos",
            Self::LinkDoNivelDeDetalheNaoReconhecido => "link_do_nivel_de_detalhe_nao_reconhecido",
            Self::VisaoDoControleNaoRestaurada { .. } => "visao_do_controle_nao_restaurada",
            Self::PaginaDaVisaoDetalhadaNaoRestaurada { .. } => {
                "pagina_da_visao_detalhada_nao_restaurada"
            }
            Self::PaginaDaVisaoResumidaNaoRestaurada { .. } => {
                "pagina_da_visao_resumida_nao_restaurada"
            }
            Self::Caixa(erro) => erro.subcodigo(),
            Self::Detalhada(erro) => erro.subcodigo(),
            Self::NivelDeDetalhe(erro) => erro.subcodigo(),
            Self::Formulario(erro) => erro.subcodigo(),
            Self::Submissao(erro) => erro.subcodigo(),
            Self::Pagina(erro) => erro.subcodigo(),
            Self::Transporte(erro) => erro.subcodigo(),
        }
    }

    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::VisaoDoControleNaoRestaurada {
                visao_deixada: Visao::Detalhada,
                ..
            } => "a tela do Controle ficou na visão detalhada: a restauração da resumida falhou",
            Self::VisaoDoControleNaoRestaurada {
                visao_deixada: Visao::Resumida,
                ..
            } => "a tela do Controle ficou na visão resumida: a restauração da detalhada falhou",
            Self::PaginaDaVisaoDetalhadaNaoRestaurada { .. } => {
                "a visão detalhada ficou na página pedida: a restauração da página falhou"
            }
            Self::PaginaDaVisaoResumidaNaoRestaurada { .. } => {
                "a lista da visão resumida ficou na página pedida: a restauração da página falhou"
            }
            Self::PaginaForaDoIntervalo { .. } => "a página pedida não existe na lista",
            Self::Transporte(erro) => erro.mensagem(),
            _ => "a visão do Controle de Processos não foi obtida como esperado",
        }
    }

    /// Status e mensagem do SEI, da requisição recusada (inclusive na restauração).
    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::VisaoDoControleNaoRestaurada { causa, .. }
            | Self::PaginaDaVisaoDetalhadaNaoRestaurada { causa, .. }
            | Self::PaginaDaVisaoResumidaNaoRestaurada { causa, .. } => causa.status_http(),
            Self::Submissao(erro) => erro.status_http(),
            Self::Pagina(erro) => erro.status_http(),
            Self::Transporte(erro) => erro.status_http(),
            _ => None,
        }
    }

    /// Campos a mais no JSON da falha de restauração.
    pub(crate) fn detalhes_json(&self) -> Option<serde_json::Value> {
        match self {
            Self::VisaoDoControleNaoRestaurada {
                visao_deixada,
                causa,
            } => Some(serde_json::json!({
                "visao_deixada": visao_deixada.valor(),
                "causa": causa.subcodigo(),
            })),
            Self::PaginaDaVisaoDetalhadaNaoRestaurada {
                visao_deixada,
                pagina_deixada,
                causa,
            } => Some(serde_json::json!({
                "visao_deixada": visao_deixada.valor(),
                "pagina_deixada": pagina_deixada,
                "causa": causa.subcodigo(),
            })),
            Self::PaginaDaVisaoResumidaNaoRestaurada {
                categoria,
                pagina_deixada,
                causa,
            } => Some(serde_json::json!({
                "visao_deixada": "D",
                "categoria": categoria.rotulo(),
                "pagina_deixada": pagina_deixada,
                "causa": causa.subcodigo(),
            })),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_fluxo;

#[cfg(test)]
mod tests_resumida;

#[cfg(test)]
mod tests_definir;
