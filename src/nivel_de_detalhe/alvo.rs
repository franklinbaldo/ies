//! Alvo de `ies unidade colunas --definir todas|LISTA` e o plano contra a
//! configuração lida (contracts/cli/unidade-definir-colunas.md). Sem rede.
use super::{ColunaDoDetalhe, NivelDeDetalhe};

/// O que `--definir` pede: as opções publicadas ou um conjunto exato.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AlvoDasColunas {
    /// Todas as opções com `disponivel: true` nesta instalação.
    Todas,
    /// Exatamente estas marcadas, as demais desmarcadas (ordem do catálogo).
    Lista(Vec<ColunaDoDetalhe>),
}

/// Erro de uso em `--definir` (saída 2, zero requisições).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ErroDaListaDeColunas {
    ListaDeColunasVazia,
    /// Item vazio entre vírgulas (`tipo,,anotacao`).
    ItemVazioNaListaDeColunas,
    /// Chave fora das 12 do catálogo; guarda a chave digitada.
    ColunaDesconhecida {
        chave: String,
    },
    ColunaRepetidaNaLista {
        coluna: ColunaDoDetalhe,
    },
}

impl ErroDaListaDeColunas {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::ListaDeColunasVazia => "lista_de_colunas_vazia",
            Self::ItemVazioNaListaDeColunas => "item_vazio_na_lista_de_colunas",
            Self::ColunaDesconhecida { .. } => "coluna_desconhecida",
            Self::ColunaRepetidaNaLista { .. } => "coluna_repetida_na_lista",
        }
    }

    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::ListaDeColunasVazia => "--definir exige `todas` ou uma lista de colunas",
            Self::ItemVazioNaListaDeColunas => "a lista de --definir tem um item vazio",
            Self::ColunaDesconhecida { .. } => {
                "coluna desconhecida em --definir (`ies unidade colunas` lista as chaves)"
            }
            Self::ColunaRepetidaNaLista { .. } => "coluna repetida na lista de --definir",
        }
    }
}

/// Opção pedida que a tela desta instalação não publica.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ColunaIndisponivelNestaInstalacao {
    pub(crate) coluna: ColunaDoDetalhe,
}

/// Marcas encontradas, marcas pedidas e as opções não publicadas,
/// todas na ordem do catálogo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PlanoDasColunas {
    pub(crate) antes: Vec<ColunaDoDetalhe>,
    pub(crate) alvo: Vec<ColunaDoDetalhe>,
    pub(crate) indisponiveis: Vec<ColunaDoDetalhe>,
}

impl AlvoDasColunas {
    /// `todas` ou chaves estáveis separadas por vírgula (espaços em volta
    /// de cada chave são ignorados).
    pub(crate) fn interpretar(texto: &str) -> Result<Self, ErroDaListaDeColunas> {
        let _ = texto;
        todo!("RED: interpretar --definir")
    }
}

impl PlanoDasColunas {
    pub(crate) fn marcar(&self) -> Vec<ColunaDoDetalhe> {
        todo!("RED: marcar")
    }

    pub(crate) fn desmarcar(&self) -> Vec<ColunaDoDetalhe> {
        todo!("RED: desmarcar")
    }

    pub(crate) fn inalterado(&self) -> bool {
        todo!("RED: inalterado")
    }
}

/// Plano do alvo contra a configuração lida; recusa opção não publicada.
pub(crate) fn planejar(
    nivel: &NivelDeDetalhe,
    alvo: &AlvoDasColunas,
) -> Result<PlanoDasColunas, ColunaIndisponivelNestaInstalacao> {
    let _ = (nivel, alvo);
    todo!("RED: planejar")
}
