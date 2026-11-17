//! `ies wizard [--perfil NOME]` (contracts/cli/wizard.md): cria um perfil
//! local em `~/.ies/<perfil>`, sem rede. Grava a origem, a sessão vazia,
//! as credenciais **sem senha** e a configuração do `ies setup`, todos
//! `0600` e em criação exclusiva; a senha vai só para o cofre do sistema
//! operacional ([`crate::cofre`]). Nada é sobrescrito: perfil existente é
//! recusa antes de qualquer gravação.
use crate::arquivo_privado::{self, FalhaDaGravacao, Politica};
use crate::cofre::{self, ChaveDoCofre, ErroCofre};
use crate::configuracao::{ArquivosDeAcesso, ErroGravacaoDaConfiguracao};
use crate::credenciais_cli::{ErroCredenciais, ler_senha_sem_eco};
use crate::instalacao::Instalacao;
use clap::Args;
use serde_json::json;
use std::io::{self, BufRead, Write};
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Args)]
pub(crate) struct WizardArgs {
    /// Nome do perfil: uma pasta em ~/.ies (padrão: o da instalação)
    #[arg(long, value_parser = nome_do_perfil)]
    perfil: Option<String>,
}

/// Instalação pública de treinamento, com a conta que ela mesma publica
/// (docs/interfaces/credenciais-de-treinamento.md).
struct Predefinicao {
    perfil: &'static str,
    nome: &'static str,
    origem: &'static str,
    usuario: &'static str,
    senha: &'static str,
    orgao: &'static str,
}

const PREDEFINICOES: [Predefinicao; 4] = [
    Predefinicao {
        perfil: "unb",
        nome: "UnB 4.0.9 (treinamento)",
        origem: "https://sei.treinamento.unb.br/sei",
        usuario: "aluno1",
        senha: "aluno1",
        orgao: "UnB",
    },
    Predefinicao {
        perfil: "pen1",
        nome: "PEN1 5.0.4 / Tramita (homologação)",
        origem: "https://sei.orgao1.tramita.processoeletronico.gov.br/sei",
        usuario: "orgao1",
        senha: "orgao1",
        orgao: "ORGAO1",
    },
    Predefinicao {
        perfil: "go",
        nome: "Goiás 5.0.3 (treinamento)",
        origem: "https://seitreinamento.go.gov.br/sei",
        usuario: "aluno05",
        senha: "abc123456",
        orgao: "SEAD",
    },
    Predefinicao {
        perfil: "ufrj",
        nome: "UFRJ (testes)",
        origem: "https://teste.sei.tic.ufrj.br/sei",
        usuario: "servidor1",
        senha: "servidor1",
        orgao: "UFRJ",
    },
];

