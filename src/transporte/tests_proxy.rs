//! Proxy do ambiente: ambiente, endereço, exceções e o `CONNECT`, com fluxo
//! injetável e um proxy falso local (`TcpListener`). O TLS através do
//! túnel não é exercitado aqui (exigiria certificado de teste).
use super::*;
use std::io::{BufRead, BufReader, Cursor};
use std::net::TcpListener;
use std::thread;

const SEI: &str = "sei.exemplo.invalid";

fn ambiente<'a>(pares: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
    move |nome| {
        pares
            .iter()
            .find(|(chave, _)| *chave == nome)
            .map(|(_, valor)| (*valor).to_owned())
    }
}

fn endereco(proxy: Option<&Proxy>) -> Option<(String, u16)> {
    proxy.map(|p| (p.host().to_owned(), p.porta()))
}

#[test]
fn variaveis_seguem_a_ordem_e_ignoram_valor_vazio() {
    let par = |h: &str, p| Some((h.to_owned(), p));
    let caso =
        |pares: &[(&str, &str)]| endereco(proxy_para(SEI, ambiente(pares)).unwrap().as_ref());
    assert_eq!(caso(&[]), None);
    assert_eq!(caso(&[("HTTPS_PROXY", "http://px:3128")]), par("px", 3128));
    assert_eq!(
        caso(&[("https_proxy", "http://a:1"), ("HTTPS_PROXY", "http://b:2")]),
        par("a", 1)
    );
    assert_eq!(
        caso(&[("https_proxy", " "), ("ALL_PROXY", "http://c:3")]),
        par("c", 3)
    );
    assert_eq!(
        caso(&[("all_proxy", "http://d:4"), ("ALL_PROXY", "http://e:5")]),
        par("d", 4)
    );
    // O destino é HTTPS: `http_proxy` não vale.
    assert_eq!(caso(&[("http_proxy", "http://f:6")]), None);
}

#[test]
fn excecoes_por_host_exato_sufixo_e_asterisco() {
    assert!(dispensado("sei.exemplo.invalid", "sei.exemplo.invalid"));
    assert!(dispensado(
        "SEI.Exemplo.Invalid",
        "x.invalid, sei.exemplo.invalid"
    ));
    assert!(dispensado("sei.exemplo.invalid", ".exemplo.invalid"));
    assert!(dispensado("exemplo.invalid", ".exemplo.invalid"));
    assert!(dispensado("qualquer.invalid", "*"));
    assert!(dispensado("sei.exemplo.invalid", " a.invalid ,, .invalid "));
    // Entrada sem ponto inicial é host exato, não sufixo.
    assert!(!dispensado("sei.exemplo.invalid", "exemplo.invalid"));
    assert!(!dispensado("maliciosoexemplo.invalid", ".exemplo.invalid"));
    assert!(!dispensado("sei.exemplo.invalid", ""));
    assert!(!dispensado("sei.exemplo.invalid", "*.exemplo.invalid"));
}

#[test]
fn excecao_minuscula_vem_antes_e_dispensa_ate_proxy_invalido() {
    let proxy = ("HTTPS_PROXY", "http://px:3128");
    let com = |pares: &[(&str, &str)]| proxy_para(SEI, ambiente(pares)).unwrap();
    assert!(com(&[proxy, ("NO_PROXY", SEI)]).is_none());
    assert!(com(&[proxy, ("no_proxy", "outro.invalid"), ("NO_PROXY", SEI)]).is_some());
    assert!(com(&[proxy, ("no_proxy", ""), ("NO_PROXY", SEI)]).is_none());
    assert!(com(&[("HTTPS_PROXY", "socks5://px:1"), ("no_proxy", "*")]).is_none());
    assert_eq!(
        proxy_para(SEI, ambiente(&[("HTTPS_PROXY", "px")])),
        Err(FalhaDoProxy::EnderecoInvalido)
    );
}

