mod anexo_documento;
mod arquivo_caixa;
mod arquivo_copia;
mod arquivo_privado;
mod arquivo_texto_documento;
mod arvore_processo;
mod bloco;
mod bloco_cli;
mod bloco_detalhe;
mod bloco_incluir;
mod bloco_incluir_cli;
mod cache_documento;
mod caixa;
mod caixa_detalhada;
#[cfg(test)]
mod cassette;
mod ciencia;
mod ciencia_cli;
mod ciencia_documento_cli;
mod classe_link_processo;
mod cli_documento;
mod cli_processo;
mod cli_unidade_aux;
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
mod documento_cancelar;
mod documento_cancelar_cli;
mod documento_criar;
mod documento_criar_cli;
mod documento_externo_incluir;
mod documento_externo_incluir_cli;
mod documento_html_privado_cli;
mod documento_obter_cli;
mod documento_salvar_cli;
mod documento_versao_cli;
mod documento_versoes;
mod documento_web;
mod dominio;
mod erro_html;
mod erro_publico;
mod erro_web;
mod esquema_cli;
mod formulario_web;
mod foto_documento;
mod historico;
mod historico_processo;
mod html_atributos;
mod indice_caminho_cli;
mod indice_local;
mod instalacao;
mod marcadores_processo;
mod mime_web;
mod nivel_de_detalhe;
mod origem_de_treinamento;
mod pagina_do_controle;
mod paginacao_controle;
mod paginas;
mod pesquisa_web;
mod processo;
mod processo_acompanhar;
mod processo_acompanhar_cli;
mod processo_andamento_registrar;
mod processo_andamento_registrar_cli;
mod processo_anexar;
mod processo_anexar_cli;
mod processo_anotar;
mod processo_anotar_cli;
mod processo_atribuir;
mod processo_atribuir_cli;
mod processo_buscar_cli;
pub mod processo_comentar;
mod processo_comentar_cli;
mod processo_comentar_web;
pub mod processo_comentarios;
mod processo_comentarios_cli;
mod processo_comentarios_web;
mod processo_concluir;
mod processo_concluir_cli;
#[allow(dead_code)] // núcleo isolado: o fluxo Web ainda não está ligado
#[path = "processo_criar/cadastro.rs"]
mod processo_criar_cadastro;
mod processo_criar_cli;
#[allow(dead_code)] // fluxo Web a integrar pela CLI
#[path = "processo_criar/fluxo.rs"]
mod processo_criar_fluxo;
#[path = "processo_criar/hipotese.rs"]
mod processo_criar_hipotese;
#[allow(dead_code)]
#[path = "processo_criar/interessados.rs"]
mod processo_criar_interessados;
#[allow(dead_code)] // etapa isolada da #245; conectar ao despacho na etapa do fluxo
#[path = "processo_criar/lista.rs"]
mod processo_criar_lista;
#[allow(dead_code)] // link do menu validado offline; fluxo Web pendente
#[path = "processo_criar/menu.rs"]
mod processo_criar_menu;
mod processo_enviar;
mod processo_enviar_cli;
mod processo_marcar;
mod processo_marcar_cli;
mod processo_obter;
mod processo_obter_cli;
mod processo_prazo;
mod processo_prazo_cli;
mod processo_reabrir;
mod processo_reabrir_cli;
mod processo_relacionar;
mod processo_relacionar_cli;
mod processo_sobrestar;
mod processo_sobrestar_cli;
mod propriedades_documento;
mod propriedades_processo;
mod relogin_cli;
mod rodape_do_documento;
mod saida;
mod saida_erro;
pub(crate) use saida_erro::erro_fonte;
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
mod unidade_indice_cli;
mod unidade_lista_cli;
mod unidade_lupa;
mod unidade_pesquisar_cli;
mod unidade_retornos;
mod unidade_retornos_cli;
mod unidade_retornos_web;
mod unidade_trocar;
mod unidade_trocar_cli;
mod url_observada;
mod visao_do_controle;

use clap::{Args, Parser, Subcommand};
use serde_json::json;
use std::path::PathBuf;
use std::process::ExitCode;

pub(crate) use cli_documento::DocumentoAcao;
pub(crate) use cli_processo::{Processo, ProcessoAcao};

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
    /// Inspeciona a localização do índice SQLite local, sem rede
    Indice(Indice),
    /// Leitura de blocos de assinatura, internos e de reunião
    Bloco(Bloco),
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

