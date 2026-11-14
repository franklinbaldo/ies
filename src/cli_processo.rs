//! Argumentos de `ies processo` (subcomandos e flags), separados de `main.rs`.
use clap::{Args, Subcommand};

#[expect(
    clippy::struct_excessive_bools,
    reason = "Flags de exclusão independentes correspondem diretamente às opções da CLI"
)]
#[derive(Debug, Args)]
pub(crate) struct Processo {
    #[arg(value_parser = crate::processo::NumeroProcesso::novo)]
    pub(crate) numero: Option<crate::processo::NumeroProcesso>,
    /// Omite documentos da leitura
    #[arg(long)]
    pub(crate) sem_documentos: bool,
    /// Omite marcadores da leitura
    #[arg(long)]
    pub(crate) sem_marcadores: bool,
    /// Omite andamentos da leitura
    #[arg(long, conflicts_with = "andamentos_completos")]
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
    /// Lista os comentários comprovadamente publicados do processo, sem escrita.
    Comentarios {
        #[arg(value_parser = crate::processo::NumeroProcesso::novo)]
        numero: crate::processo::NumeroProcesso,
    },
    /// Prévia de comentário; confirmação só após prova da escrita.
    Comentar {
        #[arg(value_parser = crate::processo::NumeroProcesso::novo)]
        numero: crate::processo::NumeroProcesso,
        #[arg(long, value_parser = crate::processo_comentar_web::texto_do_comentario)]
        texto: String,
        #[arg(long)]
        confirmar: bool,
    },
    /// Consulta, prevê e confirma anotação ou retirada.
    #[command(group(clap::ArgGroup::new("escrita_da_anotacao").args(["texto", "remover"])))]
    Anotar {
        #[arg(value_parser = crate::processo::NumeroProcesso::novo)]
        numero: crate::processo::NumeroProcesso,
        #[arg(long, conflicts_with = "remover", value_parser = crate::processo_anotar::texto_da_anotacao)]
        texto: Option<String>,
        #[arg(long, conflicts_with = "texto")]
        remover: bool,
        /// Exige --texto ou --remover.
        #[arg(long, requires = "escrita_da_anotacao")]
        confirmar: bool,
    },
    /// Registra andamento livre no histórico do processo (prévia antes da escrita)
    Andamento {
        #[arg(value_parser = crate::processo::NumeroProcesso::novo)]
        numero: crate::processo::NumeroProcesso,
        #[arg(long, value_parser = crate::processo_andamento_registrar::texto_do_andamento)]
        texto: String,
        #[arg(long)]
        confirmar: bool,
    },
    /// Inicia novo processo Web: prévia, ou confirma após validação.
    Criar {
        #[arg(long, value_parser = crate::processo_criar_cli::argumento_de_criacao)]
        tipo: String,
        #[arg(long, value_parser = crate::processo_criar_cli::argumento_de_criacao)]
        especificacao: String,
        #[arg(long, value_parser = crate::processo_criar_cli::argumento_de_criacao)]
        interessado: Vec<String>,
        #[arg(long, default_value = "publico", value_parser = ["publico", "restrito"])]
        nivel: String,
        /// ID#NOME exatos da hipótese observada no AJAX de acesso restrito.
        #[arg(long, value_name = "ID#NOME", value_parser = crate::processo_criar_cli::hipotese_legal)]
        hipotese_legal: Option<String>,
        #[arg(long)]
        confirmar: bool,
    },
    /// Pesquisa processo na Web (GET, GET e POST); com --pagina N, mais um POST
    Buscar {
        /// NUP inteiro ou parte do número, como digitado no campo Protocolo
        #[arg(value_parser = crate::processo::NumeroProcesso::novo)]
        termo: crate::processo::NumeroProcesso,
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
        #[arg(
            long,
            value_name = "DD/MM/AAAA",
            value_parser = crate::pesquisa_web::filtros::data_da_cli
        )]
        de: Option<crate::unidade_retornos::DataCivil>,
        /// Data final no formato DD/MM/AAAA
        #[arg(
            long,
            value_name = "DD/MM/AAAA",
            value_parser = crate::pesquisa_web::filtros::data_da_cli
        )]
        ate: Option<crate::unidade_retornos::DataCivil>,
    },
    /// Envia o processo a uma unidade (prévia; com --confirmar, um POST e a
    /// verificação por leitura)
    Enviar {
        #[arg(value_parser = crate::processo::NumeroProcesso::novo)]
        numero: crate::processo::NumeroProcesso,
        /// Sigla da unidade de destino, como o SEI a oferece
        #[arg(long, value_parser = crate::processo_enviar_cli::sigla_de_destino)]
        para: String,
        /// Marca "Manter processo aberto na unidade atual"
        #[arg(long)]
        manter_aberto: bool,
        /// Solicita remover as anotações na prévia; confirmação exige releitura.
        #[arg(long)]
        remover_anotacoes: bool,
        #[arg(long)]
        confirmar: bool,
    },
    /// Acompanhamento Especial: prévia e confirmação verificada por releitura
    Acompanhar {
        #[arg(value_parser = crate::processo::NumeroProcesso::novo)]
        numero: crate::processo::NumeroProcesso,
        /// Grupo exatamente como a tela do SEI oferece
        #[arg(long)]
        grupo: Option<String>,
        /// Texto livre da observação
        #[arg(long, conflicts_with = "deixar")]
        observacao: Option<String>,
        /// Retira acompanhamento (sem grupo, exige um único acompanhamento)
        #[arg(long)]
        deixar: bool,
        /// Confirma a escrita; retirada somente em treinamento, com POST único e releitura
        #[arg(long)]
        confirmar: bool,
    },
    /// Atribui ou retira marcador id#nome; prévia Web (escrita ainda não observada).
    Marcar {
        #[arg(value_parser = crate::processo::NumeroProcesso::novo)]
        numero: crate::processo::NumeroProcesso,
        #[arg(
            long,
            required_unless_present = "retirar",
            conflicts_with = "retirar",
            value_parser = crate::processo_marcar::interpretar_rotulo
        )]
        marcador: Option<crate::processo_marcar::Rotulo>,
        #[arg(long, conflicts_with = "marcador", value_parser = crate::processo_marcar::interpretar_rotulo)]
        retirar: Option<crate::processo_marcar::Rotulo>,
        #[arg(
            long,
            requires = "marcador",
            conflicts_with = "retirar",
            value_parser = crate::processo_marcar::texto_do_marcador
        )]
        texto: Option<String>,
        #[arg(long)]
        confirmar: bool,
    },
    Atribuir {
        #[arg(value_parser = crate::processo::NumeroProcesso::novo)]
        numero: crate::processo::NumeroProcesso,
        #[arg(long, required_unless_present = "remover", conflicts_with = "remover")]
        usuario: Option<String>,
        #[arg(long)]
        remover: bool,
        #[arg(long)]
        confirmar: bool,
    },
    Concluir {
        #[arg(value_parser = crate::processo::NumeroProcesso::novo)]
        numero: crate::processo::NumeroProcesso,
        #[arg(long)]
        confirmar: bool,
    },
    Reabrir {
        #[arg(value_parser = crate::processo::NumeroProcesso::novo)]
        numero: crate::processo::NumeroProcesso,
        #[arg(long)]
        confirmar: bool,
    },
    /// Lê o controle de prazo ou prepara uma prévia; escrita bloqueada nesta fatia
    #[command(group(clap::ArgGroup::new("escrita_do_prazo").args(["ate", "dias", "excluir"])))]
    Prazo {
        #[arg(value_parser = crate::processo::NumeroProcesso::novo)]
        numero: crate::processo::NumeroProcesso,
        /// Data certa no formato DD/MM/AAAA
        #[arg(
            long,
            conflicts_with_all = ["dias", "excluir"],
            value_parser = crate::processo_prazo::data_do_prazo
        )]
        ate: Option<crate::unidade_retornos::DataCivil>,
        /// Dias corridos positivos (a contagem exata é do SEI)
        #[arg(long, conflicts_with = "excluir", value_parser = crate::processo_prazo::dias_do_prazo)]
        dias: Option<u16>,
        /// Remove o controle de prazo pela lista da unidade
        #[arg(long)]
        excluir: bool,
        /// Grava uma única vez e confere por releitura independente; exige
        /// --ate, --dias ou --excluir
        #[arg(long, requires = "escrita_do_prazo")]
        confirmar: bool,
    },
    /// Registra ciência do usuário no processo: prévia e um GET mutador conferido.
    Ciente {
        #[arg(value_parser = crate::processo::NumeroProcesso::novo)]
        numero: crate::processo::NumeroProcesso,
        #[arg(long)]
        confirmar: bool,
    },
    Sobrestar {
        #[arg(value_parser = crate::processo::NumeroProcesso::novo)]
        numero: crate::processo::NumeroProcesso,
        #[arg(long, value_parser = crate::processo_sobrestar::motivo_do_sobrestamento)]
        motivo: String,
        #[arg(long)]
        confirmar: bool,
    },
    Retomar {
        #[arg(value_parser = crate::processo::NumeroProcesso::novo)]
        numero: crate::processo::NumeroProcesso,
        #[arg(long)]
        confirmar: bool,
    },
    /// Prévia de relacionar/desfazer vínculo; escrita ainda bloqueada
    Relacionar {
        #[arg(value_parser = crate::processo::NumeroProcesso::novo)]
        numero: crate::processo::NumeroProcesso,
        #[arg(long = "com", value_parser = crate::processo::NumeroProcesso::novo)]
        com: crate::processo::NumeroProcesso,
        #[arg(long)]
        remover: bool,
        #[arg(long)]
        confirmar: bool,
    },
    /// Prévia da anexação do processo NUP ao processo principal (--a).
    Anexar {
        #[arg(value_parser = crate::processo::NumeroProcesso::novo)]
        numero: crate::processo::NumeroProcesso,
        #[arg(long, value_parser = crate::processo::NumeroProcesso::novo)]
        a: crate::processo::NumeroProcesso,
        #[arg(long)]
        confirmar: bool,
    },
}
