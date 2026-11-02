//! RED/GREEN do envio injetável: resposta sintética, no máximo uma chamada.
use super::tests::{OPERACAO, instalacao, tela_iso, url_servida};
use super::*;
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

fn resposta(status_code: i32, html: &str) -> RespostaWeb<Cursor<Vec<u8>>> {
    RespostaWeb {
        status_code,
        mime: "text/html; charset=iso-8859-1".to_owned(),
        content_length: Some(html.len()),
        corpo: Cursor::new(html.as_bytes().to_vec()),
    }
}

fn enviar_com(status_code: i32, html: &str) -> (Result<String, E>, usize) {
    let (_pasta, arquivo) = cookie(0o600);
    let chamadas = Cell::new(0);
    let r = enviar_submissao_via(submissao(), &arquivo, |_, _| {
        chamadas.set(chamadas.get() + 1);
        Ok(resposta(status_code, html))
    });
    (r, chamadas.get())
}

const RESULTADO: &str = "<html><table class='pesquisaResultado'><tr><td>x</td></tr></table></html>";

#[test]
fn resposta_com_marcador_de_resultado_e_aceita_com_uma_chamada() {
    let (r, n) = enviar_com(200, RESULTADO);
    assert_eq!(r.unwrap(), RESULTADO);
    assert_eq!(n, 1);
}

#[test]
fn closure_recebe_post_corpo_cabecalhos_e_cookie_da_sessao() {
    let (_pasta, arquivo) = cookie(0o600);
    let r = enviar_submissao_via(submissao(), &arquivo, |s, cookie| {
        assert_eq!(s.metodo(), "POST");
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
fn redirecionamento_e_negacao_apos_envio_sao_falhas_de_transporte() {
    for (status, falha) in [
        (302, FalhaTransporte::Redirecionamento),
        (303, FalhaTransporte::Redirecionamento),
        (401, FalhaTransporte::AcessoNegado),
        (403, FalhaTransporte::AcessoNegado),
    ] {
        let (r, n) = enviar_com(status, "");
        assert_eq!(r.unwrap_err(), E::Transporte(falha), "{status}");
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
    let r = enviar_submissao_via::<Cursor<Vec<u8>>, _>(submissao(), &arquivo, |_, _| {
        chamadas.set(chamadas.get() + 1);
        Err(FalhaTransporte::Rede)
    });
    assert_eq!(r.unwrap_err(), E::Transporte(FalhaTransporte::Rede));
    assert_eq!(chamadas.get(), 1);
}

#[cfg(unix)]
#[test]
fn cookie_com_permissao_insegura_impede_qualquer_envio() {
    let (_pasta, arquivo) = cookie(0o644);
    let chamadas = Cell::new(0);
    let r = enviar_submissao_via(submissao(), &arquivo, |_, _| {
        chamadas.set(chamadas.get() + 1);
        Ok(resposta(200, RESULTADO))
    });
    assert_eq!(
        r.unwrap_err(),
        E::Transporte(FalhaTransporte::ArquivoPrivadoComPermissoesInseguras)
    );
    assert_eq!(chamadas.get(), 0);
}
