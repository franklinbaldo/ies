//! Leitura da visão detalhada já retornada pelo Controle Web.
//! Não executa paginação POST, comandos JS nem atribui categorias fictícias.
use crate::dominio::IdProcedimento;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ProcessoDetalhado {
    pub(crate) id: IdProcedimento,
    pub(crate) numero: String,
    pub(crate) visualizado: Option<bool>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LeituraDetalhada {
    pub(crate) unidade: String,
    pub(crate) total_declarado: usize,
    pub(crate) processos: Vec<ProcessoDetalhado>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ErroDetalhada {
    HtmlNaoInterpretado,
    TabelaAusente,
    TabelasRepetidas,
    VisualizacaoIncorreta,
    UnidadeNaoIdentificada,
    UnidadeDiferente,
    ContextoDiferente,
    CabecalhoProcessoAusente,
    CabecalhoProcessoRepetido,
    ColunasDaLinhaDiferentes,
    LinkDoProcessoAusente,
    LinksDoProcessoRepetidos,
    NumeroDoProcessoAusente,
    IdentificadorInvalido,
    IdentificadorRepetido,
    NumeroRepetido,
    LegendaAusente,
    LegendasRepetidas,
    TotalNaoDeclarado,
    FaixaInvalida,
    FaixaDescontinua,
    TotalAlterado,
    PaginasNaoObtidas,
    LimitePaginas,
}

impl ErroDetalhada {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::HtmlNaoInterpretado => "html_detalhado_nao_interpretado",
            Self::TabelaAusente => "tabela_detalhada_ausente",
            Self::TabelasRepetidas => "tabelas_detalhadas_repetidas",
            Self::VisualizacaoIncorreta => "modo_detalhado_nao_confirmado",
            Self::UnidadeNaoIdentificada => "unidade_da_detalhada_ausente",
            Self::UnidadeDiferente => "unidade_da_detalhada_diferente_da_solicitada",
            Self::ContextoDiferente => "contexto_detalhado_alterado",
            Self::CabecalhoProcessoAusente => "cabecalho_processo_detalhado_ausente",
            Self::CabecalhoProcessoRepetido => "cabecalho_processo_detalhado_repetido",
            Self::ColunasDaLinhaDiferentes => "colunas_da_linha_detalhada_diferentes",
            Self::LinkDoProcessoAusente => "link_do_processo_detalhado_ausente",
            Self::LinksDoProcessoRepetidos => "links_do_processo_detalhado_repetidos",
            Self::NumeroDoProcessoAusente => "numero_do_processo_detalhado_ausente",
            Self::IdentificadorInvalido => "identificador_detalhado_invalido",
            Self::IdentificadorRepetido => "identificador_detalhado_repetido",
            Self::NumeroRepetido => "numero_detalhado_repetido",
            Self::LegendaAusente => "legenda_detalhada_ausente",
            Self::LegendasRepetidas => "legendas_detalhadas_repetidas",
            Self::TotalNaoDeclarado => "total_detalhado_nao_declarado",
            Self::FaixaInvalida => "faixa_detalhada_invalida",
            Self::FaixaDescontinua => "faixa_detalhada_descontinua",
            Self::TotalAlterado => "total_detalhado_alterado",
            Self::PaginasNaoObtidas => "paginas_detalhadas_nao_obtidas",
            Self::LimitePaginas => "limite_paginas_detalhadas",
        }
    }
}

/// Assinatura do leitor; até o GREEN, não declara capacidade implementada.
pub(crate) fn ler_paginas(
    _paginas: &[String],
    _unidade_esperada: Option<&str>,
) -> Result<LeituraDetalhada, ErroDetalhada> {
    Err(ErroDetalhada::TabelaAusente)
}

pub(crate) fn possui_tabela(_html: &str) -> bool {
    false
}

#[cfg(test)]
mod tests;
