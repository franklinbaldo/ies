//! RED/GREEN do cliente HTTPS único nos comandos: o GET do Controle e o
//! POST atravessam o proxy do ambiente (`CONNECT` a um proxy falso local) e
//! vão direto com `NO_PROXY`; 3xx não é seguido; limites, decodificação e
//! subcódigos mantidos. A camada TLS é trocada pela identidade: o "SEI"
//! fala HTTP puro dentro do túnel. O `native-tls` real não é exercitado.
use super::envio_post::enviar_post_via;
use super::proxy::FalhaDoProxy;
use super::{FalhaTransporte, StatusHttpRecusado, decodificar_resposta, get_https_via};
use crate::formulario_web::SubmissaoDerivada;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

const SEI: &str = "sei.exemplo.invalid";

fn url() -> String {
    format!(
        "https://{SEI}/sei/controlador.php?acao=procedimento_controlar&infra_hash={}",
        "a".repeat(64)
    )
}

fn ambiente(pares: Vec<(&'static str, String)>) -> impl Fn(&str) -> Option<String> {
    move |nome| {
        pares
            .iter()
            .find(|(c, _)| *c == nome)
            .map(|(_, v)| v.clone())
    }
}

/// Camada TLS substituta: o fluxo do túnel sem cifra.
#[allow(clippy::unnecessary_wraps)]
fn sem_tls(host: &str, tcp: TcpStream) -> Result<TcpStream, FalhaTransporte> {
    assert_eq!(host, SEI, "TLS precisa usar o host do SEI");
    Ok(tcp)
}

/// O que o proxy falso recebeu: `CONNECT`, pedido no túnel e seu corpo.
struct Recebido {
    connect: String,
    pedido: String,
    corpo: Vec<u8>,
}

fn aceitar(ouvinte: &TcpListener, prazo: Duration) -> Option<TcpStream> {
    let fim = Instant::now() + prazo;
    while Instant::now() < fim {
        if let Ok((conexao, _)) = ouvinte.accept() {
            conexao.set_nonblocking(false).unwrap();
            return Some(conexao);
        }
        thread::sleep(Duration::from_millis(5));
    }
    None
}

fn cabecalhos<R: BufRead>(leitor: &mut R) -> String {
    let mut texto = String::new();
    loop {
        let mut linha = String::new();
        if leitor.read_line(&mut linha).unwrap_or(0) == 0 || linha == "\r\n" {
            return texto;
        }
        texto.push_str(&linha);
    }
}

/// Proxy falso de uma conexão: responde `status` ao `CONNECT`; se 200, lê o
/// pedido HTTP do túnel e devolve `resposta`. Depois conta as conexões que
/// chegarem em 300 ms (redirect seguido ou retry seriam uma segunda).
fn proxy_falso(
    status: u16,
    resposta: Vec<u8>,
) -> (u16, thread::JoinHandle<(Option<Recebido>, usize)>) {
    let ouvinte = TcpListener::bind("127.0.0.1:0").unwrap();
    ouvinte.set_nonblocking(true).unwrap();
    let porta = ouvinte.local_addr().unwrap().port();
    let tarefa = thread::spawn(move || {
        let Some(conexao) = aceitar(&ouvinte, Duration::from_secs(5)) else {
            return (None, 0);
        };
        let mut leitor = BufReader::new(conexao.try_clone().unwrap());
        let connect = cabecalhos(&mut leitor);
        let mut escrita = conexao;
        let _ = write!(escrita, "HTTP/1.1 {status} X\r\nContent-Length: 0\r\n\r\n");
        let mut recebido = Recebido {
            connect,
            pedido: String::new(),
            corpo: Vec::new(),
        };
        if status == 200 {
            recebido.pedido = cabecalhos(&mut leitor);
            let tamanho = recebido
                .pedido
                .lines()
                .find_map(|l| l.strip_prefix("Content-Length: "))
                .map_or(0, |n| n.trim().parse::<usize>().unwrap());
            recebido.corpo.resize(tamanho, 0);
            leitor.read_exact(&mut recebido.corpo).unwrap();
            // O cliente pode parar de ler no limite: escrita parcial é esperada.
            let _ = escrita.write_all(&resposta);
        }
        let _ = escrita.shutdown(std::net::Shutdown::Both);
        drop((leitor, escrita));
        let extras = usize::from(aceitar(&ouvinte, Duration::from_millis(300)).is_some());
        (Some(recebido), extras)
    });
    (porta, tarefa)
}

fn via_proxy(porta: u16) -> Vec<(&'static str, String)> {
    vec![(
        "HTTPS_PROXY",
        format!("http://usuario:segredo@127.0.0.1:{porta}"),
    )]
}

/// GET pelo proxy falso: resultado da leitura do Controle e o que chegou.
fn get_pelo_proxy(resposta: &[u8]) -> (Result<String, FalhaTransporte>, Recebido) {
    let (porta, tarefa) = proxy_falso(200, resposta.to_vec());
    let resultado = get_https_via(
        &url(),
        "PHPSESSID=SINTETICO",
        ambiente(via_proxy(porta)),
        sem_tls,
    )
    .and_then(decodificar_resposta);
    let (recebido, extras) = tarefa.join().unwrap();
    assert_eq!(extras, 0, "nenhum pedido além do primeiro");
    (
        resultado,
        recebido.expect("o proxy precisa receber o CONNECT"),
    )
}

#[test]
fn get_do_controle_atravessa_o_proxy_do_ambiente() {
    let corpo = "<html><body>Controle</body></html>";
    let resposta = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\
         Content-Length: {}\r\n\r\n{corpo}",
        corpo.len()
    );
    let (html, recebido) = get_pelo_proxy(resposta.as_bytes());
    assert_eq!(html.unwrap(), corpo);
    assert!(
        recebido
            .connect
            .starts_with("CONNECT sei.exemplo.invalid:443 HTTP/1.1\r\n"),
        "{}",
        recebido.connect
    );
    assert!(recebido.connect.contains("Proxy-Authorization: Basic "));
    let alvo = format!("GET {} HTTP/1.1\r\n", &url()[27..]);
    assert!(recebido.pedido.starts_with(&alvo), "{}", recebido.pedido);
    assert!(recebido.pedido.contains("Host: sei.exemplo.invalid\r\n"));
    assert!(recebido.pedido.contains("Cookie: PHPSESSID=SINTETICO\r\n"));
    // A credencial do proxy nunca segue para o SEI.
    assert!(!recebido.pedido.contains("Proxy-Authorization"));
}

