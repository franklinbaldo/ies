//! A senha do login no cofre do sistema operacional (Keychain no macOS,
//! Gerenciador de Credenciais no Windows, Secret Service no Linux): uma
//! entrada por instalação e usuário, serviço `ies:<host>` e conta igual ao
//! usuário. A senha entra e sai byte a byte como digitada; nenhum erro,
//! `Debug` ou saída a carrega. Contrato: `contracts/cli/credenciais.md`.
use crate::instalacao::Instalacao;

/// A entrada do cofre de um usuário numa instalação.
pub(crate) struct ChaveDoCofre {
    servico: String,
    usuario: String,
}

impl ChaveDoCofre {
    pub(crate) fn nova(instalacao: &Instalacao, usuario: &str) -> Self {
        Self {
            servico: format!("ies:{}", instalacao.host()),
            usuario: usuario.to_owned(),
        }
    }

    /// `ies:<host>`, o nome do serviço publicado na saída.
    pub(crate) fn servico(&self) -> &str {
        &self.servico
    }
}

/// Falha do cofre, classificada pela variante do keyring. A mensagem é a
/// do sistema (o `Display` do keyring); o erro original não é guardado
/// porque o `Debug` de `BadEncoding` traria os bytes da senha.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroCofre {
    /// Não há cofre utilizável: sem Secret Service ou sessão D-Bus
    /// (servidor, CI, contêiner) ou plataforma sem suporte.
    CofreIndisponivel { mensagem_do_sistema: String },
    /// O cofre existe e negou o acesso (bloqueado, pedido recusado).
    AcessoNegado { mensagem_do_sistema: String },
    /// Outra falha do cofre na operação pedida.
    FalhaDoCofre { mensagem_do_sistema: String },
}

impl From<keyring_core::Error> for ErroCofre {
    fn from(erro: keyring_core::Error) -> Self {
        use keyring_core::Error as E;
        // O `Display` de `Ambiguous` lista as entradas pelo `Debug` delas.
        let mensagem_do_sistema = if matches!(erro, E::Ambiguous(_)) {
            "mais de uma credencial para a entrada".to_owned()
        } else {
            erro.to_string()
        };
        if matches!(erro, E::NoDefaultStore | E::NotSupportedByStore(_)) {
            Self::CofreIndisponivel {
                mensagem_do_sistema,
            }
        } else if matches!(erro, E::NoStorageAccess(_)) {
            Self::AcessoNegado {
                mensagem_do_sistema,
            }
        } else {
            Self::FalhaDoCofre {
                mensagem_do_sistema,
            }
        }
    }
}

/// Grava (ou substitui) a senha exatamente como recebida.
pub(crate) fn salvar(chave: &ChaveDoCofre, senha: &str) -> Result<(), ErroCofre> {
    Ok(entrada(chave)?.set_password(senha)?)
}

/// A senha guardada; `None` só quando o cofre diz que a entrada não existe.
pub(crate) fn obter(chave: &ChaveDoCofre) -> Result<Option<String>, ErroCofre> {
    match entrada(chave)?.get_password() {
        Ok(senha) => Ok(Some(senha)),
        Err(keyring_core::Error::NoEntry) => Ok(None),
        Err(erro) => Err(erro.into()),
    }
}

/// Apaga a entrada; `false` quando ela já não existia.
pub(crate) fn excluir(chave: &ChaveDoCofre) -> Result<bool, ErroCofre> {
    match entrada(chave)?.delete_credential() {
        Ok(()) => Ok(true),
        Err(keyring_core::Error::NoEntry) => Ok(false),
        Err(erro) => Err(erro.into()),
    }
}

fn entrada(chave: &ChaveDoCofre) -> Result<keyring_core::Entry, ErroCofre> {
    #[cfg(not(test))]
    cofre_do_sistema()?;
    #[cfg(test)]
    cofre_falso();
    Ok(keyring_core::Entry::new(&chave.servico, &chave.usuario)?)
}

