mod anexo_documento;
mod arquivo_caixa;
mod arquivo_copia;
mod arquivo_privado;
mod arquivo_texto_documento;
mod arvore_processo;
mod cache_documento;
mod caixa;
mod caixa_detalhada;
#[cfg(test)]
mod cassette;
mod classe_link_processo;
mod colheita_do_controle;
mod configuracao;
mod controle_contexto;
mod controle_reconhecido;
mod copia_documento;
mod copia_listagem;
mod documento;
mod documento_assinar;
mod documento_assinar_cli;
mod documento_buscar_cli;
mod documento_html_privado_cli;
mod documento_obter_cli;
mod documento_salvar_cli;
mod documento_versao_cli;
mod documento_versoes;
mod documento_web;
mod dominio;
mod entidades_html;
mod erro_web;
mod esquema_cli;
mod formulario_web;
mod foto_documento;
mod hex;
mod historico;
mod historico_processo;
mod html_atributos;
mod indice_local;
mod instalacao;
mod marcadores_processo;
mod mime_web;
mod nivel_de_detalhe;
mod pagina_do_controle;
mod paginacao_controle;
mod paginas;
mod pesquisa_web;
mod processo;
mod processo_buscar_cli;
mod processo_marcar;
mod processo_marcar_cli;
mod processo_concluir;
mod processo_concluir_cli;
mod processo_enviar;
mod processo_enviar_cli;
mod processo_obter;
mod processo_obter_cli;
mod processo_reabrir;
mod processo_reabrir_cli;
mod propriedades_processo;
mod relogin_cli;
mod rodape_do_documento;
mod saida;
mod saida_erro;
mod sessao_login;
mod sessao_login_cli;
mod setup_cli;
mod status_cli;
mod status_web;
mod tela_login;
mod transporte;
mod unidade_atual_cli;
mod unidade_categoria;
mod unidade_colunas_cli;
mod unidade_colunas_definir_cli;
mod unidade_lista_cli;
mod unidade_trocar;
mod unidade_trocar_cli;
mod url_observada;
mod visao_do_controle;

use clap::{Args, Parser, Subcommand};
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
    /// Diagnóstico da conexão (ainda não implementado)
    Status(StatusArgs),
    /// Configura a conexão (ainda não implementado)
    Setup(SetupArgs),
    /// Autentica na instalação configurada e grava a sessão nos arquivos privados
    ///
    /// Credenciais só no arquivo privado 0600 de `IES_CREDENCIAIS_FILE`, com as
    /// linhas usuario=, senha= e orgao= (a sigla mostrada no selOrgao). Sem
    /// --forcar, confere antes a sessão gravada com um GET e não loga se ela
    /// valer.
    Login(LoginArgs),
    /// Mostra o contrato de saída (ainda não implementado)
    Esquema,
}

