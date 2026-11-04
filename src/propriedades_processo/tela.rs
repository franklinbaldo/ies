//! O formulário `form#frmProcedimentoCadastro` servido por
//! `procedimento_alterar` (ou `procedimento_consultar`), lido sem envio.
use super::{ErroPropriedades, FonteDasPropriedades, PropriedadesLidas};

pub(crate) fn ler_propriedades(
    html: &str,
    nup: &str,
    id: &str,
    fonte: FonteDasPropriedades,
) -> Result<PropriedadesLidas, ErroPropriedades> {
    let _ = (html, nup, id, fonte);
    Err(ErroPropriedades::HtmlDasPropriedadesNaoInterpretado)
}
