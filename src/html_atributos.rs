//! Compatibilidade dos nomes internos: parser de atributos isolado no crate ies-web.
pub(crate) use ies_web::html_atributos::{
    ErroAtributoHtml, atributo_presente, atributos, nos_com_id, nos_com_id_em,
    nos_por_elemento_e_classe, nos_por_elemento_e_classe_com_repeticao_identica,
    nos_por_elemento_e_classe_em_todas_as_ocorrencias, valor_do_atributo,
};

#[cfg(test)]
fn eh_espaco_html(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\u{c}' | '\r' | ' ')
}

/// Insere o booleano sem valor `itemscope` logo após o nome de cada tag de
/// abertura com atributos. No `tl` 0.7.8 isso corrompe o atributo seguinte;
/// cada leitor tem de devolver o mesmo resultado do HTML original.
#[cfg(test)]
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
            if fim < bytes.len() && eh_espaco_html(char::from(bytes[fim])) {
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
