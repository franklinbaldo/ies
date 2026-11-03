//! Resposta 200 do POST da pesquisa por protocolo: tabela com barra de
//! contagem, aviso de "sem resultado" ou nenhuma forma reconhecida.
//! Formas observadas na UnB 4.0.9 de treinamento em 03/11/2026 (capturas
//! privadas, só estrutura): `div.pesquisaBarraD` "Exibindo 1 - 10 de 1.234"
//! antes de `table.pesquisaResultado`; `div.pesquisaSemResultado` sozinho.
use super::{ErroResultadoPesquisa, ResultadoPesquisa};
use crate::dominio::IdProcedimento;
use crate::html_atributos::ErroAtributoHtml;

/// Posições `inicio..=fim` (base 1) declaradas pela barra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FaixaDaPesquisa {
    pub(crate) inicio: u64,
    pub(crate) fim: u64,
}

/// Uma página da lista: só números observados, total e faixa da barra.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ListaDaPesquisa {
    pub(crate) processos: Vec<ResultadoPesquisa>,
    pub(crate) total_declarado: u64,
    pub(crate) faixa: FaixaDaPesquisa,
}

/// As três respostas observadas do envio da pesquisa.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RespostaDaPesquisa {
    /// 302 interpretado (nunca seguido): só o identificador interno.
    ProcessoUnico {
        id: IdProcedimento,
    },
    Lista(ListaDaPesquisa),
    /// `div.pesquisaSemResultado`: nada para esta conta/unidade; não é
    /// prova de inexistência do processo.
    SemResultado,
}

/// Uma variante por fato verificável da resposta 200.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroRespostaDaPesquisa {
    HtmlDaRespostaNaoInterpretado,
    RespostaDaPesquisaNaoReconhecida,
    TabelaDeResultadoEAvisoDeSemResultadoNaMesmaResposta,
    BarraDeContagemDaPesquisaAusente,
    BarrasDeContagemDaPesquisaRepetidas,
    TextoDaBarraDeContagemForaDoFormato,
    NumeroDaBarraDeContagemForaDoFormato,
    FaixaDaBarraDeContagemImpossivel,
    LinhasDaTabelaDiferentesDaFaixaDaBarra,
    Resultado(ErroResultadoPesquisa),
    AtributoHtml(ErroAtributoHtml),
}

impl ErroRespostaDaPesquisa {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::HtmlDaRespostaNaoInterpretado => "html_da_resposta_da_pesquisa_nao_interpretado",
            Self::RespostaDaPesquisaNaoReconhecida => "resposta_da_pesquisa_nao_reconhecida",
            Self::TabelaDeResultadoEAvisoDeSemResultadoNaMesmaResposta => {
                "tabela_de_resultado_e_aviso_de_sem_resultado_na_mesma_resposta"
            }
            Self::BarraDeContagemDaPesquisaAusente => "barra_de_contagem_da_pesquisa_ausente",
            Self::BarrasDeContagemDaPesquisaRepetidas => "barras_de_contagem_da_pesquisa_repetidas",
            Self::TextoDaBarraDeContagemForaDoFormato => {
                "texto_da_barra_de_contagem_fora_do_formato"
            }
            Self::NumeroDaBarraDeContagemForaDoFormato => {
                "numero_da_barra_de_contagem_fora_do_formato"
            }
            Self::FaixaDaBarraDeContagemImpossivel => "faixa_da_barra_de_contagem_impossivel",
            Self::LinhasDaTabelaDiferentesDaFaixaDaBarra => {
                "linhas_da_tabela_diferentes_da_faixa_da_barra"
            }
            Self::Resultado(erro) => erro.subcodigo(),
            Self::AtributoHtml(erro) => erro.subcodigo(),
        }
    }
}

/// Número da barra: grupos de 3 dígitos separados por `.` (o primeiro com
/// 1 a 3, sem zero à esquerda); qualquer outra forma é `None`.
pub(crate) fn ler_contagem(_texto: &str) -> Option<u64> {
    todo!("GREEN #58: separador de milhar da barra")
}

/// Classifica a resposta 200 (já sem validação da Infra) em lista ou
/// "sem resultado"; o 302 é interpretado antes, em `formulario_pesquisa`.
pub(crate) fn ler_resposta_html(_html: &str) -> Result<RespostaDaPesquisa, ErroRespostaDaPesquisa> {
    todo!("GREEN #58: tabela, barra e aviso de sem resultado")
}

#[cfg(test)]
#[path = "tests_resposta.rs"]
mod tests;
