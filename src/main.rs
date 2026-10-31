mod arquivo_caixa;
mod caixa;
mod cassette;
mod configuracao;
mod documento;
mod documento_web;
mod dominio;
mod erro_web;
mod historico;
mod historico_cli;
mod instalacao;
mod paginas;
mod pesquisa_web;
#[allow(dead_code)] // Tipos da próxima operação, ainda sem adaptador de leitura.
mod processo;
mod processo_resumo_cli;
mod replay_busca_documento;
mod replay_consulta;
mod replay_documento_cli;
mod replay_pesquisa_cli;
mod resumo_processo;
mod saida;
mod status_cli;
mod transporte;
mod unidade_atual_cli;
mod unidade_lista_cli;

use clap::{Args, CommandFactory, Parser, Subcommand};
use serde_json::json;
use std::path::PathBuf;
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
    /// Diagnóstico da conexão (ainda não implementado)
    Status(StatusArgs),
    /// Configura a conexão (ainda não implementado)
    Setup(SetupArgs),
    /// Mostra o contrato de saída (ainda não implementado)
    Esquema,
}

// Flags de exclusão independentes correspondem diretamente às opções da CLI.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Args)]
struct Processo {
    numero: Option<String>,
    /// Consulta somente a identidade já exibida na Caixa de Entrada
    #[arg(long)]
    resumo: bool,
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
    /// Mostra a unidade ativa pelo rótulo publicado na página do Controle
    Atual,
    ListarProcessos {
        #[arg(long)]
        saida: Option<PathBuf>,
    },
}

// As opções do replay são flags independentes de cenários, não estado de domínio.
#[allow(clippy::struct_excessive_bools)]
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
    /// Reproduz um único HTML de documento da operação `documento_imprimir_web`
    #[arg(long)]
    documento: bool,
    /// Escolhe pelo número SEI exato entre documentos de uma cassette verificada
    #[arg(long, requires = "documento")]
    numero_documento: Option<String>,
    /// Inclui o texto visível do corpo no replay offline do documento
    #[arg(long, requires = "documento", conflicts_with = "listar_documentos")]
    texto_documento: bool,
    /// Lista as identidades distintas observadas em documentos da cassette
    #[arg(long, requires = "documento", conflicts_with_all = ["numero_documento", "passo", "texto_documento"])]
    listar_documentos: bool,
    /// Busca termo no texto de documentos já capturados e verificados, sem rede
    #[arg(long, requires = "documento", conflicts_with_all = ["listar_documentos", "numero_documento", "passo", "texto_documento"])]
    termo_documento: Option<String>,
    /// Lê resultados da pesquisa Web capturados em uma resposta POST verificada, sem rede
    #[arg(long, conflicts_with_all = ["documento", "todas_paginas", "numero_processo"])]
    resultado_pesquisa: bool,
    /// Consulta NUP nas caixas já gravadas, sem acesso à rede
    #[arg(long)]
    numero_processo: Option<String>,
    /// Reúne páginas contíguas já gravadas na cassette, sem executar POST
    #[arg(long, conflicts_with = "passo")]
    todas_paginas: bool,
}

#[derive(Debug, Args)]
struct StatusArgs {
    /// Confere apenas metadados dos arquivos privados, sem abrir conteúdos nem rede
    #[arg(long)]
    verificar_arquivos: bool,
}

