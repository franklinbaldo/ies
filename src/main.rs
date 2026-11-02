mod arquivo_caixa;
mod arquivo_copia;
mod arquivo_texto_documento;
mod caixa;
mod caixa_detalhada;
mod caixa_local;
mod cassette;
mod classe_link_processo;
mod configuracao;
mod controle_contexto;
#[allow(dead_code)] // A criação dependerá de editor Web validado.
mod copia_documento;
mod copia_listagem;
mod documento;
mod documento_html_privado_cli;
mod documento_salvar_cli;
#[allow(dead_code)] // O parser offline antecede o transporte de histórico Web.
mod documento_versoes;
mod documento_web;
mod dominio;
mod erro_web;
mod esquema_cli;
mod filtro_andamentos;
#[allow(dead_code)] // Primitivo offline; nenhum comando envia POST nesta fatia.
mod formulario_web;
mod historico;
mod historico_cli;
mod html_atributos;
mod instalacao;
mod mime_web;
mod pagina_do_controle;
mod paginacao_controle;
mod paginas;
mod paginas_capturadas;
mod pesquisa_web;
#[allow(dead_code)] // Tipos da próxima operação, ainda sem adaptador de leitura.
mod processo;
mod processo_historico_privado_cli;
mod processo_pesquisa_privada_cli;
mod processo_resumo_cli;
mod replay_args;
mod replay_busca_documento;
mod replay_cli;
mod replay_consulta;
mod replay_documento_cli;
mod replay_pesquisa_cli;
mod replay_versoes_cli;
mod resumo_exportacao;
mod resumo_pagina;
mod resumo_processo;
mod saida;
mod setup_cli;
mod status_cli;
mod status_web;
mod tela_login;
mod transporte;
mod unidade_atual_cli;
mod unidade_categoria;
mod unidade_lista_cli;

use clap::{Args, Parser, Subcommand};
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
    /// Interpreta HTML de resposta de pesquisa previamente capturada (sem rede)
    #[arg(long = "resultado-pesquisa-file", value_name = "HTML_PRIVADO")]
    resultado_pesquisa_file: Option<PathBuf>,
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
        /// Pede a página N da categoria: no máximo um POST que altera a página lembrada na sessão
        #[arg(
            long,
            value_name = "N",
            requires = "categoria",
            value_parser = clap::value_parser!(u32).range(1..)
        )]
        pagina: Option<u32>,
    },
    /// Lista Recebidos da unidade: a página observada, com total e faixa
    Recebidos(ListaDaUnidade),
    /// Lista Gerados da unidade: a página observada, com total e faixa
    Gerados(ListaDaUnidade),
}

/// Opções de uma lista independente da caixa (Recebidos ou Gerados).
#[derive(Debug, Args)]
struct ListaDaUnidade {
    /// Exporta a lista verificada para novo arquivo privado
    #[arg(long)]
    saida: Option<PathBuf>,
    /// Pede a página N da lista: no máximo um POST que altera a página lembrada na sessão
    #[arg(
        long,
        value_name = "N",
        value_parser = clap::value_parser!(u32).range(1..)
    )]
    pagina: Option<u32>,
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

/// Despacha os recortes de leitura e prévia já implementados, sem duplicar
/// verificações nem acessar serviços fora do comando explicitamente solicitado.
fn tentar_operacao_documento_ou_processo(cli: &Cli) -> Option<ExitCode> {
    processo_pesquisa_privada_cli::tentar_consultar(cli)
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
        Domain::Replay(replay) => {
            return replay_cli::executar_replay(replay, cli.unidade.as_deref(), cli.json);
        }
        Domain::Esquema => return esquema_cli::mostrar_esquema(cli.json),
        Domain::Setup(setup) => return setup_cli::executar_setup(setup, cli.json),
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
                    pagina,
                }),
        }) => {
            if let Some(categoria) = categoria {
                return unidade_categoria::listar_categoria(
                    cli.unidade.as_deref(),
                    cli.json,
                    saida.as_deref(),
                    *categoria,
                    pagina.map(|n| n as usize),
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
            esquema_cli::show_noun_help("processo");
            return ExitCode::SUCCESS;
        }
        Domain::Documento(Documento {
            numeros,
            acao: None,
            ..
        }) if numeros.is_empty() => {
            esquema_cli::show_noun_help("documento");
            return ExitCode::SUCCESS;
        }
        Domain::Unidade(Unidade { acao: None }) => {
            esquema_cli::show_noun_help("unidade");
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
