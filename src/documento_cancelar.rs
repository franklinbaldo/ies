//! Cancelamento Web de documento (#273): link do nó DOCUMENTO e motivo.
//! Deriva endereços somente de âncoras servidas; nunca executa JavaScript.
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::barra_do_no;
use crate::arvore_processo::endereco::{EnderecoRecusado, conferir_endereco_relativo_por};
use crate::consulta_do_controlador::menciona_acao;
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::atributos;
use crate::instalacao::Instalacao;

#[cfg(test)]
mod acervo;
mod erro;
mod fluxo;
#[cfg(test)]
mod tests_fluxo;

#[cfg(test)]
pub(crate) use erro::{ErroCancelar, FalhaDoCancelamento};
#[cfg(test)]
pub(crate) use fluxo::ResultadoDaVerificacao;
pub(crate) use fluxo::{DocumentoCancelado, PedidoDoCancelamento, cancelar_documento};

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroLinkCancelar {
    /// O tokenizador de atributos recusou a tag.
    Atributo(#[source] crate::html_atributos::ErroAtributoHtml),
    BarraForaDaForma,
    /// A barra do documento não pôde ser lida do HTML da tela.
    BarraDoDocumentoIlegivel(#[source] crate::arvore_processo::ErroBarraDoDocumento),
    LinksRepetidos,
    EnderecoNaoReconhecido,
    EnderecoDeOutroDocumento,
}

/// Um link assinado validado; Debug nunca revela a URL.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct LinkDeCancelamento(String);
impl LinkDeCancelamento {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}
impl std::fmt::Debug for LinkDeCancelamento {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("LinkDeCancelamento(<redigido>)")
    }
}

/// Único href `documento_cancelar` dentro da barra do documento pedido.
pub(crate) fn ler_link_cancelar(
    instalacao: &Instalacao,
    html: &str,
    documento: &str,
    processo: &str,
) -> Result<Option<LinkDeCancelamento>, ErroLinkCancelar> {
    let Some(barra) =
        barra_do_no(html, documento).map_err(ErroLinkCancelar::BarraDoDocumentoIlegivel)?
    else {
        return Ok(None);
    };
    let dom = crate::tela_html::interpretar(&barra);
    let parser = dom.parser();
    let mut links = Vec::new();
    for no in dom.query_selector("a").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let attrs = atributos(tag).map_err(ErroLinkCancelar::Atributo)?;
        let Some(bruto) = attrs.valor("href") else {
            continue;
        };
        let href = decodificar_atributo(bruto).ok_or(ErroLinkCancelar::BarraForaDaForma)?;
        if menciona_acao(&href, AcaoDoSei::DocumentoCancelar) {
            links.push(href);
        }
    }
    let href = match links.as_slice() {
        [] => return Ok(None),
        [unico] => unico,
        _ => return Err(ErroLinkCancelar::LinksRepetidos),
    };
    let mapear = |r| match r {
        EnderecoRecusado::NaoReconhecido => ErroLinkCancelar::EnderecoNaoReconhecido,
        EnderecoRecusado::DeOutroProcesso => ErroLinkCancelar::EnderecoDeOutroDocumento,
    };
    conferir_endereco_relativo_por(
        instalacao,
        href,
        AcaoDoSei::DocumentoCancelar,
        "id_procedimento",
        processo,
    )
    .map_err(mapear)?;
    let url = conferir_endereco_relativo_por(
        instalacao,
        href,
        AcaoDoSei::DocumentoCancelar,
        "id_documento",
        documento,
    )
    .map_err(mapear)?;
    Ok(Some(LinkDeCancelamento(url)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arvore_processo::tests_assinatura::arvore_assinavel;

    fn instalacao() -> Instalacao {
        Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap()
    }
    fn link(doc: &str) -> String {
        format!(
            "<a href=\"controlador.php?acao=documento_cancelar&acao_origem=arvore_visualizar&id_procedimento=10&id_documento={doc}&infra_hash={}\">Cancelar Documento</a>",
            "a".repeat(64)
        )
    }
    fn arvore(acoes: &str) -> String {
        arvore_assinavel("10", "21", "0000101", acoes, &[])
    }

    // Caso de todos tests/test_documento_mover_cancelar_010.py @323eb01.
    #[test]
    fn reconhece_o_link_assinado_so_na_barra_do_documento_pedido() {
        let url = ler_link_cancelar(&instalacao(), &arvore(&link("21")), "21", "10")
            .unwrap()
            .unwrap();
        assert!(url.as_str().contains("acao=documento_cancelar"));
        assert!(!format!("{url:?}").contains("infra_hash"));
    }

    // Caso de todos tests/test_documento_mover_cancelar_010.py @323eb01.
    #[test]
    fn recusa_link_assinado_para_outro_documento_antes_da_escrita() {
        assert_eq!(
            ler_link_cancelar(&instalacao(), &arvore(&link("99")), "21", "10"),
            Err(ErroLinkCancelar::EnderecoDeOutroDocumento)
        );
    }

    #[test]
    fn recusa_duas_acoes_de_cancelamento_na_mesma_barra() {
        assert_eq!(
            ler_link_cancelar(
                &instalacao(),
                &arvore(&(link("21") + &link("21"))),
                "21",
                "10"
            ),
            Err(ErroLinkCancelar::LinksRepetidos)
        );
    }

    #[test]
    fn nao_inventa_cancelamento_quando_barra_nao_oferece() {
        assert_eq!(
            ler_link_cancelar(&instalacao(), &arvore(""), "21", "10"),
            Ok(None)
        );
    }

    #[test]
    fn duplicata_de_infra_hash_na_url_assinada_falha_fechado() {
        let dupla = link("21").replace(
            "infra_hash=",
            &format!("infra_hash={}&infra_hash=", "a".repeat(64)),
        );
        assert_eq!(
            ler_link_cancelar(&instalacao(), &arvore(&dupla), "21", "10"),
            Err(ErroLinkCancelar::EnderecoNaoReconhecido)
        );
    }
}
