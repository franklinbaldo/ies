//! Classes do link do processo no Controle: estado observado pela classe
//! publicada, comparada por token inteiro, sem inferir pelo texto ou pelo NUP.

/// Estado declarado pela classe do link; não é propriedade imutável do processo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EstadoDoLink {
    pub(crate) visualizado: bool,
    pub(crate) sigiloso: bool,
}

/// O mesmo link publica duas classes de estado reconhecidas diferentes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) struct ClassesDeEstadoDiferentesNoLink;

const fn estado(visualizado: bool, sigiloso: bool) -> EstadoDoLink {
    EstadoDoLink {
        visualizado,
        sigiloso,
    }
}

/// `processoVisualizadoSigiloso` observado em Goiás 5.0.3 (02/11/2026);
/// `processoNaoVisualizadoSigiloso` é o par simétrico, não observado live.
const CLASSES: [(&str, EstadoDoLink); 4] = [
    ("processoVisualizado", estado(true, false)),
    ("processoNaoVisualizado", estado(false, false)),
    ("processoVisualizadoSigiloso", estado(true, true)),
    ("processoNaoVisualizadoSigiloso", estado(false, true)),
];

/// `None` quando nenhum token reconhecido aparece (ícones e outros links).
pub(crate) fn estado_do_link(
    classes: &str,
) -> Result<Option<EstadoDoLink>, ClassesDeEstadoDiferentesNoLink> {
    let mut achado = None;
    for token in classes.split_whitespace() {
        let Some(&(_, atual)) = CLASSES.iter().find(|(nome, _)| *nome == token) else {
            continue;
        };
        if achado.is_some_and(|anterior| anterior != atual) {
            return Err(ClassesDeEstadoDiferentesNoLink);
        }
        achado = Some(atual);
    }
    Ok(achado)
}
