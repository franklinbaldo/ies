//! Reconciliação da criação pela caixa Gerados: retratos de cobertura
//! comprovada, identidades opacas e diferenças conservadoras.
//! Não efetua HTTP, não afirma que um processo foi criado pela nossa escrita.
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CandidatoGerado {
    pub(crate) id: String,
    pub(crate) nup: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FotografiaDosGerados {
    por_id: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroFotografia {
    #[causa(interna)]
    Caixa(crate::erro_web::ErroCaixa),
    #[causa(nome = "gerados_sem_cobertura_completa_para_reconciliacao")]
    CoberturaIncompleta,
    #[causa(nome = "identidade_ja_observada_alterada_na_reconciliacao")]
    IdentidadeAlterada,
    #[causa(nome = "identidade_anterior_ausente_na_reconciliacao")]
    IdentidadeAnteriorAusente,
}

impl FotografiaDosGerados {
    pub(crate) fn da_tela(_html: &str, _unidade: &str) -> Result<Self, ErroFotografia> {
        Err(ErroFotografia::CoberturaIncompleta)
    }

    pub(crate) fn novos(
        &self,
        _depois: &Self,
    ) -> Result<Vec<CandidatoGerado>, ErroFotografia> {
        Err(ErroFotografia::CoberturaIncompleta)
    }
}

#[cfg(test)]
#[path = "tests_reconciliacao.rs"]
mod tests;
