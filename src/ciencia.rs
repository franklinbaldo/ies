//! Ciência pela interface Web: interpretação de aceite e confirmação por lista.
//! Oráculo de comportamento: Todos tests/test_ciclo_ciencia_010.py e
//! tests/test_documento_ciente_010.py @323eb01, sem portar a arquitetura.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AlvoDaCiencia<'a> {
    Processo {
        id_procedimento: &'a str,
    },
    Documento {
        id_procedimento: &'a str,
        id_documento: &'a str,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroCiencia {
    AceiteSemAtividade,
    AtividadesAmbiguas,
    ListaDeCienciasForaDaFormaObservada,
    AtividadesRepetidasNaLista,
}

impl ErroCiencia {
    pub(crate) const fn subcodigo(self) -> &'static str {
        match self {
            Self::AceiteSemAtividade => "aceite_da_ciencia_sem_atividade",
            Self::AtividadesAmbiguas => "atividades_da_ciencia_ambiguas",
            Self::ListaDeCienciasForaDaFormaObservada => "lista_de_ciencias_fora_da_forma_observada",
            Self::AtividadesRepetidasNaLista => "atividades_repetidas_na_lista_de_ciencias",
        }
    }
}

/// Extrai só a atividade identificada pelo aceite, vinculada ao alvo.
pub(crate) fn atividade_do_aceite(
    _html: &str,
    _alvo: AlvoDaCiencia<'_>,
) -> Result<String, ErroCiencia> {
    Err(ErroCiencia::AceiteSemAtividade)
}

/// Confirma uma atividade na lista novamente lida, nunca no HTML do aceite.
pub(crate) fn atividade_na_lista(_html: &str, _atividade: &str) -> Result<bool, ErroCiencia> {
    Ok(false)
}

#[cfg(test)]
mod tests;
