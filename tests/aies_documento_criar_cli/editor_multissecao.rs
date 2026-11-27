//! E2E sintético do editor clássico nativo: binário, HTTPS local e SQLite.
//! A forma segue os GETs originais estudados; a transição não é replay.
use super::*;
use iesei::documento_versoes::{self, DocumentoSimulado, Revisao};
use iesei::editor_documento::{self, Oferta, Secao};
use std::fs;
use std::num::NonZeroU32;

const URL: &str = "https://sei-editor.test/sei";
const DOC: &str = "0000101";
const CENA: &str = "editor-multissecao";
const ANTES: &str = "<p>Antes</p>";
const DEPOIS: &str = "<p>Depois\nopaco</p>";

fn secoes(corpo: &str) -> Vec<Secao> {
    [
        ("txaEditor_4", false, "<h1>Fixo</h1>"),
        ("txaEditor_5", false, ""),
        ("txaEditor_12", true, corpo),
        ("txaEditor_13", false, ""),
        ("txaEditor_14", false, "<hr>"),
    ]
    .into_iter()
    .map(|(nome, principal, html)| Secao {
        nome: nome.into(),
        principal,
        html: html.into(),
    })
    .collect()
}

fn preparar() -> (Connection, Instalacao) {
    let mut banco = Connection::open_in_memory().unwrap();
    banco_cassettes::iniciar(&banco).unwrap();
    let instalacao = Instalacao::interpretar(URL).unwrap();
    banco_cassettes::guardar(
        &mut banco,
        CENA,
        &ControleResumido {
            instalacao: instalacao.clone(),
            unidade: UnidadeExibida::nova("TESTE").unwrap(),
            recebidos: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
            gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
        },
    )
    .unwrap();
    let v1 = NonZeroU32::new(1).unwrap();
    documento_versoes::guardar(
        &mut banco,
        CENA,
        &DocumentoSimulado {
            instalacao: instalacao.clone(),
            id_documento: "21".into(),
            id_procedimento: "10".into(),
            numero_processo: NUP.into(),
            numero: DOC.into(),
            tipo: "Despacho".into(),
            revisoes: vec![Revisao {
                versao: v1,
                usuario: "Teste".into(),
                unidade: "TESTE".into(),
                modificacao: "26/11/2026 10:00:00".into(),
                html_corpo: ANTES.into(),
            }],
        },
    )
    .unwrap();
    editor_documento::registrar_oferta(
        &banco,
        &instalacao,
        CENA,
        "21",
        Oferta {
            editavel: true,
            assinaturas: 0,
        },
    )
    .unwrap();
    editor_documento::registrar_secoes(&banco, &instalacao, CENA, "21", v1, &secoes(ANTES))
        .unwrap();
    (banco, instalacao)
}

fn encaminhar(
    tcp: &mut TcpStream,
    tls: &TlsAcceptor,
    destino: std::net::SocketAddr,
    registro: &Mutex<Vec<String>>,
) {
    assert!(cabecalho(tcp).starts_with(b"CONNECT sei-editor.test:443 HTTP/1.1\r\n"));
    tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
        .unwrap();
    let mut canal = tls.accept(tcp).unwrap();
    let (bytes, primeira) = pedido(&mut canal);
    let mut campos = primeira.split_whitespace();
    let metodo = campos.next().unwrap();
    let acao = acao_da_rota(campos.next().unwrap());
    let mut backend = TcpStream::connect(destino).unwrap();
    backend.write_all(&bytes).unwrap();
    backend.shutdown(Shutdown::Write).unwrap();
    let mut resposta = Vec::new();
    backend.read_to_end(&mut resposta).unwrap();
    let status = std::str::from_utf8(resposta.split(|b| *b == b'\r').next().unwrap())
        .unwrap()
        .split_whitespace()
        .nth(1)
        .unwrap();
    // Só três ações permitidas na trilha; nenhum valor de sessão ou URL.
    let acao = match acao {
        "documento_verificar_assinatura" | "editor_montar" | "editor_salvar" => acao,
        _ => "outra",
    };
    registro
        .lock()
        .unwrap()
        .push(format!("{metodo} {acao}: {status}"));
    canal.write_all(&resposta).unwrap();
    canal.flush().unwrap();
}

fn conferir_trilha(trilha: &[String]) {
    let relevantes: Vec<&str> = trilha
        .iter()
        .map(String::as_str)
        .filter(|linha| !linha.contains(" outra:"))
        .collect();
    assert_eq!(
        relevantes,
        [
            "GET documento_verificar_assinatura: 200",
            "GET editor_montar: 200",
            "POST editor_salvar: 200",
            "GET documento_verificar_assinatura: 200",
            "GET editor_montar: 200",
        ]
    );
}

