//! `--categoria T --pagina N` da caixa: no máximo um POST do paginador de
//! uma tabela, derivado de `frmProcedimentoControlar` servido.
//! Contrato: contracts/cli/unidade-listar-processos-paginas.md.
//! Evidência estrutural (02/11/2026, treinamento): PEN1, PEN3, PEN4 (SEI
//! 5.0.4) e Goiás (5.0.3) publicam `infraAcaoPaginar('+'|'=', i, 'T', null)`
//! em `lnk{T}ProximaPagina*`, `lnk{T}UltimaPagina*` e `sel{T}Paginacao*`,
//! com índice a partir de 0 gravado em `hdn{T}PaginaAtual`.
mod regras;

use crate::dominio::Categoria;
use crate::erro_web::ErroCaixa;
use crate::formulario_web::{
    CodificacaoDaTela, EfeitoAvaliado, EnvioDoFormulario, ErroSubmissaoFormulario, EstadoDeSessao,
    MarcadorDeResultado, OperacaoDeFormulario, RespostaDoPost, SubmissaoDerivada, TelaServida,
};
use crate::instalacao::Instalacao;
use crate::paginas::FaixaNaPagina;
use crate::transporte::FalhaTransporte;
pub(crate) use regras::ErroControleDoFormulario;
use std::io::Read;
use std::path::Path;

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
    marcador_de_resultado: MarcadorDeResultado {
        elemento: "table",
        classe: "tabelaControle",
    },
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

/// Charset do `Content-Type` da tela, para codificar o corpo do POST.
/// Shim CONTRACT.
pub(crate) fn codificacao_da_tela(_mime: &str) -> Option<CodificacaoDaTela> {
    None
}

/// Calcula, sem rede, se e como pedir a página `pagina` (≥ 1). Shim CONTRACT.
pub(crate) fn planejar(
    _html: &str,
    _categoria: Categoria,
    _pagina: usize,
) -> Result<PlanoDaPagina, ErroPaginaDoControle> {
    Err(ErroPaginaDoControle::PaginaExigeVisaoResumida)
}

/// Deriva o POST do paginador, sem rede. Shim CONTRACT.
pub(crate) fn derivar_pagina(
    _instalacao: &Instalacao,
    _tela: &TelaServida<'_>,
    _unidade: &str,
    _categoria: Categoria,
    _indice: usize,
) -> Result<SubmissaoDerivada, ErroPaginaDoControle> {
    Err(ErroPaginaDoControle::PaginaExigeVisaoResumida)
}

/// Obtém a página: `Ok(None)` se a tela já a mostra (zero POST), senão
/// envia uma vez (`FnOnce`, sem retry) e confere a resposta. Shim CONTRACT.
pub(crate) fn obter_pagina_via<R: Read, F>(
    _instalacao: &Instalacao,
    _tela: &TelaServida<'_>,
    _unidade: &str,
    _categoria: Categoria,
    _pagina: usize,
    _cookie_file: &Path,
    _enviar: F,
) -> Result<Option<String>, ErroPaginaDoControle>
where
    F: FnOnce(SubmissaoDerivada, &str) -> Result<RespostaDoPost<R>, FalhaTransporte>,
{
    Err(ErroPaginaDoControle::PaginaExigeVisaoResumida)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_envio;
