//! Corpo de uma recusa HTTP (status diferente de 200, fora 3xx): lido uma
//! única vez, até `LIMITE`, só para expor o que o SEI disse. Nunca decide
//! tratamento; o erro da `FalhaTransporte` é a classificação.
//! Contrato: contracts/cli.md, "Status e mensagem do SEI nas recusas HTTP".
use std::io::Read;

/// Bytes do corpo da recusa lidos no máximo; o resto nunca é lido.
pub(crate) const LIMITE: usize = 64 * 1024;

/// O que a única leitura do corpo rende: a mensagem saneada e se a
/// página é a de bloqueio de segurança da instalação.
pub(crate) struct CorpoLido {
    pub(crate) mensagem: Option<String>,
    pub(crate) bloqueio_de_seguranca: bool,
}

/// Lê o corpo uma vez, até `LIMITE`.
pub(crate) fn ler<R: Read>(corpo: R, mime: &str) -> CorpoLido {
    let Some(bytes) = ler_ate_o_limite(corpo) else {
        return CorpoLido {
            mensagem: None,
            bloqueio_de_seguranca: false,
        };
    };
    let texto = decodificar(&bytes, mime);
    let (bruto, bloqueio_de_seguranca) = if crate::mime_web::e_html(mime) {
        // Até `LIMITE` bytes: o texto cabe no `HtmlServido`; se não
        // coubesse, seria um corpo sem mensagem legível.
        match crate::tela_html::HtmlServido::novo(texto) {
            Ok(html) => (
                super::texto_de_recusa::texto_do_html(&html),
                super::bloqueio_de_seguranca::e_bloqueio_de_seguranca(&html),
            ),
            Err(crate::tela_html::HtmlAcimaDoLimite) => (None, false),
        }
    } else {
        (Some(texto), false)
    };
    CorpoLido {
        mensagem: bruto.and_then(|b| super::saneamento::sanear(&b)),
        bloqueio_de_seguranca,
    }
}

/// Mensagem saneada do corpo, ou `None` se nada legível sobrou.
#[cfg(test)]
pub(crate) fn mensagem<R: Read>(corpo: R, mime: &str) -> Option<String> {
    ler(corpo, mime).mensagem
}

/// Até `LIMITE` bytes, em fluxo; erro de leitura não mascara o status.
pub(crate) fn ler_ate_o_limite<R: Read>(corpo: R) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    // `take` limita o que se pede ao leitor: nenhum byte além de `LIMITE`.
    corpo
        .take(u64::try_from(LIMITE).unwrap_or(u64::MAX))
        .read_to_end(&mut bytes)
        .ok()?;
    Some(bytes)
}

/// Charset ISO-8859-1 declarado vira ISO-8859-1; senão UTF-8, e bytes
/// inválidos caem para ISO-8859-1. Um caractere UTF-8 cortado pelo limite
/// no fim é descartado.
pub(crate) fn decodificar(bytes: &[u8], mime: &str) -> String {
    let declarado = mime.to_ascii_lowercase();
    if ["8859-1", "latin1", "latin-1", "windows-1252", "cp1252"]
        .iter()
        .any(|nome| declarado.contains(nome))
    {
        return iso_8859_1(bytes);
    }
    match std::str::from_utf8(bytes) {
        Ok(texto) => texto.to_owned(),
        // Sequência incompleta só no fim: o corte do limite; o resto vale.
        Err(erro) if erro.error_len().is_none() => {
            String::from_utf8_lossy(bytes.split_at(erro.valid_up_to()).0).into_owned()
        }
        Err(_) => iso_8859_1(bytes),
    }
}

fn iso_8859_1(bytes: &[u8]) -> String {
    bytes.iter().copied().map(char::from).collect()
}

#[cfg(test)]
#[path = "tests_corpo_de_recusa.rs"]
mod tests_corpo_de_recusa;
