//! Observações e cobertura: desconhecido nunca significa vazio.
use crate::instalacao::Instalacao;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnidadeExibida(String);

impl UnidadeExibida {
    #[must_use]
    pub fn nova(texto: &str) -> Option<Self> {
        (!texto.trim().is_empty()).then(|| Self(texto.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisaoDoControle {
    Resumida,
    Detalhada,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Proveniencia {
    Cassette { cartao: String },
    LeituraWeb { operacao: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FaixaObservada {
    inicio: usize,
    fim: usize,
}

impl FaixaObservada {
    #[must_use]
    pub fn nova(inicio: usize, fim: usize) -> Option<Self> {
        (inicio > 0 && fim >= inicio).then_some(Self { inicio, fim })
    }

    #[must_use]
    pub fn inicio(self) -> usize {
        self.inicio
    }

    #[must_use]
    pub fn fim(self) -> usize {
        self.fim
    }

    #[must_use]
    pub fn quantidade(self) -> usize {
        self.fim - self.inicio + 1
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabelaIncoerente {
    MaisLinhasQueTotal,
    FaixaMaiorQueTotal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabelaObservada<T> {
    pub total_declarado: Option<usize>,
    pub faixa_observada: Option<FaixaObservada>,
    pub linhas: Vec<T>,
}

impl<T> TabelaObservada<T> {
    /// # Errors
    ///
    /// Recusa uma tabela cuja contagem ou faixa contradiz o total declarado.
    pub fn nova(
        total_declarado: Option<usize>,
        faixa_observada: Option<FaixaObservada>,
        linhas: Vec<T>,
    ) -> Result<Self, TabelaIncoerente> {
        if total_declarado.is_some_and(|total| linhas.len() > total) {
            return Err(TabelaIncoerente::MaisLinhasQueTotal);
        }
        if faixa_observada
            .is_some_and(|faixa| total_declarado.is_some_and(|total| faixa.fim() > total))
        {
            return Err(TabelaIncoerente::FaixaMaiorQueTotal);
        }
        Ok(Self {
            total_declarado,
            faixa_observada,
            linhas,
        })
    }

    #[must_use]
    pub fn pagina_verificavel(&self) -> bool {
        match self.faixa_observada {
            Some(faixa) => faixa.quantidade() == self.linhas.len(),
            None => self.total_declarado == Some(0) && self.linhas.is_empty(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cobertura<T> {
    NaoObservada,
    Observada(T),
}

impl<T> Cobertura<TabelaObservada<T>> {
    #[must_use]
    pub fn suficiente_para_servir(&self) -> bool {
        match self {
            Self::NaoObservada => false,
            Self::Observada(tabela) => tabela.pagina_verificavel(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormaDoControle<T> {
    Resumida {
        recebidos: Cobertura<TabelaObservada<T>>,
        gerados: Cobertura<TabelaObservada<T>>,
    },
    Detalhada {
        processos: Cobertura<TabelaObservada<T>>,
    },
}

impl<T> FormaDoControle<T> {
    #[must_use]
    pub fn visao(&self) -> VisaoDoControle {
        match self {
            Self::Resumida { .. } => VisaoDoControle::Resumida,
            Self::Detalhada { .. } => VisaoDoControle::Detalhada,
        }
    }

    #[must_use]
    pub fn suficiente_para_servir(&self) -> bool {
        match self {
            Self::Resumida { recebidos, gerados } => {
                recebidos.suficiente_para_servir() && gerados.suficiente_para_servir()
            }
            Self::Detalhada { processos } => processos.suficiente_para_servir(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservacaoDoControle<T> {
    pub instalacao: Instalacao,
    pub unidade: UnidadeExibida,
    pub forma: FormaDoControle<T>,
    pub proveniencia: Proveniencia,
}

impl<T> ObservacaoDoControle<T> {
    #[must_use]
    pub fn suficiente_para_servir(&self) -> bool {
        self.forma.suficiente_para_servir()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nao_publica_pagina_parcial_como_lista_vazia() {
        let parcial =
            TabelaObservada::nova(Some(10), FaixaObservada::nova(1, 2), vec!["1", "2"]).unwrap();
        assert!(parcial.pagina_verificavel());
        assert_eq!(parcial.total_declarado, Some(10));
    }

    #[test]
    fn recusa_range_sem_linhas() {
        let vazio: TabelaObservada<()> =
            TabelaObservada::nova(Some(2), FaixaObservada::nova(1, 2), vec![]).unwrap();
        assert!(!vazio.pagina_verificavel());
    }
}
