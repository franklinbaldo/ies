//! Leitura do arquivo de um documento externo (Refs #60, fatia 3;
//! contracts/cli/documento-obter-externo.md): a resposta do GET binário de
//! `documento_download_anexo` é conferida (status, cabeçalhos únicos, tipo
//! na lista observada, `Content-Disposition`, limite) e o corpo é lido em
//! blocos, com SHA-256, para um parcial 0600 em `--saida` ou só para o
//! hash (e o texto, nos tipos de texto). Sem rede: quem chama faz o GET.
use crate::arquivo_privado::FalhaDaGravacao;
use crate::documento_web::ErroCorpoDocumento;
use crate::transporte::FalhaTransporte;
use crate::transporte::FatoDaResposta;
use crate::transporte::resposta_binaria::RespostaBinaria;
use crate::transporte::resposta_http::falha_de_leitura;
use std::io::Read;
use std::path::Path;

mod cabecalhos;
mod conteudo;
mod fluxo;

pub(crate) use fluxo::ler_anexo;

pub(crate) use conteudo::ConteudoDoAnexo;

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
    pub(crate) fn novo(bytes: u64) -> Option<Self> {
        (1..=LIMITE_MAXIMO).contains(&bytes).then_some(Self(bytes))
    }

    /// O valor de `IES_LIMITE_ANEXO_BYTES` (só dígitos), ou o padrão sem
    /// ela; `None` é entrada inválida.
    pub(crate) fn do_texto(texto: Option<&str>) -> Option<Self> {
        let Some(texto) = texto else {
            return Some(Self::default());
        };
        if texto.is_empty() || !texto.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        texto.parse().ok().and_then(Self::novo)
    }

    /// Lido do ambiente do processo.
    pub(crate) fn do_ambiente() -> Option<Self> {
        let valor = std::env::var_os(VARIAVEL_DO_LIMITE);
        match valor {
            None => Some(Self::default()),
            Some(valor) => Self::do_texto(Some(valor.to_str()?)),
        }
    }

    /// O limite do total em `chunked` pedido ao transporte.
    pub(crate) fn para_o_transporte(self) -> usize {
        usize::try_from(self.0).unwrap_or(usize::MAX)
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
    /// Diretório do cache, no qual um parcial será ligado pelo SHA-256.
    Cache {
        diretorio: &'a Path,
        host: &'a str,
    },
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
    pub(crate) nome_original: String,
    pub(crate) disposicao: Disposicao,
    pub(crate) conteudo: ConteudoDoAnexo,
}

/// `DIR/.DOC.ext.parcial-PID-N` da gravação privada única, removido no
/// `Drop` se não for concluído.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ArquivoParcial {
    parcial: crate::arquivo_privado::Parcial,
    nome: String,
}

impl ArquivoParcial {
    /// `DOC.ext`.
    pub(crate) fn nome(&self) -> &str {
        &self.nome
    }

