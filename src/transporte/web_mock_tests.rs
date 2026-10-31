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
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
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
    RespostaWeb {
        status_code: 200,
        mime: "text/html; charset=utf-8".to_owned(),
        content_length: Some(html.len()),
        corpo: Cursor::new(html.as_bytes().to_vec()),
    }
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
                "<tr id='P{id}'><td><a class='processoVisualizado'>NUP-{id}</a></td></tr>"
            )
        })
        .collect();
    format!(
        "<html><a id='lnkInfraUnidade'>TESTE</a>         <input id='hdnTipoVisualizacao' value='R'>         <input id='hdnRecebidosHashCriterios' value='recebidos'>         <input id='hdnGeradosHashCriterios' value='gerados'>         <table id='tblProcessosRecebidos'><caption>{n} registros - 1 a {n}):</caption>{linhas}</table>         <table id='tblProcessosGerados'><caption>0 registros:</caption></table></html>"
    )
}

#[test]
fn web_mock_um_get_resposta_validada_e_cobertura_inteira() {
    let (pasta, cookie) = arquivo_privado_temporario();
    let chamadas = Cell::new(0u32);
    let html = html_caixa(20);
    let lido = read_box_via(
        &url_controle(),
        &cookie,
        "https://sei.treinamento.unb.br/sei",
        |_, header_cookie| {
            assert_eq!(header_cookie, "PHPSESSID=SINTETICO");
            chamadas.set(chamadas.get() + 1);
            Ok(resposta(&html))
        },
    )
    .unwrap();
    let leitura = crate::paginas::juntar(&[lido], Some("TESTE")).unwrap();
    assert!(leitura.recebidos.completa(&leitura.caixa.recebidos));
    assert!(leitura.gerados.completa(&leitura.caixa.gerados));
    assert_eq!(leitura.caixa.recebidos.processos.len(), 20);
    let saida = crate::saida::json(&leitura, "web-mock");
    assert_eq!(saida["recebidos"]["processos"].as_array().unwrap().len(), 20);
    let json = serde_json::to_string(&saida).unwrap();
    assert!(!json.contains("infra_hash"));
    assert!(!json.contains("PHPSESSID"));
    assert_eq!(chamadas.get(), 1);
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
        (302, "text/html", Some(5), FalhaTransporte::Redirecionamento),
        (200, "application/json", Some(5), FalhaTransporte::RespostaInesperada),
        (200, "text/html", Some(crate::MAX_HTML_BYTES + 1), FalhaTransporte::RespostaExcessiva),
    ] {
        let chamadas = Cell::new(0u32);
        let obtido = read_box_via(&url, &cookie, "https://sei.treinamento.unb.br/sei", |_, _| {
            chamadas.set(chamadas.get() + 1);
            Ok(RespostaWeb {
                status_code,
                mime: mime.to_owned(),
                content_length: size,
                corpo: Cursor::new(b"<html>".to_vec()),
            })
        });
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
            let serializado = serde_json::to_vec(&crate::saida::json(&leitura, "web-mock")).unwrap();
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
