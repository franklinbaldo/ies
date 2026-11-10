//! RED/GREEN do envio de `--pagina N`: resposta sintética, no máximo um POST.
use super::tests::{Tab, Tela, instalacao, pen1, tela, url};
use super::*;
use crate::transporte::RespostaWeb;
use crate::transporte::sessao::CookieDaSessao;
use Categoria::Recebidos;
use ErroPaginaDoControle as E;
use std::cell::Cell;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

struct Pasta(PathBuf);

impl Drop for Pasta {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn cookie() -> (Pasta, PathBuf) {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static N: AtomicUsize = AtomicUsize::new(0);
    let pasta = std::env::temp_dir().join(format!(
        "ies-pagina-controle-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&pasta).unwrap();
    let arquivo = pasta.join("cookie");
    fs::write(&arquivo, "PHPSESSID=SINTETICO\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&arquivo, fs::Permissions::from_mode(0o600)).unwrap();
    }
    (Pasta(pasta), arquivo)
}

fn resposta(status_code: i32, html: &str) -> RespostaDoPost<Cursor<Vec<u8>>> {
    RespostaDoPost {
        resposta: RespostaWeb {
            status_code,
            mime: "text/html; charset=iso-8859-1".to_owned(),
            content_length: None,
            localizacao: None,
            corpo: Cursor::new(html.as_bytes().to_vec()),
        },
        localizacao: None,
    }
}

/// Página 2 de Recebidos como o servidor a devolveria.
fn segunda() -> Tela {
    Tela::nova(Tab::nova(114, 101, 114, "1"), Tab::nova(218, 1, 100, "0"))
}

fn obter(
    pagina: usize,
    status: i32,
    devolvida: &str,
    chamadas: &Cell<usize>,
) -> Result<Option<String>, E> {
    let (_pasta, arquivo) = cookie();
    let (url, html) = (url(), pen1().html());
    obter_pagina_via(
        &instalacao(),
        &tela(&url, &html),
        "TESTE",
        Recebidos,
        pagina,
        &CookieDaSessao::ler(&arquivo),
        |submissao, cookie| {
            chamadas.set(chamadas.get() + 1);
            assert_eq!(cookie, "PHPSESSID=SINTETICO");
            assert!(
                String::from_utf8_lossy(submissao.corpo()).contains("hdnRecebidosPaginaAtual=1&")
            );
            Ok(resposta(status, devolvida))
        },
    )
}

#[test]
fn pagina_dois_e_obtida_com_um_unico_post() {
    let chamadas = Cell::new(0);
    let html = obter(2, 200, &segunda().html(), &chamadas).unwrap();
    assert_eq!(chamadas.get(), 1);
    assert!(html.unwrap().contains("NUP-1114"));
}

#[test]
fn pagina_ja_exibida_ou_fora_do_intervalo_nao_envia() {
    let chamadas = Cell::new(0);
    assert_eq!(obter(1, 200, "", &chamadas), Ok(None));
    assert!(matches!(
        obter(3, 200, "", &chamadas),
        Err(E::PaginaForaDoIntervalo { .. })
    ));
    assert_eq!(chamadas.get(), 0);
}

#[test]
fn falhas_da_resposta_sao_tipadas_e_sem_nova_tentativa() {
    let login = "<html><form id='frmLogin' action='login.php'>\
                 <input name='txtUsuario'><input type='password' name='pwdSenha'></form></html>";
    let casos = [
        (
            200,
            login.to_owned(),
            E::Submissao(ErroSubmissaoFormulario::TelaDeLoginRecebidaAposEnvio),
        ),
        (302, String::new(), E::RedirecionamentoAposPaginar),
        (
            200,
            "<html><p>sem tabela</p></html>".to_owned(),
            E::Submissao(ErroSubmissaoFormulario::MarcadorDeResultadoAusenteNaResposta),
        ),
    ];
    for (status, corpo, esperado) in casos {
        let chamadas = Cell::new(0);
        assert_eq!(
            obter(2, status, &corpo, &chamadas),
            Err(esperado),
            "{status}"
        );
        assert_eq!(chamadas.get(), 1);
    }
}

#[test]
fn contexto_alterado_na_resposta_e_recusado() {
    let mut unidade = segunda();
    unidade.unidade = "OUTRA";
    let mut modo = segunda();
    modo.modo = "D";
    let mut hash_r = segunda();
    hash_r.hash_r = "eeee";
    let mut hash_g = segunda();
    hash_g.hash_g = "ffff";
    for (tela, esperado) in [
        (unidade, E::UnidadeAlteradaAposPaginar),
        (modo, E::VisualizacaoAlteradaAposPaginar),
        (hash_r, E::CriteriosRecebidosAlteradosAposPaginar),
        (hash_g, E::CriteriosGeradosAlteradosAposPaginar),
    ] {
        let chamadas = Cell::new(0);
        assert_eq!(obter(2, 200, &tela.html(), &chamadas), Err(esperado));
    }
}

#[test]
fn total_ou_faixa_diferentes_da_pagina_pedida_sao_recusados() {
    let total = Tela::nova(Tab::nova(115, 101, 115, "1"), Tab::nova(218, 1, 100, "0"));
    let repetida = Tela::nova(Tab::nova(114, 1, 100, "0"), Tab::nova(218, 1, 100, "0"));
    let chamadas = Cell::new(0);
    assert_eq!(
        obter(2, 200, &total.html(), &chamadas),
        Err(E::TotalDeclaradoAlteradoAposPaginar {
            categoria: Recebidos
        })
    );
    assert_eq!(
        obter(2, 200, &repetida.html(), &chamadas),
        Err(E::FaixaDaRespostaDiferenteDaPaginaSolicitada {
            categoria: Recebidos
        })
    );
}

#[test]
fn erro_e_debug_nao_expoem_valores_servidos() {
    let erro = E::Submissao(ErroSubmissaoFormulario::Transporte(FalhaTransporte::Rede));
    let texto = format!("{erro:?}");
    assert!(!texto.contains("cccc") && !texto.contains("PHPSESSID"));
}

#[test]
fn erro_503_do_post_com_pagina_valida_e_recusa_com_status_e_mensagem() {
    let chamadas = Cell::new(0);
    let corpo = segunda().html().replace(
        "<html>",
        "<html><div id='divInfraExcecao'>Sobrecarga.</div>",
    );
    let erro = obter(2, 503, &corpo, &chamadas).unwrap_err();
    assert_eq!(chamadas.get(), 1, "sem nova tentativa");
    let http = erro.status_http().expect("status do POST");
    assert_eq!(http.status, 503);
    assert_eq!(http.mensagem_do_servidor.as_deref(), Some("Sobrecarga."));
    assert_eq!(E::RedirecionamentoAposPaginar.status_http(), None);
}
