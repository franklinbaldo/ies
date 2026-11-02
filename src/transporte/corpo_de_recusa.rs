//! Corpo de uma recusa HTTP (status diferente de 200, fora 3xx): lido uma
//! única vez, até `LIMITE`, só para expor o que o SEI disse. Nunca decide
//! tratamento; o subcódigo da `FalhaTransporte` é a classificação.
//! Contrato: contracts/cli.md, "Status e mensagem do SEI nas recusas HTTP".
use std::io::Read;

/// Bytes do corpo da recusa lidos no máximo; o resto nunca é lido.
pub(crate) const LIMITE: usize = 64 * 1024;

/// Mensagem saneada do corpo, ou `None` se nada legível sobrou.
pub(crate) fn mensagem<R: Read>(corpo: R, mime: &str) -> Option<String> {
    let bytes = ler_ate_o_limite(corpo)?;
    let texto = decodificar(&bytes, mime);
    let bruto = if crate::mime_web::e_html(mime) {
        super::texto_de_recusa::texto_do_html(&texto)?
    } else {
        texto
    };
    super::saneamento::sanear(&bruto)
}

/// Até `LIMITE` bytes, em fluxo; erro de leitura não mascara o status.
pub(crate) fn ler_ate_o_limite<R: Read>(corpo: R) -> Option<Vec<u8>> {
    let _ = corpo;
    None
}

/// Charset ISO-8859-1 declarado vira ISO-8859-1; senão UTF-8, e bytes
/// inválidos caem para ISO-8859-1. Um caractere UTF-8 cortado pelo limite
/// no fim é descartado.
pub(crate) fn decodificar(bytes: &[u8], mime: &str) -> String {
    let _ = (bytes, mime);
    String::new()
}

#[cfg(test)]
#[path = "tests_corpo_de_recusa.rs"]
mod tests_corpo_de_recusa;
