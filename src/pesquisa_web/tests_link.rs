//! RED/GREEN do link do menu da pesquisa no Controle. HTML sintético.
use super::*;
use crate::pesquisa_web::tests_fixtures::{
    ORIGEM, controle, hash, href_do_menu, menu, url_controle, url_da_tela,
};
use crate::tela_html::HtmlServido;

fn instalacao() -> Instalacao {
    Instalacao::interpretar(ORIGEM).unwrap()
}

fn link(html: &str) -> Result<String, ErroLinkDaPesquisa> {
    link_da_pesquisa(&instalacao(), &url_controle(), &HtmlServido::de_teste(html))
}

#[test]
fn link_do_menu_e_resolvido_contra_a_url_do_controle() {
    assert_eq!(link(&controle(&menu(&href_do_menu()))), Ok(url_da_tela()));
}

#[test]
fn link_com_entidades_html_e_decodificado() {
    let href = href_do_menu().replace('&', "&amp;");
    assert_eq!(link(&controle(&menu(&href))), Ok(url_da_tela()));
}

#[test]
fn controle_sem_o_menu_da_pesquisa_e_recusado() {
    assert_eq!(
        link(&controle("")),
        Err(ErroLinkDaPesquisa::LinkDaPesquisaAusenteNoControle)
    );
}

#[test]
fn dois_links_da_pesquisa_sao_recusados() {
    let dois = format!("{}{}", menu(&href_do_menu()), menu(&href_do_menu()));
    assert_eq!(
        link(&controle(&dois)),
        Err(ErroLinkDaPesquisa::LinksDaPesquisaRepetidosNoControle)
    );
}

#[test]
fn link_absoluto_para_outro_host_e_recusado() {
    for href in [
        format!("https://outro.exemplo.org/sei/{}", href_do_menu()),
        format!("//outro.exemplo.org/sei/{}", href_do_menu()),
        format!("http://sei.exemplo.gov.br/sei/{}", href_do_menu()),
    ] {
        assert_eq!(
            link(&controle(&menu(&href))),
            Err(ErroLinkDaPesquisa::LinkDaPesquisaComEnderecoAbsoluto),
            "{href}"
        );
    }
}

#[test]
fn link_fora_da_forma_observada_e_recusado() {
    let sem_hash =
        "controlador.php?acao=protocolo_pesquisar&infra_sistema=1&infra_unidade_atual=2".to_owned();
    let hash_curto = href_do_menu().replace(&hash('b'), "bbbb");
    let duas_acoes = format!("{}&acao=protocolo_pesquisar", href_do_menu());
    let outra_acao = format!("{}&acao=procedimento_trabalhar", href_do_menu());
    let dois_hashes = format!("{}&infra_hash={}", href_do_menu(), hash('9'));
    let outra_pasta = format!("../outra/{}", href_do_menu());
    let ponto_e_virgula = href_do_menu().replace("infra_sistema=", "infra_sistema=1;");
    let chave_estranha = href_do_menu().replace("infra_sistema", "infra.sistema");
    let ajax = format!("{}&acao_ajax=x", href_do_menu());
    for href in [
        sem_hash,
        hash_curto,
        duas_acoes,
        outra_acao,
        dois_hashes,
        outra_pasta,
        ajax,
    ] {
        assert_eq!(
            link(&controle(&menu(&href))),
            Err(ErroLinkDaPesquisa::LinkDaPesquisaNaoReconhecido),
            "{href}"
        );
    }
    assert_eq!(
        link(&controle(&menu(&ponto_e_virgula))),
        Err(ErroLinkDaPesquisa::ConsultaDoLinkRecusadaPeloTransporte(
            crate::transporte::FalhaTransporte::SeparadorDeConsultaAmbiguo
        ))
    );
    assert_eq!(
        link(&controle(&menu(&chave_estranha))),
        Err(ErroLinkDaPesquisa::ConsultaDoLinkRecusadaPeloTransporte(
            crate::transporte::FalhaTransporte::ChaveDeConsultaNaoCanonica
        ))
    );
}

#[test]
fn acao_parecida_nao_e_o_link_da_pesquisa() {
    // `protocolo_pesquisa_rapida` (barra superior) não é o menu "Pesquisa".
    let rapida = href_do_menu().replace("protocolo_pesquisar", "protocolo_pesquisa_rapida");
    assert_eq!(
        link(&controle(&menu(&rapida))),
        Err(ErroLinkDaPesquisa::LinkDaPesquisaAusenteNoControle)
    );
}
