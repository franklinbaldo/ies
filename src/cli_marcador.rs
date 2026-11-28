use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub(super) struct MarcadorArgs {
    #[command(subcommand)]
    pub(super) acao: Option<MarcadorAcao>,
}

#[derive(Debug, Subcommand)]
pub(super) enum MarcadorAcao {
    /// Prévia de criação do marcador da unidade, sem escrita
    Criar {
        #[arg(long)]
        nome: String,
        #[arg(long)]
        cor: String,
        #[arg(long, default_value = "")]
        descricao: String,
        #[arg(long)]
        confirmar: bool,
    },
    /// Lista marcadores da página de catálogo observada
    Listar {
        /// Seleciona inativos, sem alternar o filtro persistente do SEI
        #[arg(long)]
        inativos: bool,
    },
}
