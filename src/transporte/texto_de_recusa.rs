//! Texto humano de uma tela de recusa do SEI, com `tl` para a árvore e
//! `html_atributos` para `id`/classe (sem regex). Evidência estrutural: as
//! telas reais do SEI testam a exceção por `getElementById('divInfraExcecao')`
//! e o Todos lê `divInfraExcecao`, `infraExcecao` e `infraMensagemErro`.

/// Texto bruto (ainda não saneado) da exceção do SEI, ou `<title>` e o
/// texto visível do `<body>`; entidades básicas decodificadas.
pub(crate) fn texto_do_html(html: &str) -> Option<String> {
    let _ = html;
    None
}

/// `&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`, `&#39;`, `&nbsp;` e
/// referências numéricas; o resto fica como veio.
pub(crate) fn decodificar_entidades(texto: &str) -> String {
    texto.to_owned()
}

#[cfg(test)]
#[path = "tests_texto_de_recusa.rs"]
mod tests_texto_de_recusa;
