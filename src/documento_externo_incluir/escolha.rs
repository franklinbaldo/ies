//! Escolha do documento externo na lista e do tipo no cadastro inicial (#332).
//! Os links e opções provêm das telas do Todos (UnB SEI 4/PEN3 SEI 5).

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

/// RED: contrato compilável antes da implementação.
pub(crate) fn escolher_externo(_html: &str) -> Result<EscolhaExterna, ErroEscolhaExterna> {
    Err(ErroEscolhaExterna::ForaDaForma)
}
pub(crate) fn identificar_tipo(_html: &str, _tipo: &str) -> Result<String, ErroEscolhaExterna> {
    Err(ErroEscolhaExterna::ForaDaForma)
}
#[cfg(test)]
#[path = "tests_escolha.rs"]
mod tests;
