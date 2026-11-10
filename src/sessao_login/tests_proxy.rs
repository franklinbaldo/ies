//! O cliente do login usa o proxy do ambiente: o `CONNECT` vai ao proxy
//! falso local sem resolver o host do SEI; a exceção conecta direto. O TLS
//! através do túnel não é exercitado (exigiria certificado de teste).
use crate::sessao_login_cli::json_do_erro;
use crate::transporte::FalhaTransporte;
use crate::transporte::cliente_https::abrir_conexao;
use crate::transporte::proxy::FalhaDoProxy;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

const SEI: &str = "sei.exemplo.invalid";

fn ambiente(pares: Vec<(&'static str, String)>) -> impl Fn(&str) -> Option<String> {
    move |nome| {
        pares
            .iter()
            .find(|(c, _)| *c == nome)
            .map(|(_, v)| v.clone())
    }
}

#[test]
fn conexao_do_login_passa_pelo_proxy_do_ambiente() {
    let ouvinte = TcpListener::bind("127.0.0.1:0").unwrap();
    let porta = ouvinte.local_addr().unwrap().port();
    let tarefa = std::thread::spawn(move || {
        let (conexao, _) = ouvinte.accept().unwrap();
        let mut linha = String::new();
        let mut leitor = BufReader::new(conexao.try_clone().unwrap());
        leitor.read_line(&mut linha).unwrap();
        let mut resto = String::new();
        while resto != "\r\n" {
            resto.clear();
            if leitor.read_line(&mut resto).unwrap() == 0 {
                break;
            }
        }
        let mut escrita = conexao;
        escrita.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
        linha
    });
    // O host `.invalid` não resolve: só o proxy pode tê-lo recebido.
    let tcp = abrir_conexao(
        SEI,
        ambiente(vec![("HTTPS_PROXY", format!("http://127.0.0.1:{porta}"))]),
    );
    assert!(tcp.is_ok(), "{:?}", tcp.err());
    assert_eq!(
        tarefa.join().unwrap(),
        "CONNECT sei.exemplo.invalid:443 HTTP/1.1\r\n"
    );
}

#[test]
fn excecao_conecta_direto_e_proxy_invalido_e_recusado() {
    let proxy = ("HTTPS_PROXY", "http://127.0.0.1:9".to_owned());
    let direto = abrir_conexao(SEI, ambiente(vec![proxy, ("NO_PROXY", SEI.to_owned())]));
    assert_eq!(direto.err(), Some(FalhaTransporte::Rede));
    let invalido = abrir_conexao(
        SEI,
        ambiente(vec![("https_proxy", "https://px:1".to_owned())]),
    );
    assert_eq!(
        invalido.err(),
        Some(FalhaTransporte::Proxy(FalhaDoProxy::EsquemaNaoSuportado))
    );
}

#[test]
fn recusa_do_tunel_publica_o_status_do_proxy_sem_status_http() {
    let erro = super::ErroLogin::Transporte(FalhaTransporte::Proxy(FalhaDoProxy::RecusouOTunel {
        status: 407,
    }));
    let valor = json_do_erro(&erro);
    assert_eq!(valor["status_do_proxy"], 407);
    assert!(valor.get("status_http").is_none());
    let valor = json_do_erro(&super::ErroLogin::Transporte(FalhaTransporte::Proxy(
        FalhaDoProxy::Inacessivel,
    )));
    assert!(valor.get("status_do_proxy").is_none());
}