#[test]
fn get_dispensado_pelo_no_proxy_conecta_direto() {
    let ouvinte = TcpListener::bind("127.0.0.1:0").unwrap();
    ouvinte.set_nonblocking(true).unwrap();
    let porta = ouvinte.local_addr().unwrap().port();
    for excecao in [("NO_PROXY", SEI), ("no_proxy", ".exemplo.invalid")] {
        let mut pares = via_proxy(porta);
        pares.push((excecao.0, excecao.1.to_owned()));
        let resultado = get_https_via(&url(), "PHPSESSID=SINTETICO", ambiente(pares), sem_tls)
            .and_then(decodificar_resposta);
        // Direto, o host `.invalid` não resolve: falha de rede, sem proxy.
        assert_eq!(resultado, Err(FalhaTransporte::Rede));
    }
    assert!(aceitar(&ouvinte, Duration::from_millis(100)).is_none());
}

#[test]
fn get_3xx_pelo_proxy_nao_e_seguido_nem_lido() {
    let resposta = b"HTTP/1.1 302 Found\r\nLocation: https://outro.invalid/sei/\r\n\
        Content-Type: text/html\r\nContent-Length: 8\r\n\r\nnao lido";
    let (erro, _) = get_pelo_proxy(resposta);
    let erro = erro.unwrap_err();
    assert_eq!(
        erro,
        FalhaTransporte::Redirecionamento(StatusHttpRecusado::sem_mensagem(302))
    );
    assert_eq!(erro.subcodigo(), "redirecionamento");
}

