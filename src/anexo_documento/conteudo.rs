//! O conteúdo guardado de um anexo lido: o texto (tipos de texto, sem
//! `--saida`), o parcial (com `--saida`) ou nada. Texto e parcial são
//! alternativos: o texto só é lido quando não há parcial.
use super::{AnexoLido, ArquivoParcial, pdf::ExtracaoPdf};

/// Um só destes: o texto lido, o parcial ainda não ligado ao nome final,
/// ou nenhum (metadados, binário, ou já publicado no cache).
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ConteudoDoAnexo {
    Texto(String),
    Pdf(ExtracaoPdf),
    Arquivo(ArquivoParcial),
    Nenhum,
}

impl AnexoLido {
    /// O texto, só nos tipos de texto sem `--saida`.
    pub(crate) fn texto(&self) -> Option<&str> {
        match &self.conteudo {
            ConteudoDoAnexo::Texto(texto) => Some(texto.as_str()),
            ConteudoDoAnexo::Pdf(pdf) => pdf.texto(),
            ConteudoDoAnexo::Arquivo(_) | ConteudoDoAnexo::Nenhum => None,
        }
    }

    pub(crate) fn estado_pdf(&self) -> Option<&'static str> {
        match &self.conteudo {
            ConteudoDoAnexo::Pdf(pdf) => Some(pdf.estado()),
            ConteudoDoAnexo::Texto(_) | ConteudoDoAnexo::Arquivo(_) | ConteudoDoAnexo::Nenhum => None,
        }
    }

    /// Tira o parcial para ligá-lo ao nome final. Sem parcial, o texto
    /// (se houver) continua no anexo.
    pub(crate) fn tirar_arquivo(&mut self) -> Option<ArquivoParcial> {
        if !matches!(self.conteudo, ConteudoDoAnexo::Arquivo(_)) {
            return None;
        }
        match std::mem::replace(&mut self.conteudo, ConteudoDoAnexo::Nenhum) {
            ConteudoDoAnexo::Arquivo(parcial) => Some(parcial),
            ConteudoDoAnexo::Texto(_) | ConteudoDoAnexo::Pdf(_) | ConteudoDoAnexo::Nenhum => None,
        }
    }
}
