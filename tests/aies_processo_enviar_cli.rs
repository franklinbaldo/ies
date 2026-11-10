//! BDD-333-01/02: CLI real, HTTPS de loopback e estado SQLite do anti-IES.
//! A hipótese sintética não substitui validação de compatibilidade com SEI.
#![cfg(unix)]

use anti_ies::atender_http;
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::documentos_simulados::{self, Processo};
use iesei::instalacao::Instalacao;
use iesei::observacao::{TabelaObservada, UnidadeExibida};
use iesei::processos::{self, Destino, ProcessoSimulado};
use native_tls::{Identity, TlsAcceptor};
use rusqlite::Connection;
use serde_json::Value;
use std::fs;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const ORIGEM: &str = "https://sei.test/sei";
const CENARIO: &str = "bdd-333-cli";
const NUP: &str = "00001.000001/2026-01";

struct Pasta(PathBuf);
impl Pasta {
    fn nova() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let pasta = std::env::temp_dir().join(format!("aies-envio-{}-{nanos}", std::process::id()));
        fs::create_dir(&pasta).unwrap();
        Self(pasta)
    }

    fn privado(&self, nome: &str, conteudo: &str) -> PathBuf {
        let caminho = self.0.join(nome);
        fs::write(&caminho, conteudo).unwrap();
        fs::set_permissions(&caminho, fs::Permissions::from_mode(0o600)).unwrap();
        caminho
    }
}
impl Drop for Pasta {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn identidade(pasta: &Path) -> (PathBuf, Identity) {
    let cert = pasta.join("cert.pem");
    let key = pasta.join("key.pem");
    let p12 = pasta.join("identidade.p12");
    assert!(
        Command::new("openssl")
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
                "-keyout"
            ])
            .arg(&key)
            .arg("-out")
            .arg(&cert)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("openssl")
            .args(["pkcs12", "-export", "-passout", "pass:teste", "-inkey"])
            .arg(&key)
            .arg("-in")
            .arg(&cert)
            .arg("-out")
            .arg(&p12)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap()
            .success()
    );
    (
        cert,
        Identity::from_pkcs12(&fs::read(p12).unwrap(), "teste").unwrap(),
    )
}