#[test]
fn endereco_do_proxy_aceita_so_http_com_porta() {
    let ok = interpretar("http://usuario:p%40ss@proxy.local:3128/").unwrap();
    assert_eq!(
        (ok.host(), ok.porta(), ok.tem_autorizacao()),
        ("proxy.local", 3128, true)
    );
    let sem_esquema = interpretar("10.0.0.1:8080").unwrap();
    assert_eq!(
        (sem_esquema.host(), sem_esquema.tem_autorizacao()),
        ("10.0.0.1", false)
    );
    let v6 = interpretar("http://[::1]:3128").unwrap();
    assert_eq!((v6.host(), v6.porta()), ("::1", 3128));
    for esquema in ["https://px:443", "socks5://px:1080", "HTTPS://px:1"] {
        assert_eq!(
            interpretar(esquema),
            Err(FalhaDoProxy::EsquemaNaoSuportado),
            "{esquema}"
        );
    }
    for ruim in [
        "",
        "http://",
        "http://px",
        "http://px:",
        "http://px:0",
        "http://px:65536",
        "http://px:3a",
        "http://:3128",
        "http://px:3128/caminho",
        "http://p x:1",
        "http://u:%zz@px:1",
        "http://px:1?x",
        "http://[::1:3128",
    ] {
        assert_eq!(
            interpretar(ruim),
            Err(FalhaDoProxy::EnderecoInvalido),
            "{ruim}"
        );
    }
}

#[test]
fn connect_leva_host_e_credencial_basic_decodificada() {
    let proxy = interpretar("http://usuario:p%40ss@px:3128").unwrap();
    // base64("usuario:p@ss")
    assert_eq!(
        pedido_connect(&proxy, SEI),
        "CONNECT sei.exemplo.invalid:443 HTTP/1.1\r\nHost: sei.exemplo.invalid:443\r\n\
         Proxy-Authorization: Basic dXN1YXJpbzpwQHNz\r\n\r\n"
    );
    let anonimo = interpretar("px:3128").unwrap();
    assert_eq!(
        pedido_connect(&anonimo, SEI),
        "CONNECT sei.exemplo.invalid:443 HTTP/1.1\r\nHost: sei.exemplo.invalid:443\r\n\r\n"
    );
    let depuracao = format!("{proxy:?}");
    assert!(
        !depuracao.contains("usuario") && !depuracao.contains("dXN1"),
        "{depuracao}"
    );
}

/// Fluxo falso: lê de `entrada`, grava em `saida`.
struct Fluxo {
    entrada: Cursor<Vec<u8>>,
    saida: Vec<u8>,
}

impl Read for Fluxo {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.entrada.read(buf)
    }
}

