//! Tabela `instalacao`: a entrada sem assinatura por `id_procedimento`.
//!
//! Medido pelo IES na `UnB` 4.0.9 (03/11/2026, sessão nova por caso): só
//! `procedimento_trabalhar&id_procedimento=N` sem hash é aceita (302 para a
//! mesma ação assinada pelo servidor), opcionalmente com `&id_documento=M`;
//! toda outra forma testada foi recusada com "Link sem assinatura" e
//! derrubou a sessão. Por isso este é o **único** lugar do IES que monta uma
//! URL sem assinatura, e só nessas duas formas. O aceite de cada forma é
//! guardado à parte: a com documento só foi medida na `UnB`.
use super::IndiceLocal;
use super::falhas::FalhaDoIndice;
use super::validacao::conferir_id_procedimento;
use crate::acao_do_sei::AcaoDoSei;
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
) -> Result<String, super::validacao::RegistroRecusado> {
    conferir_id_procedimento(id_procedimento)?;
    Ok(format!(
        "{}/controlador.php?acao={acao}&id_procedimento={id_procedimento}",
        instalacao.url_base(),
        acao = AcaoDoSei::ProcedimentoTrabalhar.como_na_url()
    ))
}

/// A mesma entrada com o documento: `…&id_procedimento=P&id_documento=M`,
/// exatamente (302 para a ação assinada com os dois ids, documento
/// selecionado). `None` se algum id não for só dígitos ASCII (1 a 20).
pub(crate) fn url_da_entrada_por_id_com_documento(
    instalacao: &Instalacao,
    id_procedimento: &str,
    id_documento: &str,
) -> Result<String, super::validacao::RegistroRecusado> {
    conferir_id_procedimento(id_documento)?;
    let mut url = url_da_entrada_por_id(instalacao, id_procedimento)?;
    url.push_str("&id_documento=");
    url.push_str(id_documento);
    Ok(url)
}

/// O que a instalação respondeu à entrada por id, enquanto vale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AceiteDaEntrada {
    /// 302 para a mesma ação assinada.
    Aceita,
    /// "Link sem assinatura" na tela de login, há menos de 30 dias.
    Recusada,
}

/// As colunas do aceite de cada forma medida.
#[derive(Clone, Copy)]
enum Forma {
    Processo,
    ComDocumento,
}

impl Forma {
    /// (estado, testado em, recusas seguidas): nomes fixos, nunca entrada.
    fn colunas(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::Processo => ("entrada_por_id", "entrada_por_id_testado_em", "recusas"),
            Self::ComDocumento => (
                "entrada_com_documento",
                "entrada_com_documento_testado_em",
                "recusas_com_documento",
            ),
        }
    }
}

impl IndiceLocal {
    /// A instalação aceita a entrada por `id_procedimento` sem hash?
    /// `None`: não há registro vigente (nunca testada, recusa expirada ou
    /// índice sem banco; a falha do banco fica nos avisos).
    pub(crate) fn entrada_por_id(&self, instalacao: &Instalacao) -> Option<AceiteDaEntrada> {
        self.aceite(instalacao, Forma::Processo)
    }

    /// A instalação aceita a forma com `id_documento`? Mesma regra.
    pub(crate) fn entrada_por_id_com_documento(
        &self,
        instalacao: &Instalacao,
    ) -> Option<AceiteDaEntrada> {
        self.aceite(instalacao, Forma::ComDocumento)
    }

    /// Registro do 302 ou da recusa da forma com documento (mesma regra de
    /// [`Self::registrar_entrada_por_id`], contada à parte).
    pub(crate) fn registrar_entrada_por_id_com_documento(
        &self,
        instalacao: &Instalacao,
        aceita: bool,
    ) {
        self.registrar_aceite(instalacao, Forma::ComDocumento, aceita);
    }

    /// O estado lido do SQLite vira o enum aqui; um inteiro fora de `{0, 1}`
    /// fica nos avisos e conta como sem registro.
    fn aceite(&self, instalacao: &Instalacao, forma: Forma) -> Option<AceiteDaEntrada> {
        let (estado, testado, _) = forma.colunas();
        let origem = instalacao.url_base();
        let lido = self.executar(None, |banco, agora| {
            let linha = banco
                .query_row(
                    &format!("SELECT {estado}, {testado} FROM instalacao WHERE origem = ?1"),
                    [&origem],
                    |l| Ok((l.get::<_, Option<i64>>(0)?, l.get::<_, Option<f64>>(1)?)),
                )
                .optional()?;
            Ok(linha.and_then(|(estado, testado)| Some((estado?, agora - testado.unwrap_or(0.0)))))
        });
        match lido? {
            (1, _) => Some(AceiteDaEntrada::Aceita),
            (0, idade) => {
                (idade < VALIDADE_DA_RECUSA_SEGUNDOS).then_some(AceiteDaEntrada::Recusada)
            }
            (outro, _) => {
                self.anotar(FalhaDoIndice::EstadoDoAceiteForaDoConjunto(outro));
                None
            }
        }
    }

    /// Registre só o 302 para a mesma ação assinada (aceite) ou a recusa
    /// característica ("Link sem assinatura" na tela de login); sessão
    /// vencida não é recusa. Aceite zera a conta; a instalação desconhecida
    /// ou de recusa expirada é recusada na primeira; a já aceita, só depois
    /// de `RECUSAS_PARA_REBAIXAR_ACEITA` seguidas.
    pub(crate) fn registrar_entrada_por_id(&self, instalacao: &Instalacao, aceita: bool) {
        self.registrar_aceite(instalacao, Forma::Processo, aceita);
    }

    fn registrar_aceite(&self, instalacao: &Instalacao, forma: Forma, aceita: bool) {
        let (coluna, testado, contagem) = forma.colunas();
        let origem = instalacao.url_base();
        self.executar((), |banco, agora| {
            banco.execute(
                "INSERT INTO instalacao (origem) VALUES (?1) ON CONFLICT (origem) DO NOTHING",
                [&origem],
            )?;
            let (estado, recusas): (Option<i64>, i64) = banco.query_row(
                &format!("SELECT {coluna}, {contagem} FROM instalacao WHERE origem = ?1"),
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
                &format!(
                    "UPDATE instalacao SET {coluna} = ?2, {testado} = ?3, {contagem} = ?4 \
                     WHERE origem = ?1"
                ),
                rusqlite::params![origem, novo, agora, recusas],
            )?;
            Ok(())
        });
    }
}
