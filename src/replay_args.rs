//! Argumentos da ferramenta de replay: não pertencem ao domínio do SEI.
use clap::Args;
use std::path::PathBuf;

// As opções do replay são flags independentes de cenários, não estado de domínio.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Args)]
pub(crate) struct Replay {
    /// Nome do cenário OKF, sem caminho, ex.: processo-andamento-go
    pub(crate) cenario: String,
    /// Raiz local do acervo contendo cassettes/ e .cache/ies/
    #[arg(long, default_value = ".")]
    pub(crate) acervo: PathBuf,
    /// Passos explicitamente escolhidos para caixa multipágina (uso avançado)
    #[arg(long)]
    pub(crate) passo: Vec<usize>,
    /// Reproduz um único HTML de documento da operação `documento_imprimir_web`
    #[arg(long)]
    pub(crate) documento: bool,
    /// Escolhe pelo número SEI exato entre documentos de uma cassette verificada
    #[arg(long, requires = "documento")]
    pub(crate) numero_documento: Option<String>,
    /// Inclui o texto visível do corpo no replay offline do documento
    #[arg(long, requires = "documento", conflicts_with = "listar_documentos")]
    pub(crate) texto_documento: bool,
    /// Salva o texto verificado em arquivo privado novo, sem exibir o corpo no terminal
    #[arg(long, requires = "texto_documento")]
    pub(crate) arquivo_texto: Option<PathBuf>,
    /// Lista as identidades distintas observadas em documentos da cassette
    #[arg(long, requires = "documento", conflicts_with_all = ["numero_documento", "passo", "texto_documento"])]
    pub(crate) listar_documentos: bool,
    /// Busca termo no texto de documentos já capturados e verificados, sem rede
    #[arg(long, requires = "documento", conflicts_with_all = ["listar_documentos", "numero_documento", "passo", "texto_documento"])]
    pub(crate) termo_documento: Option<String>,
    /// Filtra descrições dos andamentos de histórico já gravado, sem rede
    #[arg(long, conflicts_with_all = ["documento", "resultado_pesquisa", "todas_paginas"])]
    pub(crate) termo_andamento: Option<String>,
    /// Lê resultados da pesquisa Web capturados em uma resposta POST verificada, sem rede
    #[arg(long, conflicts_with_all = ["documento", "todas_paginas"])]
    pub(crate) resultado_pesquisa: bool,
    /// Consolida respostas POST de pesquisa escolhidas explicitamente, sem consultar SEI
    #[arg(long, requires = "resultado_pesquisa", requires = "passo")]
    pub(crate) unir_resultados_pesquisa: bool,
    /// Lê histórico Web de versões de documento em uma cassette, sem listar nem abrir editor
    #[arg(long, requires_all = ["documento", "numero_documento", "id_documento_interno"], conflicts_with_all = ["listar_documentos", "texto_documento", "arquivo_texto", "termo_documento"])]
    pub(crate) historico_versoes: bool,
    /// Identificador interno publicado no histórico; o número SEI não é inferido dele
    #[arg(long, requires = "historico_versoes")]
    pub(crate) id_documento_interno: Option<String>,
    /// Consulta exclusivamente o índice de uma versão observada
    #[arg(long, requires = "historico_versoes")]
    pub(crate) versao_documento: Option<u32>,
    /// Consulta NUP nas caixas já gravadas, sem acesso à rede
    #[arg(long)]
    pub(crate) numero_processo: Option<String>,
    /// Reúne páginas contíguas já gravadas na cassette, sem executar POST
    #[arg(long, conflicts_with = "passo")]
    pub(crate) todas_paginas: bool,
}
