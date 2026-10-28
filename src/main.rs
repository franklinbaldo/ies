mod caixa;
mod cassette;
mod dominio;
mod erro_web;
mod historico;
mod instalacao;
mod paginas;
#[allow(dead_code)] // Tipos da próxima operação, ainda sem adaptador de leitura.
mod processo;
mod saida;
mod transporte;

use clap::{Args, CommandFactory, Parser, Subcommand};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub(crate) const MAX_HTML_BYTES: usize = 8_000_000;

#[derive(Debug, Parser)]
#[command(name = "ies", version, about = "CLI Rust de domínio para o SEI")]
struct Cli {
    #[arg(long, global = true, help = "Devolve modelos em JSON")]
    json: bool,
    #[arg(long, global = true, help = "Unidade de trabalho do SEI")]
    unidade: Option<String>,
    #[command(subcommand)]
    command: Domain,
}

#[derive(Debug, Subcommand)]
enum Domain {
    /// Mostra um processo pelo NUP ou executa uma operação nele
    Processo(Processo),
    /// Mostra documentos pelo número SEI ou executa uma operação
    Documento(Documento),
    /// Consulta coleções da unidade
    Unidade(Unidade),
    /// Reproduz evidências históricas sem acesso ao SEI
    Replay(Replay),
    /// Ferramentas de sessão Web autorizada
    Sessao(Sessao),
    /// Diagnóstico da conexão (ainda não implementado)
    Status,
    /// Configura a conexão (ainda não implementado)
    Setup,
    /// Mostra o contrato de saída (ainda não implementado)
    Esquema,
}

#[derive(Debug, Args)]
struct Processo {
    numero: Option<String>,
    /// Omite documentos da leitura
    #[arg(long)]
    sem_documentos: bool,
    /// Omite marcadores da leitura
    #[arg(long)]
    sem_marcadores: bool,
    /// Omite andamentos da leitura
    #[arg(long)]
    sem_andamentos: bool,
    /// Omite propriedades adicionais da leitura
    #[arg(long)]
    sem_propriedades: bool,
    /// Solicita histórico completo do processo
    #[arg(long)]
    andamentos_completos: bool,
    #[command(subcommand)]
    acao: Option<ProcessoAcao>,
}

#[derive(Debug, Subcommand)]
enum ProcessoAcao {
    Buscar {
        #[arg(long)]
        assunto: Option<String>,
    },
    Enviar {
        numero: String,
        #[arg(long, required = true)]
        para: Vec<String>,
        #[arg(long)]
        confirmar: bool,
    },
    Concluir {
        numero: String,
        #[arg(long)]
        confirmar: bool,
    },
}

#[derive(Debug, Args)]
struct Documento {
    /// Números dos documentos SEI
    #[arg(num_args = 0..)]
    numeros: Vec<String>,
    /// Contexto de processo, quando conhecido
    #[arg(long)]
    processo: Option<String>,
    /// Diretório onde os conteúdos serão salvos
    #[arg(long)]
    saida: Option<String>,
    /// Oculta documentos não acessíveis
    #[arg(long)]
    ocultar_inacessiveis: bool,
    /// Inclui propriedades do cadastro
    #[arg(long)]
    com_propriedades: bool,
    #[command(subcommand)]
    acao: Option<DocumentoAcao>,
}

#[derive(Debug, Subcommand)]
enum DocumentoAcao {
    Buscar {
        #[arg(long)]
        assunto: Option<String>,
    },
    Assinar {
        numero: String,
        #[arg(long)]
        confirmar: bool,
    },
}

#[derive(Debug, Args)]
struct Unidade {
    #[command(subcommand)]
    acao: Option<UnidadeAcao>,
}

#[derive(Debug, Subcommand)]
enum UnidadeAcao {
    ListarProcessos {
        #[arg(long)]
        saida: Option<PathBuf>,
    },
}

#[derive(Debug, Args)]
struct Replay {
    /// Nome do cenário OKF, sem caminho, ex.: processo-andamento-go
    cenario: String,
    /// Raiz local do acervo contendo cassettes/ e .cache/ies/
    #[arg(long, default_value = ".")]
    acervo: PathBuf,
    /// Passos explicitamente escolhidos para caixa multipágina (uso avançado)
    #[arg(long)]
    passo: Vec<usize>,
}

