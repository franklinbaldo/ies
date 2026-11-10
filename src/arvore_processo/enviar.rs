//! A ação "Enviar Processo" publicada na barra do nó `PROCESSO`
//! (contracts/cli/processo-enviar.md; observação em
//! docs/como-o-sei-funciona/envio-do-processo.md). Uma forma nas sete
//! instalações gravadas: `<a href="controlador.php?acao=procedimento_enviar
//! &acao_origem=arvore_visualizar&…&id_procedimento=ID&arvore=1&…&infra_hash=…">`,
//! o link da tela "Enviar Processo". Sem regex e sem JavaScript executado.
use super::barra_do_processo::{FalhaDaBarra, acao_do_href, ancoras_da_barra};
use super::concluir::EnderecoDeEscrita;
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::erro_html::ErroHtml;
use crate::instalacao::Instalacao;

/// A `acao` da tela de envio e do POST.
pub(crate) const ACAO_DE_ENVIAR: &str = "procedimento_enviar";

/// Recusas da leitura da barra (só pesam quando se pede para enviar).
#[derive(Clone, Copy, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroEnviarNaBarra {
    EnderecoDoEnvioDeOutroProcesso,
    #[causa(transparente)]
    Html(ErroHtml),
}

/// O link da tela de envio na barra do processo `id`; `Ok(None)` se a
/// barra não o publica.
pub(crate) fn ler_enviar_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<EnderecoDeEscrita>, ErroEnviarNaBarra> {
    use ErroEnviarNaBarra as E;
    let ancoras = ancoras_da_barra(html).map_err(|falha| match falha {
        FalhaDaBarra::Fora => E::Html(ErroHtml::valor("barra_do_processo_fora_da_forma_observada")),
        FalhaDaBarra::Repetidas => {
            E::Html(ErroHtml::repetido("barras_do_processo_repetidas_na_arvore"))
        }
    })?;
    let links: Vec<&str> = ancoras
        .iter()
        .filter_map(|a| a.href.as_deref())
        .filter(|href| acao_do_href(href) == Some(ACAO_DE_ENVIAR))
        .collect();
    match links.as_slice() {
        [] => Ok(None),
        [href] => conferir_endereco_relativo(instalacao, href, ACAO_DE_ENVIAR, id)
            .map(|url| Some(EnderecoDeEscrita::novo(url)))
            .map_err(|recusa| match recusa {
                EnderecoRecusado::NaoReconhecido => {
                    E::Html(ErroHtml::valor("endereco_do_envio_nao_reconhecido"))
                }
                EnderecoRecusado::DeOutroProcesso => E::EnderecoDoEnvioDeOutroProcesso,
            }),
        _ => Err(ErroHtml::repetido("links_de_enviar_repetidos_na_barra").into()),
    }
}