#[test]
fn get_pelo_proxy_mantem_limite_de_8_mb_e_tipo_html() {
    let declarado =
        b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 9000000\r\n\r\n";
    assert_eq!(
        get_pelo_proxy(declarado).0,
        Err(FalhaTransporte::RespostaExcessiva)
    );
    let mut em_fluxo = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n".to_vec();
    em_fluxo.resize(em_fluxo.len() + crate::MAX_HTML_BYTES + 1, b'a');
    assert_eq!(
        get_pelo_proxy(&em_fluxo).0,
        Err(FalhaTransporte::RespostaExcessiva)
    );
    let json = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{}";
    let erro = get_pelo_proxy(json).0.unwrap_err();
    assert_eq!(erro.subcodigo(), "tipo_de_conteudo_nao_html");
}

#[test]
fn get_pelo_proxy_decodifica_charset_e_corpo_em_pedacos() {
    let mut iso = b"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=iso-8859-1\r\n\
        Transfer-Encoding: chunked\r\n\r\n6\r\n<p>a"
        .to_vec();
    iso.extend_from_slice(&[0xE7, 0xE3]);
    iso.extend_from_slice(b"\r\n4\r\n</p>\r\n0\r\n\r\n");
    assert_eq!(get_pelo_proxy(&iso).0.unwrap(), "<p>açã</p>");
}

#[test]
fn recusa_http_pelo_proxy_le_no_maximo_64_kib_do_corpo() {
    let curta = b"HTTP/1.1 503 X\r\nContent-Type: text/plain\r\n\r\nem manutencao";
    let erro = get_pelo_proxy(curta).0.unwrap_err();
    assert_eq!(erro.subcodigo(), "status_http_de_erro_do_servidor");
    let http = erro.status_http().unwrap();
    assert_eq!(
        (http.status, http.mensagem_do_servidor.as_deref()),
        (503, Some("em manutencao"))
    );
    // Só espaços nos primeiros 64 KiB: o texto depois deles nunca é lido.
    let mut longa = b"HTTP/1.1 500 X\r\nContent-Type: text/plain\r\n\r\n".to_vec();
    longa.resize(longa.len() + super::corpo_de_recusa::LIMITE, b' ');
    longa.extend_from_slice(b"ALEM_DO_LIMITE");
    let erro = get_pelo_proxy(&longa).0.unwrap_err();
    assert_eq!(
        erro,
        FalhaTransporte::StatusHttpDeErroDoServidor(StatusHttpRecusado::sem_mensagem(500))
    );
    for (status, subcodigo) in [
        (401, "acesso_negado"),
        (429, "status_http_de_limite_de_requisicoes"),
        (404, "status_http_diferente_de_200"),
    ] {
        let resposta = format!("HTTP/1.1 {status} X\r\nContent-Length: 0\r\n\r\n");
        let erro = get_pelo_proxy(resposta.as_bytes()).0.unwrap_err();
        assert_eq!(erro.subcodigo(), subcodigo);
        assert_eq!(erro.status_http().map(|h| h.status), Some(status));
    }
}

#[test]
fn proxy_que_recusa_o_tunel_ou_invalido_no_get() {
    let (porta, tarefa) = proxy_falso(407, Vec::new());
    let erro = get_https_via(
        &url(),
        "PHPSESSID=SINTETICO",
        ambiente(via_proxy(porta)),
        sem_tls,
    )
    .map(|_| ())
    .unwrap_err();
    assert_eq!(
        erro,
        FalhaTransporte::Proxy(FalhaDoProxy::RecusouOTunel { status: 407 })
    );
    assert_eq!(erro.subcodigo(), "recusou_o_tunel");
    assert_eq!(erro.status_do_proxy(), Some(407));
    assert!(erro.status_http().is_none());
    assert_eq!(tarefa.join().unwrap().1, 0);
    let invalido = ambiente(vec![("https_proxy", "socks5://127.0.0.1:1".to_owned())]);
    let erro = get_https_via(&url(), "PHPSESSID=SINTETICO", invalido, sem_tls)
        .map(|_| ())
        .unwrap_err();
    assert_eq!(erro.subcodigo(), "esquema_nao_suportado");
}