#[derive(Debug, Args)]
struct Sessao {
    #[command(subcommand)]
    acao: SessaoAcao,
}

#[derive(Debug, Subcommand)]
enum SessaoAcao {
    /// Consulta caixa via URL assinada e credenciais em arquivos privados
    LerCaixa {
        #[arg(long)]
        url_caixa_file: PathBuf,
        #[arg(long)]
        cookie_file: PathBuf,
        #[arg(long)]
        origem_file: PathBuf,
    },
}

fn erro_lista(json_mode: bool, codigo: &str, mensagem: &str, exit: u8) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({"codigo": codigo, "erro": mensagem, "retentavel": false})
        );
    } else {
        eprintln!("IES: {mensagem}");
    }
    ExitCode::from(exit)
}

fn erro_fonte(json_mode: bool, subcodigo: &str, mensagem: &str) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "fonte_invalida",
                "subcodigo": subcodigo,
                "erro": mensagem,
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: {mensagem} [{subcodigo}]");
    }
    ExitCode::from(1)
}

fn listar_caixa_sessao(
    url: &Path,
    cookie: &Path,
    origem: &Path,
    unidade: Option<&str>,
    json_mode: bool,
) -> ExitCode {
    let html = match transporte::read_box_from_file(url, cookie, origem) {
        Ok(html) => html,
        Err(falha) => return erro_fonte(json_mode, falha.subcodigo(), falha.mensagem()),
    };
    mostrar_caixa(vec![html], "sessao", unidade, json_mode)
}

fn mostrar_caixa(
    htmls: Vec<String>,
    origem: &str,
    unidade: Option<&str>,
    json_mode: bool,
) -> ExitCode {
    let leitura = match paginas::juntar(&htmls, unidade) {
        Ok(valor) => valor,
        Err(erro) => {
            if json_mode {
                println!(
                    "{}",
                    json!({
                        "codigo": "tela_inesperada",
                        "subcodigo": erro.subcodigo(),
                        "erro": erro.to_string(),
                        "categoria": erro.categoria().map(|c| c.rotulo()),
                        "id": erro.id(),
                        "retentavel": false
                    })
                );
            } else {
                eprintln!("IES: {erro}");
            }
            return ExitCode::from(1);
        }
    };
    let resultado = saida::json(&leitura, origem);
    if json_mode {
        println!("{}", serde_json::to_string_pretty(&resultado).unwrap());
    } else {
        println!("{}", saida::humano(&leitura));
    }
    ExitCode::SUCCESS
}

fn mostrar_historico(html: &str, instalacao: &str, json_mode: bool) -> ExitCode {
    let numero = match historico::numero_do_historico(html) {
        Ok(n) => n,
        Err(motivo) => return erro_fonte(json_mode, motivo, "NUP ausente ou ambíguo no histórico"),
    };
    let detalhe = match historico::ler_historico(html, &numero) {
        Ok(valor) => valor,
        Err(motivo) => return erro_fonte(json_mode, motivo, "histórico não verificável"),
    };
    if json_mode {
        println!(
            "{}",
            json!({"processo": detalhe, "instalacao": instalacao, "escopo": "historico_offline", "parcial": true})
        );
    } else {
        println!(
            "Histórico offline do processo {}: {} andamento(s), dados parciais.",
            detalhe.numero.as_str(),
            detalhe.andamentos.len()
        );
    }
    ExitCode::SUCCESS
}

