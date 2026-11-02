//! GET do Controle que preserva URL e `Content-Type`, e o POST real da
//! submissão derivada. Mesmas travas do GET: HTTPS na instalação autorizada,
//! sem redirects, 8 MB durante o recebimento e uma única tentativa.
use super::FalhaTransporte;
use crate::formulario_web::{RespostaDoPost, SubmissaoDerivada};
use crate::instalacao::Instalacao;
use std::path::Path;

/// Tela do GET com o que o POST seguinte precisa: URL exata, tipo e instalação.
pub(crate) struct TelaDoControle {
    pub(crate) url: String,
    pub(crate) html: String,
    pub(crate) mime: String,
    pub(crate) instalacao: Instalacao,
}

/// URL assinada e HTML nunca aparecem no `Debug`.
impl std::fmt::Debug for TelaDoControle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TelaDoControle")
            .field("url", &"<redigido>")
            .field("html", &"<redigido>")
            .field("mime", &self.mime)
            .finish_non_exhaustive()
    }
}

/// GET único do Controle a partir dos arquivos privados. Shim CONTRACT.
pub(crate) fn ler_tela_do_controle(
    _url_file: &Path,
    _cookie_file: &Path,
    _origem_file: &Path,
) -> Result<TelaDoControle, FalhaTransporte> {
    Err(FalhaTransporte::Rede)
}

/// POST HTTPS único, sem seguir redirects. Shim CONTRACT.
pub(crate) fn enviar_post(
    _submissao: SubmissaoDerivada,
    _cookie: &str,
) -> Result<RespostaDoPost<std::io::Empty>, FalhaTransporte> {
    Err(FalhaTransporte::Rede)
}
