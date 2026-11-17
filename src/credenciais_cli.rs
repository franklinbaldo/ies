//! `ies credenciais salvar|status|excluir --origem URL --usuario LOGIN`
//! (contracts/cli/credenciais.md): a senha do login no cofre do sistema
//! operacional, sem rede. A senha só entra pelo terminal, sem eco; a saída
//! traz o serviço `ies:<host>` e o estado, nunca o usuário ou a senha.
use crate::cofre::{self, ChaveDoCofre, ErroCofre};
use crate::instalacao::Instalacao;
use clap::{Args, Subcommand};
use serde_json::json;

#[derive(Debug, Args)]
pub(crate) struct CredenciaisArgs {
    #[command(subcommand)]
    comando: ComandoCredenciais,
}

/// A entrada do cofre: instalação e usuário.
#[derive(Debug, Args)]
struct Entrada {
    /// URL base HTTPS da instalação, como no arquivo de origem
    /// (ex.: <https://sei.exemplo.gov.br/sei>)
    #[arg(long, value_parser = origem_da_instalacao)]
    origem: Instalacao,
    /// Login do usuário no SEI
    #[arg(long, value_parser = usuario_nao_vazio)]
    usuario: String,
}

#[derive(Debug, Subcommand)]
enum ComandoCredenciais {
    /// Pede a senha sem eco e a grava (ou substitui) no cofre
    Salvar(Entrada),
    /// Diz se há senha no cofre para a instalação e o usuário
    Status(Entrada),
    /// Apaga a senha do cofre e confere, relendo, que ela não está mais lá
    Excluir(Entrada),
}

fn origem_da_instalacao(valor: &str) -> Result<Instalacao, String> {
    Instalacao::interpretar(valor)
        .ok_or_else(|| "esperada a URL base HTTPS da instalação, sem consulta nem fragmento".into())
}

fn usuario_nao_vazio(valor: &str) -> Result<String, String> {
    let usuario = valor.trim();
    if usuario.is_empty() {
        return Err("o usuário não pode ser vazio".into());
    }
    Ok(usuario.to_owned())
}

/// Por que `ies credenciais` falhou. Nenhuma variante carrega a senha.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroCredenciais {
    /// O terminal não entregou a senha sem eco (sem terminal interativo).
    SenhaNaoLida(#[source] std::io::Error),
    /// A senha digitada é vazia ou só espaços.
    SenhaVazia,
    Cofre(#[from] ErroCofre),
    /// O cofre aceitou a exclusão, mas a releitura ainda achou a senha.
    SenhaPresenteAposExcluir,
}

/// Pede a senha no terminal, sem eco. Os bytes digitados são mantidos;
/// espaços só contam para recusar a senha vazia.
pub(crate) fn ler_senha_sem_eco(pergunta: &str) -> Result<String, ErroCredenciais> {
    let senha = rpassword::prompt_password(pergunta).map_err(ErroCredenciais::SenhaNaoLida)?;
    if senha.trim().is_empty() {
        return Err(ErroCredenciais::SenhaVazia);
    }
    Ok(senha)
}

pub(crate) fn executar(args: &CredenciaisArgs, json_mode: bool) -> anyhow::Result<()> {
    let (estado, chave) = match &args.comando {
        ComandoCredenciais::Salvar(entrada) => {
            let chave = ChaveDoCofre::nova(&entrada.origem, &entrada.usuario);
            let senha = ler_senha_sem_eco("Senha (não aparece ao digitar): ")?;
            cofre::salvar(&chave, &senha)?;
            ("gravada", chave)
        }
        ComandoCredenciais::Status(entrada) => {
            let chave = ChaveDoCofre::nova(&entrada.origem, &entrada.usuario);
            let estado = if cofre::obter(&chave)?.is_some() {
                "presente"
            } else {
                "ausente"
            };
            (estado, chave)
        }
        ComandoCredenciais::Excluir(entrada) => {
            let chave = ChaveDoCofre::nova(&entrada.origem, &entrada.usuario);
            (excluir_e_conferir(&chave)?, chave)
        }
    };
    if json_mode {
        println!("{}", json!({"cofre": chave.servico(), "senha": estado}));
    } else {
        println!(
            "IES: senha {estado} no cofre do sistema ({}).",
            chave.servico()
        );
    }
    Ok(())
}

/// `excluida` ou, se não havia entrada, `ausente`; sucesso só quando a
/// releitura não acha mais a senha.
fn excluir_e_conferir(chave: &ChaveDoCofre) -> Result<&'static str, ErroCredenciais> {
    let havia = cofre::excluir(chave)?;
    if cofre::obter(chave)?.is_some() {
        return Err(ErroCredenciais::SenhaPresenteAposExcluir);
    }
    Ok(if havia { "excluida" } else { "ausente" })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chave(host: &str) -> ChaveDoCofre {
        ChaveDoCofre::nova(
            &origem_da_instalacao(&format!("https://{host}/sei")).unwrap(),
            "u",
        )
    }

    #[test]
    fn excluir_confere_pela_releitura() {
        let chave = chave("credenciais-excluir.test");
        assert_eq!(excluir_e_conferir(&chave).unwrap(), "ausente");
        cofre::salvar(&chave, "senha-falsa").unwrap();
        assert_eq!(excluir_e_conferir(&chave).unwrap(), "excluida");
        assert_eq!(cofre::obter(&chave), Ok(None));
    }

    #[test]
    fn falha_ao_excluir_nao_e_sucesso_e_a_senha_continua() {
        let chave = chave("credenciais-releitura.test");
        cofre::salvar(&chave, "senha-falsa").unwrap();
        cofre::falhar_na_proxima(&chave, keyring_core::Error::PlatformFailure("x".into()));
        assert!(matches!(
            excluir_e_conferir(&chave),
            Err(ErroCredenciais::Cofre(ErroCofre::FalhaDoCofre { .. }))
        ));
        assert_eq!(
            cofre::obter(&chave).unwrap().as_deref(),
            Some("senha-falsa")
        );
    }

    #[test]
    fn origem_e_usuario_sao_validados_pelo_clap() {
        assert!(origem_da_instalacao("http://sei.exemplo.gov.br/sei").is_err());
        assert!(origem_da_instalacao("https://sei.exemplo.gov.br/sei?x=1").is_err());
        assert!(usuario_nao_vazio("  ").is_err());
        assert_eq!(usuario_nao_vazio(" aluno1 ").unwrap(), "aluno1");
    }
}