fn conferir_sql(banco: &Connection, instalacao: &Instalacao) {
    let revisoes = documento_versoes::ler(banco, instalacao, CENA, "21")
        .unwrap()
        .unwrap();
    assert_eq!(revisoes.revisoes.len(), 2);
    for (versao, esperado) in [(1, ANTES), (2, DEPOIS)] {
        let mut consulta = banco
            .prepare(
                "SELECT nome, principal, html FROM documento_revisao_secao
                 WHERE origem=?1 AND cenario=?2 AND id_documento='21' AND versao=?3
                 ORDER BY posicao",
            )
            .unwrap();
        let persistidas: Vec<Secao> = consulta
            .query_map(rusqlite::params![instalacao.url_base(), CENA, versao], |r| {
                Ok(Secao {
                    nome: r.get(0)?,
                    principal: r.get(1)?,
                    html: r.get(2)?,
                })
            })
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(persistidas, secoes(esperado));
    }
}

#[test]
fn cli_nativa_salva_multissecao_e_preserva_revisao_anterior() {
    let pasta = Pasta::nova();
    let (cert, identity) = identidade(&pasta.0, "sei-editor.test");
    let origem = pasta.privado("origem", URL);
    let chave = ["infra", "hash"].join("_");
    let controle = pasta.privado(
        "url",
        &format!(
            "{URL}/controlador.php?acao=procedimento_controlar&{chave}={}",
            "a".repeat(64)
        ),
    );
    let cookie = pasta.privado("cookie", "PHPSESSID=FICTICIO");
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    escuta.set_nonblocking(true).unwrap();
    let destino = escuta.local_addr().unwrap();
    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    proxy.set_nonblocking(true).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let parar = Arc::new(AtomicBool::new(false));
    let parar_sim = Arc::clone(&parar);
    let simulador = thread::spawn(move || {
        let (banco, instalacao) = preparar();
        while let Some(mut cliente) = aceitar(&escuta, &parar_sim) {
            atender_http(&mut cliente, Some((&banco, &instalacao, CENA))).unwrap();
        }
        (banco, instalacao)
    });
    let registro = Arc::new(Mutex::new(Vec::new()));
    let trilha = Arc::clone(&registro);
    let parar_proxy = Arc::clone(&parar);
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(identity).unwrap();
        while let Some(mut tcp) = aceitar(&proxy, &parar_proxy) {
            encaminhar(&mut tcp, &tls, destino, &trilha);
        }
    });
    let comando = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_ies"))
            .args(args)
            .current_dir(&pasta.0)
            .env_clear()
            .env("PATH", std::env::var_os("PATH").unwrap_or_default())
            .env("HOME", &pasta.0)
            .env("IES_URL_CONTROLE_FILE", &controle)
            .env("IES_COOKIE_FILE", &cookie)
            .env("IES_ORIGEM_FILE", &origem)
            .env("SSL_CERT_FILE", &cert)
            .env("https_proxy", format!("http://127.0.0.1:{porta}"))
            .output()
            .unwrap()
    };
    let resultado = std::panic::catch_unwind(|| exercitar(&comando, &pasta, &registro));
    parar.store(true, Ordering::SeqCst);
    tunel.join().unwrap();
    let (banco, instalacao) = simulador.join().unwrap();
    if let Err(erro) = resultado {
        std::panic::resume_unwind(erro);
    }
    conferir_sql(&banco, &instalacao);
}

fn exercitar(comando: &impl Fn(&[&str]) -> Output, pasta: &Pasta, registro: &Mutex<Vec<String>>) {
    let saida = pasta.0.to_str().unwrap();
    let foto = comando(&["--json", "documento", DOC, "--versao", "1", "--saida", saida]);
    assert!(foto.status.success(), "{}", String::from_utf8_lossy(&foto.stderr));
    let base = fs::read_to_string(pasta.0.join(format!("{DOC}.v1.html"))).unwrap();
    assert_eq!(base.matches(ANTES).count(), 1);
    let editada = base.replace(ANTES, "<p>Depois\r\r\nopaco</p>");
    pasta.privado(&format!("{DOC}.v2.html"), &editada);
    let antes = registro.lock().unwrap().len();
    let previa = comando(&["--json", "documento", "salvar", DOC, "--versao", "1"]);
    assert!(previa.status.success(), "{}", String::from_utf8_lossy(&previa.stderr));
    assert_eq!(registro.lock().unwrap().len(), antes);
    let confirmar = ["--json", "documento", "salvar", DOC, "--versao", "1", "--confirmar"];
    let salvo = comando(&confirmar);
    assert!(salvo.status.success(), "{}", String::from_utf8_lossy(&salvo.stderr));
    let valor = json(&salvo);
    assert_eq!(valor["confirmado"], true);
    assert_eq!(valor["versao_criada"], 2);
    assert_eq!(valor["releitura_confirmada"], true);
    assert_eq!(valor["assinaturas_apos"], 0);
    conferir_trilha(&registro.lock().unwrap()[antes..]);
    let nova = pasta.0.join("releitura");
    fs::create_dir(&nova).unwrap();
    let relida = comando(&[
        "--json", "documento", DOC, "--versao", "2", "--saida", nova.to_str().unwrap(),
    ]);
    assert!(relida.status.success(), "{}", String::from_utf8_lossy(&relida.stderr));
    let html = fs::read_to_string(nova.join(format!("{DOC}.v2.html"))).unwrap();
    assert!(html.contains(DEPOIS));
    assert!(html.contains("<h1>Fixo</h1>"));
    assert_eq!(fs::read_to_string(pasta.0.join(format!("{DOC}.v1.html"))).unwrap(), base);
}
