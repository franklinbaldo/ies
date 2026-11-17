use super::*;
use crate::tela_html::HtmlServido;

const ORIGEM: &str = "https://sei.exemplo.gov.br/sei";

fn instalacao() -> Instalacao {
    Instalacao::interpretar(ORIGEM).expect("origem sintética")
}

/// URL relativa de cadastro com valores sintéticos (hash de 64 dígitos).
fn url_de_cadastro(id_procedimento: &str) -> String {
    format!(
        "controlador.php?acao=comentario_cadastrar&acao_origem=comentario_listar&acao_retorno=comentario_listar&arvore=1&id_procedimento={id_procedimento}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={}",
        "3".repeat(64)
    )
}

fn lista_com_botao(onclick: &str) -> String {
    format!(
        r#"<html><body><form id="frmComentarioLista">
        <button tabindex="451" type="button" accesskey="N" id="btnNovo" value="Novo" onclick="{onclick}" class="infraButton">Novo</button>
        </form></body></html>"#
    )
}

#[test]
fn botao_novo_com_location_href_e_aceito_pela_mesma_conferencia() {
    // Caso: SEI publica `<button id="btnNovo" onclick="location.href='…'">` na lista.
    let url = url_de_cadastro("10");
    let html = HtmlServido::de_teste(&lista_com_botao(&format!("location.href='{url}'")));
    assert_eq!(
        link_do_cadastro(&instalacao(), &html, "10").ok(),
        Some(format!("{ORIGEM}/{url}"))
    );
}

#[test]
fn botao_novo_da_visao_de_protocolo_aceita_o_campo_do_protocolo() {
    // Caso: visão A traz `id_rel_protocolo_protocolo` no mesmo link de Novo.
    let url = format!(
        "{}&id_rel_protocolo_protocolo=6453289",
        url_de_cadastro("10")
    );
    let html = HtmlServido::de_teste(&lista_com_botao(&format!("location.href='{url}'")));
    assert_eq!(
        link_do_cadastro(&instalacao(), &html, "10").ok(),
        Some(format!("{ORIGEM}/{url}"))
    );
}

#[test]
fn botao_com_outra_forma_de_onclick_nao_conta_como_link() {
    let url = url_de_cadastro("10");
    for onclick in [
        format!("window.open('{url}')"),
        format!("location.href='{url}'; return false;"),
        format!("location.href=\"{url}\""),
    ] {
        let html = HtmlServido::de_teste(&lista_com_botao(&onclick));
        assert!(
            matches!(
                link_do_cadastro(&instalacao(), &html, "10"),
                Err(ErroComentar::LinkDoCadastroNaoUnico)
            ),
            "forma recusada: {onclick}"
        );
    }
}

#[test]
fn botao_e_link_do_mesmo_cadastro_somam_dois_e_sao_recusados() {
    let url = url_de_cadastro("10");
    let html = HtmlServido::de_teste(&format!(
        r#"<a href="{url}">Novo</a>{}"#,
        lista_com_botao(&format!("location.href='{url}'"))
    ));
    assert!(matches!(
        link_do_cadastro(&instalacao(), &html, "10"),
        Err(ErroComentar::LinkDoCadastroNaoUnico)
    ));
}

#[test]
fn botao_de_outro_processo_e_recusado_pela_politica_de_endereco() {
    let html = HtmlServido::de_teste(&lista_com_botao(&format!(
        "location.href='{}'",
        url_de_cadastro("11")
    )));
    assert!(matches!(
        link_do_cadastro(&instalacao(), &html, "10"),
        Err(ErroComentar::LinkDoCadastroRecusado(
            EnderecoRecusado::DeOutroProcesso
        ))
    ));
}
