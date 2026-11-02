//! Credenciais do login, lidas só de arquivo privado `0600` e só quando o
//! login é necessário. Formato: `usuario=`, `senha=` e `orgao=`, uma vez
//! cada, uma por linha; linhas vazias são ignoradas.
use super::{CampoDasCredenciais, ErroLogin};
use std::fmt;
use std::path::Path;

/// Limite do arquivo de credenciais.
pub(crate) const LIMITE_DO_ARQUIVO: u64 = 4096;

pub(crate) struct Credenciais {
    usuario: String,
    senha: String,
    orgao: String,
}

/// Nenhum valor aparece no `Debug`.
impl fmt::Debug for Credenciais {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Credenciais")
            .field("usuario", &"<redigido>")
            .field("senha", &"<redigido>")
            .field("orgao", &"<redigido>")
            .finish()
    }
}

impl Credenciais {
    pub(crate) fn usuario(&self) -> &str {
        &self.usuario
    }
    pub(crate) fn senha(&self) -> &str {
        &self.senha
    }
    pub(crate) fn orgao(&self) -> &str {
        &self.orgao
    }

    /// Arquivo privado pelas travas do transporte (`0600`, sem link, limite).
    pub(crate) fn ler(caminho: &Path) -> Result<Self, ErroLogin> {
        let texto = crate::transporte::arquivo_privado(caminho, LIMITE_DO_ARQUIVO)
            .map_err(ErroLogin::ArquivoDeCredenciais)?;
        Self::interpretar(&texto)
    }

    /// `usuario` e `orgao` aparados; `senha` exatamente como escrita depois
    /// do primeiro `=`, só sem o fim de linha.
    pub(crate) fn interpretar(texto: &str) -> Result<Self, ErroLogin> {
        let _ = (texto, CampoDasCredenciais::Usuario);
        todo!("CONTRACT: interpretar credenciais")
    }
}
