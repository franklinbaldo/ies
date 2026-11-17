//! Link de "Iniciar Processo" publicado pelo Controle da sessão.
//! Não inferir URL, não executar JS e não seguir redirecionamento.
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::{
    ConsultaDoControlador, ErroConsultaDoControlador, ErroParametroDaUrl, menciona_acao,
};
use crate::instalacao::Instalacao;

/// O menu do Controle fora da forma observada.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroMenu {
    LinkDeIniciarProcessoAusente,
    LinkDeIniciarProcessoNaoReconhecido,
    LinkDeIniciarProcessoRepetido,
    /// A consulta do link não se lê em pares `chave=valor`.
    ConsultaDoLinkIlegivel(#[source] ErroConsultaDoControlador),
    /// `acao` ou `infra_hash` do link não tem exatamente um valor aceito.
    ChaveDoLinkIlegivel(#[source] ErroParametroDaUrl),
    AcaoDoLinkIlegivel(#[source] crate::consulta_do_controlador::ErroAcaoDaUrl),
}

/// O href literal do menu; apenas o padrão relativo demonstrado pelo SEI.
/// Uma forma de onclick desconhecida é recusada, não executada.
pub(crate) fn link_de_iniciar(
    instalacao: &Instalacao,
    url_controle: &str,
    html: &crate::tela_html::HtmlServido,
) -> Result<String, ErroMenu> {
    let dom = crate::tela_html::arvore(html);
    let mut candidatos = Vec::new();
    for no in dom.elementos("a") {
        let Some(href) = no.atributo("href") else {
            continue;
        };
        if menciona_acao(href, AcaoDoSei::ProcedimentoEscolherTipo) {
            candidatos.push(href.to_owned());
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
    let consulta =
        ConsultaDoControlador::ler(consulta).map_err(ErroMenu::ConsultaDoLinkIlegivel)?;
    let acao = consulta.acao().map_err(ErroMenu::AcaoDoLinkIlegivel)?;
    let hash = consulta
        .unico("infra_hash")
        .map_err(ErroMenu::ChaveDoLinkIlegivel)?;
    if acao != AcaoDoSei::ProcedimentoEscolherTipo.como_na_url()
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

    fn ancora(acao: AcaoDoSei) -> String {
        format!(
            "<a href='controlador.php?acao={acao}&amp;infra_hash={}'>Iniciar</a>",
            "a".repeat(64),
            acao = acao.como_na_url()
        )
    }

    #[test]
    fn aceita_um_link_servido_e_preserva_assinatura() {
        let (url, instalacao) = base();
        let link = link_de_iniciar(
            &instalacao,
            url,
            &crate::tela_html::HtmlServido::de_teste(&ancora(AcaoDoSei::ProcedimentoEscolherTipo)),
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
            ancora(AcaoDoSei::ProcedimentoEscolherTipo),
            ancora(AcaoDoSei::ProcedimentoEscolherTipo)
        ));
        assert_eq!(
            link_de_iniciar(&instalacao, url, &repetido),
            Err(ErroMenu::LinkDeIniciarProcessoRepetido)
        );
        assert_eq!(
            link_de_iniciar(
                &instalacao,
                url,
                &crate::tela_html::HtmlServido::de_teste(&ancora(AcaoDoSei::ProcedimentoGerar))
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
            Err(ErroMenu::AcaoDoLinkIlegivel(
                crate::consulta_do_controlador::ErroAcaoDaUrl::AcaoRepetida
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
                ErroParametroDaUrl::ParametroAusente("infra_hash")
            ))
        );
    }
}
