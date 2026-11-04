//! Leitura do arquivo de um documento externo (Refs #60, fatia 3;
//! contracts/cli/documento-obter-externo.md): a resposta do GET binário de
//! `documento_download_anexo` é conferida (status, cabeçalhos únicos, tipo
//! na lista observada, `Content-Disposition`, limite) e o corpo é lido em
//! blocos, com SHA-256, para um parcial 0600 em `--saida` ou só para o
//! hash (e o texto, nos tipos de texto). Sem rede: quem chama faz o GET.
use crate::documento_web::ErroCorpoDocumento;
use crate::transporte::resposta_binaria::RespostaBinaria;
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};
use std::io::Read;
use std::path::{Path, PathBuf};

/// Limite padrão do PDF: 64 MiB.
pub(crate) const LIMITE_PADRAO: u64 = 64 * 1024 * 1024;
/// Maior limite aceito em `IES_LIMITE_ANEXO_BYTES`: 2 GiB.
pub(crate) const LIMITE_MAXIMO: u64 = 2 * 1024 * 1024 * 1024;
pub(crate) const VARIAVEL_DO_LIMITE: &str = "IES_LIMITE_ANEXO_BYTES";

/// Bytes aceitos no anexo binário (o texto fica também abaixo de 8 MB).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LimiteDoAnexo(u64);

impl LimiteDoAnexo {
    /// De 1 a [`LIMITE_MAXIMO`].
    pub(crate) fn novo(_bytes: u64) -> Option<Self> {
        todo!()
    }

    /// `IES_LIMITE_ANEXO_BYTES`, ou o padrão sem ela.
    pub(crate) fn do_texto(_texto: Option<&str>) -> Option<Self> {
        todo!()
    }

    pub(crate) fn bytes(self) -> u64 {
        self.0
    }
}

impl Default for LimiteDoAnexo {
    fn default() -> Self {
        Self(LIMITE_PADRAO)
    }
}

/// Onde o corpo vai: só hash (e texto) ou um parcial no diretório.
#[derive(Clone, Copy, Debug)]
pub(crate) enum DestinoDoAnexo<'a> {
    SoMetadados,
    Diretorio(&'a Path),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Disposicao {
    Inline,
    Attachment,
}

impl Disposicao {
    pub(crate) fn como_texto(self) -> &'static str {
        match self {
            Self::Inline => "inline",
            Self::Attachment => "attachment",
        }
    }
}

/// O arquivo lido.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct AnexoLido {
    /// Tipo principal, em minúsculas.
    pub(crate) mime: String,
    pub(crate) tamanho: u64,
    pub(crate) sha256: String,
    pub(crate) nome_original: Option<String>,
    pub(crate) disposicao: Option<Disposicao>,
    /// Só tipos de texto, sem `--saida`.
    pub(crate) texto: Option<String>,
    /// Com `--saida`: o parcial ainda não ligado ao nome final.
    pub(crate) arquivo: Option<ArquivoParcial>,
}

/// `DIR/.DOC.ext.parcial-PID`, removido no `Drop` se não for concluído.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ArquivoParcial {
    parcial: PathBuf,
    destino: PathBuf,
    nome: String,
}

impl ArquivoParcial {
    /// `DOC.ext`.
    pub(crate) fn nome(&self) -> &str {
        &self.nome
    }

    /// Liga o parcial a `DOC.ext` sem sobrescrever e remove o parcial.
    pub(crate) fn concluir(self) -> Result<(), ErroAnexo> {
        todo!()
    }
}

impl Drop for ArquivoParcial {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.parcial);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ErroAnexo {
    /// Status, rede, limite de `chunked`, charset.
    Transporte(FalhaTransporte),
    /// HTML 200 que é a tela de login.
    TelaDeLogin,
    SemTipoDeConteudo,
    TipoNaoPermitido,
    CabecalhoRepetido,
    DisposicaoForaDaFormaObservada,
    AcimaDoLimite,
    Vazio,
    /// HTML com o contêiner de exceção do SEI; status 200 e a mensagem.
    RecusadoPeloSei(StatusHttpRecusado),
    /// O texto visível do HTML recebido.
    Corpo(ErroCorpoDocumento),
    DiretorioAusente,
    DiretorioLinkSimbolico,
    ArquivoJaExiste,
    EscritaFalhou,
}

impl ErroAnexo {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Transporte(falha) => falha.subcodigo(),
            Self::TelaDeLogin => "sessao_encerrada_rode_ies_login",
            Self::SemTipoDeConteudo => "anexo_sem_tipo_de_conteudo",
            Self::TipoNaoPermitido => "tipo_de_anexo_nao_permitido",
            Self::CabecalhoRepetido => "cabecalho_do_anexo_repetido",
            Self::DisposicaoForaDaFormaObservada => "disposicao_do_anexo_fora_da_forma_observada",
            Self::AcimaDoLimite => "anexo_acima_do_limite",
            Self::Vazio => "anexo_vazio",
            Self::RecusadoPeloSei(_) => "anexo_recusado_pelo_sei",
            Self::Corpo(erro) => erro.subcodigo(),
            Self::DiretorioAusente => "diretorio_do_anexo_ausente",
            Self::DiretorioLinkSimbolico => "diretorio_do_anexo_link_simbolico",
            Self::ArquivoJaExiste => "arquivo_do_anexo_ja_existe",
            Self::EscritaFalhou => "escrita_do_anexo_falhou",
        }
    }

    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::Transporte(falha) => falha.mensagem(),
            Self::TelaDeLogin => "a sessão foi encerrada; rode ies login",
            Self::SemTipoDeConteudo => "o anexo veio sem Content-Type",
            Self::TipoNaoPermitido => "o tipo do anexo não está na lista observada",
            Self::CabecalhoRepetido => "cabeçalho do anexo repetido",
            Self::DisposicaoForaDaFormaObservada => {
                "o Content-Disposition do anexo não tem a forma observada"
            }
            Self::AcimaDoLimite => "o anexo passa do limite (IES_LIMITE_ANEXO_BYTES)",
            Self::Vazio => "o anexo veio vazio",
            Self::RecusadoPeloSei(_) => "o SEI recusou o anexo",
            Self::Corpo(_) => "o corpo do HTML recebido não foi reconhecido",
            Self::DiretorioAusente => "o diretório de --saida não existe",
            Self::DiretorioLinkSimbolico => "o diretório de --saida passa por link simbólico",
            Self::ArquivoJaExiste => "o arquivo do anexo já existe em --saida",
            Self::EscritaFalhou => "o arquivo do anexo não foi gravado",
        }
    }

    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Transporte(falha) => falha.status_http(),
            Self::RecusadoPeloSei(status) => Some(status),
            _ => None,
        }
    }
}

/// Lê a resposta de `documento_download_anexo` do documento `numero`.
/// 3xx não chega aqui: quem chama o trata como no resto do fluxo.
pub(crate) fn ler_anexo<R: Read>(
    _resposta: RespostaBinaria<R>,
    _numero: &str,
    _limite: LimiteDoAnexo,
    _destino: DestinoDoAnexo<'_>,
) -> Result<AnexoLido, ErroAnexo> {
    todo!()
}

#[cfg(test)]
#[path = "anexo_documento/tests.rs"]
mod tests;