#[derive(Debug, Args)]
struct Bloco {
    /// Número de bloco de assinatura a detalhar; sem número, usar listar
    numero: Option<u64>,
    #[command(subcommand)]
    acao: Option<BlocoAcao>,
}
#[derive(Debug, Subcommand)]
enum BlocoAcao {
    /// Lista blocos da unidade na página atualmente observada
    Listar {
        /// Tipo de bloco oferecido pelo menu Web
        #[arg(long, value_enum, default_value = "assinatura")]
        tipo: bloco::TipoBloco,
        /// Página 1: validação da página atual; páginas adicionais em desenvolvimento
        #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
        pagina: Option<u32>,
    },
    /// Inclui documento (bloco de assinatura) ou processo (interno/reunião) em bloco; sem --confirmar, só prévia
    #[command(group(clap::ArgGroup::new("alvo").required(true).args(["documento", "processo"])))]
    Incluir {
        /// Número do bloco publicado pela lista da unidade
        bloco: u64,
        /// Número SEI do documento (bloco de assinatura)
        #[arg(long, value_parser = crate::documento::NumeroDocumento::novo)]
        documento: Option<crate::documento::NumeroDocumento>,
        /// NUP do processo (bloco interno ou de reunião)
        #[arg(long)]
        processo: Option<String>,
        /// Executa a inclusão (sem isso, só prévia)
        #[arg(long)]
        confirmar: bool,
    },
}

#[derive(Debug, Args)]
struct Indice {
    #[command(subcommand)]
    acao: Option<IndiceAcao>,
}

#[derive(Debug, Subcommand)]
enum IndiceAcao {
    /// Mostra o caminho configurado do SQLite; não abre o banco
    Caminho,
}

#[derive(Debug, Args)]
struct Unidade {
    #[command(subcommand)]
    acao: Option<UnidadeAcao>,
}

