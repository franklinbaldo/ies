//! Cancelamento Web de documento (#273). O link da ação vem da barra
//! específica do nó do documento; a operação nunca constrói URL assinada.
use crate::arvore_processo::endereco::EnderecoRecusado;
use crate::instalacao::Instalacao;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroLinkCancelar {
    BarraForaDaForma,
    LinksRepetidos,
    EnderecoNaoReconhecido,
    EnderecoDeOutroDocumento,
}

/// Um link assinado validado; Debug nunca revela a URL.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct LinkDeCancelamento(String);
impl LinkDeCancelamento {
    pub(crate) fn as_str(&self) -> &str { &self.0 }
}
impl std::fmt::Debug for LinkDeCancelamento {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("LinkDeCancelamento(<redigido>)")
    }
}

/// Extrai somente a ação do documento pedido, sem executar scripts.
pub(crate) fn ler_link_cancelar(
    _instalacao: &Instalacao, _html: &str, _documento: &str, _processo: &str,
) -> Result<Option<LinkDeCancelamento>, ErroLinkCancelar> {
    Ok(None) // RED: observar e contratar antes da implementação.
}

/// Um motivo não vazio, sem controles, representável em ISO-8859-1.
pub(crate) fn conferir_motivo(_motivo: &str) -> Result<String, &'static str> {
    Err("motivo_invalido") // RED
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arvore_processo::tests_assinatura::arvore_assinavel;

    fn instalacao() -> Instalacao {
        Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap()
    }
    fn link(doc: &str) -> String {
        format!("<a href=\"controlador.php?acao=documento_cancelar&acao_origem=arvore_visualizar&id_procedimento=10&id_documento={doc}&infra_hash={}\">Cancelar Documento</a>", "a".repeat(64))
    }
    fn arvore(acoes: &str) -> String {
        arvore_assinavel("10", "21", "0000101", acoes, &[])
    }

    // Caso de todos tests/test_documento_mover_cancelar_010.py @323eb01.
    #[test]
    fn reconhece_o_link_assinado_so_na_barra_do_documento_pedido() {
        let url = ler_link_cancelar(&instalacao(), &arvore(&link("21")), "21", "10")
            .unwrap().unwrap();
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
            ler_link_cancelar(&instalacao(), &arvore(&(link("21") + &link("21"))), "21", "10"),
            Err(ErroLinkCancelar::LinksRepetidos)
        );
    }

    // Caso de todos tests/test_documento_mover_cancelar_010.py @323eb01.
    #[test]
    fn motivo_com_espacos_e_recusado_e_acentuado_e_preservado() {
        assert!(conferir_motivo("   ").is_err());
        assert_eq!(conferir_motivo("  Revisão necessária  "), Ok("Revisão necessária".into()));
        assert!(conferir_motivo("Linha\\nquebrada").is_err());
    }
}
