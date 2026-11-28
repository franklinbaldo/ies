//! CLI real, proxy TLS loopback, anti-IES e SQLite sintéticos: criação de grupo.
//! Não constitui replay de instalação externa.
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
const CENARIO: &str = "grupo-criar-cli";
const NUP: &str = "00001.000001/2026-01";


use iesei::grupos_envio::{self, Escopo, Membro};

#[path = "aies_processo_enviar_cli/infra.rs"]
mod infra;
use infra::{aceitar, cabecalho, identidade, pedido, preparar_banco};

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

/// Arquivos efêmeros e proxy da bancada CLI, sem produção.
struct BancadaCli<'a> {
    pasta: &'a Pasta,
    controle: &'a Path,
    cookie: &'a Path,
    origem: &'a Path,
    cert: &'a Path,
    porta: u16,
}

impl BancadaCli<'_> {
    fn executar(&self, args: &[&str]) -> Output {
        let mut comando = Command::new(env!("CARGO_BIN_EXE_ies"));
        comando.env_remove("IES_ANTI_IES_TESTE");
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


type Registro = (String, String, String);
type MembroPersistido = (String, String, String);

fn estado_sql(banco: &Connection) -> (Vec<Registro>, Vec<MembroPersistido>, i64) {
    let grupos = banco
        .prepare("SELECT id_grupo,nome,descricao FROM sim_grupo_envio ORDER BY id_grupo")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
    let membros = banco
        .prepare("SELECT id_grupo,id_unidade,nome FROM sim_grupo_envio_membro ORDER BY posicao")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
    let proximo = banco
        .query_row("SELECT proximo_id FROM sim_grupo_envio_catalogo", [], |r| r.get(0))
        .unwrap();
    (grupos, membros, proximo)
}

type EstadoSql = (Vec<Registro>, Vec<MembroPersistido>, i64);

fn executar(confirmar: bool, falhar_releitura: bool) -> (Output, Vec<String>, EstadoSql) {
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
    let simulador = thread::spawn(move || {
        let (banco, instalacao) = preparar_banco(None);
        // Temporário até a composição ordenada de 0042–0045.
        banco
            .execute_batch(include_str!("../crates/iesei/migrations/0045_grupos_envio.sql"))
            .unwrap();
        let escopo = Escopo {
            instalacao: &instalacao,
            cenario: CENARIO,
            unidade: "ORIGEM",
        };
        grupos_envio::semear(
            &banco,
            escopo,
            &[
                Membro { id: "001".into(), nome: "X - Unidade".into() },
                Membro { id: "1".into(), nome: "Y - Outra".into() },
            ],
        )
        .unwrap();
        assert_eq!(estado_sql(&banco), (vec![], vec![], 1));
        while let Some(mut tcp) = aceitar(&escuta, &parar_sim) {
            atender_http(&mut tcp, Some((&banco, &instalacao, CENARIO))).unwrap();
        }
        estado_sql(&banco)
    });
    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    proxy.set_nonblocking(true).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let parar_proxy = Arc::clone(&parar);
    let trilha = Arc::new(Mutex::new(Vec::<String>::new()));
    let capturada = Arc::clone(&trilha);
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(identidade).unwrap();
        let mut enviado = false;
        while let Some(mut tcp) = aceitar(&proxy, &parar_proxy) {
            assert!(cabecalho(&mut tcp).starts_with(b"CONNECT sei.test:443 HTTP/1.1\r\n"));
            tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n").unwrap();
            let mut canal = tls.accept(tcp).unwrap();
            let (bytes, metodo, acao) = pedido(&mut canal);
            let resposta = if falhar_releitura && enviado && metodo == "GET" {
                b"HTTP/1.1 503 Service Unavailable\r\nContent-Type: text/plain\r\nContent-Length: 7\r\nConnection: close\r\n\r\nIndispo".to_vec()
            } else {
                let mut backend = TcpStream::connect(destino).unwrap();
                backend.write_all(&bytes).unwrap();
                backend.shutdown(Shutdown::Write).unwrap();
                let mut resposta = Vec::new();
                backend.read_to_end(&mut resposta).unwrap();
                resposta
            };
            if metodo == "POST" && acao == "grupo_unidade_cadastrar" {
                enviado = true;
            }
            capturada.lock().unwrap().push(format!("{metodo} {acao}"));
            canal.write_all(&resposta).unwrap();
            canal.flush().unwrap();
        }
    });
    let cli = BancadaCli {
        pasta: &pasta,
        controle: &controle,
        cookie: &cookie,
        origem: &origem,
        cert: &cert,
        porta,
    };
    let mut args = vec![
        "--json", "grupo-envio", "criar", "--nome", "Grupo teste",
        "--descricao", "Descrição", "--membro", "X", "--membro", "Y",
    ];
    if confirmar {
        args.push("--confirmar");
    }
    let saida = cli.executar(&args);
    parar.store(true, Ordering::SeqCst);
    tunel.join().unwrap();
    let finalizado = simulador.join().unwrap();
    let pedidos = trilha.lock().unwrap().clone();
    (saida, pedidos, finalizado)
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


fn post_mutador(pedidos: &[String]) -> usize {
    pedidos.iter().filter(|p| *p == "POST grupo_unidade_cadastrar").count()
}

fn conferir_persistencia(estado: EstadoSql) {
    assert_eq!(
        estado,
        (
            vec![("1".into(), "Grupo teste".into(), "Descrição".into())],
            vec![
                ("1".into(), "001".into(), "X - Unidade".into()),
                ("1".into(), "1".into(), "Y - Outra".into()),
            ],
            2,
        )
    );
}

#[test]
fn previa_resolve_membros_sem_gravar() {
    let (saida, pedidos, estado) = executar(false, false);
    assert!(saida.status.success(), "{:?} {pedidos:?}", saida);
    let p = json(&saida);
    assert_eq!(p["estado"], "previa");
    assert_eq!(p["membros"][0]["id"], "001");
    assert_eq!(p["membros"][1]["id"], "1");
    assert_eq!(post_mutador(&pedidos), 0);
    assert_eq!(estado, (vec![], vec![], 1));
}

#[test]
fn confirmacao_unica_rele_identidade_e_membros() {
    let (saida, pedidos, estado) = executar(true, false);
    assert!(saida.status.success(), "{:?} {pedidos:?}", saida);
    let p = json(&saida);
    assert_eq!(p["estado"], "criado");
    assert_eq!(p["grupo"]["id"], "1");
    assert_eq!(p["grupo"]["unidades"][0]["id"], "001");
    assert_eq!(p["grupo"]["unidades"][1]["id"], "1");
    assert_eq!(post_mutador(&pedidos), 1);
    let post = pedidos.iter().position(|p| p == "POST grupo_unidade_cadastrar").unwrap();
    let consulta = pedidos.iter().position(|p| p == "GET grupo_unidade_consultar").unwrap();
    assert!(consulta > post, "{pedidos:?}");
    conferir_persistencia(estado);
}

#[test]
fn falha_de_releitura_nao_repete_post() {
    let (saida, pedidos, estado) = executar(true, true);
    assert_eq!(saida.status.code(), Some(1));
    assert!(saida.stdout.is_empty());
    assert!(String::from_utf8_lossy(&saida.stderr).contains("releitura"));
    assert_eq!(post_mutador(&pedidos), 1);
    conferir_persistencia(estado);
}