/// O cofre da plataforma, inicializado uma vez pelo `keyring` como o
/// padrão do `keyring_core`.
#[cfg(not(test))]
fn cofre_do_sistema() -> Result<(), ErroCofre> {
    match keyring::Entry::store_status() {
        Ok(()) => Ok(()),
        Err(erro) => Err(ErroCofre::CofreIndisponivel {
            mensagem_do_sistema: erro.to_string(),
        }),
    }
}

/// Nos testes, nunca o cofre da pessoa: o cofre em memória do
/// `keyring_core`, o mesmo para o processo inteiro.
#[cfg(test)]
fn cofre_falso() {
    static FALSO: std::sync::Once = std::sync::Once::new();
    FALSO.call_once(|| {
        keyring_core::set_default_store(
            keyring_core::mock::Store::new().expect("cofre em memória"),
        );
    });
}

/// Faz a próxima operação na entrada falhar com `erro` (cofre de teste).
#[cfg(test)]
pub(crate) fn falhar_na_proxima(chave: &ChaveDoCofre, erro: keyring_core::Error) {
    let entrada = entrada(chave).unwrap();
    let falsa: &keyring_core::mock::Cred = entrada.as_any().downcast_ref().unwrap();
    falsa.set_error(erro);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chave(host: &str, usuario: &str) -> ChaveDoCofre {
        let instalacao = Instalacao::interpretar(&format!("https://{host}/sei")).unwrap();
        ChaveDoCofre::nova(&instalacao, usuario)
    }

    #[test]
    fn servico_e_o_host_da_instalacao() {
        let instalacao = Instalacao::interpretar("https://SEI.Treinamento.UnB.br/sei").unwrap();
        let chave = ChaveDoCofre::nova(&instalacao, "aluno1");
        assert_eq!(chave.servico(), "ies:sei.treinamento.unb.br");
    }

    #[test]
    fn salvar_obter_excluir_preserva_os_bytes_e_deixa_ausente() {
        let chave = chave("cofre-ida-volta.test", "u");
        assert_eq!(obter(&chave), Ok(None));
        salvar(&chave, " senha falsa com espaços ").unwrap();
        assert_eq!(
            obter(&chave).unwrap().as_deref(),
            Some(" senha falsa com espaços ")
        );
        assert_eq!(excluir(&chave), Ok(true));
        assert_eq!(obter(&chave), Ok(None));
        assert_eq!(excluir(&chave), Ok(false));
    }

    #[test]
    fn falha_do_cofre_nao_vira_ausencia_e_nao_mostra_a_senha() {
        let chave = chave("cofre-negado.test", "u");
        salvar(&chave, "senha-falsa-123").unwrap();
        falhar_na_proxima(
            &chave,
            keyring_core::Error::NoStorageAccess("bloqueado".into()),
        );
        assert!(matches!(obter(&chave), Err(ErroCofre::AcessoNegado { .. })));
        falhar_na_proxima(
            &chave,
            keyring_core::Error::BadEncoding(b"senha-falsa-123".to_vec()),
        );
        let erro = obter(&chave).unwrap_err();
        assert!(matches!(erro, ErroCofre::FalhaDoCofre { .. }));
        assert!(!format!("{erro:?}{erro}").contains("senha-falsa"));
        falhar_na_proxima(
            &chave,
            keyring_core::Error::PlatformFailure("fora do ar".into()),
        );
        assert!(matches!(
            excluir(&chave),
            Err(ErroCofre::FalhaDoCofre { .. })
        ));
        assert_eq!(obter(&chave).unwrap().as_deref(), Some("senha-falsa-123"));
    }

    #[test]
    fn cofre_ausente_e_indisponivel() {
        assert!(matches!(
            ErroCofre::from(keyring_core::Error::NoDefaultStore),
            ErroCofre::CofreIndisponivel { .. }
        ));
    }
}
