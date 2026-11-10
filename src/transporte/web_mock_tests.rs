//! Testes de contrato e medição da única leitura Web, sem rede.
//! O adaptador substituto responde exatamente uma vez com bytes sintéticos.
use super::{FalhaTransporte, RespostaWeb, read_box_via};
use std::cell::Cell;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

fn url_controle() -> String {
    format!(
        "https://sei.treinamento.unb.br/sei/controlador.php?acao=procedimento_controlar&infra_hash={}",
        "a".repeat(64)
    )
}

fn arquivo_privado_temporario() -> (PathBuf, PathBuf) {
    let pasta = std::env::temp_dir().join(format!(
        "ies-perf-web-mock-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&pasta).unwrap();
    let cookie = pasta.join("cookie");
    fs::write(&cookie, "PHPSESSID=SINTETICO").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&cookie, fs::Permissions::from_mode(0o600)).unwrap();
    }
    (pasta, cookie)
}

fn resposta(html: &str) -> RespostaWeb<Cursor<Vec<u8>>> {
    super::testes_http::resposta(
        200,
        "text/html; charset=utf-8",
        html.as_bytes().to_vec(),
        Some(html.len()),
    )
}

fn html_caixa(n: usize) -> String {
    use std::fmt::Write as _;
    let mut linhas = String::new();
    for id in 1..=n {
        write!(
            &mut linhas,
            "<tr id='P{id}'><td><a class='processoVisualizado'>NUP-{id}</a></td></tr>"
        )
        .unwrap();
    }
    format!(
        "<html><a id='lnkInfraUnidade'>TESTE</a>         <input id='hdnTipoVisualizacao' value='R'>         <input id='hdnRecebidosHashCriterios' value='recebidos'>         <input id='hdnGeradosHashCriterios' value='gerados'>         <table id='tblProcessosRecebidos'><caption>{n} registros - 1 a {n}):</caption>{linhas}</table>         <table id='tblProcessosGerados'><caption>0 registros:</caption></table></html>"
    )
}

