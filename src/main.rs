mod arquivo_caixa;
mod arquivo_copia;
mod arquivo_texto_documento;
mod caixa;
mod caixa_local;
mod cassette;
mod configuracao;
mod controle_contexto;
#[allow(dead_code)] // A criação dependerá de editor Web validado.
mod copia_documento;
mod copia_listagem;
mod documento;
mod documento_html_privado_cli;
mod documento_remoto_cli;
mod documento_salvar_cli;
mod documento_transporte;
#[allow(dead_code)] // O parser offline antecede o transporte de histórico Web.
mod documento_versoes;
mod documento_web;
mod dominio;
mod erro_web;
mod filtro_andamentos;
mod historico;
mod historico_cli;
mod instalacao;
mod mime_web;
mod paginas;
mod paginas_capturadas;
mod pesquisa_web;
#[allow(dead_code)] // Tipos da próxima operação, ainda sem adaptador de leitura.
mod processo;
mod processo_historico_privado_cli;
mod processo_resumo_cli;
mod replay_args;
mod replay_busca_documento;
mod replay_consulta;
mod replay_documento_cli;
mod replay_pesquisa_cli;
mod replay_versoes_cli;
mod resumo_exportacao;
mod resumo_pagina;
mod resumo_processo;
mod saida;
mod status_cli;
mod status_web;
mod tela_login;
mod transporte;
mod unidade_atual_cli;
mod unidade_categoria;
mod unidade_lista_cli;

use clap::{Args, CommandFactory, Parser, Subcommand};
use replay_args::Replay;
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
    /// Interpreta histórico HTML privado previamente capturado, sem rede
    #[arg(long = "historico-privado", value_name = "ARQUIVO")]
    historico_privado: Option<PathBuf>,
    /// Salva resumo da caixa em novo arquivo privado sem expor o conteúdo no terminal
    #[arg(long, requires = "resumo")]
    saida: Option<PathBuf>,
    /// Lê resumo de caixa previamente exportada, sem conectar ao SEI
    #[arg(long, requires = "resumo", conflicts_with = "saida")]
    caixa_local: Option<PathBuf>,
    /// Relata NUP na primeira página sem alegar cobertura da caixa
    #[arg(long, requires = "resumo", conflicts_with = "caixa_local")]
    observado_na_pagina: bool,
    /// Une HTML privado previamente capturado ao único GET de resumo
    #[arg(
        long = "pagina-arquivo",
        requires = "resumo",
        conflicts_with_all = ["caixa_local", "observado_na_pagina"]
    )]
    paginas_capturadas: Vec<PathBuf>,
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
// Flags do Clap são opções independentes, não um estado booleano de domínio.
#[allow(clippy::struct_excessive_bools)]
struct Documento {
    /// Números dos documentos SEI
    #[arg(num_args = 0..)]
    numeros: Vec<String>,
    /// Contexto de processo, quando conhecido
    #[arg(long)]
    processo: Option<String>,
    /// Solicita exatamente esta versão histórica (requer transporte autorizado)
    #[arg(long)]
    versao: Option<u32>,
    /// Não prepara cópia editável ao ler
    #[arg(long)]
    sem_copia: bool,
    /// Interpreta HTML privado já capturado, sem rede ou confirmação de sessão
    #[arg(long = "html-privado", value_name = "ARQUIVO")]
    html_privado: Option<PathBuf>,
    /// Link assinado de visualização previamente fornecido em arquivo privado
    #[arg(
        long = "url-documento-file",
        value_name = "ARQUIVO",
        conflicts_with = "html_privado"
    )]
    url_documento_file: Option<PathBuf>,
    /// Confirma ciência e autorização para possível efeito administrativo do GET
    #[arg(long = "autorizar-efeito-de-leitura", requires = "url_documento_file")]
    autorizar_efeito_de_leitura: bool,
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
    /// Prévia de cópia local; confirmação remota ainda bloqueada
    Salvar {
        numero: String,
        #[arg(long)]
        versao: Option<u32>,
        #[arg(long)]
        confirmar: bool,
        /// Lista as cópias locais versionadas sem consultar o SEI
        #[arg(long, conflicts_with_all = ["versao", "confirmar"])]
        listar_copias: bool,
    },
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
        /// Complementa o GET com páginas HTML privadas já capturadas, sem fazer POST
        #[arg(long = "pagina-arquivo", value_name = "HTML_PRIVADO")]
        paginas_capturadas: Vec<PathBuf>,
        /// Projeta somente categoria coberta no primeiro GET; não usa páginas privadas
        #[arg(long, value_enum, conflicts_with = "paginas_capturadas")]
        categoria: Option<unidade_categoria::CategoriaLista>,
    },
}

