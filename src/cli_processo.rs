//! Argumentos de `ies processo` (subcomandos e flags), separados de `main.rs`.
use clap::{Args, Subcommand};

// Flags de exclusão independentes correspondem diretamente às opções da CLI.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Args)]
pub(crate) struct Processo {
    pub(crate) numero: Option<String>,
    /// Omite documentos da leitura
    #[arg(long)]
    pub(crate) sem_documentos: bool,
    /// Omite marcadores da leitura
    #[arg(long)]
    pub(crate) sem_marcadores: bool,
    /// Omite andamentos da leitura
    #[arg(long)]
    pub(crate) sem_andamentos: bool,
    /// Omite propriedades adicionais da leitura
    #[arg(long)]
    pub(crate) sem_propriedades: bool,
    /// Solicita histórico completo do processo
    #[arg(long)]
    pub(crate) andamentos_completos: bool,
    #[command(subcommand)]
    pub(crate) acao: Option<ProcessoAcao>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ProcessoAcao {
    /// Registra andamento livre no histórico do processo (prévia antes da escrita)
    Andamento {
        numero: String,
        #[arg(long)]
        texto: String,
        #[arg(long)]
        confirmar: bool,
    },
    /// Inicia novo processo Web: prévia, ou confirma após validação.
    Criar {
        #[arg(long)]
        tipo: String,
        #[arg(long)]
        especificacao: String,
        #[arg(long)]
        interessado: Vec<String>,
        #[arg(long, default_value = "publico", value_parser = ["publico", "restrito"])]
        nivel: String,
        /// ID#NOME exatos da hipótese observada no AJAX de acesso restrito.
        #[arg(long, value_name = "ID#NOME")]
        hipotese_legal: Option<String>,
        #[arg(long)]
        confirmar: bool,
    },
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
        /// Solicita remover as anotações na prévia; confirmação exige releitura.
        #[arg(long)]
        remover_anotacoes: bool,
        #[arg(long)]
        confirmar: bool,
    },
    /// Prévia Web de Acompanhamento Especial (escrita indisponível até reconciliação comprovada)
    Acompanhar {
        numero: String,
        /// Grupo exatamente como a tela do SEI oferece
        #[arg(long)]
        grupo: Option<String>,
        /// Texto livre da observação
        #[arg(long, conflicts_with = "deixar")]
        observacao: Option<String>,
        /// Retira acompanhamento (sem grupo, exige um único acompanhamento)
        #[arg(long)]
        deixar: bool,
        /// Confirmação de escrita ainda bloqueada por segurança
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
    Atribuir {
        numero: String,
        #[arg(long, required_unless_present = "remover", conflicts_with = "remover")]
        usuario: Option<String>,
        #[arg(long)]
        remover: bool,
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
    /// Lê o controle de prazo ou prepara uma prévia; escrita bloqueada nesta fatia
    Prazo {
        numero: String,
        /// Data certa no formato DD/MM/AAAA
        #[arg(long, conflicts_with_all = ["dias", "excluir"])]
        ate: Option<String>,
        /// Dias corridos positivos (a contagem exata é do SEI)
        #[arg(long, conflicts_with = "excluir")]
        dias: Option<u32>,
        /// Remove o controle de prazo pela lista da unidade
        #[arg(long)]
        excluir: bool,
        /// Grava uma única vez e confere por releitura independente
        #[arg(long)]
        confirmar: bool,
    },
    /// Registra ciência do usuário no processo: prévia e um GET mutador conferido.
    Ciente {
        numero: String,
        #[arg(long)]
        confirmar: bool,
    },
    Sobrestar {
        numero: String,
        #[arg(long)]
        motivo: String,
        #[arg(long)]
        confirmar: bool,
    },
    Retomar {
        numero: String,
        #[arg(long)]
        confirmar: bool,
    },
    /// Prévia de relacionar/desfazer vínculo; escrita ainda bloqueada
    Relacionar {
        numero: String,
        #[arg(long = "com")]
        com: String,
        #[arg(long)]
        remover: bool,
        #[arg(long)]
        confirmar: bool,
    },
    /// Prévia da anexação do processo NUP ao processo principal (--a).
    Anexar {
        numero: String,
        #[arg(long)]
        a: String,
        #[arg(long)]
        confirmar: bool,
    },
}
