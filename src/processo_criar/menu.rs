//! Link de "Iniciar Processo" publicado pelo Controle da sessão.
//! Não inferir URL, não executar JS e não seguir redirecionamento.
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::{
    ConsultaDoControlador, ErroChaveDaConsulta, ErroConsultaDoControlador, menciona_acao,
};
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::valor_do_atributo;
use crate::instalacao::Instalacao;

/// O menu do Controle fora da forma observada.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroMenu {
    LinkDeIniciarProcessoAusente,
    LinkDeIniciarProcessoNaoReconhecido,
    AtributoDoLinkDeIniciarIlegivel(#[source] crate::html_atributos::ErroAtributoHtml),
    LinkDeIniciarProcessoRepetido,
    ConsultaDoLinkRecusada(#[source] crate::transporte::FalhaTransporte),
    /// A consulta do link não se lê em pares `chave=valor`.
    ConsultaDoLinkIlegivel(#[source] ErroConsultaDoControlador),
    /// `acao` ou `infra_hash` do link não tem exatamente um valor aceito.
    ChaveDoLinkIlegivel(#[source] ErroChaveDaConsulta),
}

/// O href literal do menu; apenas o padrão relativo demonstrado pelo SEI.
/// Uma forma de onclick desconhecida é recusada, não executada.
pub(crate) fn link_de_iniciar(
    instalacao: &Instalacao,
    url_controle: &str,
    html: &crate::tela_html::HtmlServido,
) -> Result<String, ErroMenu> {
    let dom = crate::tela_html::interpretar(html);
    let parser = dom.parser();
    let mut candidatos = Vec::new();
    for no in dom.query_selector("a").into_iter().flatten() {
        let tag = no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroMenu::LinkDeIniciarProcessoNaoReconhecido)?;
        let Some(bruto) =
            valor_do_atributo(tag, "href").map_err(ErroMenu::AtributoDoLinkDeIniciarIlegivel)?
        else {
            continue;
        };
        let href = match decodificar_atributo(&bruto) {
            Some(href) => href,
            None if bruto.contains(AcaoDoSei::ProcedimentoEscolherTipo.como_na_url()) => {
                return Err(ErroMenu::LinkDeIniciarProcessoNaoReconhecido);
            }
            None => continue,
        };
        let aponta = menciona_acao(&href, AcaoDoSei::ProcedimentoEscolherTipo);
        if aponta {
            candidatos.push(href);
        }
    }
    let [href] = candidatos.as_slice() else {
        return Err(if candidatos.is_empty() {
            ErroMenu::LinkDeIniciarProcessoAusente
        } else {
            ErroMenu::LinkDeIniciarProcessoRepetido
        });
    };
    // Não seguir links de outra instalação, nem admitir parâmetros inesperados
    // de ação ou identificador de sessão.
    let consulta = href
        .strip_prefix("controlador.php?")
        .ok_or(ErroMenu::LinkDeIniciarProcessoNaoReconhecido)?;
    if href.contains(['#', '\\', '\'']) || !href.bytes().all(|b| (33..=126).contains(&b)) {
        return Err(ErroMenu::LinkDeIniciarProcessoNaoReconhecido);
    }
    let consulta = ConsultaDoControlador::ler(consulta).map_err(|erro| match erro {
        ErroConsultaDoControlador::RecusadaPeloTransporte(falha) => {
            ErroMenu::ConsultaDoLinkRecusada(falha)
        }
        outro @ ErroConsultaDoControlador::ParSemIgual => ErroMenu::ConsultaDoLinkIlegivel(outro),
    })?;
    let acao = consulta.acao().map_err(ErroMenu::ChaveDoLinkIlegivel)?;
    let hash = consulta
        .unico("infra_hash")
        .map_err(ErroMenu::ChaveDoLinkIlegivel)?;
    if acao != AcaoDoSei::ProcedimentoEscolherTipo
        || consulta.valores("acao_ajax").next().is_some()
        || hash.len() != 64
        || !hash.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(ErroMenu::LinkDeIniciarProcessoNaoReconhecido);
    }
    let base = url_controle
        .split('?')
        .next()
        .ok_or(ErroMenu::LinkDeIniciarProcessoNaoReconhecido)?;
    let (pasta, _) = base
        .rsplit_once('/')
        .ok_or(ErroMenu::LinkDeIniciarProcessoNaoReconhecido)?;
    let url = format!("{pasta}/{href}");
    let decomposicao = crate::url_observada::UrlHttps::interpretar(&url)
        .ok_or(ErroMenu::LinkDeIniciarProcessoNaoReconhecido)?;
    if !instalacao.autoriza_controlador(decomposicao.host, decomposicao.caminho) {
        return Err(ErroMenu::LinkDeIniciarProcessoNaoReconhecido);
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> (&'static str, Instalacao) {
        (
            "https://sei.exemplo.test/sei/controlador.php?acao=procedimento_controlar",
            Instalacao::interpretar("https://sei.exemplo.test/sei").unwrap(),
        )
    }

    fn ancora(acao: &str) -> String {
        format!(
            "<a href='controlador.php?acao={acao}&amp;infra_hash={}'>Iniciar</a>",
            "a".repeat(64)
        )
    }

    #[test]
    fn aceita_um_link_servido_e_preserva_assinatura() {
        let (url, instalacao) = base();
        let link = link_de_iniciar(
            &instalacao,
            url,
            &crate::tela_html::HtmlServido::de_teste(&ancora(
                AcaoDoSei::ProcedimentoEscolherTipo.como_na_url(),
            )),
        )
        .unwrap();
        assert!(link.contains("acao=procedimento_escolher_tipo&infra_hash="));
        assert!(!link.contains("amp;"));
    }

    #[test]
    fn recusa_ausencia_repeticao_e_acao_errada() {
        let (url, instalacao) = base();
        assert_eq!(
            link_de_iniciar(
                &instalacao,
                url,
                &crate::tela_html::HtmlServido::de_teste("<div/>")
            ),
            Err(ErroMenu::LinkDeIniciarProcessoAusente)
        );
        let repetido = crate::tela_html::HtmlServido::de_teste(&format!(
            "{}{}",
            ancora(AcaoDoSei::ProcedimentoEscolherTipo.como_na_url()),
            ancora(AcaoDoSei::ProcedimentoEscolherTipo.como_na_url())
        ));
        assert_eq!(
            link_de_iniciar(&instalacao, url, &repetido),
            Err(ErroMenu::LinkDeIniciarProcessoRepetido)
        );
        assert_eq!(
            link_de_iniciar(
                &instalacao,
                url,
                &crate::tela_html::HtmlServido::de_teste(&ancora("procedimento_gerar"))
            ),
            Err(ErroMenu::LinkDeIniciarProcessoAusente)
        );
    }

    #[test]
    fn recusa_com_a_causa_da_consulta() {
        let (url, instalacao) = base();
        let escolher = AcaoDoSei::ProcedimentoEscolherTipo.como_na_url();
        let hash = "a".repeat(64);
        let duas_acoes = crate::tela_html::HtmlServido::de_teste(&format!(
            "<a href='controlador.php?acao={escolher}&amp;acao=procedimento_gerar&amp;infra_hash={hash}'>Iniciar</a>"
        ));
        assert_eq!(
            link_de_iniciar(&instalacao, url, &duas_acoes),
            Err(ErroMenu::ChaveDoLinkIlegivel(
                ErroChaveDaConsulta::Repetida { chave: "acao" }
            ))
        );
        let par_sem_igual = crate::tela_html::HtmlServido::de_teste(&format!(
            "<a href='controlador.php?acao={escolher}&amp;lixo&amp;infra_hash={hash}'>Iniciar</a>"
        ));
        assert_eq!(
            link_de_iniciar(&instalacao, url, &par_sem_igual),
            Err(ErroMenu::ConsultaDoLinkIlegivel(
                ErroConsultaDoControlador::ParSemIgual
            ))
        );
    }

    #[test]
    fn recusa_link_externo_ou_sem_assinatura() {
        let (url, instalacao) = base();
        let externo = crate::tela_html::HtmlServido::de_teste(&format!(
            "<a href='https://outro.test/controlador.php?acao={}&amp;infra_hash={}'>Iniciar</a>",
            AcaoDoSei::ProcedimentoEscolherTipo.como_na_url(),
            "a".repeat(64)
        ));
        assert_eq!(
            link_de_iniciar(&instalacao, url, &externo),
            Err(ErroMenu::LinkDeIniciarProcessoNaoReconhecido)
        );
        let sem_assinatura = crate::tela_html::HtmlServido::de_teste(&format!(
            "<a href='controlador.php?acao={}'>Iniciar</a>",
            AcaoDoSei::ProcedimentoEscolherTipo.como_na_url()
        ));
        assert_eq!(
            link_de_iniciar(&instalacao, url, &sem_assinatura),
            Err(ErroMenu::ChaveDoLinkIlegivel(
                ErroChaveDaConsulta::Ausente {
                    chave: "infra_hash"
                }
            ))
        );
    }
}