const FORM: &str = "<html><a id='lnkInfraUnidade'>UNIDADE TESTE</a>\
    <form id='frmPesquisaProtocolo' method='post' \
    action='controlador.php?acao=protocolo_pesquisar&amp;infra_hash=cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc'>\
    <input type='hidden' name='hdnInfraTipoPagina' value='1'>\
    <input type='text' name='txtProtocoloPesquisa' value=''>\
    <button type='submit' name='sbmPesquisar' value='Pesquisar'>Pesquisar</button>\
    <input type='hidden' name='hdnInicio' value='0'></form></html>";

fn tela() -> String {
    format!(
        "https://{SEI}/sei/controlador.php?acao=protocolo_pesquisar&infra_hash={}",
        "b".repeat(64)
    )
}

fn submissao() -> SubmissaoDerivada {
    use crate::formulario_web::{CodificacaoDaTela, TelaServida, derivar_submissao};
    let instalacao = crate::instalacao::Instalacao::interpretar("https://sei.exemplo.invalid/sei");
    let url = tela();
    let tela = TelaServida {
        url: &url,
        html: FORM,
        codificacao: CodificacaoDaTela::Iso88591,
    };
    derivar_submissao(
        &crate::formulario_web::tests::OPERACAO,
        &instalacao.unwrap(),
        &tela,
        "UNIDADE TESTE",
        &[("txtProtocoloPesquisa", "0001")],
    )
    .unwrap()
}

#[test]
fn post_atravessa_o_proxy_e_302_volta_como_dado() {
    let resposta = b"HTTP/1.1 302 Found\r\nLocation: controlador.php?acao=x\r\n\
        Content-Length: 8\r\n\r\nnao lido";
    let (porta, tarefa) = proxy_falso(200, resposta.to_vec());
    let submissao = submissao();
    let corpo = submissao.corpo().to_vec();
    let caminho = submissao.url()[27..].to_owned();
    let post = enviar_post_via(
        submissao,
        "PHPSESSID=SINTETICO",
        ambiente(via_proxy(porta)),
        sem_tls,
    )
    .unwrap();
    assert_eq!(post.resposta.status_code, 302);
    assert_eq!(post.localizacao.as_deref(), Some("controlador.php?acao=x"));
    let (recebido, extras) = tarefa.join().unwrap();
    let recebido = recebido.unwrap();
    assert_eq!(extras, 0);
    assert!(
        recebido
            .connect
            .starts_with("CONNECT sei.exemplo.invalid:443 HTTP/1.1\r\n")
    );
    assert!(
        recebido
            .pedido
            .starts_with(&format!("POST {caminho} HTTP/1.1\r\n"))
    );
    for cabecalho in [
        "Host: sei.exemplo.invalid\r\n".to_owned(),
        "Cookie: PHPSESSID=SINTETICO\r\n".to_owned(),
        "Content-Type: application/x-www-form-urlencoded\r\n".to_owned(),
        "Origin: https://sei.exemplo.invalid\r\n".to_owned(),
        format!("Referer: {}\r\n", tela()),
        format!("Content-Length: {}\r\n", corpo.len()),
    ] {
        assert!(recebido.pedido.contains(&cabecalho), "{cabecalho}");
    }
    assert_eq!(recebido.corpo, corpo);
}

#[test]
fn post_pelo_proxy_publica_status_e_mensagem_da_recusa() {
    let resposta = b"HTTP/1.1 500 X\r\nContent-Type: text/plain\r\n\r\nfalha interna";
    let (porta, tarefa) = proxy_falso(200, resposta.to_vec());
    let post = enviar_post_via(
        submissao(),
        "PHPSESSID=SINTETICO",
        ambiente(via_proxy(porta)),
        sem_tls,
    )
    .unwrap();
    assert_eq!(post.localizacao, None);
    let erro = decodificar_resposta(post.resposta).unwrap_err();
    assert_eq!(erro.subcodigo(), "status_http_de_erro_do_servidor");
    assert_eq!(
        erro.status_http()
            .and_then(|h| h.mensagem_do_servidor.as_deref()),
        Some("falha interna")
    );
    assert_eq!(tarefa.join().unwrap().1, 0);
}