    /// Liga o parcial a `DOC.ext` sem sobrescrever (`hard_link` falha se
    /// o destino existe) e remove o parcial.
    pub(crate) fn concluir(self) -> Result<(), ErroAnexo> {
        self.parcial.confirmar().map_err(|falha| match falha {
            FalhaDaGravacao::DestinoJaExiste => ErroAnexo::ArquivoJaExiste,
            #[cfg(unix)]
            FalhaDaGravacao::DiretorioGravavelPorOutros => ErroAnexo::EscritaFalhou,
            FalhaDaGravacao::SemNomeDeArquivo
            | FalhaDaGravacao::DiretorioAusente
            | FalhaDaGravacao::DiretorioNaoConsultado(_)
            | FalhaDaGravacao::DestinoNaoEhArquivoRegular
            | FalhaDaGravacao::AcimaDoLimite
            | FalhaDaGravacao::CriacaoFalhou(_)
            | FalhaDaGravacao::EscritaFalhou
            | FalhaDaGravacao::ErroDeIoNaEscrita(_)
            | FalhaDaGravacao::NivelDoCaminhoNaoEhDiretorio
            | FalhaDaGravacao::NivelDoCaminhoIlegivel(_)
            | FalhaDaGravacao::CriacaoDoDiretorioFalhou(_)
            | FalhaDaGravacao::CriacaoDoTemporarioFalhou(_)
            | FalhaDaGravacao::NomesDeTemporarioEsgotados
            | FalhaDaGravacao::ConfirmacaoFalhou(_) => ErroAnexo::EscritaFalhou,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroAnexo {
    /// Status, rede, limite de `chunked`, charset.
    Transporte(#[from] FalhaTransporte),
    /// HTML 200 que é a tela de login.
    TelaDeLogin(#[source] crate::erro_publico::SessaoEncerrada),
    SemTipoDeConteudo,
    TipoNaoPermitido,
    CabecalhoRepetido,
    DisposicaoForaDaFormaObservada(#[source] cabecalhos::ErroDisposicao),
    /// Sem `Content-Disposition`: o comando promete disposição e nome.
    CabecalhoDeDisposicaoAusente,
    AcimaDoLimite,
    Vazio,
    ConteudoComLinkAssinado,
    /// HTML com o contêiner de exceção do SEI; status 200 e a mensagem.
    /// Página de exceção do SEI (resposta 200), com o texto saneado se houver.
    RecusadoPeloSei(Option<String>),
    /// O texto visível do HTML recebido.
    Corpo(#[from] ErroCorpoDocumento),
    DiretorioAusente,
    ArquivoJaExiste,
    EscritaFalhou,
    /// A gravação do parcial no disco falhou; a causa é a da gravação.
    GravacaoDoParcialFalhou(#[source] FalhaDaGravacao),
    /// A publicação do parcial no cache falhou.
    PublicacaoDoCacheFalhou(#[source] crate::cache_documento::FalhaCache),
    /// O corpo de texto não decodifica como UTF-8 no charset declarado.
    CorpoNaoUtf8NoCharsetDeclarado(#[source] std::str::Utf8Error),
}

impl ErroAnexo {
    /// A sessão do SEI acabou (decide o relogin único): por `match` no tipo, sem
    /// percorrer a cadeia `source()`.
    pub(crate) fn sessao_encerrada(&self) -> bool {
        match self {
            Self::Transporte(erro) => erro.sessao_encerrada(),
            Self::TelaDeLogin(_) => true,
            Self::SemTipoDeConteudo
            | Self::TipoNaoPermitido
            | Self::CabecalhoRepetido
            | Self::DisposicaoForaDaFormaObservada(_)
            | Self::CabecalhoDeDisposicaoAusente
            | Self::AcimaDoLimite
            | Self::Vazio
            | Self::ConteudoComLinkAssinado
            | Self::RecusadoPeloSei(_)
            | Self::Corpo(_)
            | Self::DiretorioAusente
            | Self::ArquivoJaExiste
            | Self::EscritaFalhou
            | Self::GravacaoDoParcialFalhou(_)
            | Self::PublicacaoDoCacheFalhou(_)
            | Self::CorpoNaoUtf8NoCharsetDeclarado(_) => false,
        }
    }
}

/// A falha de leitura do corpo do anexo: a resposta excessiva é o limite do
/// anexo; as demais seguem como falha de transporte.
/// A falha ao acrescentar bytes ao parcial: só o limite tem fato próprio.
fn erro_ao_escrever(falha: FalhaDaGravacao) -> ErroAnexo {
    match falha {
        FalhaDaGravacao::AcimaDoLimite => ErroAnexo::AcimaDoLimite,
        #[cfg(unix)]
        FalhaDaGravacao::DiretorioGravavelPorOutros => ErroAnexo::EscritaFalhou,
        FalhaDaGravacao::SemNomeDeArquivo
        | FalhaDaGravacao::DiretorioAusente
        | FalhaDaGravacao::DiretorioNaoConsultado(_)
        | FalhaDaGravacao::DestinoJaExiste
        | FalhaDaGravacao::DestinoNaoEhArquivoRegular
        | FalhaDaGravacao::CriacaoFalhou(_)
        | FalhaDaGravacao::EscritaFalhou
        | FalhaDaGravacao::ErroDeIoNaEscrita(_)
        | FalhaDaGravacao::NivelDoCaminhoNaoEhDiretorio
        | FalhaDaGravacao::NivelDoCaminhoIlegivel(_)
        | FalhaDaGravacao::CriacaoDoDiretorioFalhou(_)
        | FalhaDaGravacao::CriacaoDoTemporarioFalhou(_)
        | FalhaDaGravacao::NomesDeTemporarioEsgotados
        | FalhaDaGravacao::ConfirmacaoFalhou(_) => ErroAnexo::EscritaFalhou,
    }
}

fn erro_de_leitura(erro: &std::io::Error) -> ErroAnexo {
    let falha = falha_de_leitura(erro);
    match falha.fato_da_resposta() {
        FatoDaResposta::CorpoAcimaDoLimite => ErroAnexo::AcimaDoLimite,
        FatoDaResposta::TelaDeLoginNoLugarDaResposta | FatoDaResposta::NenhumAlemDoTransporte => {
            ErroAnexo::Transporte(falha)
        }
    }
}

fn conferir_token(bytes: &[u8], lidos: &mut usize) -> Result<(), ErroAnexo> {
    const TOKEN: &[u8] = b"infra_hash";
    for &byte in bytes {
        if TOKEN.get(*lidos) == Some(&byte) {
            *lidos += 1;
        } else {
            *lidos = usize::from(TOKEN.first() == Some(&byte));
        }
        if *lidos == TOKEN.len() {
            return Err(ErroAnexo::ConteudoComLinkAssinado);
        }
    }
    Ok(())
}

/// Liga o parcial binário ao nome content-addressed; o Drop remove o
/// temporário tanto em sucesso quanto em erro.
fn publicar_cache(
    arquivo: &mut Option<ArquivoParcial>,
    destino: DestinoDoAnexo<'_>,
    numero: &str,
    hash: &str,
    extensao: &str,
) -> Result<(), ErroAnexo> {
    if let DestinoDoAnexo::Cache { diretorio, host } = destino {
        let parcial = arquivo.take().ok_or(ErroAnexo::EscritaFalhou)?;
        let raiz = diretorio
            .ancestors()
            .nth(3)
            .ok_or(ErroAnexo::EscritaFalhou)?;
        crate::cache_documento::publicar_parcial(
            parcial.parcial.temporario(),
            raiz,
            host,
            numero,
            hash,
            extensao,
        )
        .map_err(ErroAnexo::PublicacaoDoCacheFalhou)?;
    }
    Ok(())
}

/// Bloco de leitura e gravação.
const BLOCO: usize = 64 * 1024;

/// O que os cabeçalhos decidem antes de qualquer byte do corpo.
struct Conferido {
    mime: String,
    valor_do_tipo: String,
    extensao: &'static str,
    disposicao: Option<(Disposicao, String)>,
    limite: u64,
}

fn unico<'a, R>(
    resposta: &'a RespostaBinaria<R>,
    nome: &str,
) -> Result<Option<&'a str>, ErroAnexo> {
    match resposta.valores(nome).as_slice() {
        [] => Ok(None),
        [um] => Ok(Some(um)),
        _ => Err(ErroAnexo::CabecalhoRepetido),
    }
}

fn conferir<R: Read>(
    resposta: &RespostaBinaria<R>,
    limite: LimiteDoAnexo,
) -> Result<Conferido, ErroAnexo> {
    let tipo = unico(resposta, "content-type")?;
    let tamanho = unico(resposta, "content-length")?;
    let disposicao = unico(resposta, "content-disposition")?;
    let valor_do_tipo = tipo.unwrap_or_default().to_owned();
    let mime = cabecalhos::tipo_principal(&valor_do_tipo).ok_or(ErroAnexo::SemTipoDeConteudo)?;
    let extensao = cabecalhos::extensao(&mime).ok_or(ErroAnexo::TipoNaoPermitido)?;
    let disposicao = disposicao
        .map(|valor| {
            cabecalhos::ler_disposicao(valor).map_err(ErroAnexo::DisposicaoForaDaFormaObservada)
        })
        .transpose()?;
    let limite = if cabecalhos::e_texto(&mime) {
        u64::try_from(crate::MAX_HTML_BYTES)
            .map_or(limite.bytes(), |maximo| limite.bytes().min(maximo))
    } else {
        limite.bytes()
    };
    if let Some(declarado) = tamanho.and_then(|t| t.parse::<u64>().ok())
        && declarado > limite
    {
        return Err(ErroAnexo::AcimaDoLimite);
    }
    Ok(Conferido {
        mime,
        valor_do_tipo,
        extensao,
        disposicao,
        limite,
    })
}
/// Cria o parcial de `DIR/DOC.ext` pela gravação privada única (0600,
/// novo, no diretório real da pessoa, sem conferir o modo dele) depois
/// de conferir que `DOC.ext` ainda não existe.
fn abrir_parcial(
    dir: &Path,
    numero: &str,
    conferido: &Conferido,
) -> Result<ArquivoParcial, ErroAnexo> {
    use crate::arquivo_privado::{FalhaDaGravacao, Parcial, Politica};
    let nome = format!("{numero}.{}", conferido.extensao);
    let politica = Politica::exclusiva()
        .diretorio_do_usuario()
        .com_limite(conferido.limite);
    let parcial = Parcial::abrir(&dir.join(&nome), politica).map_err(|falha| match falha {
        FalhaDaGravacao::SemNomeDeArquivo
        | FalhaDaGravacao::DiretorioAusente
        | FalhaDaGravacao::NivelDoCaminhoNaoEhDiretorio => ErroAnexo::DiretorioAusente,
        falha @ (FalhaDaGravacao::DiretorioNaoConsultado(_)
        | FalhaDaGravacao::NivelDoCaminhoIlegivel(_)) => ErroAnexo::GravacaoDoParcialFalhou(falha),
        FalhaDaGravacao::DestinoJaExiste => ErroAnexo::ArquivoJaExiste,
        // Diretório do usuário: a peça não confere o modo.
        #[cfg(unix)]
        FalhaDaGravacao::DiretorioGravavelPorOutros => ErroAnexo::EscritaFalhou,
        FalhaDaGravacao::DestinoNaoEhArquivoRegular
        | FalhaDaGravacao::AcimaDoLimite
        | FalhaDaGravacao::CriacaoFalhou(_)
        | FalhaDaGravacao::CriacaoDoDiretorioFalhou(_)
        | FalhaDaGravacao::CriacaoDoTemporarioFalhou(_)
        | FalhaDaGravacao::NomesDeTemporarioEsgotados
        | FalhaDaGravacao::EscritaFalhou
        | FalhaDaGravacao::ErroDeIoNaEscrita(_)
        | FalhaDaGravacao::ConfirmacaoFalhou(_) => ErroAnexo::EscritaFalhou,
    })?;
    Ok(ArquivoParcial { parcial, nome })
}

fn abrir_parcial_cache(
    dir: &Path,
    numero: &str,
    conferido: &Conferido,
) -> Result<ArquivoParcial, ErroAnexo> {
    use crate::arquivo_privado::{Parcial, Politica};
    let nome = format!("{numero}.cache.{}", conferido.extensao);
    let parcial = Parcial::abrir(
        &dir.join(&nome),
        Politica::exclusiva()
            .diretorio_do_ies()
            .criando_diretorios()
            .com_limite(conferido.limite),
    )
    .map_err(ErroAnexo::GravacaoDoParcialFalhou)?;
    Ok(ArquivoParcial { parcial, nome })
}

/// HTML que é a tela de login ou a exceção do SEI não é o arquivo. A
/// decodificação aqui é tolerante: só serve para reconhecer a tela.
#[cfg(test)]
#[path = "anexo_documento/tests.rs"]
mod tests;
