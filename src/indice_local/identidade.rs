//! Tabela `processo`: identidade durável NUP ↔ `id_procedimento`, por
//! instalação. O NUP é opaco (comparação exata); a observação nova que
//! conflita com as antigas as substitui.
use super::IndiceLocal;
use super::validacao::{RegistroRecusado, conferir_id_procedimento, conferir_nup};
use crate::instalacao::Instalacao;
use rusqlite::OptionalExtension;

impl IndiceLocal {
    /// O `id_procedimento` já visto para este NUP. Palpite: a árvore aberta
    /// tem de mostrar o NUP pedido; se não mostrar, `esquecer_processo`.
    pub(crate) fn id_por_nup(&self, instalacao: &Instalacao, nup: &str) -> Option<String> {
        let origem = instalacao.url_base();
        let id: Option<String> = self.executar(None, |banco, _| {
            banco
                .query_row(
                    "SELECT id_procedimento FROM processo WHERE origem = ?1 AND nup = ?2",
                    [&origem, nup],
                    |l| l.get(0),
                )
                .optional()
        });
        id.filter(|id| conferir_id_procedimento(id).is_ok())
    }

    /// Nova observação: apaga, na mesma transação, a linha do mesmo NUP e a
    /// do mesmo id antes de gravar.
    pub(crate) fn registrar_processo(
        &self,
        instalacao: &Instalacao,
        nup: &str,
        id_procedimento: &str,
    ) -> Result<(), RegistroRecusado> {
        conferir_nup(nup)?;
        conferir_id_procedimento(id_procedimento)?;
        let origem = instalacao.url_base();
        self.executar((), |banco, agora| {
            banco.execute(
                "DELETE FROM processo WHERE origem = ?1 AND (nup = ?2 OR id_procedimento = ?3)",
                [&origem, nup, id_procedimento],
            )?;
            banco.execute(
                "INSERT INTO processo (origem, nup, id_procedimento, observado_em) \
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![origem, nup, id_procedimento, agora],
            )?;
            Ok(())
        });
        Ok(())
    }

    /// O servidor divergiu: esquece o NUP.
    pub(crate) fn esquecer_processo(&self, instalacao: &Instalacao, nup: &str) {
        let origem = instalacao.url_base();
        self.executar((), |banco, _| {
            banco.execute(
                "DELETE FROM processo WHERE origem = ?1 AND nup = ?2",
                [&origem, nup],
            )?;
            Ok(())
        });
    }
}
