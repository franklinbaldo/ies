//! Link do menu "Pesquisa" publicado pelo Controle (`acao=protocolo_pesquisar`).
//! Forma observada (UnB 4.0.9, PEN1 5.0.4, GO 5.0.3; capturas privadas):
//! `<a link="protocolo_pesquisar" href="controlador.php?acao=protocolo_pesquisar&…&infra_hash=<64 hex>">`,
//! exatamente um por Controle. Só esse link é seguido, da mesma resposta.
use crate::instalacao::Instalacao;

/// Uma variante por fato verificável do link no Controle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroLinkDaPesquisa {
    LinkDaPesquisaAusenteNoControle,
    LinksDaPesquisaRepetidosNoControle,
    /// `https://`, `http://` ou `//`: o menu observado é relativo.
    LinkDaPesquisaComEnderecoAbsoluto,
    /// Fora de `controlador.php?`, sem exatamente uma `acao` e um
    /// `infra_hash` de 64 hex, com chave não canônica ou fora da instalação.
    LinkDaPesquisaNaoReconhecido,
}

impl ErroLinkDaPesquisa {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::LinkDaPesquisaAusenteNoControle => "link_da_pesquisa_ausente_no_controle",
            Self::LinksDaPesquisaRepetidosNoControle => "links_da_pesquisa_repetidos_no_controle",
            Self::LinkDaPesquisaComEnderecoAbsoluto => "link_da_pesquisa_com_endereco_absoluto",
            Self::LinkDaPesquisaNaoReconhecido => "link_da_pesquisa_nao_reconhecido",
        }
    }
}

/// URL absoluta do link de pesquisa do Controle, resolvida contra a URL
/// da tela servida e conferida na instalação. Nenhuma requisição.
pub(crate) fn link_da_pesquisa(
    _instalacao: &Instalacao,
    _url_do_controle: &str,
    _html: &str,
) -> Result<String, ErroLinkDaPesquisa> {
    todo!("GREEN #58: link do menu da pesquisa")
}

#[cfg(test)]
#[path = "tests_link.rs"]
mod tests;
