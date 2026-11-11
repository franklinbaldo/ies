//! Unidade publicada no Controle de Processos em qualquer visão observada.
//!
//! Desde que as leituras deixam a sessão na visão que usaram (sem
//! restaurar), o Controle pode chegar na visão resumida (`R`, tabelas
//! Recebidos/Gerados) ou na detalhada (`D`, `tblProcessosDetalhado`).
//! Login, diagnóstico e `unidade atual` precisam reconhecer as duas.
use crate::caixa_detalhada::{self, ErroDetalhada};
use crate::erro_web::ErroCaixa;
use crate::html_atributos::nos_com_id;
use crate::paginas;

/// A página publica a tabela da visão detalhada (tag ilegível não conta).
pub(crate) fn publica_visao_detalhada(html: &str) -> bool {
    tl::parse(html, tl::ParserOptions::default()).is_ok_and(|dom| {
        nos_com_id(&dom, "tblProcessosDetalhado").is_ok_and(|tabelas| !tabelas.is_empty())
    })
}

/// Rótulo da unidade ativa publicado pelo Controle, na visão `R` ou `D`.
///
/// A visão `D` é lida pelo leitor da detalhada, com as mesmas conferências
/// (modo `D` confirmado, unidade única, tabela e linhas válidas). Uma falha
/// dela vira `IdentificacaoDaUnidadeAusenteNoControle` quando falta a
/// unidade e `HtmlInvalido` nos demais casos.
pub(crate) fn unidade_do_controle(html: &str) -> Result<String, ErroCaixa> {
    if publica_visao_detalhada(html) {
        return caixa_detalhada::ler_paginas(&[html.to_owned()], None)
            .map(|leitura| leitura.unidade)
            .map_err(|erro| match erro {
                ErroDetalhada::UnidadeNaoIdentificada => {
                    ErroCaixa::IdentificacaoDaUnidadeAusenteNoControle
                }
                ErroDetalhada::VisualizacaoIncorreta
                | ErroDetalhada::UnidadeDiferente
                | ErroDetalhada::ContextoDiferente
                | ErroDetalhada::ColunasDaLinhaDiferentes
                | ErroDetalhada::ClassesDeEstadoDiferentesNoLink(_)
                | ErroDetalhada::EstadoDoLinkDoProcessoAusente
                | ErroDetalhada::TotalNaoDeclarado
                | ErroDetalhada::FaixaSemIntervaloComPaginacao
                | ErroDetalhada::FaixaDescontinua
                | ErroDetalhada::TotalAlterado
                | ErroDetalhada::LimitePaginas
                | ErroDetalhada::RotuloDaCelulaDiferenteDoCabecalho
                | ErroDetalhada::ItensDaCelulaNaoSeparados
                | ErroDetalhada::CabecalhoDiferenteEntrePaginas
                | ErroDetalhada::TextoForaDosMarcadoresNaCelula
                | ErroDetalhada::CabecalhoDaVisaoDetalhadaNaoReconhecido
                | ErroDetalhada::CabecalhoProcessoDetalhadoAusente
                | ErroDetalhada::CabecalhoProcessoDetalhadoRepetido
                | ErroDetalhada::ColunaDaVisaoDetalhadaRepetida
                | ErroDetalhada::FaixaDetalhadaInvalida
                | ErroDetalhada::HtmlDetalhadoNaoInterpretado(_)
                | ErroDetalhada::TotalNaoNumerico(_)
                | ErroDetalhada::NumeroDaFaixaIlegivel(_)
                | ErroDetalhada::ControleDaPaginaNaoInterpretado(_)
                | ErroDetalhada::AtributoDaDetalhadaIlegivel(_)
                | ErroDetalhada::IdentificadorDetalhadoInvalido(_)
                | ErroDetalhada::IdentificadorDetalhadoRepetido
                | ErroDetalhada::ItensNaoPrevistosNaCelulaDetalhada
                | ErroDetalhada::LegendaDetalhadaAusente
                | ErroDetalhada::LegendasDetalhadasRepetidas
                | ErroDetalhada::LinkDoProcessoDetalhadoAusente
                | ErroDetalhada::LinksDoProcessoDetalhadoRepetidos
                | ErroDetalhada::MarcadorDaCelulaDetalhadaForaDaFormaObservada
                | ErroDetalhada::NumeroDetalhadoRepetido
                | ErroDetalhada::NumeroDoProcessoDetalhadoAusente(_)
                | ErroDetalhada::TabelaDetalhadaAusente
                | ErroDetalhada::TabelasDetalhadasRepetidas => ErroCaixa::HtmlDaCaixaInvalido,
            });
    }
    paginas::juntar(&[html.to_owned()], None)?
        .unidade
        .ok_or(ErroCaixa::IdentificacaoDaUnidadeAusenteNoControle)
}

#[cfg(test)]
#[path = "controle_reconhecido/tests.rs"]
mod tests;
