//! Credenciais do login, lidas só de arquivo privado `0600` e só quando o
//! login é necessário. Formato: `usuario=`, `senha=` e `orgao=`, uma vez
//! cada, uma por linha; linhas vazias são ignoradas. Sem a linha `senha=`,
//! a senha vem do cofre do sistema operacional ([`crate::cofre`]).
use super::{CampoDasCredenciais, ErroLogin};
use crate::cofre::{self, ChaveDoCofre, ErroCofre};
use crate::instalacao::Instalacao;
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
    /// Sem `senha=`, a do cofre para o usuário nesta instalação.
    pub(crate) fn ler(caminho: &Path, instalacao: &Instalacao) -> Result<Self, ErroLogin> {
        let texto = crate::arquivo_privado::ler(caminho, LIMITE_DO_ARQUIVO)
            .map_err(ErroLogin::ArquivoDeCredenciais)?;
        Self::interpretar_com_cofre(&texto, |usuario| {
            cofre::obter(&ChaveDoCofre::nova(instalacao, usuario))
        })
    }

    /// Só o arquivo: sem `senha=`, a senha é ausente.
    #[cfg(test)]
    pub(crate) fn interpretar(texto: &str) -> Result<Self, ErroLogin> {
        Self::interpretar_com_cofre(texto, |_| Ok(None))
    }

    /// `usuario` e `orgao` aparados; `senha` exatamente como escrita depois
    /// do primeiro `=`, só sem o fim de linha. Sem a linha `senha=`, o
    /// cofre é consultado (uma vez, pelo usuário lido): entrada ausente é
    /// `CampoDasCredenciaisAusente(Senha)`, falha do cofre é `Cofre`.
    pub(crate) fn interpretar_com_cofre(
        texto: &str,
        senha_do_cofre: impl FnOnce(&str) -> Result<Option<String>, ErroCofre>,
    ) -> Result<Self, ErroLogin> {
        let mut valores: [Option<String>; 3] = [None, None, None];
        for linha in texto.lines().filter(|linha| !linha.is_empty()) {
            let (chave, valor) = linha
                .split_once('=')
                .ok_or(ErroLogin::LinhaDasCredenciaisSemCampoConhecido)?;
            let [usuario, senha, orgao] = &mut valores;
            let (campo, guardado) = match chave {
                "usuario" => (CampoDasCredenciais::Usuario, usuario),
                "senha" => (CampoDasCredenciais::Senha, senha),
                "orgao" => (CampoDasCredenciais::Orgao, orgao),
                _ => return Err(ErroLogin::LinhaDasCredenciaisSemCampoConhecido),
            };
            if guardado.is_some() {
                return Err(ErroLogin::CampoDasCredenciaisRepetido(campo));
            }
            let valor = if campo == CampoDasCredenciais::Senha {
                valor
            } else {
                valor.trim()
            };
            if valor.is_empty() {
                return Err(ErroLogin::CampoDasCredenciaisVazio(campo));
            }
            *guardado = Some(valor.to_owned());
        }
        let [usuario, senha, orgao] = valores;
        let usuario = usuario.ok_or(ErroLogin::CampoDasCredenciaisAusente(
            CampoDasCredenciais::Usuario,
        ))?;
        let orgao = orgao.ok_or(ErroLogin::CampoDasCredenciaisAusente(
            CampoDasCredenciais::Orgao,
        ))?;
        let senha = match senha {
            Some(senha) => senha,
            None => match senha_do_cofre(&usuario)? {
                Some(senha) if !senha.is_empty() => senha,
                Some(_) => {
                    return Err(ErroLogin::CampoDasCredenciaisVazio(
                        CampoDasCredenciais::Senha,
                    ));
                }
                None => {
                    return Err(ErroLogin::CampoDasCredenciaisAusente(
                        CampoDasCredenciais::Senha,
                    ));
                }
            },
        };
        Ok(Self {
            usuario,
            senha,
            orgao,
        })
    }
}
