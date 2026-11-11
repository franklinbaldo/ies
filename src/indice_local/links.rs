//! Tabela `link_assinado`: URLs assinadas **literais**, válidas só na
//! geração de sessão em que foram vistas. A unidade faz parte da chave: o
//! hash cobre `infra_unidade_atual`, e trocar de unidade invalida os links
//! do menu (medido pelo Todos). O índice nunca monta nem edita uma URL
//! assinada: guarda a que a tela publicou e a devolve igual.
use super::IndiceLocal;
use super::validacao::{RegistroRecusado, UrlAssinada, conferir_chave, conferir_url};
use crate::acao_do_sei::AcaoDoSei;
use crate::instalacao::Instalacao;
use rusqlite::OptionalExtension;

/// Chave de um link assinado. `geracao_sessao` é opaca (o SHA-256 do cookie);
/// `unidade` é o id de `infra_unidade_atual`; `alvo` identifica o alvo
/// (por exemplo o `id_procedimento`), vazio quando a tela não tem alvo.
/// Sem `Debug`: a geração deriva do cookie.
#[derive(Clone, Copy)]
pub(crate) struct ChaveDoLink<'a> {
    pub(crate) geracao_sessao: &'a str,
    pub(crate) unidade: &'a str,
    pub(crate) acao: AcaoDoSei,
    pub(crate) alvo: &'a str,
}

/// Unidade da chave dos links: o `infra_unidade_atual` único e decimal da
/// URL privada do Controle. `None` se ausente, repetido ou não decimal.
pub(crate) fn unidade_dos_links(url_do_controle: &str) -> Option<String> {
    let consulta = url_do_controle.split_once('?')?.1;
    let mut valores = consulta
        .split('&')
        .filter_map(|par| par.strip_prefix("infra_unidade_atual="));
    let unidade = valores.next()?;
    let decimal = !unidade.is_empty() && unidade.bytes().all(|b| b.is_ascii_digit());
    (decimal && valores.next().is_none()).then(|| unidade.to_owned())
}

impl IndiceLocal {
    /// A URL assinada já vista nesta geração de sessão e unidade, literal.
    /// A linha que não passa mais na conferência da gravação não volta.
    pub(crate) fn link(
        &self,
        instalacao: &Instalacao,
        chave: ChaveDoLink<'_>,
    ) -> Option<UrlAssinada> {
        conferir_chave(chave).ok()?;
        let origem = instalacao.url_base();
        let url: Option<String> = self.executar(None, |banco, _| {
            banco
                .query_row(
                    "SELECT url FROM link_assinado WHERE origem = ?1 AND geracao_sessao = ?2 \
                     AND unidade = ?3 AND acao = ?4 AND objeto = ?5",
                    [
                        origem.as_str(),
                        chave.geracao_sessao,
                        chave.unidade,
                        chave.acao.como_na_url(),
                        chave.alvo,
                    ],
                    |l| l.get(0),
                )
                .optional()
        });
        let url = url?;
        conferir_url(instalacao, chave, &url).ok()?;
        Some(UrlAssinada::guardada(url))
    }

    /// Guarda a URL absoluta que a tela publicou, sem editá-la. A mesma
    /// chave observada de novo substitui a anterior.
    pub(crate) fn registrar_link(
        &self,
        instalacao: &Instalacao,
        chave: ChaveDoLink<'_>,
        url: &str,
    ) -> Result<(), RegistroRecusado> {
        conferir_chave(chave)?;
        conferir_url(instalacao, chave, url)?;
        let origem = instalacao.url_base();
        self.executar((), |banco, agora| {
            banco.execute(
                "INSERT INTO link_assinado \
                 (origem, geracao_sessao, unidade, acao, objeto, url, observado_em) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) \
                 ON CONFLICT (origem, geracao_sessao, unidade, acao, objeto) \
                 DO UPDATE SET url = excluded.url, observado_em = excluded.observado_em",
                rusqlite::params![
                    origem,
                    chave.geracao_sessao,
                    chave.unidade,
                    chave.acao.como_na_url(),
                    chave.alvo,
                    url,
                    agora
                ],
            )?;
            Ok(())
        });
        Ok(())
    }

    /// O link guardado não abriu a tela esperada: esquece-o.
    pub(crate) fn esquecer_link(&self, instalacao: &Instalacao, chave: ChaveDoLink<'_>) {
        let origem = instalacao.url_base();
        self.executar((), |banco, _| {
            banco.execute(
                "DELETE FROM link_assinado WHERE origem = ?1 AND geracao_sessao = ?2 \
                 AND unidade = ?3 AND acao = ?4 AND objeto = ?5",
                [
                    origem.as_str(),
                    chave.geracao_sessao,
                    chave.unidade,
                    chave.acao.como_na_url(),
                    chave.alvo,
                ],
            )?;
            Ok(())
        });
    }

    /// Apaga os links da instalação de toda geração diferente da atual
    /// (depois de um `ies login`).
    #[cfg(test)]
    pub(crate) fn descartar_outras_geracoes(&self, instalacao: &Instalacao, geracao_atual: &str) {
        let origem = instalacao.url_base();
        self.executar((), |banco, _| {
            banco.execute(
                "DELETE FROM link_assinado WHERE origem = ?1 AND geracao_sessao <> ?2",
                [&origem, geracao_atual],
            )?;
            Ok(())
        });
    }

    /// Apaga **todos** os links assinados da instalação, de toda geração e
    /// unidade (depois de `ies unidade trocar`, #205): nenhum link da
    /// unidade anterior é reaproveitado. Devolve quantos apagou; `None` com o
    /// índice desligado ou com falha do banco.
    pub(crate) fn esquecer_links_da_instalacao(&self, instalacao: &Instalacao) -> Option<usize> {
        let origem = instalacao.url_base();
        self.executar(None, |banco, _| {
            banco
                .execute("DELETE FROM link_assinado WHERE origem = ?1", [&origem])
                .map(Some)
        })
    }
}
