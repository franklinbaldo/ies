//! Argumentos do subcomando de documento separados de main.rs para respeitar 20 KB.
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub(crate) enum DocumentoAcao {
    /// Inclui PDF externo: inspeção local, prévia Web (--consultar-sei) ou gravação (--confirmar).
    IncluirExterno {
        #[arg(long, required = true, value_name = "NUP", value_parser = crate::processo::NumeroProcesso::novo)]
        processo: crate::processo::NumeroProcesso,
        #[arg(long, required = true, value_name = "ARQUIVO.pdf")]
        arquivo: std::path::PathBuf,
        #[arg(long, required = true, value_name = "TIPO", value_parser = crate::documento_externo_incluir_cli::tipo_externo)]
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
        #[arg(value_parser = crate::documento::NumeroDocumento::novo)]
        numero: crate::documento::NumeroDocumento,
        #[arg(long)]
        confirmar: bool,
    },
    /// Inclui documento interno: prévia Web; confirmar faz escrita única com releitura.
    Criar {
        #[arg(long, required = true, value_parser = crate::processo::NumeroProcesso::novo)]
        processo: crate::processo::NumeroProcesso,
        #[arg(long, required = true, value_parser = crate::documento_criar_cli::tipo_de_documento)]
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
        #[arg(value_parser = crate::documento::NumeroDocumento::novo)]
        numero: crate::documento::NumeroDocumento,
        #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
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
        #[arg(value_parser = crate::documento_buscar_cli::termo_de_documento)]
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
        #[arg(long, value_name = "DD/MM/AAAA", value_parser = crate::pesquisa_web::filtros::data_da_cli)]
        de: Option<crate::unidade_retornos::DataCivil>,
        /// Data final no formato DD/MM/AAAA
        #[arg(long, value_name = "DD/MM/AAAA", value_parser = crate::pesquisa_web::filtros::data_da_cli)]
        ate: Option<crate::unidade_retornos::DataCivil>,
    },
    /// Cancela um documento com motivo, prévia e confirmação irreversível
    Cancelar {
        #[arg(value_parser = crate::documento::NumeroDocumento::novo)]
        numero: crate::documento::NumeroDocumento,
        #[arg(long, required = true)]
        motivo: String,
        #[arg(long)]
        confirmar: bool,
    },
    /// Assina com a senha do login (`IES_CREDENCIAIS_FILE`); prévia sem POST
    Assinar {
        #[arg(value_parser = crate::documento::NumeroDocumento::novo)]
        numero: crate::documento::NumeroDocumento,
        /// Cargo/função exatamente como uma opção da tela de assinatura
        #[arg(long, value_name = "TEXTO")]
        cargo: Option<String>,
        /// Assina (não se desfaz); exige --cargo
        #[arg(long, requires = "cargo")]
        confirmar: bool,
    },
}
