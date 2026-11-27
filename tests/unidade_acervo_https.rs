//! E2E sintético da consulta histórica: binário real, HTTPS em loopback.
//! O replay original continua separado; não é paridade externa.
#![cfg(unix)]

use native_tls::TlsAcceptor;
use serde_json::Value;
use std::io::Write;
use std::net::TcpListener;
use std::process::{Command, Output};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

#[path = "aies_documento_criar_cli/fixture_tls.rs"]
mod fixture_tls;
#[path = "aies_pdf_externo_cli/bancada.rs"]
mod bancada;

use fixture_tls::{Pasta, identidade};

const ORIGEM: &str = "https://sei.test/sei";
const A: &str = "00001.000001/2024-01";
const B: &str = "00001.000002/2024-02";

fn link(acao: &str) -> String {
    format!("controlador.php?acao={acao}&amp;infra_hash={}", "a".repeat(64))
}

fn controle() -> String {
    format!(
        "<html><body><a id='lnkInfraUnidade'>PPI</a>\
         <input id='hdnTipoVisualizacao' value='R'>\
         <table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
         <table id='tblProcessosGerados'><caption>0 registros:</caption></table>\
         <a href='{}'>Estatísticas</a></body></html>",
        link("gerar_estatisticas_unidade")
    )
}

fn formulario() -> String {
    format!(
        "<a id='lnkInfraUnidade'>PPI</a><form id='frmEstatisticas' method='post' action='{}'>\
         <select name='selOrgao'><option value='null' selected>Todos</option></select>\
         <input name='txtPeriodoDe'><input name='txtPeriodoA'>\
         <input type='submit' name='sbmPesquisar' value='Pesquisar'></form>",
        link("gerar_estatisticas_unidade")
    )
}

fn relatorio(vazio: bool) -> String {
    let total = if vazio {
        "0".to_owned()
    } else {
        format!(
            "<a onclick=\"abrirDetalhe('{}&amp;tabela_estatisticas=4');\">3</a>",
            link("estatisticas_detalhar_unidade")
        )
    };
    format!(
        "<table><caption>Processos com andamento aberto na unidade ao final do período:</caption>\
         <tr class='totalEstatisticas'><td>TOTAL:</td><td>{total}</td></tr></table>"
    )
}

fn pagina(inicio: usize, fim: usize, nups: &[&str]) -> String {
    let linhas: String = nups
        .iter()
        .map(|nup| format!("<tr><td>O</td><td>PPI</td><td>{nup}</td><td>T</td><td>2</td><td>2024</td></tr>"))
        .collect();
    format!(
        "<form id='frmEstatisticasLista' method='post' action='{}'>\
         <input type='hidden' name='hdnInfraPaginaAtual' value='0'>\
         <table><caption>Lista de Registros de Detalhamento (3 registros - {inicio} a {fim}):</caption>\
         <tr><th>Órgão</th><th>Unidade</th><th>Processo</th><th>Tipo</th><th>Mês</th><th>Ano</th></tr>\
         {linhas}</table></form>",
        link("estatisticas_detalhar_unidade")
    )
}

#[derive(Clone, Copy)]
enum Caso {
    Completo,
    Repetido,
    Vazio,
}

