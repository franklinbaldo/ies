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
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
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
        if texto.trim().is_empty() {
            return Err(ErroDaListaDeColunas::ListaDeColunasVazia);
        }
        if texto.trim() == "todas" {
            return Ok(Self::Todas);
        }
        let mut colunas = Vec::new();
        for item in texto.split(',').map(str::trim) {
            if item.is_empty() {
                return Err(ErroDaListaDeColunas::ItemVazioNaListaDeColunas);
            }
            let coluna = ColunaDoDetalhe::pela_chave(item).ok_or_else(|| {
                ErroDaListaDeColunas::ColunaDesconhecida {
                    chave: item.to_owned(),
                }
            })?;
            if colunas.contains(&coluna) {
                return Err(ErroDaListaDeColunas::ColunaRepetidaNaLista { coluna });
            }
            colunas.push(coluna);
        }
        colunas.sort_unstable();
        Ok(Self::Lista(colunas))
    }
}

impl PlanoDasColunas {
    pub(crate) fn marcar(&self) -> Vec<ColunaDoDetalhe> {
        self.alvo
            .iter()
            .filter(|coluna| !self.antes.contains(coluna))
            .copied()
            .collect()
    }

    pub(crate) fn desmarcar(&self) -> Vec<ColunaDoDetalhe> {
        self.antes
            .iter()
            .filter(|coluna| !self.alvo.contains(coluna))
            .copied()
            .collect()
    }

    pub(crate) fn inalterado(&self) -> bool {
        self.antes == self.alvo
    }
}

/// Plano do alvo contra a configuração lida; recusa opção não publicada.
pub(crate) fn planejar(
    nivel: &NivelDeDetalhe,
    alvo: &AlvoDasColunas,
) -> Result<PlanoDasColunas, ColunaIndisponivelNestaInstalacao> {
    let disponiveis: Vec<_> = nivel
        .opcoes
        .iter()
        .filter(|opcao| opcao.disponivel)
        .map(|opcao| opcao.coluna)
        .collect();
    let alvo = match alvo {
        AlvoDasColunas::Todas => disponiveis,
        AlvoDasColunas::Lista(colunas) => {
            if let Some(&coluna) = colunas.iter().find(|c| !disponiveis.contains(c)) {
                return Err(ColunaIndisponivelNestaInstalacao { coluna });
            }
            // A ordem do catálogo é a de `ColunaDoDetalhe` (derive `Ord`).
            let mut ordenadas = colunas.clone();
            ordenadas.sort_unstable();
            ordenadas
        }
    };
    Ok(PlanoDasColunas {
        antes: nivel.marcadas(),
        alvo,
        indisponiveis: nivel
            .opcoes
            .iter()
            .filter(|opcao| !opcao.disponivel)
            .map(|opcao| opcao.coluna)
            .collect(),
    })
}