#[derive(Debug, Subcommand)]
enum UnidadeAcao {
    /// Lista as unidades já guardadas no índice local, sem rede
    Listar {
        /// Recolhe uma lista nova pela lupa (ainda não disponível nesta fatia)
        #[arg(long)]
        atualizar: bool,
    },
    /// Pesquisa no índice local por sigla ou descrição, sem rede
    Pesquisar { termo: String },
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
    /// Lista os retornos programados da unidade na tela Web do SEI
    Retornos {
        /// Mes AAAA-MM (ausente: mes marcado como hoje pelo SEI)
        #[arg(long, value_name = "AAAA-MM")]
        mes: Option<String>,
    },
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

/// Despacha os recortes de leitura e prévia já implementados, sem duplicar
/// verificações nem acessar serviços fora do comando explicitamente solicitado.
fn tentar_operacao_documento_ou_processo(cli: &Cli) -> Option<ExitCode> {
    processo_criar_cli::tentar_criar(cli)
        .or_else(|| processo_comentarios_cli::tentar_ler(cli))
        .or_else(|| processo_comentar_cli::tentar_comentar(cli))
        .or_else(|| processo_anotar_cli::tentar_anotar(cli))
        .or_else(|| processo_andamento_registrar_cli::tentar_registrar(cli))
        .or_else(|| processo_buscar_cli::tentar_buscar(cli))
        .or_else(|| documento_buscar_cli::tentar_buscar(cli))
        .or_else(|| documento_html_privado_cli::tentar_ler(cli))
        .or_else(|| documento_salvar_cli::tentar_salvar_documento(cli))
        .or_else(|| documento_salvar_cli::tentar_validar_versao(cli))
        .or_else(|| documento_obter_cli::tentar_obter(cli))
        .or_else(|| processo_obter_cli::tentar_obter(cli))
        .or_else(|| processo_atribuir_cli::tentar_atribuir(cli))
        .or_else(|| processo_marcar_cli::tentar_marcar(cli))
        .or_else(|| processo_concluir_cli::tentar_concluir(cli))
        .or_else(|| processo_acompanhar_cli::tentar_acompanhar(cli))
        .or_else(|| processo_reabrir_cli::tentar_reabrir(cli))
        .or_else(|| processo_prazo_cli::tentar_prazo(cli))
        .or_else(|| ciencia_cli::tentar_ciente(cli))
        .or_else(|| ciencia_documento_cli::tentar_ciente_documento(cli))
        .or_else(|| processo_anexar_cli::tentar_anexar(cli))
        .or_else(|| processo_sobrestar_cli::tentar_operar(cli))
        .or_else(|| documento_assinar_cli::tentar_assinar(cli))
        .or_else(|| documento_cancelar_cli::tentar_cancelar(cli))
        .or_else(|| documento_criar_cli::tentar_criar(cli))
        .or_else(|| documento_externo_incluir_cli::tentar_incluir(cli))
        .or_else(|| processo_enviar_cli::tentar_enviar(cli))
        .or_else(|| processo_relacionar_cli::tentar_relacionar(cli))
}

/// `ies unidade colunas`: leitura; com `--definir`, prévia ou escrita.
fn run(cli: &Cli) -> Saida {
    if let Some(status) = unidade_indice_cli::tentar_consulta_local(cli) {
        return Ok(status);
    }
    if let Some(status) = tentar_operacao_documento_ou_processo(cli) {
        return Ok(status);
    }
    match &cli.command {
        Domain::Bloco(bloco) => return bloco_cli::executar(cli, bloco),
        Domain::Esquema => return Ok(esquema_cli::mostrar_esquema(cli.json)),
        Domain::Setup(setup) => return Ok(setup_cli::executar_setup(setup, cli.json)),
        Domain::Login(LoginArgs { forcar }) => {
            return Ok(sessao_login_cli::executar_login(
                cli.json,
                *forcar,
                cli.unidade.as_deref(),
            ));
        }
        Domain::Status(args) => {
            return Ok(status_cli::mostrar_status(
                cli.json,
                args,
                cli.unidade.as_deref(),
            ));
        }
        Domain::Unidade(Unidade {
            acao: Some(UnidadeAcao::Atual),
        }) => {
            return Ok(unidade_atual_cli::mostrar_unidade_atual(
                cli.unidade.as_deref(),
                cli.json,
            ));
        }
        Domain::Unidade(Unidade {
            acao: Some(UnidadeAcao::Recebidos(lista)),
        }) => {
            return Ok(cli_unidade_aux::listar(
                cli,
                lista,
                unidade_categoria::CategoriaLista::Recebidos,
            ));
        }
        Domain::Unidade(Unidade {
            acao: Some(UnidadeAcao::Gerados(lista)),
        }) => {
            return Ok(cli_unidade_aux::listar(
                cli,
                lista,
                unidade_categoria::CategoriaLista::Gerados,
            ));
        }
        Domain::Unidade(Unidade {
            acao: Some(UnidadeAcao::Retornos { mes }),
        }) => {
            return Ok(unidade_retornos_cli::listar(
                cli.unidade.as_deref(),
                cli.json,
                mes.as_deref(),
            ));
        }
        Domain::Unidade(Unidade {
            acao: Some(UnidadeAcao::ListarProcessos { saida, pagina }),
        }) => {
            return Ok(unidade_lista_cli::listar_processos_da_unidade(
                cli.unidade.as_deref(),
                cli.json,
                saida.as_deref(),
                pagina.map(|n| n as usize),
            ));
        }
        Domain::Unidade(Unidade {
            acao: Some(UnidadeAcao::Colunas { definir, confirmar }),
        }) => {
            return Ok(cli_unidade_aux::colunas(
                cli,
                definir.as_deref(),
                *confirmar,
            ));
        }
        Domain::Unidade(Unidade {
            acao: Some(UnidadeAcao::Trocar { sigla, confirmar }),
        }) => {
            return Ok(unidade_trocar_cli::trocar(
                sigla,
                *confirmar,
                cli.unidade.as_deref(),
                cli.json,
            ));
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
        return Ok(status);
    }
    if let Some(status) = processo_obter_cli::validar_numero(cli) {
        return Ok(status);
    }
    match &cli.command {
        Domain::Processo(Processo {
            numero: None,
            acao: None,
            ..
        }) => {
            esquema_cli::show_noun_help("processo");
            return Ok(ExitCode::SUCCESS);
        }
        Domain::Documento(Documento {
            numeros,
            acao: None,
            ..
        }) if numeros.is_empty() => {
            esquema_cli::show_noun_help("documento");
            return Ok(ExitCode::SUCCESS);
        }
        Domain::Indice(Indice { acao: None }) => {
            println!("ies indice caminho");
            return Ok(ExitCode::SUCCESS);
        }
        Domain::Unidade(Unidade { acao: None }) => {
            esquema_cli::show_noun_help("unidade");
            return Ok(ExitCode::SUCCESS);
        }
        _ => {}
    }
    Ok(comando_nao_implementado(cli))
}

fn comando_nao_implementado(cli: &Cli) -> ExitCode {
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

/// O que um comando devolve: o código de saída, ou o erro que sobe até o
/// `main`, o único lugar que o mostra (#695).
pub(crate) type Saida = Result<ExitCode, Box<dyn std::error::Error>>;

/// Mostra o erro que subiu: o próprio tipo (`Debug`), sem texto à parte, e
/// o status e a mensagem do SEI se uma recusa HTTP estiver na cadeia.
fn mostrar(json_mode: bool, erro: &(dyn std::error::Error + 'static)) -> ExitCode {
    let http = std::iter::successors(Some(erro), |e| e.source())
        .find_map(|e| e.downcast_ref::<transporte::StatusHttpRecusado>());
    if json_mode {
        let mut valor = json!({"erro": format!("{erro:?}")});
        if let Some(http) = http {
            if http.status != 200 {
                valor["status_http"] = json!(http.status);
            }
            if let Some(texto) = &http.mensagem_do_servidor {
                valor["mensagem_do_servidor"] = json!(texto);
            }
        }
        println!("{valor}");
    } else {
        match http {
            Some(h) => eprintln!(
                "IES: {erro:?} (HTTP {}{})",
                h.status,
                h.mensagem_do_servidor
                    .as_deref()
                    .map(|t| format!(": {t}"))
                    .unwrap_or_default()
            ),
            None => eprintln!("IES: {erro:?}"),
        }
    }
    ExitCode::from(1)
}

fn main() -> ExitCode {
    match Cli::try_parse() {
        Ok(cli) => match run(&cli) {
            Ok(codigo) => codigo,
            Err(erro) => mostrar(cli.json, erro.as_ref()),
        },
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
