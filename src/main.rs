mod caixa;
mod cassette;
mod dominio;
mod paginas;
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
        /// Card OKF de provenance da cassette do Internet Archive
        #[arg(long)]
        cassette_card: Option<PathBuf>,
        /// ZIP original baixado do Internet Archive
        #[arg(long)]
        zip_file: Option<PathBuf>,
        /// Índice da interação, quando a cassette contém várias caixas
        #[arg(long)]
        passo: Vec<usize>,
        /// URL da caixa já assinada, obtida na mesma sessão
        #[arg(long)]
        url_caixa_file: Option<PathBuf>,
        /// Arquivo local 0600 contendo apenas os pares Cookie da sessão
        #[arg(long)]
        cookie_file: Option<PathBuf>,
        /// Arquivo privado 0600 contendo a origem HTTPS autorizada (sem caminho)
        #[arg(long)]
        origem_file: Option<PathBuf>,
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

fn salvar_json(destino: &Path, conteudo: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let mut cfg = std::fs::OpenOptions::new();
    cfg.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        cfg.mode(0o600);
    }
    let mut file = cfg
        .open(destino)
        .map_err(|_| "não foi possível criar o arquivo de saída")?;
    file.write_all(conteudo)
        .map_err(|_| "falha ao escrever a saída")?;
    Ok(())
}

struct ConsultaUnidade<'a> {
    output: Option<&'a Path>,
    card: Option<&'a Path>,
    zip: Option<&'a Path>,
    passo: &'a [usize],
    url: Option<&'a Path>,
    cookie: Option<&'a Path>,
    origem_file: Option<&'a Path>,
    unidade: Option<&'a str>,
    json_mode: bool,
}

fn listar_processos(args: ConsultaUnidade<'_>) -> ExitCode {
    let ConsultaUnidade {
        output,
        card,
        zip,
        passo,
        url,
        cookie,
        origem_file,
        unidade,
        json_mode,
    } = args;
    let offline = card.is_some() || zip.is_some() || !passo.is_empty();
    let live = url.is_some() || cookie.is_some() || origem_file.is_some();
    if offline == live
        || (offline && (card.is_none() || zip.is_none()))
        || (live && (url.is_none() || cookie.is_none() || origem_file.is_none()))
    {
        return erro_lista(
            json_mode,
            "entrada_invalida",
            "informe --cassette-card e --zip-file, ou --url-caixa-file, --cookie-file e --origem-file (sem misturar)",
            2,
        );
    }
    let (htmls, origin) = if offline {
        match cassette::read_cassette_pages(card.unwrap(), zip.unwrap(), passo) {
            Ok(resultado) => resultado,
            Err(motivo) => {
                let subcodigo = if motivo.contains("SHA-") {
                    "integridade_cassette"
                } else {
                    "cassette_invalida"
                };
                return erro_fonte(
                    json_mode,
                    subcodigo,
                    "cassette ou ZIP inválidos; confira os hashes e a seleção de passos",
                );
            }
        }
    } else {
        match transporte::read_box_from_file(url.unwrap(), cookie.unwrap(), origem_file.unwrap()) {
            Ok(html) => (vec![html], "sessao".to_owned()),
            Err(falha) => return erro_fonte(json_mode, falha.subcodigo(), falha.mensagem()),
        }
    };
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
    let result = saida::json(&leitura, &origin);
    let serialized = match serde_json::to_vec_pretty(&result) {
        Ok(value) => value,
        Err(_) => return erro_lista(json_mode, "saida_invalida", "falha na serialização JSON", 1),
    };
    if let Some(dest) = output {
        if let Err(message) = salvar_json(dest, &serialized) {
            return erro_lista(json_mode, "saida_invalida", &message, 1);
        }
        if !json_mode {
            println!("Resultado gravado no arquivo solicitado.");
        }
    }
    if json_mode {
        println!("{}", String::from_utf8_lossy(&serialized));
    } else if output.is_none() {
        println!("{}", saida::humano(&leitura));
    }
    ExitCode::SUCCESS
}

fn show_noun_help(noun: &str) {
    let mut root = Cli::command();
    if let Some(command) = root.find_subcommand_mut(noun) {
        let _ = command.print_help();
        println!();
    }
}

fn run(cli: Cli) -> ExitCode {
    if let Domain::Unidade(Unidade {
        acao:
            Some(UnidadeAcao::ListarProcessos {
                saida,
                cassette_card,
                zip_file,
                passo,
                url_caixa_file,
                cookie_file,
                origem_file,
            }),
    }) = &cli.command
    {
        // Sem origem explícita, preserva o erro de stub do contrato provisório.
        if cassette_card.is_some()
            || zip_file.is_some()
            || !passo.is_empty()
            || url_caixa_file.is_some()
            || cookie_file.is_some()
            || origem_file.is_some()
        {
            return listar_processos(ConsultaUnidade {
                output: saida.as_deref(),
                card: cassette_card.as_deref(),
                zip: zip_file.as_deref(),
                passo,
                url: url_caixa_file.as_deref(),
                cookie: cookie_file.as_deref(),
                origem_file: origem_file.as_deref(),
                unidade: cli.unidade.as_deref(),
                json_mode: cli.json,
            });
        }
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
