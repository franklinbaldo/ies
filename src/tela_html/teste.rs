//! Auxiliares de teste dos leitores de tela.

/// Insere o booleano sem valor `itemscope` logo após o nome de cada tag de
/// abertura com atributos. No `tl` 0.7.8 isso corrompe o atributo seguinte;
/// cada leitor tem de devolver o mesmo resultado do HTML original.
pub(crate) fn com_booleano_antes_dos_atributos(html: &str) -> String {
    let bytes = html.as_bytes();
    let mut saida = String::with_capacity(html.len() * 2);
    let mut inicio = 0;
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'<' && bytes[i + 1].is_ascii_alphabetic() {
            let mut fim = i + 1;
            while fim < bytes.len() && bytes[fim].is_ascii_alphanumeric() {
                fim += 1;
            }
            if fim < bytes.len() && matches!(bytes[fim], b'\t' | b'\n' | 0x0c | b'\r' | b' ') {
                saida.push_str(&html[inicio..fim]);
                saida.push_str(" itemscope");
                inicio = fim;
            }
            i = fim;
        } else {
            i += 1;
        }
    }
    saida.push_str(&html[inicio..]);
    saida
}
