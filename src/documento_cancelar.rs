//! Cancelamento Web de documento (#273): link do nó DOCUMENTO e motivo.
//! Deriva endereços somente de âncoras servidas; nunca executa JavaScript.
use crate::arvore_processo::barra_do_no;
use crate::arvore_processo::endereco::{EnderecoRecusado, conferir_endereco_relativo_por};
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::atributos;
use crate::instalacao::Instalacao;

mod erro;
mod fluxo;
#[cfg(test)]
mod acervo;
#[cfg(test)]
mod tests_fluxo;

pub(crate) use erro::{ErroCancelar, FalhaDoCancelamento};
pub(crate) use fluxo::{DocumentoCancelado, PedidoDoCancelamento, cancelar_documento};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroLinkCancelar {
    BarraForaDaForma,
    LinksRepetidos,
    EnderecoNaoReconhecido,
    EnderecoDeOutroDocumento,
}
impl ErroLinkCancelar {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::BarraForaDaForma => "barra_do_cancelamento_fora_da_forma",
            Self::LinksRepetidos => "links_de_cancelamento_repetidos",
            Self::EnderecoNaoReconhecido => "endereco_de_cancelamento_nao_reconhecido",
            Self::EnderecoDeOutroDocumento => "endereco_de_cancelamento_de_outro_documento",
        }
    }
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
    use ErroLinkCancelar as E;
    let Some(barra) = barra_do_no(html, documento).map_err(|_| E::BarraForaDaForma)? else {
        return Ok(None);
    };
    let dom = tl::parse(&barra, tl::ParserOptions::default()).map_err(|_| E::BarraForaDaForma)?;
    let parser = dom.parser();
    let mut links = Vec::new();
    for no in dom.query_selector("a").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let attrs = atributos(tag).map_err(|_| E::BarraForaDaForma)?;
        let Some(bruto) = attrs.valor("href").map_err(|_| E::BarraForaDaForma)? else {
            continue;
        };
        let href = decodificar_atributo(bruto).ok_or(E::BarraForaDaForma)?;
        let acao = href
            .split_once('?')
            .and_then(|(_, query)| query.split('&').find_map(|p| p.strip_prefix("acao=")));
        if acao == Some("documento_cancelar") {
            links.push(href);
        }
    }
    let href = match links.as_slice() {
        [] => return Ok(None),
        [unico] => unico,
        _ => return Err(E::LinksRepetidos),
    };
    let mapear = |r| match r {
        EnderecoRecusado::NaoReconhecido => E::EnderecoNaoReconhecido,
        EnderecoRecusado::DeOutroProcesso => E::EnderecoDeOutroDocumento,
    };
    conferir_endereco_relativo_por(
        instalacao,
        href,
        "documento_cancelar",
        "id_procedimento",
        processo,
    )
    .map_err(mapear)?;
    let url = conferir_endereco_relativo_por(
        instalacao,
        href,
        "documento_cancelar",
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
