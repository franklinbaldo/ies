//! Leitura do arquivo de um documento externo (Refs #60, fatia 3;
//! contracts/cli/documento-obter-externo.md): a resposta do GET binário de
//! `documento_download_anexo` é conferida (status, cabeçalhos únicos, tipo
//! na lista observada, `Content-Disposition`, limite) e o corpo é lido em
//! blocos, com SHA-256, para um parcial 0600 em `--saida` ou só para o
//! hash (e o texto, nos tipos de texto). Sem rede: quem chama faz o GET.
use crate::arquivo_privado::FalhaDaGravacao;
use crate::documento_web::ErroCorpoDocumento;
use crate::transporte::FatoDaResposta;
use crate::transporte::resposta_binaria::RespostaBinaria;
use crate::transporte::resposta_http::falha_de_leitura;
use crate::transporte::{FalhaTransporte, RespostaWeb, recusar_status};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

mod cabecalhos;

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
    pub(crate) nome_original: Option<String>,
    pub(crate) disposicao: Option<Disposicao>,
    /// Só tipos de texto, sem `--saida`.
    pub(crate) texto: Option<String>,
    /// Com `--saida`: o parcial ainda não ligado ao nome final.
    pub(crate) arquivo: Option<ArquivoParcial>,
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
            FalhaDaGravacao::SemNomeDeArquivo
            | FalhaDaGravacao::DiretorioAusente
            | FalhaDaGravacao::DiretorioGravavelPorOutros
            | FalhaDaGravacao::DestinoNaoEhArquivoRegular
            | FalhaDaGravacao::AcimaDoLimite
            | FalhaDaGravacao::CriacaoFalhou
            | FalhaDaGravacao::EscritaFalhou
            | FalhaDaGravacao::ConfirmacaoFalhou => ErroAnexo::EscritaFalhou,
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
            | Self::AcimaDoLimite
            | Self::Vazio
            | Self::ConteudoComLinkAssinado
            | Self::RecusadoPeloSei(_)
            | Self::Corpo(_)
            | Self::DiretorioAusente
            | Self::ArquivoJaExiste
            | Self::EscritaFalhou => false,
        }
    }
}

/// A falha de leitura do corpo do anexo: a resposta excessiva é o limite do
/// anexo; as demais seguem como falha de transporte.
fn erro_de_leitura(erro: &std::io::Error) -> ErroAnexo {
    let falha = falha_de_leitura(erro);
    match falha.fato_da_resposta() {
        FatoDaResposta::CorpoAcimaDoLimite => ErroAnexo::AcimaDoLimite,
        FatoDaResposta::TelaDeLoginNoLugarDaResposta | FatoDaResposta::NenhumAlemDoTransporte => {
            ErroAnexo::Transporte(falha)
        }
    }
}