fn aceitar(listener: &TcpListener, parar: &AtomicBool) -> Option<TcpStream> {
    loop {
        match listener.accept() {
            Ok((tcp, endereco)) => {
                assert!(endereco.ip().is_loopback());
                tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                tcp.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
                return Some(tcp);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if parar.load(Ordering::SeqCst) {
                    return None;
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => panic!("aceitar TLS de teste: {e}"),
        }
    }
}

fn cabecalho(fonte: &mut impl Read) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut byte = [0];
    while !bytes.ends_with(b"\r\n\r\n") {
        assert!(bytes.len() < 16_384);
        fonte.read_exact(&mut byte).unwrap();
        bytes.push(byte[0]);
    }
    bytes
}

fn pedido(fonte: &mut impl Read) -> (Vec<u8>, String, String) {
    let mut bytes = cabecalho(fonte);
    let cabecalho = std::str::from_utf8(&bytes).unwrap();
    let primeira = cabecalho.lines().next().unwrap();
    let mut partes = primeira.split_whitespace();
    let metodo = partes.next().unwrap().to_owned();
    let alvo = partes.next().unwrap();
    let acao = alvo
        .split_once('?')
        .and_then(|(_, q)| {
            q.split('&').find_map(|item| {
                item.strip_prefix("acao=")
                    .or_else(|| item.strip_prefix("acao_ajax="))
            })
        })
        .unwrap_or("sem_acao")
        .to_owned();
    let tamanho = cabecalho
        .lines()
        .find_map(|linha| {
            let (nome, valor) = linha.split_once(':')?;
            nome.eq_ignore_ascii_case("content-length")
                .then(|| valor.trim().parse::<usize>().unwrap())
        })
        .unwrap_or(0);
    assert!(tamanho < 64_000);
    let mut corpo = vec![0; tamanho];
    fonte.read_exact(&mut corpo).unwrap();
    bytes.extend(corpo);
    (bytes, metodo, acao)
}

/// Simula um SEI que aceita o envio, mas ignora o checkbox de remoção.
/// Só altera a requisição entre a CLI real e o anti-IES isolado.
fn omitir_checkbox_do_envio(pedido: Vec<u8>) -> Vec<u8> {
    let bruto = String::from_utf8(pedido).unwrap();
    let (cabecalho, corpo) = bruto.split_once("\r\n\r\n").unwrap();
    let campos: Vec<_> = corpo.split('&').collect();
    assert!(
        campos
            .iter()
            .any(|campo| campo.starts_with("chkSinRemoverAnotacoes="))
    );
    let corpo = campos
        .into_iter()
        .filter(|campo| !campo.starts_with("chkSinRemoverAnotacoes="))
        .collect::<Vec<_>>()
        .join("&");
    let cabecalho = cabecalho
        .split("\r\n")
        .map(|linha| {
            if linha.to_ascii_lowercase().starts_with("content-length:") {
                format!("Content-Length: {}", corpo.len())
            } else {
                linha.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\r\n");
    format!("{cabecalho}\r\n\r\n{corpo}").into_bytes()
}

/// Executa o único desvio sintético controlado antes de entregar ao anti-IES.
fn pedido_com_ensaio(canal: &mut impl Read, ensaio: EnsaioRemocao) -> (Vec<u8>, String, String) {
    let (bytes, metodo, acao) = pedido(canal);
    if ensaio == EnsaioRemocao::IgnorarNoPost && metodo == "POST" && acao == "procedimento_enviar" {
        return (omitir_checkbox_do_envio(bytes), metodo, acao);
    }
    (bytes, metodo, acao)
}

fn preparar_banco(anotacao: Option<&str>) -> (Connection, Instalacao) {
    let mut banco = Connection::open_in_memory().unwrap();
    banco_cassettes::iniciar(&banco).unwrap();
    let origem = Instalacao::interpretar(ORIGEM).unwrap();
    banco_cassettes::guardar(
        &mut banco,
        CENARIO,
        &ControleResumido {
            instalacao: origem.clone(),
            unidade: UnidadeExibida::nova("ORIGEM").unwrap(),
            recebidos: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
            gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
        },
    )
    .unwrap();
    documentos_simulados::registrar_processo(
        &banco,
        &origem,
        CENARIO,
        &Processo {
            id: "10".into(),
            nup: NUP.into(),
            unidade: "ORIGEM".into(),
        },
    )
    .unwrap();
    processos::guardar(
        &mut banco,
        &origem,
        CENARIO,
        &ProcessoSimulado {
            id: "10".into(),
            numero: NUP.into(),
            unidade_origem: "ORIGEM".into(),
            aberto_em: vec!["ORIGEM".into()],
            anotacao_origem: anotacao.map(str::to_owned),
            destinos: vec![
                Destino {
                    id: "2".into(),
                    sigla: "DESTINO".into(),
                    descricao: "DESTINO - Unidade de teste".into(),
                },
                Destino {
                    id: "3".into(),
                    sigla: "ARQUIVO".into(),
                    descricao: "ARQUIVO - Unidade de teste".into(),
                },
            ],
        },
    )
    .unwrap();
    (banco, origem)
}

/// Arquivos efêmeros e proxy da bancada CLI, sem produção.
struct BancadaCli<'a> {
    pasta: &'a Pasta,
    controle: &'a Path,
    cookie: &'a Path,
    origem: &'a Path,
    cert: &'a Path,
    porta: u16,
    ensaio_local: bool,
}

impl BancadaCli<'_> {
    fn executar(&self, args: &[&str]) -> Output {
        let mut comando = Command::new(env!("CARGO_BIN_EXE_ies"));
        if self.ensaio_local {
            comando.env("IES_ANTI_IES_TESTE", "1");
        } else {
            comando.env_remove("IES_ANTI_IES_TESTE");
        }
        comando
            .args(args)
            .env("IES_URL_CONTROLE_FILE", self.controle)
            .env("IES_COOKIE_FILE", self.cookie)
            .env("IES_ORIGEM_FILE", self.origem)
            .env("SSL_CERT_FILE", self.cert)
            .env("https_proxy", format!("http://127.0.0.1:{}", self.porta))
            .env("HOME", &self.pasta.0)
            .env_remove("HTTPS_PROXY")
            .env_remove("all_proxy")
            .env_remove("ALL_PROXY")
            .env_remove("NO_PROXY")
            .env_remove("no_proxy")
            .env_remove("IES_CONFIG_FILE")
            .env_remove("IES_CREDENCIAIS_FILE")
            .output()
            .expect("binário da CLI")
    }
}

/// A troca é disparada antes da segunda CLI, mantendo o mesmo SQLite.
fn selecionar_unidade_destino(banco: &mut Connection, instalacao: &Instalacao, unidade: &str) {
    let intermediario = processos::ler(banco, instalacao, CENARIO, "10")
        .unwrap()
        .unwrap();
    assert_eq!(intermediario.aberto_em, ["DESTINO"]);
    banco_cassettes::guardar(
        banco,
        CENARIO,
        &ControleResumido {
            instalacao: instalacao.clone(),
            unidade: UnidadeExibida::nova(unidade).unwrap(),
            recebidos: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
            gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
        },
    )
    .unwrap();
}

/// Trilha tem apenas método e ação; nunca URL completa, corpo ou cookie.
fn executar(
    comandos: &[&[&str]],
    falhar_releitura: bool,
    anotacao: Option<&'static str>,
) -> (Vec<Output>, Vec<String>, ProcessoSimulado) {
    executar_com_ensaio_local(
        comandos,
        falhar_releitura,
        anotacao,
        false,
        EnsaioRemocao::Normal,
    )
}

fn executar_com_ensaio_local(
    comandos: &[&[&str]],
    falhar_releitura: bool,
    anotacao: Option<&'static str>,
    ensaio_local: bool,
    ensaio: EnsaioRemocao,
) -> (Vec<Output>, Vec<String>, ProcessoSimulado) {
    executar_com_troca_de_unidade(
        comandos,
        falhar_releitura,
        anotacao,
        ensaio_local,
        ensaio,
        None,
    )
}

/// No ensaio de duas CLI, troca a unidade ativa entre os subprocessos,
/// preservando o mesmo banco SQLite e sem alterar o fluxo de cada comando.
fn executar_com_troca_de_unidade(
    comandos: &[&[&str]],
    falhar_releitura: bool,
    anotacao: Option<&'static str>,
    ensaio_local: bool,
    ensaio: EnsaioRemocao,
    trocar_apos_primeiro: Option<&'static str>,
) -> (Vec<Output>, Vec<String>, ProcessoSimulado) {
    let pasta = Pasta::nova();
    let (cert, identidade) = identidade(&pasta.0);
    let origem = pasta.privado("origem", ORIGEM);
    let nome_hash = ["infra", "hash"].join("_");
    let controle = pasta.privado(
        "url",
        &format!(
            "{ORIGEM}/controlador.php?acao=procedimento_controlar&{nome_hash}={}",
            "a".repeat(64)
        ),
    );
    let cookie = pasta.privado("cookie", "PHPSESSID=FICTICIO");
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    escuta.set_nonblocking(true).unwrap();
    let destino = escuta.local_addr().unwrap();
    let parar = Arc::new(AtomicBool::new(false));
    let parar_sim = Arc::clone(&parar);
    let trocar = Arc::new(AtomicBool::new(false));
    let trocar_sim = Arc::clone(&trocar);
    let recusar_destino = Arc::new(AtomicBool::new(false));
    let recusar_no_simulador = Arc::clone(&recusar_destino);
    let simulador = thread::spawn(move || {
        let (mut banco, instalacao) = preparar_banco(anotacao);
        let antes = processos::ler(&banco, &instalacao, CENARIO, "10")
            .unwrap()
            .unwrap();
        assert_eq!(antes.aberto_em, ["ORIGEM"]);
        while let Some(mut tcp) = aceitar(&escuta, &parar_sim) {
            if trocar_sim.swap(false, Ordering::SeqCst) {
                selecionar_unidade_destino(
                    &mut banco,
                    &instalacao,
                    trocar_apos_primeiro.expect("segunda unidade especificada"),
                );
            }
            if recusar_no_simulador.swap(false, Ordering::SeqCst) {
                // Hipótese sintética: o destino deixou de ser oferecido entre
                // o autocomplete e o POST. O anti-IES recebe a requisição
                // original, recusa e não altera a tramitação no SQLite.
                let mut processo = processos::ler(&banco, &instalacao, CENARIO, "10")
                    .unwrap()
                    .unwrap();
                processo.destinos.retain(|destino| destino.id != "2");
                processos::guardar(&mut banco, &instalacao, CENARIO, &processo).unwrap();
            }
            atender_http(&mut tcp, Some((&banco, &instalacao, CENARIO))).unwrap();
        }
        processos::ler(&banco, &instalacao, CENARIO, "10")
            .unwrap()
            .unwrap()
    });
    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    proxy.set_nonblocking(true).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let parar_proxy = Arc::clone(&parar);
    let trilha = Arc::new(Mutex::new(Vec::<String>::new()));
    let capturada = Arc::clone(&trilha);
    let recusar_no_proxy = Arc::clone(&recusar_destino);
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(identidade).unwrap();
        let mut enviado = false;
        while let Some(mut tcp) = aceitar(&proxy, &parar_proxy) {
            assert!(cabecalho(&mut tcp).starts_with(b"CONNECT sei.test:443 HTTP/1.1\r\n"));
            tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .unwrap();
            let mut canal = tls.accept(tcp).unwrap();
            let (bytes, metodo, acao) = pedido_com_ensaio(&mut canal, ensaio);
            // A hipótese de incerteza só atinge a releitura posterior à escrita.
            let bloquear = falhar_releitura && enviado && metodo == "GET";
            if ensaio == EnsaioRemocao::TelaSemCheckboxEnvioRecusado
                && metodo == "POST"
                && acao == "procedimento_enviar"
            {
                recusar_no_proxy.store(true, Ordering::SeqCst);
            }
            let mut resposta = if bloquear {
                b"HTTP/1.1 503 Service Unavailable\r\nContent-Type: text/plain\r\nContent-Length: 7\r\nConnection: close\r\n\r\nIndispo".to_vec()
            } else {
                let mut backend = TcpStream::connect(destino).unwrap();
                backend.write_all(&bytes).unwrap();
                backend.shutdown(Shutdown::Write).unwrap();
                let mut resposta = Vec::new();
                backend.read_to_end(&mut resposta).unwrap();
                resposta
            };
            resposta = ensaio::ajustar_resposta(resposta, ensaio, &metodo, &acao);
            if metodo == "POST" && acao == "procedimento_enviar" {
                enviado = true;
            }
            capturada.lock().unwrap().push(format!("{metodo} {acao}"));
            canal.write_all(&resposta).unwrap();
            canal.flush().unwrap();
            resposta.clear();
        }
    });
    let cli = BancadaCli {
        pasta: &pasta,
        controle: &controle,
        cookie: &cookie,
        origem: &origem,
        cert: &cert,
        porta,
        ensaio_local,
    };
    let saidas = comandos
        .iter()
        .enumerate()
        .map(|(indice, args)| {
            if trocar_apos_primeiro.is_some() && indice == 1 {
                trocar.store(true, Ordering::SeqCst);
            }
            cli.executar(args)
        })
        .collect();
    parar.store(true, Ordering::SeqCst);
    tunel.join().unwrap();
    let finalizado = simulador.join().unwrap();
    let pedidos = trilha.lock().unwrap().clone();
    (saidas, pedidos, finalizado)
}

