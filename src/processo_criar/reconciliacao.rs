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
    /// Exige unidade e cobertura integral dos Gerados no Controle R.
    /// Uma página parcial, ainda que apresente o primeiro candidato,
    /// não comprova ausência nem unicidade depois da escrita.
    pub(crate) fn da_tela(html: &str, unidade: &str) -> Result<Self, ErroFotografia> {
        let leitura = crate::paginas::juntar(&[html.to_owned()], Some(unidade))
            .map_err(ErroFotografia::Caixa)?;
        let tabela = &leitura.caixa.gerados;
        let total = tabela
            .total_declarado
            .ok_or(ErroFotografia::CoberturaIncompleta)?;
        if tabela.processos.len() != total
            || (total != 0 && leitura.gerados.faixa_observada() != Some((1, total)))
        {
            return Err(ErroFotografia::CoberturaIncompleta);
        }
        let por_id = tabela
            .processos
            .iter()
            .map(|p| (p.id.as_str().to_owned(), p.numero.clone()))
            .collect();
        Ok(Self { por_id })
    }

    /// Identifica *candidatos* novos, não afirma que o POST os criou.
    /// Mudança/desaparecimento de item anterior torna a comparação
    /// inconclusiva (não é prova de que repetir a escrita é seguro).
    pub(crate) fn novos(
        &self,
        depois: &Self,
    ) -> Result<Vec<CandidatoGerado>, ErroFotografia> {
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

#[cfg(test)]
#[path = "tests_reconciliacao.rs"]
mod tests;
