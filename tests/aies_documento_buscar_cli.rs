//! BDD-61-03: CLI real -> TLS local -> anti-IES -> SQLite efêmero.
//! A observação "sem resultado" é explícita no banco e limitada ao termo/unidade.
#![cfg(unix)]
use anti_ies::atender_http;
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::documento_versoes::{self, DocumentoSimulado, Revisao};
use iesei::instalacao::Instalacao;
use iesei::observacao::{TabelaObservada, UnidadeExibida};
use iesei::pesquisa_documento::{self, LinhaPaginada, PaginaObservada};
use native_tls::{Identity, TlsAcceptor};
use rusqlite::Connection;
use serde_json::Value;
use std::fs;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::num::NonZeroU32;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const ORIGEM: &str = "https://sei.test/sei";
const CENARIO: &str = "bdd-61";
const TERMO_EXATO: &str = "0011519";
const TERMO: &str = "SEM-REGISTRO-XYZ";
const TERMO_PAGINADO: &str = "00050";

#[derive(Clone, Copy)]
enum EstadoDeBusca {
    SemResultado,
    DocumentoExato,
    Paginas,
}

struct Pasta(PathBuf);
impl Pasta {
    fn nova() -> Self {
        let instante = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let pasta = std::env::temp_dir().join(format!(
            "aies-buscar-{}-{}",
            std::process::id(),
            instante.as_nanos()
        ));
        fs::create_dir(&pasta).unwrap();
        Self(pasta)
    }
    fn privado(&self, nome: &str, texto: &str) -> PathBuf {
        let caminho = self.0.join(nome);
        fs::write(&caminho, texto).unwrap();
        fs::set_permissions(&caminho, fs::Permissions::from_mode(0o600)).unwrap();
        caminho
    }
}
impl Drop for Pasta {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn certificado(pasta: &Path) -> (PathBuf, Identity) {
    let cert = pasta.join("cert.pem");
    let key = pasta.join("key.pem");
    let p12 = pasta.join("identidade.p12");
    let criado = Command::new("openssl")
        .args([
            "req",
            "-x509",
            "-newkey",
            "rsa:2048",
            "-nodes",
            "-sha256",
            "-days",
            "1",
            "-subj",
            "/CN=sei.test",
            "-addext",
            "subjectAltName=DNS:sei.test",
            "-keyout",
        ])
        .arg(&key)
        .arg("-out")
        .arg(&cert)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert!(criado.success());
    let convertido = Command::new("openssl")
        .args(["pkcs12", "-export", "-passout", "pass:teste", "-inkey"])
        .arg(&key)
        .arg("-in")
        .arg(&cert)
        .arg("-out")
        .arg(&p12)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert!(convertido.success());
    (
        cert,
        Identity::from_pkcs12(&fs::read(p12).unwrap(), "teste").unwrap(),
    )
}

fn aceitar(listener: &TcpListener) -> TcpStream {
    listener.set_nonblocking(true).unwrap();
    let inicio = Instant::now();
    loop {
        match listener.accept() {
            Ok((tcp, peer)) => {
                assert!(peer.ip().is_loopback());
                tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                tcp.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
                return tcp;
            }
            Err(erro) if erro.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(
                    inicio.elapsed() < Duration::from_secs(3),
                    "requisição ausente"
                );
                thread::sleep(Duration::from_millis(10));
            }
            Err(erro) => panic!("accept: {erro}"),
        }
    }
}

fn cabecalho(canal: &mut impl Read) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut byte = [0u8];
    while !bytes.ends_with(b"\r\n\r\n") {
        assert!(bytes.len() < 16_384);
        canal.read_exact(&mut byte).unwrap();
        bytes.push(byte[0]);
    }
    bytes
}

fn encaminhar(canal: &mut impl Read, destino: &mut impl Write) {
    let cab = cabecalho(canal);
    let texto = std::str::from_utf8(&cab).unwrap();
    let tamanho = texto
        .lines()
        .find_map(|linha| {
            let (nome, valor) = linha.split_once(':')?;
            nome.eq_ignore_ascii_case("content-length")
                .then(|| valor.trim().parse::<usize>().unwrap())
        })
        .unwrap_or(0);
    assert!(tamanho < 64_000);
    destino.write_all(&cab).unwrap();
    let mut corpo = vec![0; tamanho];
    canal.read_exact(&mut corpo).unwrap();
    destino.write_all(&corpo).unwrap();
    destino.flush().unwrap();
}

