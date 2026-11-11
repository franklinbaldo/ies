//! Cabeçalhos do anexo (Refs #60, fatia 3): tipo principal do
//! `Content-Type` (parâmetros vazios tolerados, como o
//! `text/html;;charset=iso-8859-1` gravado), `Content-Disposition` no
//! subconjunto da RFC 6266 (`inline`/`attachment`, `filename` e
//! `filename*` da RFC 5987) e o nome original saneado, que é só
//! apresentação: nunca vira caminho.
use super::Disposicao;

/// Tipos aceitos, pela evidência: PDF (`UnB`, oráculo Todos), texto plano
/// (`documento_txt`) e HTML recebido (cassettes de PEN1, PEN2 e PEN4).
const PERMITIDOS: [(&str, &str); 3] = [
    ("application/pdf", "pdf"),
    ("text/plain", "txt"),
    ("text/html", "html"),
];

/// Maior nome original publicado, em caracteres.
const MAX_NOME: usize = 255;

/// O tipo principal em minúsculas, ou `None` se vazio.
pub(super) fn tipo_principal(valor: &str) -> Option<String> {
    let principal = valor.split(';').next().unwrap_or_default().trim();
    (!principal.is_empty()).then(|| principal.to_ascii_lowercase())
}

/// A extensão do tipo permitido.
pub(super) fn extensao(mime: &str) -> Option<&'static str> {
    PERMITIDOS
        .iter()
        .find(|(tipo, _)| *tipo == mime)
        .map(|(_, ext)| *ext)
}

pub(super) fn e_texto(mime: &str) -> bool {
    mime.starts_with("text/")
}

/// O `Content-Disposition` fora do subconjunto observado da RFC 6266.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroDisposicao {
    AspasSemFechamento,
    TipoDeDisposicaoNaoObservado,
    ParametroSemIgual,
    NomeDeArquivoRepetido,
    NomeSimplesForaDaForma,
    NomeEstendidoSemCharsetOuIdioma,
    PercentualInvalidoNoNomeEstendido,
    CharsetDoNomeEstendidoNaoAceito,
    NomeEstendidoUtf8Invalido,
}

/// Disposição e nome original saneado.
pub(super) fn ler_disposicao(valor: &str) -> Result<(Disposicao, Option<String>), ErroDisposicao> {
    let partes = dividir(valor)?;
    let mut partes = partes.into_iter();
    let tipo = partes.next().unwrap_or_default();
    let disposicao = match tipo.trim().to_ascii_lowercase().as_str() {
        "inline" => Disposicao::Inline,
        "attachment" => Disposicao::Attachment,
        _ => return Err(ErroDisposicao::TipoDeDisposicaoNaoObservado),
    };
    let (mut simples, mut estendido) = (None, None);
    for parte in partes {
        let parte = parte.trim();
        if parte.is_empty() {
            continue;
        }
        let (nome, valor) = parte
            .split_once('=')
            .ok_or(ErroDisposicao::ParametroSemIgual)?;
        let nome = nome.trim().to_ascii_lowercase();
        let valor = valor.trim();
        let alvo = match nome.as_str() {
            "filename" => &mut simples,
            "filename*" => &mut estendido,
            _ => continue,
        };
        if alvo.is_some() {
            return Err(ErroDisposicao::NomeDeArquivoRepetido);
        }
        *alvo = Some(if nome == "filename" {
            sem_aspas(valor)?
        } else {
            estendido_rfc5987(valor)?
        });
    }
    Ok((
        disposicao,
        estendido.or(simples).and_then(|n| sanear_nome(&n)),
    ))
}

/// Divide por `;` fora de aspas; aspas sem fim são recusa.
fn dividir(valor: &str) -> Result<Vec<String>, ErroDisposicao> {
    let (mut partes, mut atual, mut aspas, mut escape) = (Vec::new(), String::new(), false, false);
    for c in valor.chars() {
        if escape {
            escape = false;
        } else if aspas && c == '\\' {
            escape = true;
        } else if c == '"' {
            aspas = !aspas;
        } else if c == ';' && !aspas {
            partes.push(std::mem::take(&mut atual));
            continue;
        }
        atual.push(c);
    }
    if aspas || escape {
        return Err(ErroDisposicao::AspasSemFechamento);
    }
    partes.push(atual);
    Ok(partes)
}

/// `"texto"` com `\x` → `x`, ou um token sem aspas.
fn sem_aspas(valor: &str) -> Result<String, ErroDisposicao> {
    let Some(dentro) = valor.strip_prefix('"') else {
        return if valor.is_empty() || valor.contains('"') {
            Err(ErroDisposicao::NomeSimplesForaDaForma)
        } else {
            Ok(valor.to_owned())
        };
    };
    let dentro = dentro
        .strip_suffix('"')
        .ok_or(ErroDisposicao::NomeSimplesForaDaForma)?;
    let mut saida = String::new();
    let mut escape = false;
    for c in dentro.chars() {
        if escape || c != '\\' {
            saida.push(c);
            escape = false;
        } else {
            escape = true;
        }
    }
    Ok(saida)
}

/// `charset'idioma'%XX…` com charset UTF-8 ou ISO-8859-1.
fn estendido_rfc5987(valor: &str) -> Result<String, ErroDisposicao> {
    let mut campos = valor.splitn(3, '\'');
    let (Some(charset), Some(_idioma), Some(codificado)) =
        (campos.next(), campos.next(), campos.next())
    else {
        return Err(ErroDisposicao::NomeEstendidoSemCharsetOuIdioma);
    };
    let mut bytes = Vec::new();
    let mut resto = codificado.bytes();
    while let Some(b) = resto.next() {
        if b == b'%' {
            let alto = resto.next().and_then(|h| char::from(h).to_digit(16));
            let baixo = resto.next().and_then(|h| char::from(h).to_digit(16));
            let (Some(alto), Some(baixo)) = (alto, baixo) else {
                return Err(ErroDisposicao::PercentualInvalidoNoNomeEstendido);
            };
            bytes.push(
                u8::try_from(alto * 16 + baixo)
                    .map_err(|_| ErroDisposicao::PercentualInvalidoNoNomeEstendido)?,
            );
        } else {
            bytes.push(b);
        }
    }
    match charset.to_ascii_lowercase().as_str() {
        "utf-8" => String::from_utf8(bytes).map_err(|_| ErroDisposicao::NomeEstendidoUtf8Invalido),
        "iso-8859-1" => Ok(bytes.into_iter().map(char::from).collect()),
        _ => Err(ErroDisposicao::CharsetDoNomeEstendidoNaoAceito),
    }
}

/// Último componente (por `/` ou `\`), sem controles, aparado, até
/// [`MAX_NOME`] caracteres; vazio, `.` ou `..` não é nome.
fn sanear_nome(nome: &str) -> Option<String> {
    let base = nome.rsplit(['/', '\\']).next().unwrap_or_default();
    let limpo: String = base.chars().filter(|c| !c.is_control()).collect();
    let limpo = limpo.trim();
    if limpo.is_empty() || limpo == "." || limpo == ".." {
        return None;
    }
    Some(limpo.chars().take(MAX_NOME).collect())
}
