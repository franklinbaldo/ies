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
pub(crate) fn ler_de_arquivos(
    _url_file: &Path,
    _cookie_file: &Path,
    _origem_file: &Path,
) -> Result<String, ErroDocumentoTransporte> {
    Err(ErroDocumentoTransporte::UrlNaoEhVisualizacaoAssinada)
}