fn args(confirmar: bool) -> Vec<&'static str> {
    let mut comando = vec![
        "--json",
        "processo",
        "enviar",
        NUP,
        "--para",
        "DESTINO",
        "--manter-aberto",
    ];
    if confirmar {
        comando.push("--confirmar");
    }
    comando
}

fn json(saida: &Output) -> Value {
    serde_json::from_slice(&saida.stdout).unwrap_or_else(|erro| {
        panic!(
            "JSON inválido {erro}; exit {:?}; stdout {}; stderr {}",
            saida.status.code(),
            String::from_utf8_lossy(&saida.stdout),
            String::from_utf8_lossy(&saida.stderr)
        )
    })
}

#[test]
fn bdd_333_01_02_previa_nao_muta_e_confirmacao_unica_atualiza_sqlite() {
    let previa = args(false);
    let confirmacao = args(true);
    let (saidas, pedidos, depois) =
        executar(&[&previa, &confirmacao], false, Some("Nota da origem"));
    assert_eq!(
        saidas[0].status.code(),
        Some(0),
        "prévia {} pedidos {pedidos:?}",
        String::from_utf8_lossy(&saidas[0].stdout)
    );
    let p = json(&saidas[0]);
    assert_eq!(p["confirmado"], false);
    assert_eq!(p["alterado"], false);
    assert_eq!(p["aberto_em"], serde_json::json!(["ORIGEM"]));
    assert_eq!(
        saidas[1].status.code(),
        Some(0),
        "confirmação stdout {} stderr {} pedidos {pedidos:?}",
        String::from_utf8_lossy(&saidas[1].stdout),
        String::from_utf8_lossy(&saidas[1].stderr)
    );
    let c = json(&saidas[1]);
    assert_eq!(c["confirmado"], true);
    assert_eq!(c["alterado"], true);
    assert_eq!(c["depois"], serde_json::json!(["ORIGEM", "DESTINO"]));
    assert_eq!(depois.aberto_em, ["ORIGEM", "DESTINO"]);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| *p == "POST procedimento_enviar")
            .count(),
        1
    );
    assert!(
        pedidos
            .iter()
            .rposition(|p| p == "GET procedimento_visualizar")
            > pedidos.iter().position(|p| p == "POST procedimento_enviar"),
        "a confirmação depende de uma leitura posterior ao POST: {pedidos:?}"
    );
}