// Caso de todos tests/sei_falso.py @323eb01
#[test]
fn web_mock_um_get_resposta_validada_e_cobertura_inteira() {
    let (pasta, cookie) = arquivo_privado_temporario();
    let mut pedidos = Vec::new();
    let html = html_caixa(20);
    let url = url_controle();
    let mut respostas = std::collections::VecDeque::from([Ok(resposta(&html))]);
    let lido = read_box_via(
        &url,
        &cookie,
        "https://sei.treinamento.unb.br/sei",
        |destino, header_cookie| {
            assert_eq!(header_cookie, "PHPSESSID=SINTETICO");
            assert!(super::validar_origem(destino, "https://sei.treinamento.unb.br/sei").is_ok());
            super::testes_http::atender(&mut pedidos, &mut respostas, "GET")
        },
    )
    .unwrap();
    let leitura = crate::paginas::juntar(&[lido], Some("TESTE")).unwrap();
    assert!(leitura.recebidos.completa(&leitura.caixa.recebidos));
    assert!(leitura.gerados.completa(&leitura.caixa.gerados));
    assert_eq!(leitura.caixa.recebidos.processos.len(), 20);
    let saida = crate::saida::json(&leitura, "web-mock");
    assert_eq!(saida[0]["processos"].as_array().unwrap().len(), 20);
    let json = serde_json::to_string(&saida).unwrap();
    assert!(!json.contains("infra_hash"));
    assert!(!json.contains("PHPSESSID"));
    assert_eq!(pedidos, ["GET"]);
    assert!(respostas.is_empty());
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn web_mock_origem_diferente_recusada_antes_de_cookie_ou_get() {
    let chamadas = Cell::new(0u32);
    let resultado = read_box_via(
        &url_controle(),
        Path::new("/inexistente/cookie"),
        "https://dominio-invalido.exemplo/sei",
        |_, _| {
            chamadas.set(chamadas.get() + 1);
            Ok(resposta("<html></html>"))
        },
    );
    assert_eq!(resultado, Err(FalhaTransporte::OrigemNaoAutorizada));
    assert_eq!(chamadas.get(), 0);
}

#[test]
fn web_mock_redirecionamento_mime_e_limite_rejeitados_sem_retry() {
    let (pasta, cookie) = arquivo_privado_temporario();
    let url = url_controle();
    for (status_code, mime, size, esperado) in [
        (
            302,
            "text/html",
            Some(5),
            FalhaTransporte::Status(super::StatusHttpRecusado::sem_mensagem(302)),
        ),
        (
            200,
            "application/json",
            Some(5),
            FalhaTransporte::TipoDeConteudoNaoHtml,
        ),
        (
            200,
            "text/html",
            Some(crate::MAX_HTML_BYTES + 1),
            FalhaTransporte::RespostaExcessiva,
        ),
    ] {
        let chamadas = Cell::new(0u32);
        let obtido = read_box_via(
            &url,
            &cookie,
            "https://sei.treinamento.unb.br/sei",
            |_, _| {
                chamadas.set(chamadas.get() + 1);
                Ok(RespostaWeb {
                    status_code,
                    mime: mime.to_owned(),
                    content_length: size,
                    localizacao: None,
                    corpo: Cursor::new(b"<html>".to_vec()),
                })
            },
        );
        assert_eq!(obtido, Err(esperado));
        assert_eq!(chamadas.get(), 1);
    }
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn web_mock_benchmark_p50_p95_inclui_transporte_parser_e_json() {
    let (pasta, cookie) = arquivo_privado_temporario();
    for n in [20, 1_000] {
        let html = html_caixa(n);
        let mut tempos = Vec::new();
        for _ in 0..25 {
            let chamadas = Cell::new(0u32);
            let inicio = Instant::now();
            let lido = read_box_via(
                &url_controle(),
                &cookie,
                "https://sei.treinamento.unb.br/sei",
                |_, _| {
                    chamadas.set(chamadas.get() + 1);
                    Ok(resposta(&html))
                },
            )
            .unwrap();
            let leitura = crate::paginas::juntar(&[lido], Some("TESTE")).unwrap();
            assert!(leitura.recebidos.completa(&leitura.caixa.recebidos));
            assert!(leitura.gerados.completa(&leitura.caixa.gerados));
            let serializado =
                serde_json::to_vec(&crate::saida::json(&leitura, "web-mock")).unwrap();
            assert_ne!(serializado.len(), 0);
            assert_eq!(leitura.caixa.recebidos.processos.len(), n);
            assert_eq!(chamadas.get(), 1);
            tempos.push(inicio.elapsed().as_micros());
        }
        tempos.sort_unstable();
        let p50 = tempos[12];
        let p95 = tempos[23];
        println!(
            "WEB_MOCK_BENCH processos={n} requests=1 bytes={} p50_us={p50} p95_us={p95} amostras=25 rede=NOT_RUN",
            html.len()
        );
    }
    fs::remove_dir_all(pasta).unwrap();
}

// Caso de todos tests/test_predicados_de_sessao_010.py @323eb01
#[test]
fn tela_de_login_na_caixa_indica_sessao_vencida_sem_repetir() {
    let (pasta, cookie) = arquivo_privado_temporario();
    let chamadas = Cell::new(0);
    let resultado = read_box_via(
        &url_controle(),
        &cookie,
        "https://sei.treinamento.unb.br/sei",
        |_, _| {
            chamadas.set(chamadas.get() + 1);
            Ok(resposta(
                "<form id='frmLogin'><input name='txtUsuario'></form>",
            ))
        },
    );
    fs::remove_dir_all(pasta).unwrap();
    assert_eq!(
        resultado,
        Err(FalhaTransporte::PaginaDeLoginRecebidaNoControle)
    );
    assert_eq!(chamadas.get(), 1);
}

/// Matriz sintética #237: cada recusa ocorre antes de abrir o cookie
/// (arquivo deliberadamente inexistente) e antes de invocar o transporte.
#[test]
fn matriz_url_host_porta_userinfo_hash_e_duplicatas_sem_cookie() {
    let origem = "https://sei.treinamento.unb.br/sei";
    let hash = "a".repeat(64);
    let base = format!(
        "https://sei.treinamento.unb.br/sei/controlador.php?acao=procedimento_controlar&infra_hash={hash}"
    );
    let casos = [
        base.replace("sei.treinamento.unb.br", "outro.exemplo.test"),
        base.replace("sei.treinamento.unb.br", "sei.treinamento.unb.br:8443"),
        base.replace("https://sei", "https://usuario@sei"),
        format!(
            "//outro.exemplo.test/sei/controlador.php?acao=procedimento_controlar&infra_hash={hash}"
        ),
        base.replace(&hash, "nao-hexadecimal"),
        format!("{base}&infra_hash={hash}"),
        format!("{base}&acao=procedimento_controlar"),
    ];
    for (indice, url) in casos.iter().enumerate() {
        let chamadas = Cell::new(0usize);
        let obtido = read_box_via(
            url,
            Path::new("/arquivo/privado/inexistente"),
            origem,
            |_, _| {
                chamadas.set(chamadas.get() + 1);
                Ok(resposta("<html></html>"))
            },
        );
        assert!(
            matches!(
                obtido,
                Err(FalhaTransporte::UrlInvalida | FalhaTransporte::OrigemNaoAutorizada)
            ),
            "caso {indice} deveria recusar a URL antes do cookie: {obtido:?}"
        );
        assert_eq!(chamadas.get(), 0, "caso {indice} enviou cookie");
    }
}

#[test]
fn matriz_get_3xx_nao_302_recusados_sem_follow() {
    let (pasta, cookie) = arquivo_privado_temporario();
    for codigo in [301, 303, 307, 308] {
        let chamadas = Cell::new(0usize);
        let resultado = read_box_via(
            &url_controle(),
            &cookie,
            "https://sei.treinamento.unb.br/sei",
            |_, _| {
                chamadas.set(chamadas.get() + 1);
                Ok(super::testes_http::resposta(
                    codigo,
                    "text/html",
                    Vec::new(),
                    Some(0),
                ))
            },
        );
        let Err(FalhaTransporte::Status(status)) = resultado else {
            panic!("esperada recusa tipada de {codigo}");
        };
        assert_eq!(i32::from(status.status), codigo);
        assert_eq!(chamadas.get(), 1);
    }
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn matriz_get_erro_de_transporte_nao_repete_pedido() {
    let (pasta, cookie) = arquivo_privado_temporario();
    let chamadas = Cell::new(0usize);
    let resultado = read_box_via::<Cursor<Vec<u8>>, _>(
        &url_controle(),
        &cookie,
        "https://sei.treinamento.unb.br/sei",
        |_, _| {
            chamadas.set(chamadas.get() + 1);
            Err(FalhaTransporte::Rede)
        },
    );
    assert_eq!(resultado, Err(FalhaTransporte::Rede));
    assert_eq!(chamadas.get(), 1);
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn matriz_get_limite_em_stream_sem_content_length() {
    let (pasta, cookie) = arquivo_privado_temporario();
    let chamadas = Cell::new(0usize);
    let resultado = read_box_via(
        &url_controle(),
        &cookie,
        "https://sei.treinamento.unb.br/sei",
        |_, _| {
            chamadas.set(chamadas.get() + 1);
            Ok(super::testes_http::resposta(
                200,
                "text/html",
                vec![b'x'; crate::MAX_HTML_BYTES + 1],
                None,
            ))
        },
    );
    assert_eq!(resultado, Err(FalhaTransporte::RespostaExcessiva));
    assert_eq!(chamadas.get(), 1);
    fs::remove_dir_all(pasta).unwrap();
}

/// Regressão documentada na #241: parâmetro não reservado duplicado
/// é aceito pela validação de URL antes da leitura do cookie.
/// Mantido ignorado até a correção independente do código de produção.
#[test]
#[ignore = "issue #241: rejeitar parâmetros genéricos duplicados"]
fn parametro_generico_duplicado_deve_ser_recusado_antes_do_cookie() {
    let url = format!("{}&parametro=1&parametro=2", url_controle());
    let chamadas = Cell::new(0usize);
    let resultado = read_box_via(
        &url,
        Path::new("/arquivo/privado/inexistente"),
        "https://sei.treinamento.unb.br/sei",
        |_, _| {
            chamadas.set(chamadas.get() + 1);
            Ok(resposta("<html></html>"))
        },
    );
    assert!(
        matches!(resultado, Err(FalhaTransporte::UrlInvalida)),
        "URL duplicada não recusada: {resultado:?}"
    );
    assert_eq!(chamadas.get(), 0);
}
