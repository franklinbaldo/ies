//! Onde o processo está aberto: o `.html` do nó `PROCESSO` (Refs #59;
//! contracts/cli/processo-obter-situacao.md). Formas observadas na `UnB`
//! 4.0.9, em GO 5.0.3 e na PEN1 4.0.12: "Processo aberto somente na
//! unidade <a>", "Processo aberto nas unidades:<br /><a>…" (com
//! atribuição por unidade) e "Processo não possui andamentos abertos.".
//! O texto vem do leitor de literais; as âncoras, do tokenizador de
//! atributos. Qualquer outra forma é "não lida", nunca falha.

/// A pessoa a quem o processo está atribuído na unidade: login (texto do
/// link) e nome (`alt` = `title`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Atribuicao {
    pub(crate) sigla: String,
    pub(crate) nome: String,
}

/// Uma unidade onde o processo está aberto: sigla (texto do link), nome
/// (`alt` = `title`) e a atribuição publicada na mesma linha.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UnidadeAberta {
    pub(crate) sigla: String,
    pub(crate) nome: String,
    pub(crate) atribuido_a: Option<Atribuicao>,
}

/// Por que `aberto_em` não foi lido.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MotivoAbertoEm {
    RaizPublicaOVinculoDeAnexacao,
    HtmlDaRaizNaoPublicado,
    HtmlDaRaizForaDaFormaObservada,
}

impl MotivoAbertoEm {
    pub(crate) fn como_texto(self) -> &'static str {
        match self {
            Self::RaizPublicaOVinculoDeAnexacao => "raiz_publica_o_vinculo_de_anexacao",
            Self::HtmlDaRaizNaoPublicado => "html_da_raiz_nao_publicado",
            Self::HtmlDaRaizForaDaFormaObservada => "html_da_raiz_fora_da_forma_observada",
        }
    }
}

/// `Unidades(vec![])`: "Processo não possui andamentos abertos.".
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AbertoEm {
    Unidades(Vec<UnidadeAberta>),
    NaoLido(MotivoAbertoEm),
}

/// O `aberto_em` da árvore; nunca falha (um `.html` fora da forma do
/// literal é recusado depois pelo leitor do vínculo).
pub(super) fn ler_aberto_em(_html: &str) -> AbertoEm {
    AbertoEm::NaoLido(MotivoAbertoEm::HtmlDaRaizNaoPublicado)
}

/// As unidades do texto do `.html` da raiz, ou `None` fora das formas
/// observadas.
pub(super) fn ler_unidades(_texto: &str) -> Option<Vec<UnidadeAberta>> {
    None
}
