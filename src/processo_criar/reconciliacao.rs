//! Reconciliação da criação pela caixa Gerados: retratos de cobertura
//! comprovada, identidades opacas e diferenças conservadoras.
//! Não efetua HTTP, não afirma que um processo foi criado pela nossa escrita.
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CandidatoGerado {
    pub(crate) id: String,
    pub(crate) nup: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FotografiaDosGerados {
    por_id: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroFotografia {
    Caixa(#[from] crate::erro_web::ErroCaixa),
    CoberturaIncompleta,
    NupRepetido,
    IdentidadeAlterada,
    IdentidadeAnteriorAusente,
    LinkDoCandidatoNaoComprovado,
    LinksDoCandidatoRepetidos,
}

impl FotografiaDosGerados {
    /// Exige unidade e cobertura integral dos Gerados no Controle R.
    /// Uma página parcial, ainda que apresente o primeiro candidato,
    /// não comprova ausência nem unicidade depois da escrita.
    pub(crate) fn da_tela(html: &str, unidade: &str) -> Result<Self, ErroFotografia> {
        let leitura = crate::paginas::juntar(&[html.to_owned()], Some(unidade))?;
        let tabela = &leitura.caixa.gerados;
        let total = tabela
            .total_declarado
            .ok_or(ErroFotografia::CoberturaIncompleta)?;
        if tabela.processos.len() != total
            || (total != 0 && leitura.gerados.faixa_observada() != Some((1, total)))
        {
            return Err(ErroFotografia::CoberturaIncompleta);
        }
        // A tabela já recusa ID duplicado. Também exigimos um NUP por ID:
        // dois IDs apontando o mesmo processo não formam duas identidades
        // independentes para reconciliar uma escrita numa conta compartilhada.
        let mut nups = HashSet::new();
        for processo in &tabela.processos {
            if !nups.insert(processo.numero.as_str()) {
                return Err(ErroFotografia::NupRepetido);
            }
        }
        let por_id = tabela
            .processos
            .iter()
            .map(|p| (p.id.as_str().to_owned(), p.numero.clone()))
            .collect();
        Ok(Self { por_id })
    }

    /// A fotografia anterior ao POST pode vetar um Location que aponta
    /// para identidade já existente. Não prova causalidade por si só.
    pub(crate) fn contem_id(&self, id: &str) -> bool {
        self.por_id.contains_key(id)
    }

    /// Identifica *candidatos* novos, não afirma que o POST os criou.
    /// Mudança/desaparecimento de item anterior torna a comparação
    /// inconclusiva (não é prova de que repetir a escrita é seguro).
    pub(crate) fn novos(&self, depois: &Self) -> Result<Vec<CandidatoGerado>, ErroFotografia> {
        for (id, numero) in &self.por_id {
            match depois.por_id.get(id) {
                Some(atual) if atual == numero => {}
                Some(_) => return Err(ErroFotografia::IdentidadeAlterada),
                None => return Err(ErroFotografia::IdentidadeAnteriorAusente),
            }
        }
        Ok(depois
            .por_id
            .iter()
            .filter(|(id, _)| !self.por_id.contains_key(*id))
            .map(|(id, nup)| CandidatoGerado {
                id: id.clone(),
                nup: nup.clone(),
            })
            .collect())
    }
}

#[path = "desfechos.rs"]
pub(crate) mod desfechos;

#[cfg(test)]
#[path = "tests_reconciliacao.rs"]
mod tests;