fn executar_replay(replay: &Replay, unidade: Option<&str>, json_mode: bool) -> ExitCode {
    let cenario = &replay.cenario;
    if cenario.is_empty()
        || cenario.contains('/')
        || cenario.contains('\\')
        || cenario.contains("..")
        || !cenario
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return erro_lista(json_mode, "entrada_invalida", "nome de cenário inválido", 2);
    }
    let pasta = replay.acervo.join("cassettes");
    let mut encontrados = Vec::new();
    let dirs = match std::fs::read_dir(&pasta) {
        Ok(dirs) => dirs,
        Err(_) => {
            return erro_fonte(
                json_mode,
                "acervo_ausente",
                "diretório de cassettes não encontrado",
            );
        }
    };
    for dir in dirs.flatten() {
        if !dir.path().is_dir() {
            continue;
        }
        if let Ok(cards) = std::fs::read_dir(dir.path()) {
            for file in cards.flatten() {
                let name = file.file_name();
                let name = name.to_string_lossy();
                if name == format!("{cenario}.md")
                    || name.starts_with(&format!("{cenario}--")) && name.ends_with(".md")
                {
                    encontrados.push(file.path());
                }
            }
        }
    }
    if encontrados.len() != 1 {
        return erro_fonte(
            json_mode,
            "cenario_ambiguo_ou_ausente",
            "cenário não encontrado ou com várias revisões; use um identificador mais específico",
        );
    }
    let card = &encontrados[0];
    let zip_name = match cassette::archive_filename(card) {
        Ok(name) => name,
        Err(_) => return erro_fonte(json_mode, "card_invalido", "metadados de origem inválidos"),
    };
    let zip = replay.acervo.join(".cache/ies").join(zip_name);
    if !zip.is_file() {
        return erro_fonte(
            json_mode,
            "zip_ausente",
            "ZIP não localizado no cache .cache/ies; obtenha o arquivo indicado no card OKF",
        );
    }
    if cenario.contains("andamento") {
        if !replay.passo.is_empty() {
            return erro_lista(
                json_mode,
                "entrada_invalida",
                "passos não são necessários para histórico",
                2,
            );
        }
        match cassette::read_cassette_history_auto(card, &zip) {
            Ok((html, origem)) => mostrar_historico(&html, &origem, json_mode),
            Err(_) => erro_fonte(
                json_mode,
                "historico_invalido",
                "histórico ausente ou ambíguo na cassette",
            ),
        }
    } else {
        match cassette::read_cassette_pages(card, &zip, &replay.passo) {
            Ok((htmls, origem)) => mostrar_caixa(htmls, &origem, unidade, json_mode),
            Err(_) => erro_fonte(
                json_mode,
                "cassette_invalida",
                "caixa ausente ou ambígua na cassette",
            ),
        }
    }
}

fn show_noun_help(noun: &str) {
    let mut root = Cli::command();
    if let Some(command) = root.find_subcommand_mut(noun) {
        let _ = command.print_help();
        println!();
    }
}

fn run(cli: Cli) -> ExitCode {
    match &cli.command {
        Domain::Replay(replay) => return executar_replay(replay, cli.unidade.as_deref(), cli.json),
        Domain::Sessao(Sessao {
            acao:
                SessaoAcao::LerCaixa {
                    url_caixa_file,
                    cookie_file,
                    origem_file,
                },
        }) => {
            return listar_caixa_sessao(
                url_caixa_file,
                cookie_file,
                origem_file,
                cli.unidade.as_deref(),
                cli.json,
            );
        }
        _ => {}
    }
    match &cli.command {
        Domain::Processo(Processo {
            numero: None,
            acao: None,
            ..
        }) => {
            show_noun_help("processo");
            return ExitCode::SUCCESS;
        }
        Domain::Documento(Documento {
            numeros,
            acao: None,
            ..
        }) if numeros.is_empty() => {
            show_noun_help("documento");
            return ExitCode::SUCCESS;
        }
        Domain::Unidade(Unidade { acao: None }) => {
            show_noun_help("unidade");
            return ExitCode::SUCCESS;
        }
        _ => {}
    }
    let detail = format!("{:?}", cli.command);
    if cli.json {
        println!(
            "{}",
            json!({
                "erro": "Comando definido preliminarmente, mas sem implementação SEI",
                "codigo": "nao_implementado",
                "retentavel": false,
                "dados": { "comando": detail },
                "proximo_passo": "Implementar após aprovação do contrato CLI"
            })
        );
    } else {
        eprintln!("IES: comando ainda não implementado no SEI ({detail})");
    }
    ExitCode::from(1)
}

fn main() -> ExitCode {
    match Cli::try_parse() {
        Ok(cli) => run(cli),
        Err(err) => {
            let status = if matches!(
                err.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) {
                0
            } else {
                2
            };
            let _ = err.print();
            ExitCode::from(status)
        }
    }
}
