//! RED/GREEN da extração do texto da tela de recusa: exceção do SEI,
//! depois `<title>` e texto visível; entidades básicas decodificadas.
use super::texto_do_html;
use crate::tela_html::HtmlServido;
use crate::transporte::saneamento::sanear;

fn extraido(html: &str) -> Option<String> {
    texto_do_html(&HtmlServido::de_teste(html)).and_then(|t| sanear(&t))
}

/// Tela de exceção sintética no formato que o SEI testa por
/// `getElementById('divInfraExcecao')`.
const EXCECAO: &str = "<html><head><title>SEI - Erro</title>\
    <script>var infra = '<b>nao</b>';</script><style>p{color:red}</style></head>\
    <body><div id='divInfraBarraSistema'>Menu Sair</div>\
    <div id=\"divInfraExcecao\" class=\"infraExcecao\">\n  Erro acessando o banco &amp; \
    <b>dados</b>.\n</div></body></html>";

#[test]
fn excecao_do_sei_tem_prioridade_sobre_o_resto_da_tela() {
    assert_eq!(
        extraido(EXCECAO).as_deref(),
        Some("Erro acessando o banco & dados.")
    );
}

#[test]
fn classe_infra_excecao_ou_mensagem_de_erro_quando_nao_ha_o_id() {
    let html = "<html><title>T</title><body><p>x</p>\
                <div class='alerta infraExcecao'>Usuário inativo.</div></body></html>";
    assert_eq!(extraido(html).as_deref(), Some("Usuário inativo."));
    let html = "<html><body><div class='infraMensagemErro'>Sem permissão.</div></body></html>";
    assert_eq!(extraido(html).as_deref(), Some("Sem permissão."));
}

#[test]
fn sem_conteiner_usa_title_e_texto_visivel_sem_script_nem_style() {
    let html = "<html><head><title>503 Service Unavailable</title>\
                <style>body{x:y}</style></head><body><h1>Service Unavailable</h1>\
                <script>alert('x')</script><noscript>ligue o JS</noscript>\
                <p>The server is temporarily unable.</p></body></html>";
    assert_eq!(
        extraido(html).as_deref(),
        Some("503 Service Unavailable — Service Unavailable The server is temporarily unable.")
    );
}

#[test]
fn html_sem_texto_nao_e_mensagem() {
    assert_eq!(
        extraido("<html><body><script>x()</script></body></html>"),
        None
    );
    assert_eq!(extraido(""), None);
}

#[test]
fn atributo_booleano_antes_do_id_nao_muda_a_extracao() {
    let corrompido = crate::tela_html::teste::com_booleano_antes_dos_atributos(EXCECAO);
    assert_eq!(extraido(&corrompido), extraido(EXCECAO));
}
