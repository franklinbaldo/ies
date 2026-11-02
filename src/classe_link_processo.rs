//! Classes do link do processo no Controle: estado observado pela classe
//! publicada, comparada por token inteiro, sem inferir pelo texto ou pelo NUP.

/// Estado declarado pela classe do link; não é propriedade imutável do processo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EstadoDoLink {
    pub(crate) visualizado: bool,
    pub(crate) sigiloso: bool,
}

/// O mesmo link publica duas classes de estado reconhecidas diferentes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ClassesDeEstadoDiferentesNoLink;

/// SHIM (CONTRACT): ainda reconhece só as duas classes antigas.
pub(crate) fn estado_do_link(
    classes: &str,
) -> Result<Option<EstadoDoLink>, ClassesDeEstadoDiferentesNoLink> {
    let tokens: Vec<&str> = classes.split_whitespace().collect();
    let visualizado = if tokens.contains(&"processoVisualizado") {
        Some(true)
    } else if tokens.contains(&"processoNaoVisualizado") {
        Some(false)
    } else {
        None
    };
    Ok(visualizado.map(|visualizado| EstadoDoLink {
        visualizado,
        sigiloso: false,
    }))
}