fn preparar_estado(banco: &mut Connection, instalacao: &Instalacao, estado: EstadoDeBusca) {
    banco_cassettes::guardar(
        banco,
        CENARIO,
        &ControleResumido {
            instalacao: instalacao.clone(),
            unidade: UnidadeExibida::nova("TESTE").unwrap(),
            recebidos: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
            gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
        },
    )
    .unwrap();
    if matches!(estado, EstadoDeBusca::DocumentoExato) {
        // A identidade e as revisões já pertencem ao mesmo SQLite do anti-IES.
        documento_versoes::guardar(
            banco,
            CENARIO,
            &DocumentoSimulado {
                instalacao: instalacao.clone(),
                id_documento: "42".into(),
                id_procedimento: "7".into(),
                numero_processo: "00001.000001/2026-01".into(),
                numero: TERMO_EXATO.into(),
                tipo: "Ofício".into(),
                revisoes: vec![Revisao {
                    versao: NonZeroU32::new(1).unwrap(),
                    usuario: "Usuário sintético".into(),
                    unidade: "TESTE".into(),
                    modificacao: "01/11/2026".into(),
                    html_corpo: "<p>Conteúdo de teste</p>".into(),
                }],
            },
        )
        .unwrap();
    } else if matches!(estado, EstadoDeBusca::SemResultado) {
        // Zero explicitamente observado; ausência de registro seria desconhecido.
        iesei::pesquisa_documento::registrar_sem_resultado(
            banco,
            instalacao,
            CENARIO,
            &UnidadeExibida::nova("TESTE").unwrap(),
            TERMO,
        )
        .unwrap();
    } else {
        preparar_paginas(banco, instalacao);
    }
}

fn preparar_paginas(banco: &mut Connection, instalacao: &Instalacao) {
    let unidade = UnidadeExibida::nova("TESTE").unwrap();
    let mut linhas = Vec::new();
    for posicao in 1..=11 {
        let id_documento = (3000 + posicao).to_string();
        documento_versoes::guardar(
            banco,
            CENARIO,
            &DocumentoSimulado {
                instalacao: instalacao.clone(),
                id_documento: id_documento.clone(),
                id_procedimento: (2000 + posicao).to_string(),
                numero_processo: format!("00002.{posicao:06}/2026-00"),
                numero: format!("{:07}", 5000 + posicao),
                tipo: "Despacho".into(),
                revisoes: vec![Revisao {
                    versao: NonZeroU32::new(1).unwrap(),
                    usuario: "Pessoa de teste".into(),
                    unidade: "TESTE".into(),
                    modificacao: "12/10/2026".into(),
                    html_corpo: "<p>Registro sintético</p>".into(),
                }],
            },
        )
        .unwrap();
        linhas.push(LinhaPaginada {
            id_documento,
            rotulo: format!("Despacho {posicao}"),
            unidade: Some("TESTE".into()),
            inclusao: Some("12/10/2026".into()),
            id_anexo: None,
        });
    }
    // Goiás, pesquisa-paginas-go#3/#4: total 27.338 e faixa 11–20,
    // com uma linha na segunda página (replay do acervo em 04/11).
    // Números de documentos e conteúdo são sintéticos: isto não é
    // confronto independente dos corpos originais da cassette.
    for pagina in [
        PaginaObservada {
            pagina: 1,
            inicio: 1,
            fim: 10,
            total_declarado: 27_338,
            linhas: linhas[..10].to_vec(),
        },
        PaginaObservada {
            pagina: 2,
            inicio: 11,
            fim: 20,
            total_declarado: 27_338,
            linhas: linhas[10..].to_vec(),
        },
    ] {
        pesquisa_documento::registrar_pagina(
            banco,
            instalacao,
            CENARIO,
            &unidade,
            TERMO_PAGINADO,
            &pagina,
        )
        .unwrap();
    }
}

fn executar(pasta: &Pasta, args: &[&str]) -> Output {
    executar_com_estado(pasta, args, EstadoDeBusca::SemResultado)
}

fn executar_com_estado(pasta: &Pasta, args: &[&str], estado: EstadoDeBusca) -> Output {
    let requisicoes = if matches!(estado, EstadoDeBusca::Paginas) && args.contains(&"--pagina") {
        4
    } else {
        3
    };
    let (cert, identity) = certificado(&pasta.0);
    let origem = pasta.privado("origem", ORIGEM);
    let url = pasta.privado(
        "url",
        &format!(
            "{ORIGEM}/controlador.php?acao=procedimento_controlar&infra_hash={}",
            "a".repeat(64)
        ),
    );
    let cookie = pasta.privado("cookie", "PHPSESSID=FICTICIO");
    let servidor = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = servidor.local_addr().unwrap();
    let servidor_thread = thread::spawn(move || {
        let mut banco = Connection::open_in_memory().unwrap();
        banco_cassettes::iniciar(&banco).unwrap();
        let instalacao = Instalacao::interpretar(ORIGEM).unwrap();
        preparar_estado(&mut banco, &instalacao, estado);
        for _ in 0..requisicoes {
            let mut tcp = aceitar(&servidor);
            atender_http(&mut tcp, Some((&banco, &instalacao, CENARIO))).unwrap();
        }
    });
    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(identity).unwrap();
        for _ in 0..requisicoes {
            let mut tcp = aceitar(&proxy);
            assert!(cabecalho(&mut tcp).starts_with(b"CONNECT sei.test:443 HTTP/1.1\r\n"));
            tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .unwrap();
            let mut canal = tls.accept(tcp).unwrap();
            let mut destino = TcpStream::connect(endereco).unwrap();
            encaminhar(&mut canal, &mut destino);
            destino.shutdown(Shutdown::Write).unwrap();
            let mut resposta = Vec::new();
            destino.read_to_end(&mut resposta).unwrap();
            canal.write_all(&resposta).unwrap();
            canal.flush().unwrap();
        }
    });
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .current_dir(&pasta.0)
        .args(args)
        .env("IES_URL_CONTROLE_FILE", url)
        .env("IES_COOKIE_FILE", cookie)
        .env("IES_ORIGEM_FILE", origem)
        .env("SSL_CERT_FILE", cert)
        .env("https_proxy", format!("http://127.0.0.1:{porta}"))
        .env("HOME", &pasta.0)
        .env_remove("HTTPS_PROXY")
        .env_remove("all_proxy")
        .env_remove("ALL_PROXY")
        .env_remove("NO_PROXY")
        .env_remove("no_proxy")
        .env_remove("IES_CONFIG_FILE")
        .env_remove("IES_CREDENCIAIS_FILE")
        .output()
        .unwrap();
    tunel.join().unwrap();
    servidor_thread.join().unwrap();
    saida
}