/// Lê a resposta de `documento_download_anexo` do documento `numero`.
/// 3xx não chega aqui: quem chama o trata como no resto do fluxo.
pub(crate) fn ler_anexo<R: Read>(
    resposta: RespostaBinaria<R>,
    numero: &str,
    limite: LimiteDoAnexo,
    destino: DestinoDoAnexo<'_>,
) -> Result<AnexoLido, ErroAnexo> {
    if resposta.status != 200 {
        // As mesmas recusas de status do transporte (corpo até 64 KiB).
        let mime = resposta
            .valores("content-type")
            .first()
            .map_or_else(String::new, |v| (*v).to_owned());
        return Err(ErroAnexo::Transporte(recusar_status(RespostaWeb {
            status_code: i32::from(resposta.status),
            mime,
            content_length: None,
            localizacao: None,
            corpo: resposta.corpo,
        })));
    }
    let conferido = conferir(&resposta, limite)?;
    let RespostaBinaria { corpo, .. } = resposta;
    let mut arquivo = match destino {
        DestinoDoAnexo::SoMetadados => None,
        DestinoDoAnexo::Diretorio(dir) => Some(abrir_parcial(dir, numero, &conferido)?),
        DestinoDoAnexo::Cache { diretorio, .. } => {
            Some(abrir_parcial_cache(diretorio, numero, &conferido)?)
        }
    };
    let texto = cabecalhos::e_texto(&conferido.mime);
    let mut guardado = Vec::new();
    let mut hasher = Sha256::new();
    let mut tamanho = 0u64;
    // O token não pode chegar ao cache, mesmo se cruzar dois blocos.
    let mut token_lido = 0usize;
    let mut leitor = corpo.take(conferido.limite.saturating_add(1));
    let mut bloco = vec![0u8; BLOCO];
    loop {
        let lido = match leitor.read(&mut bloco) {
            Ok(0) => break,
            Ok(lido) => lido,
            Err(erro) if erro.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(erro) => {
                return Err(erro_de_leitura(&erro));
            }
        };
        tamanho = u64::try_from(lido)
            .ok()
            .and_then(|lido| tamanho.checked_add(lido))
            .ok_or(ErroAnexo::AcimaDoLimite)?;
        if tamanho > conferido.limite {
            return Err(ErroAnexo::AcimaDoLimite);
        }
        conferir_token(&bloco[..lido], &mut token_lido)?;
        hasher.update(&bloco[..lido]);
        if let Some(aberto) = &mut arquivo {
            aberto
                .parcial
                .escrever(&bloco[..lido])
                .map_err(|falha| match falha {
                    FalhaDaGravacao::AcimaDoLimite => ErroAnexo::AcimaDoLimite,
                    FalhaDaGravacao::SemNomeDeArquivo
                    | FalhaDaGravacao::DiretorioAusente
                    | FalhaDaGravacao::DiretorioGravavelPorOutros
                    | FalhaDaGravacao::DestinoJaExiste
                    | FalhaDaGravacao::DestinoNaoEhArquivoRegular
                    | FalhaDaGravacao::CriacaoFalhou
                    | FalhaDaGravacao::EscritaFalhou
                    | FalhaDaGravacao::ConfirmacaoFalhou => ErroAnexo::EscritaFalhou,
                })?;
        }
        if texto {
            guardado.extend_from_slice(&bloco[..lido]);
        }
    }
    if tamanho == 0 {
        return Err(ErroAnexo::Vazio);
    }
    if conferido.mime == "text/html" {
        conferir_html(&guardado, &conferido.valor_do_tipo)?;
    }
    if let Some(aberto) = &mut arquivo {
        aberto
            .parcial
            .sincronizar()
            .map_err(|_| ErroAnexo::EscritaFalhou)?;
    }
    let hash = hex::encode(hasher.finalize());
    publicar_cache(&mut arquivo, destino, numero, &hash, conferido.extensao)?;
    let texto = if texto && arquivo.is_none() {
        Some(texto_do_anexo(
            &conferido.mime,
            &guardado,
            &conferido.valor_do_tipo,
        )?)
    } else {
        None
    };
    Ok(AnexoLido {
        mime: conferido.mime,
        tamanho,
        sha256: hash,
        nome_original: conferido.nome_original,
        disposicao: conferido.disposicao,
        texto,
        arquivo,
    })
}