#[derive(Debug, Args)]
struct SetupArgs {
    #[arg(long = "config-file")]
    destino: PathBuf,
    #[arg(long = "url-controle-file")]
    url_controle: PathBuf,
    #[arg(long = "cookie-file")]
    cookie: PathBuf,
    #[arg(long = "origem-file")]
    origem: PathBuf,
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

fn nome_cenario_aceito(cenario: &str) -> bool {
    !cenario.is_empty()
        && !cenario.contains("..")
        && cenario
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn localizar_cards(pasta: &std::path::Path, cenario: &str) -> std::io::Result<Vec<PathBuf>> {
    let mut encontrados = Vec::new();
    for dir in std::fs::read_dir(pasta)?.flatten() {
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
    Ok(encontrados)
}

fn executar_replay(replay: &Replay, unidade: Option<&str>, json_mode: bool) -> ExitCode {
    if !nome_cenario_aceito(&replay.cenario) {
        return erro_lista(json_mode, "entrada_invalida", "nome de cenário inválido", 2);
    }
    let cenario = &replay.cenario;
    let pasta = replay.acervo.join("cassettes");
    let Ok(encontrados) = localizar_cards(&pasta, cenario) else {
        return erro_fonte(
            json_mode,
            "acervo_ausente",
            "diretório de cassettes não encontrado",
        );
    };
    if encontrados.len() != 1 {
        return erro_fonte(
            json_mode,
            "cenario_ambiguo_ou_ausente",
            "cenário não encontrado ou com várias revisões; use um identificador mais específico",
        );
    }
    let card = &encontrados[0];
    let Ok(zip_name) = cassette::archive_filename(card) else {
        return erro_fonte(json_mode, "card_invalido", "metadados de origem inválidos");
    };
    let zip = replay.acervo.join(".cache/ies").join(zip_name);
    if !zip.is_file() {
        return erro_fonte(
            json_mode,
            "zip_ausente",
            "ZIP não localizado no cache .cache/ies; obtenha o arquivo indicado no card OKF",
        );
    }
    if replay.resultado_pesquisa {
        return replay_pesquisa_cli::mostrar(card, &zip, &replay.passo, json_mode);
    }
    if replay.documento && (replay.numero_processo.is_some() || replay.todas_paginas) {
        return erro_lista(
            json_mode,
            "opcoes_incompativeis",
            "documento não é caixa de processos",
            2,
        );
    }
    if replay.documento {
        if let Some(termo) = replay.termo_documento.as_deref() {
            return replay_busca_documento::mostrar(card, &zip, &replay.passo, termo, json_mode);
        }
        return replay_documento_cli::mostrar(
            card,
            &zip,
            &replay.passo,
            replay.numero_documento.as_deref(),
            replay.listar_documentos,
            replay.texto_documento,
            json_mode,
        );
    }
    match cassette::read_cassette_auto(card, &zip, &replay.passo, replay.todas_paginas) {
        Ok(cassette::ReplayLido::Historico(html, origem)) => historico_cli::mostrar_historico(
            &html,
            &origem,
            replay.numero_processo.as_deref(),
            json_mode,
        ),
        Ok(cassette::ReplayLido::Caixa(htmls, origem)) => {
            if let Some(numero) = replay.numero_processo.as_deref() {
                replay_consulta::mostrar(&htmls, &origem, numero, unidade, json_mode)
            } else {
                unidade_lista_cli::mostrar_caixa(
                    &htmls,
                    &origem,
                    unidade,
                    json_mode,
                    replay.todas_paginas,
                    None,
                )
            }
        }
        Err(_) => erro_fonte(
            json_mode,
            "cassette_invalida",
            "replay não verificável para as operações registradas",
        ),
    }
}

fn show_noun_help(noun: &str) {
    let mut root = Cli::command();
    if let Some(command) = root.find_subcommand_mut(noun) {
        let _ = command.print_help();
        println!();
    }
}

fn mostrar_esquema(json_mode: bool) -> ExitCode {
    let mut comandos: Vec<_> = Cli::command()
        .get_subcommands()
        .map(|c| c.get_name().to_owned())
        .collect();
    comandos.sort();
    if json_mode {
        println!(
            "{}",
            json!({"comandos": comandos, "versao": env!("CARGO_PKG_VERSION")})
        );
    } else {
        println!("Comandos do IES: {}", comandos.join(", "));
    }
    ExitCode::SUCCESS
}

fn validar_documentos(numeros: &[String], json_mode: bool) -> Option<ExitCode> {
    for numero in numeros {
        if let Err(erro) = documento::NumeroDocumento::novo(numero) {
            if json_mode {
                println!(
                    "{}",
                    json!({
                        "codigo": "entrada_invalida",
                        "subcodigo": erro.subcodigo(),
                        "erro": "número SEI de documento inválido",
                        "retentavel": false
                    })
                );
            } else {
                eprintln!(
                    "IES: número SEI de documento inválido [{}]",
                    erro.subcodigo()
                );
            }
            return Some(ExitCode::from(2));
        }
    }
    None
}

fn executar_setup(setup: &SetupArgs, json_mode: bool) -> ExitCode {
    let arquivos = configuracao::ArquivosDeAcesso {
        url_controle: setup.url_controle.clone(),
        cookie: setup.cookie.clone(),
        origem: setup.origem.clone(),
    };
    match configuracao::salvar_arquivos_de_acesso(&setup.destino, &arquivos) {
        Ok(()) => {
            if json_mode {
                println!(
                    "{}",
                    json!({"configuracao": "gravada", "conexao": "nao_verificada"})
                );
            } else {
                println!("IES: caminhos privados configurados; conexão ainda não verificada.");
            }
            ExitCode::SUCCESS
        }
        Err(codigo) => erro_fonte(
            json_mode,
            codigo,
            "não foi possível gravar a configuração local",
        ),
    }
}

fn run(cli: &Cli) -> ExitCode {
    if let Some(status) = processo_resumo_cli::tentar_resumo(cli) {
        return status;
    }
    match &cli.command {
        Domain::Replay(replay) => return executar_replay(replay, cli.unidade.as_deref(), cli.json),
        Domain::Esquema => return mostrar_esquema(cli.json),
        Domain::Setup(setup) => return executar_setup(setup, cli.json),
        Domain::Status(args) => {
            return status_cli::mostrar_status(cli.json, args.verificar_arquivos);
        }
        Domain::Unidade(Unidade {
            acao: Some(UnidadeAcao::Atual),
        }) => return unidade_atual_cli::mostrar_unidade_atual(cli.unidade.as_deref(), cli.json),
        Domain::Unidade(Unidade {
            acao: Some(UnidadeAcao::ListarProcessos { saida }),
        }) => {
            return unidade_lista_cli::listar_processos_da_unidade(
                cli.unidade.as_deref(),
                cli.json,
                saida.as_deref(),
            );
        }
        _ => {}
    }
    if let Domain::Documento(Documento {
        numeros,
        acao: None,
        ..
    }) = &cli.command
        && let Some(status) = validar_documentos(numeros, cli.json)
    {
        return status;
    }
    // Shim de contrato: valida o NUP antes de qualquer adaptador ou acesso Web.
    if let Domain::Processo(Processo {
        numero: Some(numero),
        acao: None,
        ..
    }) = &cli.command
        && let Err(erro) = processo::NumeroProcesso::novo(numero)
    {
        if cli.json {
            println!(
                "{}",
                json!({
                    "codigo": "entrada_invalida",
                    "subcodigo": erro.subcodigo(),
                    "erro": "número de processo inválido",
                    "retentavel": false
                })
            );
        } else {
            eprintln!("IES: número de processo inválido [{}]", erro.subcodigo());
        }
        return ExitCode::from(2);
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
        Ok(cli) => run(&cli),
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

#[cfg(test)]
mod testes_nome_cenario {
    use super::nome_cenario_aceito;

    #[test]
    fn aceita_apenas_nome_de_cenario_sem_caminho() {
        assert!(nome_cenario_aceito("processo-busca-unb--67c8f2ed35b6"));
        for nome in ["", "../processo", "a/b", "a\\b", "a..b", "cenário", "a b"] {
            assert!(!nome_cenario_aceito(nome), "não aceito: {nome}");
        }
    }
}
