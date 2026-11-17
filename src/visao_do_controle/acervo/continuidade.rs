//! Continuidade entre páginas gravadas da mesma lista: na mesma cassette,
//! mesma lista, unidade, critério e total, a página seguinte começa logo
//! depois da anterior e não repete ids. Uma repetição da mesma faixa é só
//! contada (com os ids iguais ou não: a caixa muda entre requisições).
use super::{Lista, PaginaLida};

/// Relação entre duas páginas gravadas, em ordem, da mesma lista.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Relacao {
    /// `b.inicio == a.fim + 1` e nenhum id em comum.
    Continua,
    /// `b.inicio == a.fim + 1`, mas com id repetido.
    ContinuaComIdRepetido,
    /// `b.inicio > a.fim + 1`.
    Lacuna,
    /// `a.inicio < b.inicio <= a.fim`.
    Sobreposta,
    /// Mesma faixa, mesmos ids na mesma ordem.
    MesmaPaginaIgual,
    /// Mesma faixa, ids diferentes.
    MesmaPaginaDiferente,
    /// Página anterior à de `a` (voltou).
    Anterior,
}

impl Relacao {
    pub(super) fn nome(self) -> &'static str {
        match self {
            Self::Continua => "continua (sem lacuna e sem id repetido)",
            Self::ContinuaComIdRepetido => "continua com id repetido",
            Self::Lacuna => "lacuna entre as faixas",
            Self::Sobreposta => "faixas sobrepostas",
            Self::MesmaPaginaIgual => "mesma faixa, mesmos ids",
            Self::MesmaPaginaDiferente => "mesma faixa, ids diferentes",
            Self::Anterior => "voltou a uma faixa anterior",
        }
    }
}

/// Um par comparável e a relação encontrada.
#[derive(Clone, Debug)]
pub(super) struct Par<'a> {
    pub(super) a: &'a PaginaLida,
    pub(super) b: &'a PaginaLida,
    pub(super) relacao: Relacao,
}

fn mesma_lista(a: &PaginaLida, b: &PaginaLida) -> bool {
    a.card == b.card
        && a.lista == b.lista
        && a.unidade == b.unidade
        && a.criterio == b.criterio
        && a.total == b.total
}

/// Compara cada página com a anterior comparável da mesma lista (páginas
/// sem linhas não entram).
pub(super) fn pares(paginas: &[PaginaLida]) -> Vec<Par<'_>> {
    let mut saida = Vec::new();
    for (i, b) in paginas.iter().enumerate() {
        let Some((bi, bf)) = b.faixa else { continue };
        let Some(a) = paginas
            .iter()
            .take(i)
            .rev()
            .find(|a| a.faixa.is_some() && a.lista == b.lista && a.card == b.card)
        else {
            continue;
        };
        if !mesma_lista(a, b) {
            continue;
        }
        let (ai, af) = a.faixa.unwrap_or_default();
        let repetido = b.ids.iter().any(|id| a.ids.contains(id));
        let relacao = if (bi, bf) == (ai, af) {
            if a.ids == b.ids {
                Relacao::MesmaPaginaIgual
            } else {
                Relacao::MesmaPaginaDiferente
            }
        } else if bi == af + 1 {
            if repetido {
                Relacao::ContinuaComIdRepetido
            } else {
                Relacao::Continua
            }
        } else if bi > af + 1 {
            Relacao::Lacuna
        } else if bi > ai {
            Relacao::Sobreposta
        } else {
            Relacao::Anterior
        };
        saida.push(Par { a, b, relacao });
    }
    saida
}

/// Páginas gravadas além da primeira (faixa começando depois de 1).
pub(super) fn alem_da_primeira(paginas: &[PaginaLida]) -> impl Iterator<Item = &PaginaLida> {
    paginas
        .iter()
        .filter(|p| p.faixa.is_some_and(|(inicio, _)| inicio > 1))
}

pub(super) fn rotulo(lista: &Lista) -> &'static str {
    match lista {
        Lista::Recebidos => "recebidos",
        Lista::Gerados => "gerados",
        Lista::Detalhada => "D",
    }
}
