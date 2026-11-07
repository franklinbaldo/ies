//! A opção Externo da lista e o tipo no cadastro ainda não recarregado (#332).
//! O formato vem das telas do Todos/UnB SEI 4 e PEN3 SEI 5.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum EscolhaExterna {
    Url(String),
    Formulario,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroEscolhaExterna {
    NaoOferecido,
    Ambiguo,
    ForaDaForma,
}
pub(crate) fn escolher_externo(_html: &str) -> Result<EscolhaExterna, ErroEscolhaExterna> {
    Err(ErroEscolhaExterna::ForaDaForma)
}
pub(crate) fn identificar_tipo(
    _html: &str,
    _tipo: &str,
) -> Result<String, ErroEscolhaExterna> {
    Err(ErroEscolhaExterna::ForaDaForma)
}
#[cfg(test)]
#[path = "tests_escolha.rs"]
mod tests;