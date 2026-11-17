//! Natureza de um `DOCUMENTO`: o ícone (a7 a a9) e o cancelamento (ícone
//! `documento_cancelado` e dica a6 `Documento Cancelado: <motivo>`), Refs
//! #59, contracts/cli/processo-obter-situacao.md. A dica sai quando difere
//! do rótulo (em e-mails, data, remetente, destinatários e assunto). Ícone
//! fora da forma e cancelamento com uma só marca são erros tipados da
//! leitura (Refs #749): a árvore não é publicada com estado presumido.
use super::{ErroArvore, Literal};

const ICONE_CANCELADO: &str = "documento_cancelado";
const DICA_CANCELADO: &str = "Documento Cancelado:";

/// `icone`: radical opaco de `svg/<radical>.svg`; `cancelamento`: o estado
/// lido, em que o motivo só existe com o documento cancelado.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NaturezaDoDocumento {
    pub(crate) icone: String,
    pub(crate) cancelamento: Cancelamento,
    /// a6 decodificada quando difere do rótulo (a5); `None` quando igual.
    pub(crate) dica: Option<String>,
}

/// Cancelamento lido: as duas marcas (ícone e dica) ou nenhuma.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Cancelamento {
    Ativo,
    /// As duas marcas lidas: o ícone e a dica com o motivo.
    Cancelado {
        motivo: String,
    },
}

impl Cancelamento {
    pub(crate) fn cancelado(&self) -> bool {
        matches!(self, Self::Cancelado { .. })
    }

    pub(crate) fn motivo(&self) -> Option<&str> {
        match self {
            Self::Cancelado { motivo } => Some(motivo),
            Self::Ativo => None,
        }
    }
}

/// Lê a6 a a9 de um `DOCUMENTO` já com a aridade conferida. Ícone fora da
/// forma observada ou cancelamento com uma só marca é recusa tipada.
pub(super) fn ler_natureza(
    a: &[Literal; super::ARGUMENTOS],
) -> Result<NaturezaDoDocumento, ErroArvore> {
    let icone = match (&a[7], &a[8], &a[9]) {
        (Literal::Texto(a7), Literal::Texto(a8), Literal::Texto(a9)) if a7 == a8 && a8 == a9 => {
            radical(a7)
        }
        _ => None,
    }
    .ok_or(ErroArvore::IconeDoDocumentoForaDaFormaObservada)?;
    let motivo = match &a[6] {
        // Retire apenas o separador visual depois de ':'; o motivo é opaco.
        // `trim()` perderia espaços do usuário e tornaria a releitura incerta.
        Literal::Texto(dica) => dica
            .strip_prefix(DICA_CANCELADO)
            .map(|resto| resto.strip_prefix(' ').unwrap_or(resto)),
        Literal::Nulo | Literal::Booleano(_) | Literal::Inteiro(_) => None,
    };
    let cancelamento = match (icone.as_str(), motivo) {
        (ICONE_CANCELADO, Some(motivo)) => Cancelamento::Cancelado {
            motivo: motivo.to_owned(),
        },
        (ICONE_CANCELADO, None) => return Err(ErroArvore::IconeDeCancelamentoSemMotivoNaDica),
        (_, Some(_)) => return Err(ErroArvore::DicaDeCancelamentoSemIconeDeCancelado),
        (_, None) => Cancelamento::Ativo,
    };
    Ok(NaturezaDoDocumento {
        icone,
        cancelamento,
        dica: match (&a[5], &a[6]) {
            (Literal::Texto(rotulo), Literal::Texto(dica)) if dica != rotulo => Some(dica.clone()),
            _ => None,
        },
    })
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
