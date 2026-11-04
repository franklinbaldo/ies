//! O link das propriedades na barra do nó `PROCESSO` (Refs #59;
//! contracts/cli/processo-obter-propriedades.md). Forma observada na `UnB`
//! 4.0.9 (03 e 04/11/2026): a linha `Nos[i].acoes = '<literal>';` do nó
//! raiz, com `<a href="controlador.php?acao=procedimento_alterar&…">` (ou
//! `procedimento_consultar`, com o processo aberto só noutra unidade).
use crate::instalacao::Instalacao;
use crate::propriedades_processo::{EnderecoDasPropriedades, ErroPropriedades};

/// O link da tela de propriedades publicado na barra do processo `id`;
/// `Ok(None)` se a barra não publica nenhum dos dois.
pub(crate) fn ler_link_das_propriedades(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDasPropriedades>, ErroPropriedades> {
    let _ = (instalacao, html, id);
    Ok(None)
}
