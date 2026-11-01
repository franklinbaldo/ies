//! URL assinada de visualização fornecida pelo operador: sem login ou POST.
use crate::transporte::FalhaTransporte;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ErroDocumentoTransporte {
    Fonte(FalhaTransporte),
    UrlNaoEhVisualizacaoAssinada,
    OrigemDoDocumentoNaoAutorizada,
}
impl ErroDocumentoTransporte {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::Fonte(e) => e.subcodigo(),
            Self::UrlNaoEhVisualizacaoAssinada => "url_documento_sem_visualizacao_assinada",
            Self::OrigemDoDocumentoNaoAutorizada => "origem_documento_nao_autorizada",
        }
    }
}

pub(crate) fn validar_link(_url: &str, _origem: &str) -> Result<(), ErroDocumentoTransporte> {
    Err(ErroDocumentoTransporte::UrlNaoEhVisualizacaoAssinada)
}
struct Resposta<R: std::io::Read> {
    status_code: i32,
    mime: String,
    content_length: Option<usize>,
    corpo: R,
}
fn decodificar<R: std::io::Read>(_resposta: Resposta<R>) -> Result<String, ErroDocumentoTransporte> {
    Err(ErroDocumentoTransporte::UrlNaoEhVisualizacaoAssinada)
}
fn ler_via<R: std::io::Read, F>(
    _url: &str, _cookie_file: &Path, _origem: &str, _requisitar: F,
) -> Result<String, ErroDocumentoTransporte>
where F: FnOnce(&str, &str) -> Result<Resposta<R>, ErroDocumentoTransporte> {
    Err(ErroDocumentoTransporte::UrlNaoEhVisualizacaoAssinada)
}
pub(crate) fn ler_de_arquivos(
    _url_file: &Path,
    _cookie_file: &Path,
    _origem_file: &Path,
) -> Result<String, ErroDocumentoTransporte> {
    Err(ErroDocumentoTransporte::UrlNaoEhVisualizacaoAssinada)
}

#[cfg(test)]
#[path = "documento_transporte/tests.rs"]
mod tests;