/// Por que o assistente parou. Nenhuma variante carrega a senha.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroWizard {
    /// Sem pasta pessoal absoluta (`HOME`; `USERPROFILE` no Windows).
    PastaPessoalDesconhecida,
    /// A entrada padrão acabou antes da resposta.
    EntradaEncerrada,
    EntradaIlegivel(#[source] io::Error),
    /// A opção não é um número da lista.
    OpcaoInvalida,
    /// A URL não é a base HTTPS de uma instalação (como no arquivo de origem).
    OrigemNaoReconhecida,
    UsuarioVazio,
    OrgaoVazio,
    /// O perfil não é um único nome de pasta (sem `/`, `\`, `.`, `..` ou
    /// caminho absoluto).
    PerfilInvalido,
    /// `~/.ies/<perfil>` já existe: o assistente não sobrescreve.
    PerfilJaExiste,
    /// A pasta do perfil não pôde ser consultada (o motivo do sistema).
    PerfilNaoConsultado(io::ErrorKind),
    Senha(#[from] ErroCredenciais),
    Cofre(#[from] ErroCofre),
    Gravacao(#[from] FalhaDaGravacao),
    Configuracao(#[from] ErroGravacaoDaConfiguracao),
}

/// O que o assistente reuniu antes de gravar.
struct Respostas {
    instalacao: Instalacao,
    usuario: String,
    orgao: String,
    senha: String,
    perfil: String,
}

/// O perfil gravado (caminhos e serviço; nenhuma credencial).
#[derive(Debug)]
struct PerfilGravado {
    perfil: String,
    servico: String,
    config: PathBuf,
    credenciais: PathBuf,
}

pub(crate) fn executar(args: &WizardArgs, json_mode: bool) -> anyhow::Result<()> {
    let pasta_pessoal = std::env::home_dir()
        .filter(|pasta| pasta.is_absolute())
        .ok_or(ErroWizard::PastaPessoalDesconhecida)?;
    // Perguntas em stderr: stdout fica só com o resultado.
    let gravado = configurar(
        &mut io::stdin().lock(),
        &mut io::stderr(),
        || ler_senha_sem_eco("Senha (não aparece ao digitar): "),
        &pasta_pessoal,
        args.perfil.as_deref(),
    )?;
    let config = gravado.config.display().to_string();
    let credenciais = gravado.credenciais.display().to_string();
    if json_mode {
        println!(
            "{}",
            json!({
                "perfil": gravado.perfil,
                "cofre": gravado.servico,
                "config_file": config,
                "credenciais_file": credenciais,
                "conexao": "nao_verificada",
            })
        );
    } else {
        println!(
            "IES: perfil '{}' criado; senha no cofre do sistema ({}); conexão ainda não verificada.\n\
             Para usar o perfil, defina no terminal:\n  \
             IES_CONFIG_FILE={config}\n  IES_CREDENCIAIS_FILE={credenciais}\n\
             e rode `ies login`.",
            gravado.perfil, gravado.servico
        );
    }
    Ok(())
}

/// Pergunta, confere o perfil e só então grava: cofre primeiro (sem cofre,
/// nada vai ao disco), depois os arquivos do perfil.
fn configurar(
    entrada: &mut impl BufRead,
    saida: &mut impl Write,
    ler_senha: impl FnOnce() -> Result<String, ErroCredenciais>,
    pasta_pessoal: &Path,
    perfil: Option<&str>,
) -> Result<PerfilGravado, ErroWizard> {
    let respostas = perguntar(entrada, saida, ler_senha, perfil)?;
    let pasta = pasta_pessoal.join(".ies").join(&respostas.perfil);
    match std::fs::symlink_metadata(&pasta) {
        Ok(_) => return Err(ErroWizard::PerfilJaExiste),
        Err(erro) if erro.kind() == io::ErrorKind::NotFound => {}
        Err(erro) => return Err(ErroWizard::PerfilNaoConsultado(erro.kind())),
    }
    let chave = ChaveDoCofre::nova(&respostas.instalacao, &respostas.usuario);
    cofre::salvar(&chave, &respostas.senha)?;
    let arquivos = ArquivosDeAcesso {
        url_controle: pasta.join("url-controle"),
        cookie: pasta.join("cookie"),
        origem: pasta.join("origem"),
    };
    let credenciais = pasta.join("credenciais");
    let config = pasta.join("ies-config.json");
    let novo = Politica::exclusiva()
        .diretorio_do_ies()
        .criando_diretorios();
    let origem = format!("{}\n", respostas.instalacao.url_base());
    arquivo_privado::gravar(&arquivos.origem, origem.as_bytes(), novo)?;
    // Sessão vazia: o primeiro `ies login` a grava.
    arquivo_privado::gravar(&arquivos.cookie, b"", novo)?;
    arquivo_privado::gravar(&arquivos.url_controle, b"", novo)?;
    let linhas = format!("usuario={}\norgao={}\n", respostas.usuario, respostas.orgao);
    arquivo_privado::gravar(&credenciais, linhas.as_bytes(), novo)?;
    crate::configuracao::salvar_arquivos_de_acesso(&config, &arquivos)?;
    Ok(PerfilGravado {
        perfil: respostas.perfil,
        servico: chave.servico().to_owned(),
        config,
        credenciais,
    })
}

fn perguntar(
    entrada: &mut impl BufRead,
    saida: &mut impl Write,
    ler_senha: impl FnOnce() -> Result<String, ErroCredenciais>,
    perfil: Option<&str>,
) -> Result<Respostas, ErroWizard> {
    let outra = PREDEFINICOES.len() + 1;
    let opcoes: Vec<String> = (1..)
        .zip(&PREDEFINICOES)
        .map(|(numero, predefinicao)| format!("  {numero}) {}\n", predefinicao.nome))
        .collect();
    let opcoes = opcoes.concat();
    let lista = format!(
        "IES — novo perfil local (sem rede)\nInstalação:\n{opcoes}  \
         {outra}) Outra instalação (URL, órgão, usuário e senha)\n"
    );
    let escolha = responder(entrada, saida, &format!("{lista}Opção [1-{outra}]: "))?;
    let indice = escolha
        .parse::<usize>()
        .ok()
        .and_then(|numero| numero.checked_sub(1))
        .ok_or(ErroWizard::OpcaoInvalida)?;
    if let Some(p) = PREDEFINICOES.get(indice) {
        return Ok(Respostas {
            instalacao: Instalacao::interpretar(p.origem)
                .ok_or(ErroWizard::OrigemNaoReconhecida)?,
            usuario: p.usuario.to_owned(),
            orgao: p.orgao.to_owned(),
            senha: p.senha.to_owned(),
            perfil: perfil.unwrap_or(p.perfil).to_owned(),
        });
    }
    if indice != PREDEFINICOES.len() {
        return Err(ErroWizard::OpcaoInvalida);
    }
    let url = responder(
        entrada,
        saida,
        "URL base da instalação (ex.: https://sei.exemplo.gov.br/sei): ",
    )?;
    let instalacao = Instalacao::interpretar(&url).ok_or(ErroWizard::OrigemNaoReconhecida)?;
    let orgao = responder(entrada, saida, "Órgão (a sigla da tela de login): ")?;
    if orgao.is_empty() {
        return Err(ErroWizard::OrgaoVazio);
    }
    let usuario = responder(entrada, saida, "Usuário: ")?;
    if usuario.is_empty() {
        return Err(ErroWizard::UsuarioVazio);
    }
    let perfil = if let Some(perfil) = perfil {
        perfil.to_owned()
    } else {
        let padrao = instalacao.host().to_owned();
        let nome = responder(entrada, saida, &format!("Nome do perfil [{padrao}]: "))?;
        if nome.is_empty() {
            padrao
        } else {
            nome_do_perfil(&nome).map_err(|_| ErroWizard::PerfilInvalido)?
        }
    };
    let senha = ler_senha()?;
    Ok(Respostas {
        instalacao,
        usuario,
        orgao,
        senha,
        perfil,
    })
}

/// Uma linha da entrada, aparada; fim da entrada é erro.
fn responder(
    entrada: &mut impl BufRead,
    saida: &mut impl Write,
    pergunta: &str,
) -> Result<String, ErroWizard> {
    // A pergunta é cortesia: falha ao escrevê-la não muda a resposta.
    let _ = write!(saida, "{pergunta}").and_then(|()| saida.flush());
    let mut linha = String::new();
    match entrada.read_line(&mut linha) {
        Ok(0) => Err(ErroWizard::EntradaEncerrada),
        Ok(_) => Ok(linha.trim().to_owned()),
        Err(erro) => Err(ErroWizard::EntradaIlegivel(erro)),
    }
}

/// Um único nome de pasta: sem separador, `.`, `..`, raiz ou prefixo.
fn nome_do_perfil(nome: &str) -> Result<String, String> {
    let mut partes = Path::new(nome).components();
    match (partes.next(), partes.next()) {
        (Some(Component::Normal(parte)), None)
            if parte == nome && !nome.contains(['/', '\\']) && !nome.starts_with('.') =>
        {
            Ok(nome.to_owned())
        }
        _ => Err("o perfil é um único nome de pasta, sem '/', '\\', '.' ou '..'".into()),
    }
}

#[cfg(test)]
#[path = "wizard_cli/tests.rs"]
mod tests;
