//! Andamentos de `ies processo NUP` (contracts/cli/processo-obter-arvore.md,
//! seção "Andamentos"): o link assinado do histórico publicado pela árvore,
//! a lista `table#tblHistorico` com a legenda "Lista de Andamentos (N …):"
//! e a troca da visão resumida (`R`) pela completa (`P`) pelo formulário
//! servido. As linhas são lidas por [`crate::historico::ler_historico`];
//! aqui só se acrescenta o que a fatia publica. Estrutura em
//! docs/como-o-sei-funciona/historico-do-processo.md.
mod link;
mod lista;
mod visao;

#[cfg(test)]
pub(crate) mod tests_apoio;
#[cfg(test)]
mod tests_link;
#[cfg(test)]
mod tests_lista;
#[cfg(test)]
mod tests_visao;
use crate::historico::ErroHistorico;
use crate::processo::Andamento;
pub(crate) use link::{EnderecoDoHistorico, ler_link_do_historico};
pub(crate) use lista::ler_andamentos;
pub(crate) use visao::{CAMPO_TIPO, VER_HISTORICO_COMPLETO, conferir_regra_de_ver_historico};

/// `acao` do link guardado no índice local (objeto = id do processo).
pub(crate) const ACAO_DO_HISTORICO: &str = "procedimento_consultar_historico";

/// Visão servida, lida de `hdnTipoHistorico` (`R` ou `P`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TipoDoHistorico {
    Resumido,
    Completo,
}

impl TipoDoHistorico {
    pub(crate) fn como_texto(self) -> &'static str {
        match self {
            Self::Resumido => "resumido",
            Self::Completo => "completo",
        }
    }
}

/// O que a página publicou: visão, total da legenda e as linhas lidas.
/// `itens.len() <= total_declarado` (o contrário é recusa).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AndamentosLidos {
    pub(crate) tipo: TipoDoHistorico,
    pub(crate) total_declarado: u64,
    pub(crate) itens: Vec<Andamento>,
}

/// Falhas do link do histórico na árvore: uma variante por fato.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroLinkDoHistorico {
    HtmlDaArvoreNaoInterpretado,
    LinkDoHistoricoAusenteNaArvore,
    LinksDoHistoricoRepetidosNaArvore,
    LinkDoHistoricoForaDaFormaObservada,
    EnderecoDoHistoricoNaoReconhecido,
    EnderecoDoHistoricoDeOutroProcesso,
}

impl ErroLinkDoHistorico {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::HtmlDaArvoreNaoInterpretado => "html_da_arvore_nao_interpretado",
            Self::LinkDoHistoricoAusenteNaArvore => "link_do_historico_ausente_na_arvore",
            Self::LinksDoHistoricoRepetidosNaArvore => "links_do_historico_repetidos_na_arvore",
            Self::LinkDoHistoricoForaDaFormaObservada => {
                "link_do_historico_fora_da_forma_observada"
            }
            Self::EnderecoDoHistoricoNaoReconhecido => "endereco_do_historico_nao_reconhecido",
            Self::EnderecoDoHistoricoDeOutroProcesso => "endereco_do_historico_de_outro_processo",
        }
    }
}

/// Falhas da página do histórico e da troca de visão.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroAndamentos {
    /// Título, tabela ou linhas (leitor de `crate::historico`).
    Historico(ErroHistorico),
    LegendaDosAndamentosAusente,
    LegendaDosAndamentosForaDaFormaObservada,
    AndamentosAlemDoTotalDeclarado,
    TipoDoHistoricoNaoPublicado,
    CharsetDoHistoricoNaoSuportado,
    RegraDeVerHistoricoNaoObservada,
    VisaoDoHistoricoDiferenteDaPedida,
}

impl ErroAndamentos {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::Historico(erro) => erro.subcodigo(),
            Self::LegendaDosAndamentosAusente => "legenda_dos_andamentos_ausente",
            Self::LegendaDosAndamentosForaDaFormaObservada => {
                "legenda_dos_andamentos_fora_da_forma_observada"
            }
            Self::AndamentosAlemDoTotalDeclarado => "andamentos_alem_do_total_declarado",
            Self::TipoDoHistoricoNaoPublicado => "tipo_do_historico_nao_publicado",
            Self::CharsetDoHistoricoNaoSuportado => "charset_do_historico_nao_suportado",
            Self::RegraDeVerHistoricoNaoObservada => "regra_de_ver_historico_nao_observada",
            Self::VisaoDoHistoricoDiferenteDaPedida => "visao_do_historico_diferente_da_pedida",
        }
    }

    /// A página é de outro NUP: um link guardado errou.
    pub(crate) fn diverge_do_pedido(self) -> bool {
        self == Self::Historico(ErroHistorico::NumeroDoHistoricoDiferenteDoSolicitado)
    }
}