impl Write for Fluxo {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.saida.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn tunel(resposta: &[u8]) -> (Result<(), FalhaDoProxy>, Fluxo) {
    let mut fluxo = Fluxo {
        entrada: Cursor::new(resposta.to_vec()),
        saida: Vec::new(),
    };
    let proxy = interpretar("px:3128").unwrap();
    (abrir_tunel(&mut fluxo, &proxy, SEI), fluxo)
}

#[test]
fn tunel_aceito_nao_consome_bytes_depois_dos_cabecalhos() {
    let (resultado, fluxo) =
        tunel(b"HTTP/1.1 200 Connection established\r\nVia: px\r\n\r\nTLS-DEPOIS");
    assert_eq!(resultado, Ok(()));
    assert!(
        fluxo
            .saida
            .starts_with(b"CONNECT sei.exemplo.invalid:443 HTTP/1.1\r\n")
    );
    let mut resto = String::new();
    let mut entrada = fluxo.entrada;
    entrada.read_to_string(&mut resto).unwrap();
    assert_eq!(resto, "TLS-DEPOIS");
    assert_eq!(tunel(b"HTTP/1.0 200 OK\n\n").0, Ok(()));
}

#[test]
fn tunel_recusado_traz_o_status_do_proxy() {
    for status in [407_u16, 403, 502] {
        let resposta = format!("HTTP/1.1 {status} Recusa\r\nContent-Length: 0\r\n\r\n");
        assert_eq!(
            tunel(resposta.as_bytes()).0,
            Err(FalhaDoProxy::RecusouOTunel { status })
        );
    }
}

#[test]
fn resposta_ilegivel_ou_truncada_do_proxy() {
    let longa = format!("HTTP/1.1 200 OK\r\nX: {}\r\n\r\n", "a".repeat(20_000));
    for ruim in [
        &b""[..],
        b"HTTP/1.1 200 OK\r\n",
        b"SSH-2.0-x\r\n\r\n",
        b"HTTP/1.1 2000 OK\r\n\r\n",
        b"HTTP/2 200\r\n\r\n",
        longa.as_bytes(),
    ] {
        assert_eq!(tunel(ruim).0, Err(FalhaDoProxy::RespostaNaoInterpretavel));
    }
}

/// Proxy falso: lê o `CONNECT`, responde `status` e, se 200, ecoa uma linha.
fn proxy_falso(status: u16) -> (u16, thread::JoinHandle<String>) {
    let ouvinte = TcpListener::bind("127.0.0.1:0").unwrap();
    let porta = ouvinte.local_addr().unwrap().port();
    let tarefa = thread::spawn(move || {
        let (conexao, _) = ouvinte.accept().unwrap();
        let mut leitor = BufReader::new(conexao.try_clone().unwrap());
        let mut pedido = String::new();
        loop {
            let mut linha = String::new();
            leitor.read_line(&mut linha).unwrap();
            pedido.push_str(&linha);
            if linha == "\r\n" || linha.is_empty() {
                break;
            }
        }
        let mut escrita = conexao;
        write!(escrita, "HTTP/1.1 {status} X\r\n\r\n").unwrap();
        if status == 200 {
            let mut eco = String::new();
            leitor.read_line(&mut eco).unwrap();
            escrita.write_all(eco.as_bytes()).unwrap();
        }
        pedido
    });
    (porta, tarefa)
}

#[test]
fn proxy_local_abre_o_tunel_e_repassa_os_bytes() {
    let (porta, tarefa) = proxy_falso(200);
    let proxy = interpretar(&format!("http://127.0.0.1:{porta}")).unwrap();
    let mut fluxo = conectar_via(&proxy, SEI, Duration::from_secs(5)).unwrap();
    fluxo.write_all(b"ping\n").unwrap();
    let mut eco = String::new();
    BufReader::new(fluxo).read_line(&mut eco).unwrap();
    assert_eq!(eco, "ping\n");
    let pedido = tarefa.join().unwrap();
    assert!(
        pedido.starts_with("CONNECT sei.exemplo.invalid:443 HTTP/1.1\r\n"),
        "{pedido}"
    );
}

#[test]
fn proxy_local_que_recusa_ou_nao_atende() {
    let (porta, tarefa) = proxy_falso(407);
    let proxy = interpretar(&format!("127.0.0.1:{porta}")).unwrap();
    assert_eq!(
        conectar_via(&proxy, SEI, Duration::from_secs(5)).err(),
        Some(FalhaDoProxy::RecusouOTunel { status: 407 })
    );
    tarefa.join().unwrap();
    let fechada = TcpListener::bind("127.0.0.1:0").unwrap();
    let porta = fechada.local_addr().unwrap().port();
    drop(fechada);
    let proxy = interpretar(&format!("127.0.0.1:{porta}")).unwrap();
    assert_eq!(
        conectar_via(&proxy, SEI, Duration::from_secs(5)).err(),
        Some(FalhaDoProxy::Inacessivel)
    );
}

#[test]
fn subcodigos_nomeiam_o_fato() {
    let casos = [
        (FalhaDoProxy::EnderecoInvalido, "endereco_invalido"),
        (FalhaDoProxy::EsquemaNaoSuportado, "esquema_nao_suportado"),
        (FalhaDoProxy::Inacessivel, "inacessivel"),
        (
            FalhaDoProxy::RespostaNaoInterpretavel,
            "resposta_nao_interpretavel",
        ),
        (
            FalhaDoProxy::RecusouOTunel { status: 407 },
            "recusou_o_tunel",
        ),
    ];
    for (falha, _subcodigo) in casos {
        assert_ne!(falha.mensagem(), "nao_implementado");
        let _transporte = crate::transporte::FalhaTransporte::Proxy(falha.clone());
    }
    assert_eq!(
        crate::transporte::FalhaTransporte::Proxy(FalhaDoProxy::RecusouOTunel { status: 407 })
            .status_do_proxy(),
        Some(407)
    );
}
