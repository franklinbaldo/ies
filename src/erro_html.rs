//! A tela não está na forma observada ([ADR 0005](../docs/arquitetura/0005-erros-por-tipo.md)).
//!
//! Um elemento ausente, repetido, com valor fora da forma ou um HTML
//! ilegível não sustenta decisão: só diz onde a leitura falhou. Por isso é
//! um tipo só, e não um `enum` por leitor. `onde` é o diagnóstico e a folha
//! pública (o `subcodigo` legado); `problema` diz o que se viu. `pub`
//! porque compõe `enum` públicos (`ErroComentarios`); o módulo é privado.
use crate::erro_publico::{Acao, Causa};
use crate::html_atributos::ErroAtributoHtml;
use crate::transporte::StatusHttpRecusado;

/// Onde a leitura da tela falhou e o que se viu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ErroHtml {
    /// O nome público do ponto da leitura (`link_do_processo_ausente`).
    pub(crate) onde: &'static str,
    pub(crate) problema: Problema,
    pub(crate) mensagem: &'static str,
}

/// O que a leitura viu no lugar da forma observada.
#[derive(Clone, Copy, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum Problema {
    /// O elemento não está na tela.
    Ausente,
    /// O elemento aparece mais de uma vez.
    Repetido,
    /// O valor não está na forma observada.
    Valor,
    /// O HTML não pôde ser interpretado.
    Html,
    /// A tag de abertura não pôde ser lida.
    #[causa(interna)]
    Atributo(ErroAtributoHtml),
}

impl ErroHtml {
    pub(crate) const fn ausente(onde: &'static str, mensagem: &'static str) -> Self {
        Self::novo(onde, Problema::Ausente, mensagem)
    }

    pub(crate) const fn repetido(onde: &'static str, mensagem: &'static str) -> Self {
        Self::novo(onde, Problema::Repetido, mensagem)
    }

    pub(crate) const fn valor(onde: &'static str, mensagem: &'static str) -> Self {
        Self::novo(onde, Problema::Valor, mensagem)
    }

    pub(crate) const fn html(onde: &'static str, mensagem: &'static str) -> Self {
        Self::novo(onde, Problema::Html, mensagem)
    }

    /// A causa sobe: a tag ilegível fica no caminho.
    pub(crate) const fn atributo(
        onde: &'static str,
        mensagem: &'static str,
        erro: ErroAtributoHtml,
    ) -> Self {
        Self::novo(onde, Problema::Atributo(erro), mensagem)
    }

    const fn novo(onde: &'static str, problema: Problema, mensagem: &'static str) -> Self {
        Self {
            onde,
            problema,
            mensagem,
        }
    }
}

impl Causa for ErroHtml {
    fn caminho(&self, c: &mut Vec<&'static str>) {
        c.push("html");
        self.problema.caminho(c);
    }

    /// `onde`: mantém o `subcodigo` público de antes do tipo único.
    fn folha(&self) -> &'static str {
        self.onde
    }

    fn acao(&self) -> Acao {
        Acao::Reportar
    }

    fn status_http(&self) -> Option<&StatusHttpRecusado> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erro_publico::{caminho_de, folha};

    #[test]
    fn folha_e_onde_e_caminho_e_o_problema() {
        let erro = ErroHtml::ausente("link_do_processo_ausente", "m");
        assert_eq!(folha(&erro), "link_do_processo_ausente");
        assert_eq!(caminho_de(&erro), ["html", "ausente"]);
        assert_eq!(erro.acao(), Acao::Reportar);
        assert_eq!(erro.status_http(), None);
        assert_eq!(
            caminho_de(&ErroHtml::repetido("x", "m")),
            ["html", "repetido"]
        );
        assert_eq!(caminho_de(&ErroHtml::valor("x", "m")), ["html", "valor"]);
        assert_eq!(caminho_de(&ErroHtml::html("x", "m")), ["html", "html"]);
    }

    #[test]
    fn atributo_carrega_a_causa_e_mantem_onde() {
        let erro = ErroHtml::atributo(
            "link_ilegivel",
            "m",
            ErroAtributoHtml::FimDaTagDeAberturaAusente,
        );
        assert_eq!(folha(&erro), "link_ilegivel");
        assert_eq!(
            caminho_de(&erro),
            ["html", "atributo", "fim_da_tag_de_abertura_ausente"]
        );
    }
}
