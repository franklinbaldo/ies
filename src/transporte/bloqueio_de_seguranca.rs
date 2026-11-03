//! Página de bloqueio de segurança da instalação (WAF), vista em GO
//! (03/11/2026) depois de rajadas de requisições: HTTP 403, `<title>Acesso
//! Negado</title>` e, no texto visível do corpo, a frase "Sua requisição
//! foi bloqueada por nossa política de segurança", seguida de um contato e
//! do código de bloqueio. A sessão continuou válida. Reconhecida pela
//! estrutura da página (título e frase no texto visível), sem regex; é o
//! reconhecimento de uma tela, como o da tela de login, não a leitura da
//! mensagem para decidir tratamento.
use super::texto_de_recusa::{decodificar_entidades, texto_visivel};

const TITULO: &str = "Acesso Negado";
const FRASE: &str = "bloqueada por nossa política de segurança";

fn normalizado(texto: &str) -> String {
    decodificar_entidades(texto)
        .replace('\u{a0}', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// `true` só para a página de bloqueio; o 403 genérico não é.
pub(crate) fn e_bloqueio_de_seguranca(html: &str) -> bool {
    let Ok(dom) = tl::parse(html, tl::ParserOptions::default()) else {
        return false;
    };
    let parser = dom.parser();
    let primeiro = |elemento: &str| dom.query_selector(elemento).and_then(|mut nos| nos.next());
    let titulo = primeiro("title").map(|h| normalizado(&texto_visivel(h, parser)));
    let corpo = primeiro("body").map(|h| normalizado(&texto_visivel(h, parser)));
    titulo.as_deref() == Some(TITULO) && corpo.is_some_and(|c| c.contains(FRASE))
}
