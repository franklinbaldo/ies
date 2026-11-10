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
            .map_err(CredenciaisIlegiveis::Arquivo)?;
        Self::interpretar(&texto)
    }

    /// `usuario` e `orgao` aparados; `senha` exatamente como escrita depois
    /// do primeiro `=`, só sem o fim de linha.
    pub(crate) fn interpretar(texto: &str) -> Result<Self, ErroLogin> {
        use CampoDasCredenciais as C;
        let mut valores: [Option<String>; 3] = [None, None, None];
        for linha in texto.lines().filter(|linha| !linha.is_empty()) {
            let (chave, valor) = linha
                .split_once('=')
                .ok_or(ErroLogin::LinhaDasCredenciaisSemCampoConhecido)?;
            let (campo, indice) = match chave {
                "usuario" => (C::Usuario, 0),
                "senha" => (C::Senha, 1),
                "orgao" => (C::Orgao, 2),
                _ => return Err(ErroLogin::LinhaDasCredenciaisSemCampoConhecido),
            };
            if valores[indice].is_some() {
                return Err(ErroLogin::CampoDasCredenciaisRepetido(campo));
            }
            let valor = if campo == C::Senha {
                valor
            } else {
                valor.trim()
            };
            if valor.is_empty() {
                return Err(ErroLogin::CampoDasCredenciaisVazio(campo));
            }
            valores[indice] = Some(valor.to_owned());
        }
        let [usuario, senha, orgao] = valores;
        Ok(Self {
            usuario: usuario.ok_or(ErroLogin::CampoDasCredenciaisAusente(C::Usuario))?,
            senha: senha.ok_or(ErroLogin::CampoDasCredenciaisAusente(C::Senha))?,
            orgao: orgao.ok_or(ErroLogin::CampoDasCredenciaisAusente(C::Orgao))?,
        })
    }
}

/// O arquivo privado de credenciais ausente, inseguro ou ilegível. Tipo próprio da etapa; o nome
/// público é o motivo do transporte (`transparente`).
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum CredenciaisIlegiveis {
    Arquivo(#[from] crate::transporte::FalhaTransporte),
}
