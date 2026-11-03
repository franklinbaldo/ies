//! `--pagina N` da pesquisa: `navegar(inicio)` da tela de resultado.
//! Regra servida (UnB 4.0.9, 03/11/2026, captura privada só estrutural):
//! `document.getElementById('hdnInicio').value = inicio;`, `onSubmitForm()` e
//! `document.getElementById('frmPesquisaProtocolo').submit();` (sem botão),
//! com links `javascript:navegar('10')`…`navegar('90')` e "Próxima".
use super::formulario_pesquisa::CAMPOS_OCULTOS_DA_PESQUISA;
use super::resposta::{FaixaDaPesquisa, ListaDaPesquisa, RespostaDaPesquisa};
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDeSessao, OperacaoDeFormulario,
};

/// `navegar(inicio)`: `submit()` por script, só `hdnInicio` reescrito.
pub(crate) const PAGINAR_PESQUISA_DE_PROTOCOLO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "protocolo_pesquisar",
    formulario: "frmPesquisaProtocolo",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: CAMPOS_OCULTOS_DA_PESQUISA,
    campos_ocultos_reescritos: &["hdnInicio"],
    campos_usuario: super::formulario_pesquisa::CAMPOS_USUARIO_DA_PESQUISA,
    resultado: super::formulario_pesquisa::MARCADORES_DA_RESPOSTA,
    unidade: ConferenciaDaUnidade::NaTelaDoFormulario,
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::PaginaOuFiltroLembrado,
    },
};

/// Uma variante por fato verificável da paginação da pesquisa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroPaginacaoDaPesquisa {
    PaginaForaDoIntervalo { solicitada: u32, paginas: u64 },
    RegraDeNavegacaoDaPesquisaNaoObservada,
    PaginadorDaPesquisaNaoPublicado,
    LinkDePaginacaoDaPesquisaForaDaRegra,
    ProtocoloDaTelaDeResultadoDiferenteDoPesquisado,
    RedirecionamentoAoPaginarAPesquisa,
    PaginaDaPesquisaSemResultadoAposPaginar,
    TotalDaPesquisaAlteradoAposPaginar,
    FaixaDaPesquisaDiferenteDaPaginaSolicitada,
    HtmlDaTelaDeResultadoNaoInterpretado,
}

impl ErroPaginacaoDaPesquisa {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::PaginaForaDoIntervalo { .. } => "pagina_fora_do_intervalo",
            Self::RegraDeNavegacaoDaPesquisaNaoObservada => {
                "regra_de_navegacao_da_pesquisa_nao_observada"
            }
            Self::PaginadorDaPesquisaNaoPublicado => "paginador_da_pesquisa_nao_publicado",
            Self::LinkDePaginacaoDaPesquisaForaDaRegra => {
                "link_de_paginacao_da_pesquisa_fora_da_regra"
            }
            Self::ProtocoloDaTelaDeResultadoDiferenteDoPesquisado => {
                "protocolo_da_tela_de_resultado_diferente_do_pesquisado"
            }
            Self::RedirecionamentoAoPaginarAPesquisa => "redirecionamento_ao_paginar_a_pesquisa",
            Self::PaginaDaPesquisaSemResultadoAposPaginar => {
                "pagina_da_pesquisa_sem_resultado_apos_paginar"
            }
            Self::TotalDaPesquisaAlteradoAposPaginar => "total_da_pesquisa_alterado_apos_paginar",
            Self::FaixaDaPesquisaDiferenteDaPaginaSolicitada => {
                "faixa_da_pesquisa_diferente_da_pagina_solicitada"
            }
            Self::HtmlDaTelaDeResultadoNaoInterpretado => {
                "html_da_tela_de_resultado_da_pesquisa_nao_interpretado"
            }
        }
    }
}

/// O POST de página a enviar: `hdnInicio` e a faixa que a resposta deve ter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PedidoDePaginaDaPesquisa {
    pub(crate) inicio: u64,
    pub(crate) total: u64,
    pub(crate) esperada: FaixaDaPesquisa,
}

/// Página N a partir da resposta da página 1, sem rede. `Ok(None)`: a
/// resposta já é a página pedida (N = 1). N fora do intervalo é erro antes
/// de qualquer POST de página.
pub(crate) fn planejar_pagina(
    _resposta: &RespostaDaPesquisa,
    _html_da_resposta: Option<&str>,
    _solicitada: u32,
) -> Result<Option<PedidoDePaginaDaPesquisa>, ErroPaginacaoDaPesquisa> {
    todo!("GREEN #58: regra de navegar()")
}

/// Confere a resposta do POST de página contra o pedido.
pub(crate) fn conferir_pagina(
    _pedido: &PedidoDePaginaDaPesquisa,
    _resposta: RespostaDaPesquisa,
) -> Result<ListaDaPesquisa, ErroPaginacaoDaPesquisa> {
    todo!("GREEN #58: conferência da página obtida")
}

#[cfg(test)]
#[path = "tests_paginacao.rs"]
mod tests;
