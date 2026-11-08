//! Tabela `documento` (Refs #60, fatia 2): número SEI ↔ (`id_procedimento`,
//! `id_documento`), por instalação, aprendido de toda árvore já lida. O
//! número é opaco (zeros à esquerda inclusos, comparação exata); os ids vão
//! para a entrada por id montada e por isso só se guardam dígitos ASCII. A
//! observação nova que conflita com as antigas as substitui.
use super::IndiceLocal;
use super::validacao::{RegistroRecusado, conferir_id_procedimento, conferir_numero_sei};
use crate::instalacao::Instalacao;
use rusqlite::OptionalExtension;

/// Onde a última árvore lida mostrou o número SEI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DocumentoGuardado {
    pub(crate) id_procedimento: String,
    pub(crate) id_documento: String,
}

impl IndiceLocal {
    /// Palpite: a árvore do processo tem de mostrar o número pedido; se não
    /// mostrar, `esquecer_documento`.
    pub(crate) fn documento_por_numero(
        &self,
        instalacao: &Instalacao,
        numero: &str,
    ) -> Option<DocumentoGuardado> {
        conferir_numero_sei(numero).ok()?;
        let origem = instalacao.url_base();
        let linha: Option<(String, String)> = self.executar(None, |banco, _| {
            banco
                .query_row(
                    "SELECT id_procedimento, id_documento FROM documento \
                     WHERE origem = ?1 AND numero_sei = ?2",
                    [&origem, numero],
                    |l| Ok((l.get(0)?, l.get(1)?)),
                )
                .optional()
        });
        let (id_procedimento, id_documento) = linha?;
        let ids_validos = conferir_id_procedimento(&id_procedimento).is_ok()
            && conferir_id_procedimento(&id_documento).is_ok();
        ids_validos.then_some(DocumentoGuardado {
            id_procedimento,
            id_documento,
        })
    }

    /// Os documentos `(número SEI, id_documento)` de uma árvore conferida do
    /// processo `id_procedimento`, numa transação. Entrada inválida, e todo
    /// número ou id que se repete na mesma árvore, invalida qualquer palpite
    /// anterior; cada linha inequívoca substitui a associação antiga.
    pub(crate) fn registrar_documentos(
        &self,
        instalacao: &Instalacao,
        id_procedimento: &str,
        documentos: &[(&str, &str)],
    ) -> Result<(), RegistroRecusado> {
        conferir_id_procedimento(id_procedimento)?;
        let unico = |numero: &str, id: &str| {
            documentos
                .iter()
                .filter(|(n, i)| *n == numero || *i == id)
                .count()
                == 1
        };
        // Filtra entradas válidas uma única vez e preserva a ordem das
        // associações observadas. Duplicatas continuam invalidando palpites.
        let (validos, ambiguos): (Vec<_>, Vec<_>) = documentos
            .iter()
            .copied()
            .filter(|(numero, id)| {
                conferir_numero_sei(numero).is_ok()
                    && conferir_id_procedimento(id).is_ok()
            })
            .partition(|(numero, id)| unico(numero, id));
        if validos.is_empty() && ambiguos.is_empty() {
            return Ok(());
        }
        let origem = instalacao.url_base();
        self.executar((), |banco, agora| {
            for (numero, id) in &ambiguos {
                banco.execute(
                    "DELETE FROM documento WHERE origem = ?1 \
                     AND (numero_sei = ?2 OR id_documento = ?3)",
                    [origem.as_str(), numero, id],
                )?;
            }
            for (numero, id) in &validos {
                banco.execute(
                    "DELETE FROM documento WHERE origem = ?1 \
                     AND (numero_sei = ?2 OR id_documento = ?3)",
                    [origem.as_str(), numero, id],
                )?;
                banco.execute(
                    "INSERT INTO documento \
                     (origem, numero_sei, id_procedimento, id_documento, observado_em) \
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![origem, numero, id_procedimento, id, agora],
                )?;
            }
            Ok(())
        });
        Ok(())
    }

    /// O servidor divergiu: esquece o número.
    pub(crate) fn esquecer_documento(&self, instalacao: &Instalacao, numero: &str) {
        let origem = instalacao.url_base();
        self.executar((), |banco, _| {
            banco.execute(
                "DELETE FROM documento WHERE origem = ?1 AND numero_sei = ?2",
                [&origem, numero],
            )?;
            Ok(())
        });
    }
}
