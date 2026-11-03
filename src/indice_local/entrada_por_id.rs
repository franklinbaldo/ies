//! Tabela `instalacao`: a entrada sem assinatura por `id_procedimento`.
//!
//! Medido pelo IES na `UnB` 4.0.9 (03/11/2026, sessão nova por caso): só
//! `procedimento_trabalhar&id_procedimento=N` sem hash é aceita (302 para a
//! mesma ação assinada pelo servidor); toda outra forma testada foi recusada
//! com "Link sem assinatura" e derrubou a sessão. Por isso este é o **único**
//! lugar do IES que monta uma URL sem assinatura, e só nesta forma.
use super::IndiceLocal;
use super::validacao::conferir_id_procedimento;
use crate::instalacao::Instalacao;
use rusqlite::OptionalExtension;

/// Um upgrade do SEI pode mudar o comportamento: a recusa deixa de valer.
pub(crate) const VALIDADE_DA_RECUSA_SEGUNDOS: f64 = 30.0 * 24.0 * 3600.0;
/// Recusas seguidas que rebaixam uma instalação **já aceita**. A desconhecida
/// (ou de recusa expirada) é marcada na primeira: cada tentativa derruba a
/// sessão de todos os agentes da conta.
pub(crate) const RECUSAS_PARA_REBAIXAR_ACEITA: i64 = 2;

/// `https://host[/base]/controlador.php?acao=procedimento_trabalhar&id_procedimento=N`,
/// exatamente. `None` se o id não for só dígitos ASCII (1 a 20). Nenhum
/// outro parâmetro é aceito: o contexto do link sem o hash é recusado e
/// derruba a sessão.
pub(crate) fn url_da_entrada_por_id(
    instalacao: &Instalacao,
    id_procedimento: &str,
) -> Option<String> {
    conferir_id_procedimento(id_procedimento).ok()?;
    Some(format!(
        "{}/controlador.php?acao=procedimento_trabalhar&id_procedimento={id_procedimento}",
        instalacao.url_base()
    ))
}

impl IndiceLocal {
    /// A instalação aceita a entrada por `id_procedimento` sem hash?
    /// `None`: não se sabe (nunca testada, recusa expirada ou índice desligado).
    pub(crate) fn entrada_por_id(&self, instalacao: &Instalacao) -> Option<bool> {
        let origem = instalacao.url_base();
        self.executar(None, |banco, agora| {
            let linha = banco
                .query_row(
                    "SELECT entrada_por_id, entrada_por_id_testado_em FROM instalacao \
                     WHERE origem = ?1",
                    [&origem],
                    |l| Ok((l.get::<_, Option<i64>>(0)?, l.get::<_, Option<f64>>(1)?)),
                )
                .optional()?;
            Ok(match linha {
                Some((Some(1), _)) => Some(true),
                Some((Some(0), testado)) => {
                    (agora - testado.unwrap_or(0.0) < VALIDADE_DA_RECUSA_SEGUNDOS).then_some(false)
                }
                _ => None,
            })
        })
    }

    /// Registre só o 302 para a mesma ação assinada (aceite) ou a recusa
    /// característica ("Link sem assinatura" na tela de login); sessão
    /// vencida não é recusa. Aceite zera a conta; a instalação desconhecida
    /// ou de recusa expirada é recusada na primeira; a já aceita, só depois
    /// de `RECUSAS_PARA_REBAIXAR_ACEITA` seguidas.
    pub(crate) fn registrar_entrada_por_id(&self, instalacao: &Instalacao, aceita: bool) {
        let origem = instalacao.url_base();
        self.executar((), |banco, agora| {
            banco.execute(
                "INSERT INTO instalacao (origem) VALUES (?1) ON CONFLICT (origem) DO NOTHING",
                [&origem],
            )?;
            let (estado, recusas): (Option<i64>, i64) = banco.query_row(
                "SELECT entrada_por_id, recusas FROM instalacao WHERE origem = ?1",
                [&origem],
                |l| Ok((l.get(0)?, l.get(1)?)),
            )?;
            let (novo, recusas) = if aceita {
                (1, 0)
            } else {
                let recusas = recusas.saturating_add(1);
                let segue_aceita = estado == Some(1) && recusas < RECUSAS_PARA_REBAIXAR_ACEITA;
                (i64::from(segue_aceita), recusas)
            };
            banco.execute(
                "UPDATE instalacao SET entrada_por_id = ?2, entrada_por_id_testado_em = ?3, \
                 recusas = ?4 WHERE origem = ?1",
                rusqlite::params![origem, novo, agora, recusas],
            )?;
            Ok(())
        });
    }
}