#[test]
fn bdd_333_02_nao_confirmado_depois_do_post_nao_repete_escrita() {
    let confirmacao = args(true);
    let (saidas, pedidos, depois) = executar(&[&confirmacao], true, Some("Nota da origem"));
    assert_eq!(
        saidas[0].status.code(),
        Some(1),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saidas[0].stdout),
        String::from_utf8_lossy(&saidas[0].stderr)
    );
    assert_eq!(json(&saidas[0])["codigo"], "nao_confirmado");
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| *p == "POST procedimento_enviar")
            .count(),
        1
    );
    assert_eq!(
        depois.aberto_em,
        ["ORIGEM", "DESTINO"],
        "a mutação ocorreu apesar de não haver prova independente"
    );
}

#[path = "aies_processo_enviar_cli/ensaio.rs"]
mod ensaio;
use ensaio::EnsaioRemocao;

#[path = "aies_processo_enviar_cli/remocao.rs"]
mod remocao;

#[path = "aies_processo_enviar_cli/duas_unidades.rs"]
mod duas_unidades;

#[path = "aies_processo_enviar_cli/anotacao_preexistente.rs"]
mod anotacao_preexistente;

#[path = "aies_processo_enviar_cli/limpeza_parcial.rs"]
mod limpeza_parcial;
