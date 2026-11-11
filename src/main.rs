mod acao_do_sei;
mod anexo_documento;
mod arquivo_caixa;
mod arquivo_copia;
mod arquivo_privado;
mod arquivo_texto_documento;
mod arvore_processo;
mod bloco;
mod bloco_cli;
mod bloco_detalhe;
mod bloco_estado;
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
mod consulta_do_controlador;
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
#[path = "processo_criar/cadastro.rs"]
mod processo_criar_cadastro;
mod processo_criar_cli;
#[path = "processo_criar/fluxo.rs"]
mod processo_criar_fluxo;
#[path = "processo_criar/hipotese.rs"]
mod processo_criar_hipotese;
#[path = "processo_criar/interessados.rs"]
mod processo_criar_interessados;
#[path = "processo_criar/lista.rs"]
mod processo_criar_lista;
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
mod sessao_login;
mod sessao_login_cli;
mod setup_cli;
mod status_cli;
mod status_web;
mod tela_html;
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
mod uso_invalido;
mod visao_do_controle;

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

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
    #[arg(num_args = 0.., value_parser = crate::documento::NumeroDocumento::novo)]
    numeros: Vec<crate::documento::NumeroDocumento>,
    /// Contexto de processo, quando conhecido
    #[arg(long, value_parser = crate::processo::NumeroProcesso::novo)]
    processo: Option<crate::processo::NumeroProcesso>,
    /// Solicita exatamente esta versão histórica (requer transporte autorizado)
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
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
    Pesquisar {
        #[arg(value_parser = clap::builder::NonEmptyStringValueParser::new())]
        termo: String,
    },
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
        #[arg(
            long,
            value_name = "todas|LISTA",
            value_parser = nivel_de_detalhe::AlvoDasColunas::interpretar
        )]
        definir: Option<nivel_de_detalhe::AlvoDasColunas>,
        /// Salva a configuração pedida (um POST) e confere por leitura
        #[arg(long, requires = "definir")]
        confirmar: bool,
    },
    /// Troca a unidade ativa da sessão; vale para toda a sessão da conta (todos os agentes que a usam). Sem --confirmar, só prévia
    Trocar {
        /// Sigla como a tela Trocar Unidade a mostra (byte a byte)
        #[arg(value_parser = unidade_trocar_cli::sigla_da_troca)]
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
        #[arg(long, value_name = "AAAA-MM", value_parser = unidade_retornos_web::mes_da_cli)]
        mes: Option<(u16, u8)>,
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
fn tentar_operacao_documento_ou_processo(cli: &Cli) -> Option<anyhow::Result<()>> {
    processo_criar_cli::tentar_criar(cli)
        .or_else(|| processo_comentarios_cli::tentar_ler(cli))
        .or_else(|| processo_comentar_cli::tentar_comentar(cli))
        .or_else(|| processo_anotar_cli::tentar_anotar(cli))
        .or_else(|| processo_andamento_registrar_cli::tentar_registrar(cli))
        .or_else(|| processo_buscar_cli::tentar_buscar(cli))
        .or_else(|| documento_buscar_cli::tentar_buscar(cli))
        .or_else(|| documento_html_privado_cli::tentar_ler(cli))
        .or_else(|| documento_salvar_cli::tentar_salvar_documento(cli))
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
fn run(cli: &Cli) -> anyhow::Result<()> {
    if let Some(saida) = unidade_indice_cli::tentar_consulta_local(cli) {
        return saida;
    }
    if let Some(saida) = tentar_operacao_documento_ou_processo(cli) {
        return saida;
    }
    match &cli.command {
        Domain::Bloco(bloco) => return bloco_cli::executar(cli, bloco),
        Domain::Esquema => {
            esquema_cli::mostrar_esquema(cli.json);
            return Ok(());
        }
        Domain::Setup(setup) => return setup_cli::executar_setup(setup, cli.json),
        Domain::Login(LoginArgs { forcar }) => {
            return sessao_login_cli::executar_login(cli.json, *forcar, cli.unidade.as_deref());
        }
        Domain::Status(args) => {
            return status_cli::mostrar_status(cli.json, args, cli.unidade.as_deref());
        }
        Domain::Unidade(Unidade { acao: Some(acao) }) => {
            return cli_unidade_aux::executar(cli, acao);
        }
        Domain::Processo(_) | Domain::Documento(_) | Domain::Unidade(_) | Domain::Indice(_) => {}
    }
    match &cli.command {
        Domain::Processo(Processo {
            numero: None,
            acao: None,
            ..
        }) => {
            esquema_cli::show_noun_help("processo");
            return Ok(());
        }
        Domain::Documento(Documento {
            numeros,
            acao: None,
            ..
        }) if numeros.is_empty() => {
            esquema_cli::show_noun_help("documento");
            return Ok(());
        }
        Domain::Indice(Indice { acao: None }) => {
            println!("ies indice caminho");
            return Ok(());
        }
        Domain::Unidade(Unidade { acao: None }) => {
            esquema_cli::show_noun_help("unidade");
            return Ok(());
        }
        Domain::Processo(_)
        | Domain::Documento(_)
        | Domain::Unidade(_)
        | Domain::Indice(_)
        | Domain::Bloco(_)
        | Domain::Status(_)
        | Domain::Setup(_)
        | Domain::Login(_)
        | Domain::Esquema => {}
    }
    Err(ComandoSemImplementacaoNoSei(format!("{:?}", cli.command)).into())
}

/// O comando existe na superfície Clap, mas ainda não tem fluxo SEI.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
struct ComandoSemImplementacaoNoSei(String);

/// O erro que sobe é impresso pelo próprio Rust (`Error: …`, com o `Debug`
/// e a cadeia de causas) e sai com 1; uso inválido é do clap (saída 2).
fn main() -> anyhow::Result<()> {
    run(&Cli::parse())
}