// Flags de exclusão independentes correspondem diretamente às opções da CLI.
#[allow(clippy::struct_excessive_bools)]
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
    /// Pesquisa processo na Web (GET, GET e POST); com --pagina N, mais um POST
    Buscar {
        /// NUP inteiro ou parte do número, como digitado no campo Protocolo
        termo: String,
        /// Pede a página N do resultado (um POST que altera a página lembrada)
        #[arg(
            long,
            value_name = "N",
            value_parser = clap::value_parser!(u32).range(1..)
        )]
        pagina: Option<u32>,
        /// Tipo exato da lista publicada na tela
        #[arg(long, value_name = "TIPO")]
        tipo: Option<String>,
        /// Assunto publicado pela tela (autocompletar exige ID observado)
        #[arg(long, value_name = "TEXTO")]
        assunto: Option<String>,
        /// Data inicial no formato DD/MM/AAAA
        #[arg(long, value_name = "DD/MM/AAAA")]
        de: Option<String>,
        /// Data final no formato DD/MM/AAAA
        #[arg(long, value_name = "DD/MM/AAAA")]
        ate: Option<String>,
    },
    /// Envia o processo a uma unidade (prévia; com --confirmar, um POST e a
    /// verificação por leitura)
    Enviar {
        numero: String,
        /// Sigla da unidade de destino, como o SEI a oferece
        #[arg(long, required = true)]
        para: Vec<String>,
        /// Marca "Manter processo aberto na unidade atual"
        #[arg(long)]
        manter_aberto: bool,
        #[arg(long)]
        confirmar: bool,
    },
    /// Atribui ou retira marcador id#nome; prévia Web (escrita ainda não observada).
    Marcar {
        numero: String,
        #[arg(long, required_unless_present = "retirar", conflicts_with = "retirar")]
        marcador: Option<String>,
        #[arg(long, conflicts_with = "marcador")]
        retirar: Option<String>,
        #[arg(long, requires = "marcador", conflicts_with = "retirar")]
        texto: Option<String>,
        #[arg(long)]
        confirmar: bool,
    },
    Concluir {
        numero: String,
        #[arg(long)]
        confirmar: bool,
    },
    Reabrir {
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
    /// Interpreta HTML privado já capturado, sem rede ou confirmação de sessão
    #[arg(long = "html-privado", value_name = "ARQUIVO")]
    html_privado: Option<PathBuf>,
    /// Diretório onde os conteúdos serão salvos
    #[arg(long)]
    saida: Option<String>,
    /// Desativa a gravação automática do cache local de documentos
    #[arg(long)]
    sem_cache: bool,
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
    /// Pesquisa documento na Web (GET, GET e POST); com --pagina N, mais um POST
    Buscar {
        /// Número SEI exato ou parcial, como digitado no campo Protocolo
        termo: String,
        /// Pede a página N do resultado (um POST que altera a página lembrada)
        #[arg(
            long,
            value_name = "N",
            value_parser = clap::value_parser!(u32).range(1..)
        )]
        pagina: Option<u32>,
        /// Tipo exato da lista publicada na tela
        #[arg(long, value_name = "TIPO")]
        tipo: Option<String>,
        /// Assunto publicado pela tela (autocompletar exige ID observado)
        #[arg(long, value_name = "TEXTO")]
        assunto: Option<String>,
        /// Data inicial no formato DD/MM/AAAA
        #[arg(long, value_name = "DD/MM/AAAA")]
        de: Option<String>,
        /// Data final no formato DD/MM/AAAA
        #[arg(long, value_name = "DD/MM/AAAA")]
        ate: Option<String>,
    },
    /// Assina com a senha do login (`IES_CREDENCIAIS_FILE`); prévia sem POST
    Assinar {
        numero: String,
        /// Cargo/função exatamente como uma opção da tela de assinatura
        #[arg(long, value_name = "TEXTO")]
        cargo: Option<String>,
        /// Assina (não se desfaz); exige --cargo
        #[arg(long, requires = "cargo")]
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
    /// Lista os processos da unidade na visão detalhada, com colunas; a sessão fica na D
    ListarProcessos {
        /// Exporta a lista verificada para novo arquivo privado
        #[arg(long)]
        saida: Option<PathBuf>,
        /// Pede a página N da visão detalhada (um POST que altera a página lembrada)
        #[arg(
            long,
            value_name = "N",
            value_parser = clap::value_parser!(u32).range(1..)
        )]
        pagina: Option<u32>,
    },
    /// Mostra as 12 opções do Nível de detalhe; com --definir, prévia da escrita
    Colunas {
        /// Colunas a deixar marcadas: `todas` (as publicadas) ou chaves separadas por vírgula; sem --confirmar, só prévia
        #[arg(long, value_name = "todas|LISTA")]
        definir: Option<String>,
        /// Salva a configuração pedida (um POST) e confere por leitura
        #[arg(long, requires = "definir")]
        confirmar: bool,
    },
    /// Troca a unidade ativa da sessão; vale para toda a sessão da conta (todos os agentes que a usam). Sem --confirmar, só prévia
    Trocar {
        /// Sigla como a tela Trocar Unidade a mostra (byte a byte)
        sigla: String,
        /// Envia a troca (um POST) e confere pelo Controle da unidade nova
        #[arg(long)]
        confirmar: bool,
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
    /// Pede a página N da lista: um POST que altera a página lembrada (na visão detalhada, troca para a resumida)
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

/// Credenciais só por arquivo privado `0600` em `IES_CREDENCIAIS_FILE`.
#[derive(Debug, Args)]
struct LoginArgs {
    /// Faz novo login mesmo com a sessão gravada válida
    #[arg(long)]
    forcar: bool,
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

fn erro_fonte(json_mode: bool, subcodigo: &str, mensagem: &str) -> ExitCode {
    saida_erro::erro_fonte_http(json_mode, subcodigo, mensagem, None)
}

/// Despacha os recortes de leitura e prévia já implementados, sem duplicar
/// verificações nem acessar serviços fora do comando explicitamente solicitado.
fn tentar_operacao_documento_ou_processo(cli: &Cli) -> Option<ExitCode> {
    processo_buscar_cli::tentar_buscar(cli)
        .or_else(|| documento_buscar_cli::tentar_buscar(cli))
        .or_else(|| documento_html_privado_cli::tentar_ler(cli))
        .or_else(|| documento_salvar_cli::tentar_salvar_documento(cli))
        .or_else(|| documento_salvar_cli::tentar_validar_versao(cli))
        .or_else(|| documento_obter_cli::tentar_obter(cli))
        .or_else(|| processo_obter_cli::tentar_obter(cli))
        .or_else(|| processo_marcar_cli::tentar_marcar(cli))
        .or_else(|| processo_concluir_cli::tentar_concluir(cli))
        .or_else(|| processo_reabrir_cli::tentar_reabrir(cli))
        .or_else(|| documento_assinar_cli::tentar_assinar(cli))
        .or_else(|| processo_enviar_cli::tentar_enviar(cli))
}

/// `ies unidade recebidos|gerados`: uma lista independente da caixa.
fn listar(
    cli: &Cli,
    lista: &ListaDaUnidade,
    categoria: unidade_categoria::CategoriaLista,
) -> ExitCode {
    unidade_categoria::listar_categoria(
        cli.unidade.as_deref(),
        cli.json,
        lista.saida.as_deref(),
        categoria,
        lista.pagina.map(|n| n as usize),
    )
}

/// `ies unidade colunas`: leitura; com `--definir`, prévia ou escrita.
fn colunas(cli: &Cli, definir: Option<&str>, confirmar: bool) -> ExitCode {
    match definir {
        None => unidade_colunas_cli::mostrar_colunas(cli.unidade.as_deref(), cli.json),
        Some(definir) => unidade_colunas_definir_cli::definir_colunas(
            cli.unidade.as_deref(),
            cli.json,
            definir,
            confirmar,
        ),
    }
}

fn run(cli: &Cli) -> ExitCode {
    if let Some(status) = tentar_operacao_documento_ou_processo(cli) {
        return status;
    }
    match &cli.command {
        Domain::Esquema => return esquema_cli::mostrar_esquema(cli.json),
        Domain::Setup(setup) => return setup_cli::executar_setup(setup, cli.json),
        Domain::Login(LoginArgs { forcar }) => {
            return sessao_login_cli::executar_login(cli.json, *forcar, cli.unidade.as_deref());
        }
        Domain::Status(args) => {
            return status_cli::mostrar_status(cli.json, args, cli.unidade.as_deref());
        }
        Domain::Unidade(Unidade {
            acao: Some(UnidadeAcao::Atual),
        }) => return unidade_atual_cli::mostrar_unidade_atual(cli.unidade.as_deref(), cli.json),
        Domain::Unidade(Unidade {
            acao: Some(UnidadeAcao::Recebidos(lista)),
        }) => return listar(cli, lista, unidade_categoria::CategoriaLista::Recebidos),
        Domain::Unidade(Unidade {
            acao: Some(UnidadeAcao::Gerados(lista)),
        }) => return listar(cli, lista, unidade_categoria::CategoriaLista::Gerados),
        Domain::Unidade(Unidade {
            acao: Some(UnidadeAcao::ListarProcessos { saida, pagina }),
        }) => {
            return unidade_lista_cli::listar_processos_da_unidade(
                cli.unidade.as_deref(),
                cli.json,
                saida.as_deref(),
                pagina.map(|n| n as usize),
            );
        }
        Domain::Unidade(Unidade {
            acao: Some(UnidadeAcao::Colunas { definir, confirmar }),
        }) => return colunas(cli, definir.as_deref(), *confirmar),
        Domain::Unidade(Unidade {
            acao: Some(UnidadeAcao::Trocar { sigla, confirmar }),
        }) => {
            return unidade_trocar_cli::trocar(sigla, *confirmar, cli.unidade.as_deref(), cli.json);
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
    if let Some(status) = processo_obter_cli::validar_numero(cli) {
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
