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
    TituloSemNumeroSei,
    TituloSemTipoDocumento,
    IdentificacaoDaInstalacaoAusenteNoTitulo,
}

impl ErroTituloDocumento {
    pub fn subcodigo(self) -> &'static str {
        match self {
            Self::HtmlNaoInterpretado => "html_documento_nao_interpretado",
            Self::TituloAusente => "titulo_documento_ausente",
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
    let titulo = dom
        .query_selector("title")
        .and_then(|mut items| items.next())
        .and_then(|node| node.get(parser))
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ParagrafosDocumento {
    pub documento: IdentidadeDoDocumento,
    pub paragrafos: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErroParagrafosDocumento {
    Titulo(ErroTituloDocumento),
    CorpoDaImpressaoAusente,
    ParagrafosDaImpressaoAusentes,
    TextoDaImpressaoAcimaDoLimite,
    AindaNaoImplementado,
}

impl ErroParagrafosDocumento {
    pub fn subcodigo(self) -> &'static str {
        match self {
            Self::Titulo(erro) => erro.subcodigo(),
            Self::CorpoDaImpressaoAusente => "corpo_da_impressao_ausente",
            Self::ParagrafosDaImpressaoAusentes => "paragrafos_da_impressao_ausentes",
            Self::TextoDaImpressaoAcimaDoLimite => "texto_da_impressao_acima_do_limite",
            Self::AindaNaoImplementado => "paragrafos_da_impressao_nao_implementados",
        }
    }
}

pub fn paragrafos_impressos(_html: &str) -> Result<ParagrafosDocumento, ErroParagrafosDocumento> {
    Err(ErroParagrafosDocumento::AindaNaoImplementado)
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
    fn titulo_generico_com_digitos_nao_e_documento_sei() {
        assert_eq!(
            identidade_do_titulo("<title>Portal - 123 - Página</title>").unwrap_err(),
            ErroTituloDocumento::IdentificacaoDaInstalacaoAusenteNoTitulo
        );
    }
    #[test]
    fn paragrafo_de_impressao_nao_expoe_scripts_nem_url_assinada() {
        let html = "<html><head><title>SEI/UnB - 0011354 - Ofício</title></head>\
            <body><script>var cookie='SEGREDO_SCRIPT';</script>\
            <p>Primeiro parágrafo</p>\
            <p><a href='?infra_hash=SEGREDO_URL'>Segundo parágrafo</a></p></body></html>";
        let obtido = paragrafos_impressos(html).unwrap();
        assert_eq!(obtido.documento.numero, "0011354");
        assert_eq!(obtido.paragrafos, ["Primeiro parágrafo", "Segundo parágrafo"]);
        let saida = serde_json::to_string(&obtido).unwrap();
        assert!(!saida.contains("SEGREDO_SCRIPT"));
        assert!(!saida.contains("SEGREDO_URL"));
    }

    #[test]
    fn corpo_ou_paragrafos_ausentes_sao_falhas_distintas() {
        let titulo = "<title>SEI/UnB - 0011354 - Ofício</title>";
        assert_eq!(
            paragrafos_impressos(titulo).unwrap_err(),
            ErroParagrafosDocumento::CorpoDaImpressaoAusente
        );
        assert_eq!(
            paragrafos_impressos(&format!("{titulo}<body><script>segredo</script></body>")).unwrap_err(),
            ErroParagrafosDocumento::ParagrafosDaImpressaoAusentes
        );
    }

}
