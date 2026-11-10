//! RED/GREEN do envio injetável: resposta sintética, no máximo uma chamada.
use super::tests::{OPERACAO, instalacao, tela_iso, url_servida};
use super::*;
use crate::transporte::sessao::CookieDaSessao;
use ErroSubmissaoFormulario as E;
use std::cell::Cell;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

const FORM: &str = "<html><a id='lnkInfraUnidade'>UNIDADE TESTE</a>\
    <form id='frmPesquisaProtocolo' method='post' \
    action='controlador.php?acao=protocolo_pesquisar&amp;infra_hash=cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc'>\
    <input type='hidden' name='hdnInfraTipoPagina' value='1'>\
    <input type='text' name='txtProtocoloPesquisa' value=''>\
    <button type='submit' name='sbmPesquisar' value='Pesquisar'>Pesquisar</button>\
    <input type='hidden' name='hdnInicio' value='0'></form></html>";

fn submissao() -> SubmissaoDerivada {
    let url = url_servida();
    derivar_submissao(
        &OPERACAO,
        &instalacao(),
        &tela_iso(&url, FORM),
        "UNIDADE TESTE",
        &[("txtProtocoloPesquisa", "0001")],
    )
    .unwrap()
}

struct Pasta(PathBuf);

impl Drop for Pasta {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn cookie(modo: u32) -> (Pasta, PathBuf) {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static N: AtomicUsize = AtomicUsize::new(0);
    let pasta = std::env::temp_dir().join(format!(
        "ies-formulario-web-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&pasta).unwrap();
    let arquivo = pasta.join("cookie");
    fs::write(&arquivo, "PHPSESSID=SINTETICO\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&arquivo, fs::Permissions::from_mode(modo)).unwrap();
    }
    #[cfg(not(unix))]
    let _ = modo;
    (Pasta(pasta), arquivo)
}

fn resposta(status_code: i32, html: &str) -> RespostaDoPost<Cursor<Vec<u8>>> {
    crate::transporte::testes_http::resposta_post(
        status_code,
        "text/html; charset=iso-8859-1",
        html.as_bytes().to_vec(),
        Some(html.len()),
        None,
    )
}

fn html(r: Result<RespostaDoEnvio, E>) -> Result<String, E> {
    r.map(|resposta| match resposta {
        RespostaDoEnvio::Html(html) => html,
        RespostaDoEnvio::Redirecionamento(_) => panic!("302 inesperado"),
    })
}

fn enviar_com(status_code: i32, html_: &str) -> (Result<String, E>, usize) {
    let (_pasta, arquivo) = cookie(0o600);
    let chamadas = Cell::new(0);
    let r = enviar_submissao_via(submissao(), &CookieDaSessao::ler(&arquivo), |_, _| {
        chamadas.set(chamadas.get() + 1);
        Ok(resposta(status_code, html_))
    });
    (html(r), chamadas.get())
}

const RESULTADO: &str = "<html><table class='pesquisaResultado'><tr><td>x</td></tr></table></html>";

#[test]
fn resposta_com_marcador_de_resultado_e_aceita_com_uma_chamada() {
    let (r, n) = enviar_com(200, RESULTADO);
    assert_eq!(r.unwrap(), RESULTADO);
    assert_eq!(n, 1);
}

#[test]
fn marcador_com_booleano_antes_da_classe_e_reconhecido() {
    let html = crate::html_atributos::com_booleano_antes_dos_atributos(RESULTADO);
    let (r, n) = enviar_com(200, &html);
    assert_eq!(r.unwrap(), html);
    assert_eq!(n, 1);
}

#[test]
fn closure_recebe_post_corpo_cabecalhos_e_cookie_da_sessao() {
    let (_pasta, arquivo) = cookie(0o600);
    let r = enviar_submissao_via(submissao(), &CookieDaSessao::ler(&arquivo), |s, cookie| {
        assert_eq!(SubmissaoDerivada::METODO, "POST");
        assert_eq!(
            SubmissaoDerivada::TIPO_DE_CONTEUDO,
            "application/x-www-form-urlencoded"
        );
        assert_eq!(
            s.url(),
            "https://sei.exemplo.gov.br/sei/controlador.php?acao=protocolo_pesquisar&infra_hash=cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
        );
        assert_eq!(s.origem(), "https://sei.exemplo.gov.br");
        assert_eq!(s.referer(), url_servida());
        assert_eq!(
            s.corpo(),
            b"hdnInfraTipoPagina=1&txtProtocoloPesquisa=0001&sbmPesquisar=Pesquisar&hdnInicio=0"
        );
        assert_eq!(cookie, "PHPSESSID=SINTETICO");
        Ok(resposta(200, RESULTADO))
    });
    assert!(r.is_ok());
}

#[test]
fn redirecionamento_nao_302_e_negacao_apos_envio_sao_falhas_de_transporte() {
    for (status, falha) in [
        (301, "redirecionamento"),
        (303, "redirecionamento"),
        (307, "redirecionamento"),
        (401, "acesso_negado"),
        (403, "acesso_negado"),
    ] {
        let (r, n) = enviar_com(status, "");
        let E::Transporte(obtida) = r.unwrap_err() else {
            panic!("{status}: esperava falha de transporte")
        };
        assert_eq!(obtida.subcodigo(), falha, "{status}");
        assert_eq!(
            obtida.status_http().map(|h| i32::from(h.status)),
            Some(status)
        );
        assert_eq!(n, 1);
    }
}

#[test]
fn tela_de_login_com_200_apos_envio_e_recusa_propria() {
    let login = "<html><form id='frmLogin'><input name='txtUsuario'></form></html>";
    let (r, _) = enviar_com(200, login);
    assert_eq!(r.unwrap_err(), E::TelaDeLoginRecebidaAposEnvio);
}

#[test]
fn resposta_200_sem_marcador_de_resultado_e_recusada() {
    let formulario_de_volta = "<html><form id='frmPesquisaProtocolo'></form></html>";
    let (r, _) = enviar_com(200, formulario_de_volta);
    assert_eq!(r.unwrap_err(), E::MarcadorDeResultadoAusenteNaResposta);
}

#[test]
fn falha_de_rede_nao_repete_o_envio() {
    let (_pasta, arquivo) = cookie(0o600);
    let chamadas = Cell::new(0);
    let r = enviar_submissao_via::<Cursor<Vec<u8>>, _>(
        submissao(),
        &CookieDaSessao::ler(&arquivo),
        |_, _| {
            chamadas.set(chamadas.get() + 1);
            Err(FalhaTransporte::Rede)
        },
    );
    assert_eq!(r.unwrap_err(), E::Transporte(FalhaTransporte::Rede));
    assert_eq!(chamadas.get(), 1);
}

#[cfg(unix)]
#[test]
fn cookie_com_permissao_insegura_impede_qualquer_envio() {
    let (_pasta, arquivo) = cookie(0o644);
    let chamadas = Cell::new(0);
    let r = enviar_submissao_via(submissao(), &CookieDaSessao::ler(&arquivo), |_, _| {
        chamadas.set(chamadas.get() + 1);
        Ok(resposta(200, RESULTADO))
    });
    assert_eq!(
        r.unwrap_err(),
        E::Transporte(FalhaTransporte::ArquivoPrivadoComPermissoesInseguras)
    );
    assert_eq!(chamadas.get(), 0);
}

/// Leitor que acusa qualquer leitura: o corpo do 302 não deve ser lido.
struct CorpoQueNaoDeveSerLido;

impl std::io::Read for CorpoQueNaoDeveSerLido {
    fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
        panic!("corpo do 302 lido")
    }
}

fn enviar_com_redirecionamento(
    status: u16,
    localizacao: Option<&str>,
) -> (Result<RespostaDoEnvio, E>, usize) {
    let (_pasta, arquivo) = cookie(0o600);
    let chamadas = Cell::new(0);
    let r = enviar_submissao_via(submissao(), &CookieDaSessao::ler(&arquivo), |_, _| {
        chamadas.set(chamadas.get() + 1);
        Ok(RespostaDoPost {
            resposta: RespostaWeb {
                status_code: i32::from(status),
                mime: "text/html; charset=ISO-8859-1".to_owned(),
                content_length: Some(0),
                localizacao: None,
                corpo: CorpoQueNaoDeveSerLido,
            },
            localizacao: localizacao.map(str::to_owned),
        })
    });
    (r, chamadas.get())
}

const LOCALIZACAO: &str = "controlador.php?acao=procedimento_trabalhar&id_procedimento=7&infra_hash=dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";

#[test]
fn resposta_302_volta_como_dado_sem_seguir_nem_ler_o_corpo() {
    let (r, n) = enviar_com_redirecionamento(302, Some(LOCALIZACAO));
    let RespostaDoEnvio::Redirecionamento(redirecionamento) = r.unwrap() else {
        panic!("esperado 302 como dado");
    };
    assert_eq!(n, 1);
    assert_eq!(redirecionamento.localizacao(), Some(LOCALIZACAO));
    assert_eq!(redirecionamento.url_do_envio(), submissao().url());
}

// Caso de todos tests/test_redirect_seguro_010.py @323eb01
#[test]
fn location_http_do_302_e_entregue_sem_segunda_requisicao() {
    let local = "http://sei.exemplo.gov.br/sei/controlador.php";
    let (resposta, chamadas) = enviar_com_redirecionamento(302, Some(local));
    let RespostaDoEnvio::Redirecionamento(redirecionamento) = resposta.unwrap() else {
        panic!("esperado 302 como dado");
    };
    assert_eq!(chamadas, 1);
    assert_eq!(redirecionamento.localizacao(), Some(local));
}

// Caso de todos tests/test_redirect_seguro_010.py @323eb01
#[test]
fn post_com_307_ou_308_nao_reenvia_o_formulario() {
    for status in [307, 308] {
        let (resposta, chamadas) = enviar_com_redirecionamento(
            status,
            Some("http://sei.exemplo.gov.br/sei/controlador.php"),
        );
        let Err(E::Transporte(falha)) = resposta else {
            panic!("esperada recusa tipada");
        };
        assert_eq!(falha.status_http().map(|s| s.status), Some(status));
        assert_eq!(chamadas, 1);
    }
}

#[test]
fn resposta_302_sem_location_volta_para_a_operacao_decidir() {
    let (r, _) = enviar_com_redirecionamento(302, None);
    let RespostaDoEnvio::Redirecionamento(redirecionamento) = r.unwrap() else {
        panic!("esperado 302 como dado");
    };
    assert_eq!(redirecionamento.localizacao(), None);
}

#[test]
fn debug_do_302_nao_imprime_location_nem_url_assinada() {
    let (r, _) = enviar_com_redirecionamento(302, Some(LOCALIZACAO));
    let texto = format!("{:?}", r.unwrap());
    assert!(!texto.contains("dddd"), "{texto}");
    assert!(!texto.contains("cccc"), "{texto}");
    assert!(!texto.contains("procedimento_trabalhar"), "{texto}");
    assert!(texto.contains("<redigido>"), "{texto}");
}

#[test]
fn erro_500_apos_envio_expoe_status_e_mensagem_pela_submissao() {
    let (r, n) = enviar_com(
        500,
        "<html><div id='divInfraExcecao'>Falha ao paginar.</div></html>",
    );
    assert_eq!(n, 1);
    let erro = r.unwrap_err();
    assert_eq!(erro.subcodigo(), "status_http_de_erro_do_servidor");
    let http = erro.status_http().expect("status do POST");
    assert_eq!(http.status, 500);
    assert_eq!(
        http.mensagem_do_servidor.as_deref(),
        Some("Falha ao paginar.")
    );
    assert_eq!(E::TelaDeLoginRecebidaAposEnvio.status_http(), None);
}

/// Validação da Infra na estrutura observada (textarea oculto + `alert`).
fn com_validacao(corpo: &str, texto: &str) -> String {
    format!(
        "<html><body>{corpo}<textarea id=\"txaInfraValidacao\" name=\"txaInfraValidacao\" \
         style=\"display:none\">{texto}</textarea><script>self.setTimeout(\
         'alert(document.getElementById(\\'txaInfraValidacao\\').value)',300);</script>\
         </body></html>"
    )
}

#[test]
fn resposta_200_com_validacao_da_infra_e_recusa_com_a_mensagem() {
    let formulario_de_volta = "<form id='frmPesquisaProtocolo'></form>";
    let marcador = "<table class='pesquisaResultado'><tr><td>x</td></tr></table>";
    for corpo in [formulario_de_volta, marcador] {
        let (r, n) = enviar_com(200, &com_validacao(corpo, "Protocolo inv&aacute;lido."));
        let erro = r.unwrap_err();
        assert_eq!(n, 1);
        assert_eq!(erro.subcodigo(), "validacao_recusada_pelo_sei", "{corpo}");
        let http = erro.status_http().expect("mensagem do SEI");
        assert_eq!(http.status, 200);
        assert_eq!(
            http.mensagem_do_servidor.as_deref(),
            Some("Protocolo inválido.")
        );
    }
    // Textarea vazio não é recusa: vale o marcador, como antes.
    let vazio = com_validacao(marcador, "");
    assert_eq!(enviar_com(200, &vazio).0.unwrap(), vazio);
    let sem_marcador = com_validacao(formulario_de_volta, "");
    assert_eq!(
        enviar_com(200, &sem_marcador).0.unwrap_err(),
        E::MarcadorDeResultadoAusenteNaResposta
    );
}