#[test]
fn post_dispensado_pelo_no_proxy_conecta_direto() {
    let ouvinte = TcpListener::bind("127.0.0.1:0").unwrap();
    ouvinte.set_nonblocking(true).unwrap();
    let porta = ouvinte.local_addr().unwrap().port();
    let mut pares = via_proxy(porta);
    pares.push(("NO_PROXY", "*".to_owned()));
    let post = enviar_post_via(submissao(), "PHPSESSID=SINTETICO", ambiente(pares), sem_tls);
    assert_eq!(post.map(|_| ()).unwrap_err(), FalhaTransporte::Rede);
    assert!(aceitar(&ouvinte, Duration::from_millis(100)).is_none());
}

#[test]
fn dado_pdf_quando_envio_multipart_em_fluxo_entao_ha_um_post_com_tamanho_exato() {
    // Dado o corpo filArquivo observado no Todos #775 (UnB de treinamento);
    // quando o HTTPS envia o multipart; então o proxy sintético vê bytes exatos
    // e Content-Length, sem uma segunda tentativa ou acesso real ao SEI.
    use super::cliente_https::{self, EnvioEmFluxo};
    use crate::documento_externo_incluir::multipart::preparar_pdf;
    use crate::formulario_web::CodificacaoDaTela;
    let pdf: &[u8] = b"%PDF-1.4\n%%EOF\n";
    let plano = preparar_pdf(
        "prova.pdf",
        pdf.len() as u64,
        "----ies332",
        CodificacaoDaTela::Iso88591,
    )
    .unwrap();
    let mut esperado = Vec::new();
    plano.corpo(pdf).read_to_end(&mut esperado).unwrap();
    let referer = url();
    let destino = format!(
        "https://{SEI}/sei/controlador.php?acao=documento_upload_anexo&infra_hash={}",
        "c".repeat(64)
    );
    let recibo_servido = format!(
        "94e336aa#prova.pdf#application/pdf#{}#11/10/2026 11:07:50#",
        pdf.len()
    );
    let resposta = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=iso-8859-1\r\nContent-Length: {}\r\n\r\n{recibo_servido}",
        recibo_servido.len()
    );
    let (porta, tarefa) = proxy_falso(200, resposta.into_bytes());
    let post = cliente_https::requisitar_em_fluxo_via(
        &destino,
        "PHPSESSID=SINTETICO",
        EnvioEmFluxo {
            tipo: plano.tipo_conteudo(),
            origem: "https://sei.exemplo.invalid",
            referer: &referer,
            tamanho: plano.tamanho_total(),
            corpo: plano.corpo(pdf),
        },
        ambiente(via_proxy(porta)),
        sem_tls,
    )
    .unwrap();
    let recibo = crate::documento_externo_incluir::recibo::interpretar_http(
        cliente_https::resposta_web(post),
        "prova.pdf",
        pdf.len() as u64,
    )
    .unwrap();
    assert_eq!(recibo.nome_upload, "94e336aa");
    let (recebido, extras) = tarefa.join().unwrap();
    assert_eq!(extras, 0, "o upload não pode ser repetido");
    let recebido = recebido.unwrap();
    assert!(
        recebido
            .pedido
            .contains("Content-Type: multipart/form-data; boundary=----ies332\r\n")
    );
    assert!(
        recebido
            .pedido
            .contains(&format!("Content-Length: {}\r\n", esperado.len()))
    );
    assert!(recebido.pedido.starts_with(&format!(
        "POST /sei/controlador.php?acao=documento_upload_anexo&infra_hash={} HTTP/1.1\r\n",
        "c".repeat(64)
    )));
    assert_eq!(recebido.corpo, esperado);
}
