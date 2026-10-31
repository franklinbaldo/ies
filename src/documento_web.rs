//! Identificação offline de documento pelo título do HTML observado.
//! Não é leitura integral de documento nem consulta da instalação.
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IdentidadeDoDocumento {
    pub numero: String,
    pub tipo: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErroTituloDocumento {
    HtmlNaoInterpretado,
    TituloAusente,
    TitulosDeDocumentoRepetidos,
    TituloSemNumeroSei,
    TituloSemTipoDocumento,
    IdentificacaoDaInstalacaoAusenteNoTitulo,
}

impl ErroTituloDocumento {
    pub fn subcodigo(self) -> &'static str {
        match self {
            Self::HtmlNaoInterpretado => "html_documento_nao_interpretado",
            Self::TituloAusente => "titulo_documento_ausente",
            Self::TitulosDeDocumentoRepetidos => "titulos_de_documento_repetidos",
            Self::TituloSemNumeroSei => "titulo_documento_sem_numero_sei",
            Self::TituloSemTipoDocumento => "titulo_documento_sem_tipo",
            Self::IdentificacaoDaInstalacaoAusenteNoTitulo => {
                "titulo_documento_sem_identificacao_sei"
            }
        }
    }
}

pub fn identidade_do_titulo(html: &str) -> Result<IdentidadeDoDocumento, ErroTituloDocumento> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroTituloDocumento::HtmlNaoInterpretado)?;
    let parser = dom.parser();
    let mut titulos = dom
        .query_selector("title")
        .ok_or(ErroTituloDocumento::TituloAusente)?;
    let primeiro = titulos.next().ok_or(ErroTituloDocumento::TituloAusente)?;
    if titulos.next().is_some() {
        return Err(ErroTituloDocumento::TitulosDeDocumentoRepetidos);
    }
    let titulo = primeiro
        .get(parser)
        .map(|node| node.inner_text(parser).into_owned())
        .ok_or(ErroTituloDocumento::TituloAusente)?;
    let mut campos = titulo.splitn(3, " - ");
    let instalacao = campos.next().unwrap_or_default();
    if !instalacao.starts_with("SEI/") || instalacao.len() <= 4 {
        return Err(ErroTituloDocumento::IdentificacaoDaInstalacaoAusenteNoTitulo);
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
            ErroTituloDocumento::IdentificacaoDaInstalacaoAusenteNoTitulo
        );
    }
    #[test]
    fn dois_titulos_sei_na_mesma_resposta_nao_escolhem_primeiro_documento() {
        let html = "<html><head><title>SEI/UnB - 0007 - Ofício</title><title>SEI/UnB - 0008 - Memorando</title></head></html>";
        assert_eq!(
            identidade_do_titulo(html),
            Err(ErroTituloDocumento::TitulosDeDocumentoRepetidos)
        );
    }
    #[test]
    fn titulo_generico_com_digitos_nao_e_documento_sei() {
        assert_eq!(
            identidade_do_titulo("<title>Portal - 123 - Página</title>").unwrap_err(),
            ErroTituloDocumento::IdentificacaoDaInstalacaoAusenteNoTitulo
        );
    }
}

/// Erros de extração textual; não convertidos em mensagens livres.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErroCorpoDocumento {
    HtmlNaoInterpretado,
    CorpoAusente,
    CorposDeDocumentoRepetidos,
    CorpoVazio,
    CorpoComConteudoExecutavel,
    TextoAcimaDoLimite,
    AninhamentoExcessivo,
}
impl ErroCorpoDocumento {
    pub fn subcodigo(self) -> &'static str {
        match self {
            Self::HtmlNaoInterpretado => "html_corpo_documento_nao_interpretado",
            Self::CorpoAusente => "corpo_documento_ausente",
            Self::CorposDeDocumentoRepetidos => "corpos_de_documento_repetidos",
            Self::CorpoVazio => "corpo_documento_vazio",
            Self::CorpoComConteudoExecutavel => "corpo_documento_contem_script_ou_estilo",
            Self::TextoAcimaDoLimite => "texto_documento_acima_do_limite",
            Self::AninhamentoExcessivo => "arvore_documento_aninhamento_excessivo",
        }
    }
}
/// Percorre nós do corpo sem executar scripts; separa blocos visíveis.
fn anexar_texto(
    no: &tl::Node<'_>,
    parser: &tl::Parser<'_>,
    saida: &mut String,
    profundidade: usize,
) -> Result<(), ErroCorpoDocumento> {
    if profundidade > 256 {
        return Err(ErroCorpoDocumento::AninhamentoExcessivo);
    }
    match no {
        tl::Node::Raw(valor) => saida.push_str(valor.as_utf8_str().as_ref()),
        tl::Node::Comment(_) => {}
        tl::Node::Tag(tag) => {
            let nome = tag.name().as_utf8_str();
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
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroCorpoDocumento::HtmlNaoInterpretado)?;
    let parser = dom.parser();
    let mut corpos = dom
        .query_selector("body")
        .ok_or(ErroCorpoDocumento::CorpoAusente)?;
    let primeiro = corpos.next().ok_or(ErroCorpoDocumento::CorpoAusente)?;
    if corpos.next().is_some() {
        return Err(ErroCorpoDocumento::CorposDeDocumentoRepetidos);
    }
    let corpo = primeiro
        .get(parser)
        .ok_or(ErroCorpoDocumento::CorpoAusente)?;
    let tag = corpo.as_tag().ok_or(ErroCorpoDocumento::CorpoAusente)?;
    for nome in ["script", "style", "iframe", "noscript"] {
        if tag
            .query_selector(parser, nome)
            .is_some_and(|mut nodes| nodes.next().is_some())
        {
            return Err(ErroCorpoDocumento::CorpoComConteudoExecutavel);
        }
    }
    let mut bruto = String::new();
    anexar_texto(corpo, parser, &mut bruto, 0)?;
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
mod testes_texto {
    use super::*;
    #[test]
    fn extrai_somente_texto_visivel_de_paragrafos() {
        let html = "<html><head><title>SEI/UnB - 0007 - Ofício</title><style>SEGREDO_CSS</style></head><body><p>Olá,</p><p>mundo!</p></body></html>";
        assert_eq!(texto_do_corpo(html), Ok("Olá, mundo!".to_owned()));
    }
    #[test]
    fn corpo_sem_texto_tem_erro_tipado() {
        assert_eq!(
            texto_do_corpo("<html><body></body></html>"),
            Err(ErroCorpoDocumento::CorpoVazio)
        );
    }
    #[test]
    fn corpo_ausente_tem_erro_distinto() {
        assert_eq!(
            texto_do_corpo("<html><head></head></html>"),
            Err(ErroCorpoDocumento::CorpoAusente)
        );
    }
    #[test]
    fn duas_areas_de_corpo_nao_omitem_texto_silenciosamente() {
        let html = "<html><body><p>Primeiro documento</p></body><body><p>Segundo documento</p></body></html>";
        assert_eq!(
            texto_do_corpo(html),
            Err(ErroCorpoDocumento::CorposDeDocumentoRepetidos)
        );
    }
    #[test]
    fn recusa_scripts_no_corpo_para_nao_expor_tokens() {
        let html = "<body><script>infra_hash=SEGREDO</script><p>Conteúdo</p></body>";
        assert_eq!(
            texto_do_corpo(html),
            Err(ErroCorpoDocumento::CorpoComConteudoExecutavel)
        );
    }
}
