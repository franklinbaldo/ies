//! A ação "Enviar Processo" publicada na barra do nó `PROCESSO`
//! (contracts/cli/processo-enviar.md; observação em
//! docs/como-o-sei-funciona/envio-do-processo.md). Uma forma nas sete
//! instalações gravadas: `<a href="controlador.php?acao=procedimento_enviar
//! &acao_origem=arvore_visualizar&…&id_procedimento=ID&arvore=1&…&infra_hash=…">`,
//! o link da tela "Enviar Processo". Sem regex e sem JavaScript executado.
use super::barra_do_processo::{acao_do_href, ancoras_da_barra};
use super::concluir::EnderecoDeEscrita;
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::acao_do_sei::AcaoDoSei;
use crate::instalacao::Instalacao;

/// Recusas da leitura da barra (só pesam quando se pede para enviar).
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroEnviarNaBarra {
    Barra(#[from] crate::arvore_processo::ErroBarraDoProcesso),
    EnderecoDoEnvioDeOutroProcesso,
    EnderecoDoEnvioNaoReconhecido,
    LinksDeEnviarRepetidosNaBarra,
}

/// O link da tela de envio na barra do processo `id`; `Ok(None)` se a
/// barra não o publica.
pub(crate) fn ler_enviar_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroEnviarNaBarra> {
    let ancoras = ancoras_da_barra(html)?;
    let links: Vec<&str> = ancoras
        .iter()
        .filter_map(|a| a.href.as_deref())
        .filter(|href| acao_do_href(href) == Some(AcaoDoSei::ProcedimentoEnviar))
        .collect();
    match links.as_slice() {
        [] => Ok(None),
        [href] => conferir_endereco_relativo(instalacao, href, AcaoDoSei::ProcedimentoEnviar, id)
            .map(|url| Some(EnderecoDeEscrita::novo(url)))
            .map_err(|recusa| match recusa {
                EnderecoRecusado::NaoReconhecido => {
                    ErroEnviarNaBarra::EnderecoDoEnvioNaoReconhecido
                }
                EnderecoRecusado::DeOutroProcesso => {
                    ErroEnviarNaBarra::EnderecoDoEnvioDeOutroProcesso
                }
            }),
        _ => Err(ErroEnviarNaBarra::LinksDeEnviarRepetidosNaBarra),
    }
}
