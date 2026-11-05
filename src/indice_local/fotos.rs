//! Tabela `foto` (#207): o SHA-256 da foto `DOC.vN.html` que
//! `ies documento DOC --saida` gravou (ou achou igual), por instalação,
//! número SEI e versão. A prévia de `documento salvar` confere a base
//! local por ele; sem registro, a base não é aceita. A observação nova da
//! mesma versão substitui a antiga.
use super::IndiceLocal;
use super::validacao::{RegistroRecusado, conferir_numero_sei};
use crate::documento_versoes::VersaoDocumento;
use crate::instalacao::Instalacao;
use rusqlite::OptionalExtension;

fn sha_valido(sha: &str) -> bool {
    sha.len() == 64 && sha.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

impl IndiceLocal {
    pub(crate) fn registrar_foto(
        &self,
        instalacao: &Instalacao,
        numero: &str,
        versao: VersaoDocumento,
        sha256: &str,
    ) -> Result<(), RegistroRecusado> {
        conferir_numero_sei(numero)?;
        if !sha_valido(sha256) {
            return Err(RegistroRecusado::Sha256Invalido);
        }
        let origem = instalacao.url_base();
        self.executar((), |banco, agora| {
            banco.execute(
                "INSERT OR REPLACE INTO foto \
                 (origem, numero_sei, versao, sha256, observado_em) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![origem, numero, versao.numero(), sha256, agora],
            )?;
            Ok(())
        });
        Ok(())
    }

    pub(crate) fn foto(
        &self,
        instalacao: &Instalacao,
        numero: &str,
        versao: VersaoDocumento,
    ) -> Option<String> {
        conferir_numero_sei(numero).ok()?;
        let origem = instalacao.url_base();
        let sha: Option<String> = self.executar(None, |banco, _| {
            banco
                .query_row(
                    "SELECT sha256 FROM foto \
                     WHERE origem = ?1 AND numero_sei = ?2 AND versao = ?3",
                    rusqlite::params![origem, numero, versao.numero()],
                    |l| l.get(0),
                )
                .optional()
        });
        sha.filter(|s| sha_valido(s))
    }
}