fn conferir_token(bytes: &[u8], lidos: &mut usize) -> Result<(), ErroAnexo> {
    const TOKEN: &[u8] = b"infra_hash";
    for &byte in bytes {
        if byte == TOKEN[*lidos] {
            *lidos += 1;
        } else {
            *lidos = usize::from(byte == TOKEN[0]);
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
        .map_err(|_| ErroAnexo::EscritaFalhou)?;
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
    nome_original: Option<String>,
    disposicao: Option<Disposicao>,
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
    let (disposicao, nome_original) = match disposicao {
        Some(valor) => {
            let (d, nome) = cabecalhos::ler_disposicao(valor)
                .map_err(ErroAnexo::DisposicaoForaDaFormaObservada)?;
            (Some(d), nome)
        }
        None => (None, None),
    };
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
        nome_original,
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
    if !dir.is_dir() {
        return Err(ErroAnexo::DiretorioAusente);
    }
    let parcial = Parcial::abrir(&dir.join(&nome), politica).map_err(|falha| match falha {
        FalhaDaGravacao::SemNomeDeArquivo
        | FalhaDaGravacao::DiretorioAusente
        | FalhaDaGravacao::NivelDoCaminhoNaoEhDiretorio
        | FalhaDaGravacao::NivelDoCaminhoIlegivel(_) => ErroAnexo::DiretorioAusente,
        FalhaDaGravacao::DestinoJaExiste => ErroAnexo::ArquivoJaExiste,
        // Diretório do usuário: a peça não confere o modo.
        FalhaDaGravacao::DiretorioGravavelPorOutros
        | FalhaDaGravacao::DestinoNaoEhArquivoRegular
        | FalhaDaGravacao::AcimaDoLimite
        | FalhaDaGravacao::CriacaoFalhou
        | FalhaDaGravacao::CriacaoDoDiretorioFalhou(_)
        | FalhaDaGravacao::CriacaoDoTemporarioFalhou(_)
        | FalhaDaGravacao::NomesDeTemporarioEsgotados
        | FalhaDaGravacao::EscritaFalhou
        | FalhaDaGravacao::ConfirmacaoFalhou => ErroAnexo::EscritaFalhou,
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
    .map_err(|_| ErroAnexo::EscritaFalhou)?;
    Ok(ArquivoParcial { parcial, nome })
}

/// HTML que é a tela de login ou a exceção do SEI não é o arquivo. A
/// decodificação aqui é tolerante: só serve para reconhecer a tela.
fn conferir_html(bytes: &[u8], tipo: &str) -> Result<(), ErroAnexo> {
    use crate::transporte::corpo_de_recusa::decodificar;
    use crate::transporte::texto_de_recusa::{e_excecao_do_sei, texto_do_html};
    let html = decodificar(bytes, tipo);
    if crate::tela_login::e_tela_login(&html) {
        return Err(ErroAnexo::TelaDeLogin(crate::erro_publico::SessaoEncerrada));
    }
    if e_excecao_do_sei(&html) {
        let mensagem = texto_do_html(&html).and_then(|t| crate::transporte::saneamento::sanear(&t));
        return Err(ErroAnexo::RecusadoPeloSei(mensagem));
    }
    Ok(())
}

/// O texto publicado: `text/plain` no charset declarado (ISO-8859-1) ou
/// UTF-8 estrito, `\r\n` → `\n` e outros controles (fora `\n`, `\t`)
/// → `�`; `text/html` pelo texto visível do corpo.
fn texto_do_anexo(mime: &str, bytes: &[u8], tipo: &str) -> Result<String, ErroAnexo> {
    let declarado = tipo.to_ascii_lowercase();
    let latin1 = ["8859-1", "latin1", "latin-1", "windows-1252", "cp1252"]
        .iter()
        .any(|nome| declarado.contains(nome));
    let texto = if latin1 {
        bytes.iter().copied().map(char::from).collect()
    } else {
        String::from_utf8(bytes.to_vec()).map_err(|_| {
            ErroAnexo::Transporte(FalhaTransporte::CorpoNaoDecodificavelNoCharsetDeclarado)
        })?
    };
    if mime == "text/html" {
        return crate::documento_web::texto_do_corpo(&texto).map_err(ErroAnexo::Corpo);
    }
    Ok(texto
        .replace("\r\n", "\n")
        .chars()
        .map(|c| {
            if c.is_control() && c != '\n' && c != '\t' {
                '\u{fffd}'
            } else {
                c
            }
        })
        .collect())
}

#[cfg(test)]
#[path = "anexo_documento/tests.rs"]
mod tests;