#[test]
fn bdd_61_03_cli_sem_resultado_observado_na_unidade() {
    let pasta = Pasta::nova();
    let saida = executar(&pasta, &["--json", "documento", "buscar", TERMO]);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let json: Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["resultado"], "sem_resultado");
    assert_eq!(json["termo"], TERMO);
    assert_eq!(json["unidade"], "TESTE");
    assert_eq!(json["origem"], "web");
    assert_eq!(json["requisicoes"], 3);
    assert_eq!(json["total_declarado"], 0);
    assert!(json["faixa_observada"].is_null());
    assert_eq!(json["documentos"].as_array().unwrap().len(), 0);
}

#[test]
fn bdd_61_03_cli_humana_nao_confunde_sem_resultado_com_inexistencia() {
    let pasta = Pasta::nova();
    let saida = executar(&pasta, &["documento", "buscar", TERMO]);
    assert_eq!(saida.status.code(), Some(0));
    let texto = String::from_utf8(saida.stdout).unwrap();
    assert!(texto.contains("nenhum resultado para esta conta e unidade"));
    assert!(texto.contains("não prova inexistência"));
    assert!(texto.contains("3 requisições"));
}

#[test]
fn bdd_61_03_cli_recusa_termo_invalido_antes_de_ler_sessao() {
    // O BDD prescreve saída 2 tanto para vazio quanto para controle.
    // Sem servidor/proxy de teste: qualquer tentativa de rede/configuração
    // no caminho de validação deixa de produzir a recusa contratada.
    let pasta = Pasta::nova();
    for (termo, causa) in [
        ("   ", "vazio"),
        ("123\n", "controle"),
        ("123\t", "controle"),
    ] {
        let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
            .current_dir(&pasta.0)
            .args(["--json", "documento", "buscar", termo])
            .env("HOME", &pasta.0)
            .env("https_proxy", "http://127.0.0.1:1")
            .env_remove("IES_CONFIG_FILE")
            .env_remove("IES_URL_CONTROLE_FILE")
            .env_remove("IES_COOKIE_FILE")
            .env_remove("IES_ORIGEM_FILE")
            .output()
            .unwrap();
        assert_eq!(
            saida.status.code(),
            Some(2),
            "termo={termo:?}: {}",
            String::from_utf8_lossy(&saida.stderr)
        );
        let erro: Value = serde_json::from_slice(&saida.stdout).unwrap();
        assert_eq!(erro["codigo"], "entrada_invalida");
        assert_eq!(erro["subcodigo"], causa);
        assert_eq!(erro["retentavel"], false);
    }
}

#[test]
fn bdd_61_01_cli_documento_unico_por_numero_exato_sem_seguir_302() {
    let pasta = Pasta::nova();
    let saida = executar_com_estado(
        &pasta,
        &["--json", "documento", "buscar", TERMO_EXATO],
        EstadoDeBusca::DocumentoExato,
    );
    assert_eq!(
        saida.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let json: Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(json["resultado"], "documento_unico");
    assert_eq!(json["termo"], TERMO_EXATO);
    assert_eq!(json["unidade"], "TESTE");
    assert_eq!(json["origem"], "web");
    assert_eq!(json["requisicoes"], 3);
    assert_eq!(json["documento"]["id"], "42");
    assert_eq!(json["processo"]["id"], "7");
    // O redirecionamento só fornece IDs internos; não inventar NUP ou SEI.
    assert!(json["documento"].get("numero_sei").is_none());
    assert!(json["processo"].get("numero").is_none());
}

#[path = "aies_documento_buscar_cli/paginacao.rs"]
mod paginacao;