type Passo = (&'static str, &'static str, &'static str, String);

fn roteiro(caso: Caso) -> Vec<Passo> {
    let mut passos = vec![
        ("GET", "procedimento_controlar", "", controle()),
        ("GET", "gerar_estatisticas_unidade", "", formulario()),
        (
            "POST",
            "gerar_estatisticas_unidade",
            "selOrgao=&txtPeriodoDe=29%2F02%2F2024&txtPeriodoA=29%2F02%2F2024&sbmPesquisar=Pesquisar",
            relatorio(matches!(caso, Caso::Vazio)),
        ),
    ];
    if !matches!(caso, Caso::Vazio) {
        passos.push(("GET", "estatisticas_detalhar_unidade", "", pagina(1, 2, &[A, A])));
        let ultima = match caso {
            Caso::Completo => pagina(3, 3, &[B]),
            Caso::Repetido => pagina(1, 2, &[A, A]),
            Caso::Vazio => unreachable!(),
        };
        passos.push(("POST", "estatisticas_detalhar_unidade", "hdnInfraPaginaAtual=1", ultima));
    }
    passos
}

fn servir(escuta: TcpListener, parar: &AtomicBool, tls: &TlsAcceptor, passos: Vec<Passo>) -> usize {
    let mut passos = passos.into_iter();
    let mut total = 0;
    loop {
        let mut tcp = match escuta.accept() {
            Ok((tcp, peer)) => {
                assert!(peer.ip().is_loopback());
                tcp
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if parar.load(Ordering::SeqCst) {
                    break;
                }
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            Err(e) => panic!("accept: {e}"),
        };
        tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        tcp.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
        let (connect, _, _) = bancada::pedido(&mut tcp);
        assert!(connect.starts_with(b"CONNECT sei.test:443 HTTP/1.1\r\n"));
        tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n").unwrap();
        let mut canal = tls.accept(tcp).unwrap();
        let (bytes, metodo, acao) = bancada::pedido(&mut canal);
        let (m, a, corpo, html) = passos.next().expect("requisição além do roteiro");
        assert_eq!((metodo.as_str(), acao.as_str()), (m, a));
        let pedido = std::str::from_utf8(&bytes).unwrap();
        let (cabecalho, recebido) = pedido.split_once("\r\n\r\n").unwrap();
        assert_eq!(recebido, corpo);
        if total == 3 {
            assert!(cabecalho.lines().next().unwrap().contains("tabela_estatisticas=4"));
        }
        write!(
            canal,
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{html}",
            html.len()
        ).unwrap();
        canal.flush().unwrap();
        total += 1;
    }
    assert!(passos.next().is_none(), "roteiro não consumido");
    total
}

fn executar(caso: Caso) -> (Output, usize) {
    let pasta = Pasta::nova();
    let (cert, identidade_tls) = identidade(&pasta.0, "sei.test");
    let origem = pasta.privado("origem", ORIGEM);
    let controle = pasta.privado(
        "url",
        &format!(
            "{ORIGEM}/controlador.php?acao=procedimento_controlar&infra_hash={}",
            "a".repeat(64)
        ),
    );
    let cookie = pasta.privado("cookie", "PHPSESSID=FICTICIO");
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    escuta.set_nonblocking(true).unwrap();
    let porta = escuta.local_addr().unwrap().port();
    let parar = Arc::new(AtomicBool::new(false));
    let sinal = Arc::clone(&parar);
    let servidor = thread::spawn(move || {
        servir(escuta, &sinal, &TlsAcceptor::new(identidade_tls).unwrap(), roteiro(caso))
    });
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["--json", "unidade", "acervo", "--em", "2024-02-29"])
        .env("IES_URL_CONTROLE_FILE", controle)
        .env("IES_COOKIE_FILE", cookie)
        .env("IES_ORIGEM_FILE", origem)
        .env("SSL_CERT_FILE", cert)
        .env("https_proxy", format!("http://127.0.0.1:{porta}"))
        .env("HOME", &pasta.0)
        .env_remove("HTTPS_PROXY")
        .env_remove("ALL_PROXY")
        .env_remove("all_proxy")
        .env_remove("NO_PROXY")
        .env_remove("no_proxy")
        .env_remove("IES_CONFIG_FILE")
        .env_remove("IES_CREDENCIAIS_FILE")
        .output();
    parar.store(true, Ordering::SeqCst);
    let total = servidor.join().unwrap();
    (saida.unwrap(), total)
}

#[test]
fn acervo_historico_binario_preserva_linhas_e_conta_nups() {
    let (saida, total) = executar(Caso::Completo);
    assert!(saida.status.success(), "{}", String::from_utf8_lossy(&saida.stderr));
    let json: Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(total, 5);
    assert_eq!(json["unidade"], "PPI");
    assert_eq!(json["data"], "29/02/2024");
    assert_eq!(json["total_declarado"], 3);
    assert_eq!(json["linhas_observadas"], 3);
    assert_eq!(json["nups_distintos"], 2);
    let processos = json["processos"].as_array().unwrap();
    assert_eq!(processos.len(), 3);
    assert_eq!(processos[0]["processo"], A);
    assert_eq!(processos[1]["processo"], A);
    assert_eq!(processos[2]["processo"], B);
}

#[test]
fn acervo_historico_binario_recusa_pagina_repetida_sem_parcial() {
    let (saida, total) = executar(Caso::Repetido);
    assert_eq!(total, 5);
    assert_eq!(saida.status.code(), Some(1));
    assert!(saida.stdout.is_empty());
    assert!(String::from_utf8_lossy(&saida.stderr).contains("FaixaIncompleta"));
}

#[test]
fn acervo_historico_binario_zero_explicito_sem_detalhe() {
    let (saida, total) = executar(Caso::Vazio);
    assert!(saida.status.success(), "{}", String::from_utf8_lossy(&saida.stderr));
    let json: Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(total, 3);
    assert_eq!(json["total_declarado"], 0);
    assert_eq!(json["linhas_observadas"], 0);
    assert_eq!(json["nups_distintos"], 0);
    assert!(json["processos"].as_array().unwrap().is_empty());
}
