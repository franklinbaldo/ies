//! Natureza de um `DOCUMENTO`: o ícone (a7 a a9) e o cancelamento (ícone
//! `documento_cancelado` e dica a6 `Documento Cancelado: <motivo>`), Refs
//! #59, contracts/cli/processo-obter-situacao.md. A dica sai quando difere
//! do rótulo (em e-mails, data, remetente, destinatários e assunto).
use super::Literal;

const ICONE_CANCELADO: &str = "documento_cancelado";
const DICA_CANCELADO: &str = "Documento Cancelado:";

/// `icone`: radical opaco de `svg/<radical>.svg`; `cancelado`: `None` com
/// só uma das duas marcas ou sem ícone lido.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NaturezaDoDocumento {
    pub(crate) icone: Option<String>,
    pub(crate) cancelado: Option<bool>,
    pub(crate) motivo_do_cancelamento: Option<String>,
    /// a6 decodificada quando difere do rótulo (a5); `None` quando igual.
    pub(crate) dica: Option<String>,
}

/// Lê a6 a a9 de um `DOCUMENTO` já com a aridade conferida; forma
/// desconhecida é `None`, nunca recusa.
pub(super) fn ler_natureza(a: &[Literal; super::ARGUMENTOS]) -> NaturezaDoDocumento {
    let icone = match (&a[7], &a[8], &a[9]) {
        (Literal::Texto(a7), Literal::Texto(a8), Literal::Texto(a9)) if a7 == a8 && a8 == a9 => {
            radical(a7)
        }
        _ => None,
    };
    let motivo = match &a[6] {
        // Retire apenas o separador visual depois de ':'; o motivo é opaco.
        // `trim()` perderia espaços do usuário e tornaria a releitura incerta.
        Literal::Texto(dica) => dica
            .strip_prefix(DICA_CANCELADO)
            .map(|resto| resto.strip_prefix(' ').unwrap_or(resto)),
        _ => None,
    };
    let (cancelado, motivo_do_cancelamento) = match (icone.as_deref(), motivo) {
        (Some(ICONE_CANCELADO), Some(motivo)) => (Some(true), Some(motivo.to_owned())),
        (Some(_), None) if icone.as_deref() != Some(ICONE_CANCELADO) => (Some(false), None),
        // Sem ícone lido, ou só uma das duas marcas: não observado.
        _ => (None, None),
    };
    NaturezaDoDocumento {
        icone,
        cancelado,
        motivo_do_cancelamento,
        dica: match (&a[5], &a[6]) {
            (Literal::Texto(rotulo), Literal::Texto(dica)) if dica != rotulo => Some(dica.clone()),
            _ => None,
        },
    }
}

/// `svg/<radical>.svg`, com ou sem `?<decimal>`; radical de minúsculas
/// ASCII, dígitos e `_`.
fn radical(icone: &str) -> Option<String> {
    let resto = icone.strip_prefix("svg/")?;
    let (caminho, versao) = match resto.split_once('?') {
        Some((caminho, versao)) => (caminho, Some(versao)),
        None => (resto, None),
    };
    if versao.is_some_and(|v| v.is_empty() || !v.bytes().all(|b| b.is_ascii_digit())) {
        return None;
    }
    let radical = caminho.strip_suffix(".svg")?;
    let valido = !radical.is_empty()
        && radical
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    valido.then(|| radical.to_owned())
}
