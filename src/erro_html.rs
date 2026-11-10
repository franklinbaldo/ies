//! A tela não está na forma observada ([ADR 0005](../docs/arquitetura/0005-erros-por-tipo.md)).
//!
//! Um elemento ausente, repetido, com valor fora da forma ou um HTML
//! ilegível não sustenta decisão: só diz onde a leitura falhou. Por isso é
//! um tipo só, e não um `enum` por leitor. `onde` é o diagnóstico e a folha
//! pública (o `subcodigo` legado); `problema` diz o que se viu. `pub`
//! porque compõe `enum` públicos (`ErroComentarios`); o módulo é privado.
use crate::html_atributos::ErroAtributoHtml;

/// Onde a leitura da tela falhou e o que se viu. A igualdade é por `onde` e
/// `problema`: a mensagem é apresentação, não classificação.
#[derive(Clone, Copy, Debug, Eq)]
pub struct ErroHtml {
    /// O nome público do ponto da leitura (`link_do_processo_ausente`).
    pub(crate) onde: &'static str,
    pub(crate) problema: Problema,
    /// A genérica do problema, salvo [`ErroHtml::com_mensagem`].
    pub(crate) mensagem: &'static str,
}

/// O que a leitura viu no lugar da forma observada.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
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
    Atributo(#[from] ErroAtributoHtml),
}

impl Problema {
    const fn mensagem(self) -> &'static str {
        match self {
            Self::Ausente => "a tela não tem o elemento observado",
            Self::Repetido => "o elemento aparece mais de uma vez",
            Self::Valor => "o valor não está na forma observada",
            Self::Html => "o HTML não pôde ser interpretado",
            Self::Atributo(_) => "a tag de abertura não pôde ser lida",
        }
    }
}

impl ErroHtml {
    pub(crate) const fn ausente(onde: &'static str) -> Self {
        Self::novo(onde, Problema::Ausente)
    }

    pub(crate) const fn repetido(onde: &'static str) -> Self {
        Self::novo(onde, Problema::Repetido)
    }

    pub(crate) const fn valor(onde: &'static str) -> Self {
        Self::novo(onde, Problema::Valor)
    }

    pub(crate) const fn html(onde: &'static str) -> Self {
        Self::novo(onde, Problema::Html)
    }

    /// A causa sobe: a tag ilegível fica no caminho.
    pub(crate) const fn atributo(onde: &'static str, erro: ErroAtributoHtml) -> Self {
        Self::novo(onde, Problema::Atributo(erro))
    }

    /// O texto próprio do ponto, no lugar da genérica do problema.
    pub(crate) const fn com_mensagem(self, mensagem: &'static str) -> Self {
        Self { mensagem, ..self }
    }

    const fn novo(onde: &'static str, problema: Problema) -> Self {
        Self {
            onde,
            problema,
            mensagem: problema.mensagem(),
        }
    }
}

impl PartialEq for ErroHtml {
    fn eq(&self, outro: &Self) -> bool {
        self.onde == outro.onde && self.problema == outro.problema
    }
}

impl std::fmt::Display for ErroHtml {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.mensagem)
    }
}

impl std::error::Error for ErroHtml {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erro_publico::Causa;

    #[test]
    fn folha_e_onde_e_caminho_e_o_problema() {
        let erro = ErroHtml::ausente("link_do_processo_ausente");
        assert_eq!(erro.mensagem, "a tela não tem o elemento observado");
        assert_eq!(erro.status_http(), None);
    }

    #[test]
    fn com_mensagem_troca_so_o_texto() {
        let erro = ErroHtml::valor("x").com_mensagem("própria");
        assert_eq!(erro.mensagem, "própria");
        assert_eq!(erro.problema, Problema::Valor);
        assert_eq!(erro, ErroHtml::valor("x"), "a mensagem não classifica");
        assert_ne!(erro, ErroHtml::ausente("x"));
    }

    #[derive(Debug, thiserror::Error)]
    #[error("{self:?}")]
    enum Leitor {
        Html(#[from] ErroHtml),
    }

    #[test]
    fn variante_transparente_gera_from() {
        let leitor: Leitor = ErroHtml::html("x").into();
        assert!(matches!(leitor, Leitor::Html(_)));
    }

    #[test]
    fn atributo_carrega_a_causa_e_mantem_onde() {
        let _erro =
            ErroHtml::atributo("link_ilegivel", ErroAtributoHtml::FimDaTagDeAberturaAusente);
    }
}
