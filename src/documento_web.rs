//! Identificação offline de documento pelo título do HTML observado.
//! Não é leitura integral de documento nem consulta da instalação.
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IdentidadeDoDocumento {
    pub numero: String,
    pub tipo: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, crate::erro_publico::Causa)]
pub enum ErroTituloDocumento {
    #[causa(nome = "titulo_documento_sem_numero_sei")]
    TituloSemNumeroSei,
    #[causa(nome = "titulo_documento_sem_tipo")]
    TituloSemTipoDocumento,
    #[causa(interna)]
    Html(crate::erro_html::ErroHtml),
}

impl ErroTituloDocumento {}

pub fn identidade_do_titulo(html: &str) -> Result<IdentidadeDoDocumento, ErroTituloDocumento> {
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| {
        ErroTituloDocumento::Html(crate::erro_html::ErroHtml::html(
            "html_documento_nao_interpretado",
            "o HTML não pôde ser interpretado",
        ))
    })?;
    let parser = dom.parser();
    // O <title> de SVG no corpo descreve um ícone, não um documento SEI.
    // Em um HTML com <head>, é esse cabeçalho que define a identidade.
    let mut cabecalhos = dom.query_selector("head").into_iter().flatten();
    let titulos: Vec<_> =
        if let Some(cabecalho) = cabecalhos.next() {
            if cabecalhos.next().is_some() {
                return Err(ErroTituloDocumento::Html(
                    crate::erro_html::ErroHtml::repetido(
                        "cabecalhos_html_repetidos",
                        "o elemento aparece mais de uma vez",
                    ),
                ));
            }
            let tag = cabecalho.get(parser).and_then(tl::Node::as_tag).ok_or(
                ErroTituloDocumento::Html(crate::erro_html::ErroHtml::ausente(
                    "titulo_documento_ausente",
                    "a tela não tem o elemento observado",
                )),
            )?;
            tag.query_selector(parser, "title")
                .into_iter()
                .flatten()
                .collect()
        } else {
            // Fragmentos sem head admitem title externo, mas nunca aquele
            // inserido no corpo, em ícones SVG ou em template inerte.
            let mut titulos_de_conteudo = Vec::new();
            for nome in ["body", "svg", "template"] {
                for area in dom.query_selector(nome).into_iter().flatten() {
                    let Some(tag) = area.get(parser).and_then(tl::Node::as_tag) else {
                        continue;
                    };
                    titulos_de_conteudo
                        .extend(tag.query_selector(parser, "title").into_iter().flatten());
                }
            }
            dom.query_selector("title")
                .into_iter()
                .flatten()
                .filter(|titulo| !titulos_de_conteudo.contains(titulo))
                .collect()
        };
    let mut titulos = titulos.into_iter();
    let primeiro = titulos.next().ok_or(ErroTituloDocumento::Html(
        crate::erro_html::ErroHtml::ausente(
            "titulo_documento_ausente",
            "a tela não tem o elemento observado",
        ),
    ))?;
    if titulos.next().is_some() {
        return Err(ErroTituloDocumento::Html(
            crate::erro_html::ErroHtml::repetido(
                "titulos_de_documento_repetidos",
                "o elemento aparece mais de uma vez",
            ),
        ));
    }
    let titulo = primeiro
        .get(parser)
        .map(|node| node.inner_text(parser).into_owned())
        .ok_or(ErroTituloDocumento::Html(
            crate::erro_html::ErroHtml::ausente(
                "titulo_documento_ausente",
                "a tela não tem o elemento observado",
            ),
        ))?;
    let mut campos = titulo.splitn(3, " - ");
    let instalacao = campos.next().unwrap_or_default();
    if !instalacao.starts_with("SEI/") || instalacao.len() <= 4 {
        return Err(ErroTituloDocumento::Html(
            crate::erro_html::ErroHtml::ausente(
                "titulo_documento_sem_identificacao_sei",
                "a tela não tem o elemento observado",
            ),
        ));
    }
    let numero = campos
        .next()
        .ok_or(ErroTituloDocumento::TituloSemNumeroSei)?;
    if numero.is_empty() || !numero.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ErroTituloDocumento::TituloSemNumeroSei);
    }
    let tipo = campos
        .next()
        .ok_or(ErroTituloDocumento::TituloSemTipoDocumento)?;
    if tipo.trim().is_empty() {
        return Err(ErroTituloDocumento::TituloSemTipoDocumento);
    }
    Ok(IdentidadeDoDocumento {
        numero: numero.to_owned(),
        tipo: tipo.trim().to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn titulo_dentro_de_svg_sem_head_nao_identifica_documento() {
        let html = "<svg><title>SEI/UnB - 0007 - Ofício</title></svg>";
        assert_eq!(
            identidade_do_titulo(html),
            Err(ErroTituloDocumento::Html(
                crate::erro_html::ErroHtml::ausente(
                    "titulo_documento_ausente",
                    "a tela não tem o elemento observado"
                )
            ))
        );
    }

    #[test]
    fn titulo_em_body_ou_template_sem_head_eh_ignorado() {
        for html in [
            "<body><title>SEI/UnB - 0007 - Ofício</title></body>",
            "<template><title>SEI/UnB - 0007 - Ofício</title></template>",
        ] {
            assert_eq!(
                identidade_do_titulo(html),
                Err(ErroTituloDocumento::Html(
                    crate::erro_html::ErroHtml::ausente(
                        "titulo_documento_ausente",
                        "a tela não tem o elemento observado"
                    )
                ))
            );
        }
    }

    #[test]
    fn titulo_externo_em_fragmento_sem_head_eh_preservado() {
        let html = "<title>SEI/UnB - 0007 - Ofício</title>\
                    <body><svg><title>SEI/UnB - 9999 - Outro</title></svg></body>";
        assert_eq!(identidade_do_titulo(html).unwrap().numero, "0007");
    }

    #[test]
    fn dois_titulos_externos_sem_head_nao_escolhem_primeiro() {
        let html = "<title>SEI/UnB - 0007 - Ofício</title>\
                    <title>SEI/UnB - 0008 - Memorando</title>";
        assert_eq!(
            identidade_do_titulo(html),
            Err(ErroTituloDocumento::Html(
                crate::erro_html::ErroHtml::repetido(
                    "titulos_de_documento_repetidos",
                    "o elemento aparece mais de uma vez"
                )
            ))
        );
    }

    #[test]
    fn preserva_zeros_na_identidade_observada() {
        let html = "<html><head><title>SEI/UnB - 0011354 - Ofício</title></head></html>";
        let identidade = identidade_do_titulo(html).unwrap();
        assert_eq!(identidade.numero, "0011354");
        assert_eq!(identidade.tipo, "Ofício");
    }

    #[test]
    fn titulo_sem_numero_nao_fabrica_documento() {
        assert_eq!(
            identidade_do_titulo("<title>SEI - Documento</title>").unwrap_err(),
            ErroTituloDocumento::Html(crate::erro_html::ErroHtml::ausente(
                "titulo_documento_sem_identificacao_sei",
                "a tela não tem o elemento observado"
            ))
        );
    }
    #[test]
    fn dois_titulos_sei_na_mesma_resposta_nao_escolhem_primeiro_documento() {
        let html = "<html><head><title>SEI/UnB - 0007 - Ofício</title><title>SEI/UnB - 0008 - Memorando</title></head></html>";
        assert_eq!(
            identidade_do_titulo(html),
            Err(ErroTituloDocumento::Html(
                crate::erro_html::ErroHtml::repetido(
                    "titulos_de_documento_repetidos",
                    "o elemento aparece mais de uma vez"
                )
            ))
        );
    }
    #[test]
    fn titulo_de_svg_no_corpo_nao_duplica_titulo_html_do_documento() {
        let html = "<html><head><title>SEI/UnB - 0007 - Ofício</title></head>
                    <body><svg><title>Ícone de navegação</title></svg><p>Texto</p></body></html>";
        let identidade = identidade_do_titulo(html).unwrap();
        assert_eq!(identidade.numero, "0007");
        assert_eq!(identidade.tipo, "Ofício");
    }

    #[test]
    fn titulo_svg_nao_suprimenta_head_html_sem_titulo() {
        let html = "<html><head><meta charset='utf-8'></head>
                    <body><svg><title>SEI/UnB - 0007 - Ofício</title></svg></body></html>";
        assert_eq!(
            identidade_do_titulo(html),
            Err(ErroTituloDocumento::Html(
                crate::erro_html::ErroHtml::ausente(
                    "titulo_documento_ausente",
                    "a tela não tem o elemento observado"
                )
            ))
        );
    }

    #[test]
    fn dois_heads_html_nao_podem_selecionar_o_primeiro_titulo() {
        let html = "<html><head><title>SEI/UnB - 0007 - Ofício</title></head>
                    <head><title>SEI/UnB - 0008 - Memorando</title></head></html>";
        assert_eq!(
            identidade_do_titulo(html),
            Err(ErroTituloDocumento::Html(
                crate::erro_html::ErroHtml::repetido(
                    "cabecalhos_html_repetidos",
                    "o elemento aparece mais de uma vez"
                )
            ))
        );
    }

    #[test]
    fn dois_titulos_no_mesmo_head_permanecem_recusados() {
        let html = "<html><head><title>SEI/UnB - 0007 - Ofício</title>
                    <title>SEI/UnB - 0008 - Memorando</title></head></html>";
        assert_eq!(
            identidade_do_titulo(html),
            Err(ErroTituloDocumento::Html(
                crate::erro_html::ErroHtml::repetido(
                    "titulos_de_documento_repetidos",
                    "o elemento aparece mais de uma vez"
                )
            ))
        );
    }

    #[test]
    fn titulo_generico_com_digitos_nao_e_documento_sei() {
        assert_eq!(
            identidade_do_titulo("<title>Portal - 123 - Página</title>").unwrap_err(),
            ErroTituloDocumento::Html(crate::erro_html::ErroHtml::ausente(
                "titulo_documento_sem_identificacao_sei",
                "a tela não tem o elemento observado"
            ))
        );
    }
}

/// Erros de extração textual; não convertidos em mensagens livres.
#[derive(Debug, Clone, Copy, PartialEq, Eq, crate::erro_publico::Causa)]
pub enum ErroCorpoDocumento {
    #[causa(nome = "corpo_documento_vazio")]
    CorpoVazio,
    #[causa(nome = "corpo_documento_contem_script_ou_estilo")]
    CorpoComConteudoExecutavel,
    #[causa(nome = "texto_documento_acima_do_limite")]
    TextoAcimaDoLimite,
    #[causa(interna)]
    Html(crate::erro_html::ErroHtml),
}
impl ErroCorpoDocumento {}
/// Ícones SVG e templates não renderizados não compõem o texto do documento.
fn ignorar_subarvore_decorativa(nome: &str) -> bool {
    nome.eq_ignore_ascii_case("svg") || nome.eq_ignore_ascii_case("template")
}

/// Percorre nós do corpo sem executar scripts; separa blocos visíveis.
fn anexar_texto(
    no: &tl::Node<'_>,
    parser: &tl::Parser<'_>,
    saida: &mut String,
    profundidade: usize,
) -> Result<(), ErroCorpoDocumento> {
    if profundidade > 256 {
        return Err(ErroCorpoDocumento::Html(crate::erro_html::ErroHtml::valor(
            "arvore_documento_aninhamento_excessivo",
            "o valor não está na forma observada",
        )));
    }
    match no {
        tl::Node::Raw(valor) => saida.push_str(valor.as_utf8_str().as_ref()),
        tl::Node::Comment(_) => {}
        tl::Node::Tag(tag) => {
            let nome = tag.name().as_utf8_str();
            if ignorar_subarvore_decorativa(nome.as_ref()) {
                return Ok(());
            }
            if matches!(nome.as_ref(), "script" | "style" | "iframe" | "noscript") {
                return Err(ErroCorpoDocumento::CorpoComConteudoExecutavel);
            }
            let quebra = matches!(
                nome.as_ref(),
                "p" | "div"
                    | "li"
                    | "br"
                    | "tr"
                    | "td"
                    | "th"
                    | "table"
                    | "h1"
                    | "h2"
                    | "h3"
                    | "h4"
                    | "h5"
                    | "h6"
                    | "section"
                    | "article"
            );
            if quebra {
                saida.push(' ');
            }
            for filho in tag.children().top().iter() {
                if let Some(no) = filho.get(parser) {
                    anexar_texto(no, parser, saida, profundidade + 1)?;
                }
            }
            if quebra {
                saida.push(' ');
            }
        }
    }
    if saida.len() > 1_000_000 {
        return Err(ErroCorpoDocumento::TextoAcimaDoLimite);
    }
    Ok(())
}

/// Extrai o texto do body observado sem executar HTML nem serializar atributos.
/// Recusa conteúdo executável no corpo para evitar expor scripts ou segredos.
pub fn texto_do_corpo(html: &str) -> Result<String, ErroCorpoDocumento> {
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| {
        ErroCorpoDocumento::Html(crate::erro_html::ErroHtml::html(
            "html_corpo_documento_nao_interpretado",
            "o HTML não pôde ser interpretado",
        ))
    })?;
    let parser = dom.parser();
    let mut corpos = dom.query_selector("body").ok_or(ErroCorpoDocumento::Html(
        crate::erro_html::ErroHtml::ausente(
            "corpo_documento_ausente",
            "a tela não tem o elemento observado",
        ),
    ))?;
    let primeiro = corpos.next().ok_or(ErroCorpoDocumento::Html(
        crate::erro_html::ErroHtml::ausente(
            "corpo_documento_ausente",
            "a tela não tem o elemento observado",
        ),
    ))?;
    if corpos.next().is_some() {
        return Err(ErroCorpoDocumento::Html(
            crate::erro_html::ErroHtml::repetido(
                "corpos_de_documento_repetidos",
                "o elemento aparece mais de uma vez",
            ),
        ));
    }
    let corpo = primeiro.get(parser).ok_or(ErroCorpoDocumento::Html(
        crate::erro_html::ErroHtml::ausente(
            "corpo_documento_ausente",
            "a tela não tem o elemento observado",
        ),
    ))?;
    if corpo.as_tag().is_none() {
        return Err(ErroCorpoDocumento::Html(
            crate::erro_html::ErroHtml::ausente(
                "corpo_documento_ausente",
                "a tela não tem o elemento observado",
            ),
        ));
    }
    let mut bruto = String::new();
    anexar_texto(corpo, parser, &mut bruto, 0)?;
    // Entidades decodificadas antes de juntar espaços: `&nbsp;` é espaço.
    let bruto = crate::entidades_html::decodificar_texto(&bruto);
    let texto = bruto.split_whitespace().collect::<Vec<_>>().join(" ");
    if texto.is_empty() {
        return Err(ErroCorpoDocumento::CorpoVazio);
    }
    if texto.len() > 250_000 {
        return Err(ErroCorpoDocumento::TextoAcimaDoLimite);
    }
    Ok(texto)
}
#[cfg(test)]
#[path = "documento_web/testes_texto.rs"]
mod testes_texto;