#[derive(Debug, Args)]
struct StatusArgs {
    /// Confere apenas metadados dos arquivos privados, sem abrir conteúdos nem rede
    #[arg(long, conflicts_with = "verificar_sessao_local")]
    verificar_arquivos: bool,
    /// Confere URL, origem e cookie em arquivos privados, sem consultar SEI
    #[arg(long, conflicts_with = "verificar_arquivos")]
    verificar_sessao_local: bool,
    /// Consulta o Controle Web autenticado configurado: um GET com possíveis efeitos
    #[arg(long, conflicts_with_all = ["verificar_arquivos", "verificar_sessao_local"])]
    verificar_controle_web: bool,
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

fn executar_replay_documento(
    card: &std::path::Path,
    zip: &std::path::Path,
    replay: &Replay,
    json_mode: bool,
) -> ExitCode {
    if replay.historico_versoes {
        return replay_versoes_cli::mostrar(
            card,
            zip,
            &replay.passo,
            replay.numero_documento.as_deref(),
            replay.id_documento_interno.as_deref(),
            replay.versao_documento,
            json_mode,
        );
    }
    if let Some(termo) = replay.termo_documento.as_deref() {
        return replay_busca_documento::mostrar(card, zip, &replay.passo, termo, json_mode);
    }
    replay_documento_cli::mostrar(
        card,
        zip,
        &replay.passo,
        replay.numero_documento.as_deref(),
        replay.listar_documentos,
        &replay_documento_cli::SaidaDocumento {
            texto: replay.texto_documento,
            arquivo: replay.arquivo_texto.as_deref(),
        },
        json_mode,
    )
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
        return replay_pesquisa_cli::mostrar(
            card,
            &zip,
            &replay.passo,
            replay.numero_processo.as_deref(),
            replay.unir_resultados_pesquisa,
            json_mode,
        );
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
        return executar_replay_documento(card, &zip, replay, json_mode);
    }
    match cassette::read_cassette_auto(card, &zip, &replay.passo, replay.todas_paginas) {
        Ok(cassette::ReplayLido::Historico(html, origem)) => historico_cli::mostrar_historico(
            &html,
            &origem,
            replay.numero_processo.as_deref(),
            replay.termo_andamento.as_deref(),
            json_mode,
        ),
        Ok(cassette::ReplayLido::Caixa(htmls, origem)) => {
            if replay.termo_andamento.is_some() {
                return historico_cli::erro_filtro_em_caixa(json_mode);
            }
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

/// Despacha os recortes de leitura e prévia já implementados, sem duplicar
/// verificações nem acessar serviços fora do comando explicitamente solicitado.
fn tentar_operacao_documento_ou_processo(cli: &Cli) -> Option<ExitCode> {
    documento_remoto_cli::tentar_ler(cli)
        .or_else(|| documento_html_privado_cli::tentar_ler(cli))
        .or_else(|| documento_salvar_cli::tentar_salvar_documento(cli))
        .or_else(|| documento_salvar_cli::tentar_validar_versao(cli))
        .or_else(|| processo_historico_privado_cli::tentar_ler(cli))
        .or_else(|| processo_resumo_cli::tentar_resumo(cli))
}

fn run(cli: &Cli) -> ExitCode {
    if let Some(status) = tentar_operacao_documento_ou_processo(cli) {
        return status;
    }
    match &cli.command {
        Domain::Replay(replay) => return executar_replay(replay, cli.unidade.as_deref(), cli.json),
        Domain::Esquema => return mostrar_esquema(cli.json),
        Domain::Setup(setup) => return executar_setup(setup, cli.json),
        Domain::Status(args) => {
            return status_cli::mostrar_status(cli.json, args, cli.unidade.as_deref());
        }
        Domain::Unidade(Unidade {
            acao: Some(UnidadeAcao::Atual),
        }) => return unidade_atual_cli::mostrar_unidade_atual(cli.unidade.as_deref(), cli.json),
        Domain::Unidade(Unidade {
            acao:
                Some(UnidadeAcao::ListarProcessos {
                    saida,
                    paginas_capturadas,
                    categoria,
                }),
        }) => {
            if let Some(categoria) = categoria {
                return unidade_categoria::listar_categoria(
                    cli.unidade.as_deref(),
                    cli.json,
                    saida.as_deref(),
                    *categoria,
                );
            }
            return unidade_lista_cli::listar_processos_da_unidade(
                cli.unidade.as_deref(),
                cli.json,
                saida.as_deref(),
                paginas_capturadas,
            );
        }
        _ => {}
    }
    if let Domain::Documento(Documento {
        numeros,
        acao: None,
        ..
    }) = &cli.command
        && let Some(status) = documento_salvar_cli::validar_documentos(numeros, cli.json)
    {
        return status;
    }
    if let Some(status) = processo_resumo_cli::validar_numero(cli) {
        return status;
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
#[path = "testes_nome_cenario.rs"]
mod testes_nome_cenario;
