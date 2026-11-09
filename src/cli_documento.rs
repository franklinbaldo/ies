//! Argumentos do subcomando de documento separados de main.rs para respeitar 20 KB.
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub(crate) enum DocumentoAcao {
    /// Inclui PDF externo: inspeção local, prévia Web (--consultar-sei) ou gravação (--confirmar).
    IncluirExterno {
        #[arg(long, required = true, value_name = "NUP")]
        processo: String,
        #[arg(long, required = true, value_name = "ARQUIVO.pdf")]
        arquivo: std::path::PathBuf,
        #[arg(long, required = true, value_name = "TIPO")]
        tipo: String,
        /// Consulta a lista e o cadastro externo no SEI (sem upload nem escrita).
        #[arg(long)]
        consultar_sei: bool,
        /// Envia o PDF e cadastra o documento, uma única vez, com releitura independente
        #[arg(long)]
        confirmar: bool,
    },
    /// Prévia e ciência do usuário da sessão no documento (GET único, releitura).
    Ciente {
        numero: String,
        #[arg(long)]
        confirmar: bool,
    },
    /// Inclui documento interno: prévia Web; confirmar faz escrita única com releitura.
    Criar {
        #[arg(long, required = true)]
        processo: String,
        #[arg(long, required = true)]
        tipo: String,
        #[arg(long)]
        descricao: Option<String>,
        #[arg(long, value_parser = ["publico", "restrito", "sigiloso"])]
        nivel: Option<String>,
        #[arg(long)]
        confirmar: bool,
    },
    /// Prévia de cópia local; confirmação somente no anti-IES de ensaio
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
    /// Cancela um documento com motivo, prévia e confirmação irreversível
    Cancelar {
        numero: String,
        #[arg(long, required = true)]
        motivo: String,
        #[arg(long)]
        confirmar: bool,
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
