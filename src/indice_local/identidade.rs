//! Tabela `processo`: identidade durável NUP ↔ `id_procedimento`, por
//! instalação. O NUP é opaco (comparação exata); a observação nova que
//! conflita com as antigas as substitui.
use super::IndiceLocal;
use super::validacao::{conferir_id_procedimento, conferir_nup};
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

    /// O NUP já visto para este `id_procedimento` (o mesmo palpite).
    pub(crate) fn nup_por_id(&self, instalacao: &Instalacao, id: &str) -> Option<String> {
        // Cache local é melhor esforço: registro fora da forma é ignorado.
        conferir_id_procedimento(id).ok()?;
        let origem = instalacao.url_base();
        self.executar(None, |banco, _| {
            banco
                .query_row(
                    "SELECT nup FROM processo WHERE origem = ?1 AND id_procedimento = ?2",
                    [&origem, id],
                    |l| l.get(0),
                )
                .optional()
        })
    }

    /// Nova observação: apaga, na mesma transação, a linha do mesmo NUP e a
    /// do mesmo id antes de gravar. Entrada inválida não grava e fica nos
    /// avisos (`RegistroRecusado`).
    pub(crate) fn registrar_processo(
        &self,
        instalacao: &Instalacao,
        nup: &str,
        id_procedimento: &str,
    ) {
        let conferido = conferir_nup(nup).and_then(|()| conferir_id_procedimento(id_procedimento));
        if !self.anotar_recusa(conferido) {
            return;
        }
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
