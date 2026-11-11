//! Página de bloqueio de segurança da instalação (WAF), vista em GO
//! (03/11/2026) depois de rajadas de requisições: HTTP 403, `<title>Acesso
//! Negado</title>` e, no texto visível do corpo, a frase "Sua requisição
//! foi bloqueada por nossa política de segurança", seguida de um contato e
//! do código de bloqueio. A sessão continuou válida. Reconhecida pela
//! estrutura da página (título e frase no texto visível), sem regex; é o
//! reconhecimento de uma tela, como o da tela de login, não a leitura da
//! mensagem para decidir tratamento.
//!
//! Na PEN2 (Tramita, 04/11/2026), o Cloudflare devolveu no segundo GET de
//! `ies login` um 403 com `<title>Attention Required! | Cloudflare</title>`
//! e "Sorry, you have been blocked" no texto visível: o mesmo fato, outro
//! par de título e frase. A mesma página veio na PEN3 no POST da
//! pesquisa com termo só de dígitos; a pesquisa seguinte passou.
use super::texto_de_recusa::texto_visivel;

/// (título exato, frase no texto visível do corpo), por página observada.
const PAGINAS: [(&str, &str); 2] = [
    ("Acesso Negado", "bloqueada por nossa política de segurança"),
    (
        "Attention Required! | Cloudflare",
        "Sorry, you have been blocked",
    ),
];

fn normalizado(texto: &str) -> String {
    html_escape::decode_html_entities(texto)
        .into_owned()
        .replace('\u{a0}', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// `true` só para a página de bloqueio; o 403 genérico não é.
pub(crate) fn e_bloqueio_de_seguranca(html: &crate::tela_html::HtmlServido) -> bool {
    let dom = crate::tela_html::interpretar(html);
    let parser = dom.parser();
    let primeiro = |elemento: &str| dom.query_selector(elemento).and_then(|mut nos| nos.next());
    let titulo = primeiro("title").map(|h| normalizado(&texto_visivel(h, parser)));
    let corpo = primeiro("body").map(|h| normalizado(&texto_visivel(h, parser)));
    PAGINAS.iter().any(|(t, frase)| {
        titulo.as_deref() == Some(*t) && corpo.as_deref().is_some_and(|c| c.contains(frase))
    })
}
