//! BDD-270-01/02/03: CLI Rust real -> HTTPS loopback -> anti-IES -> SQLite.
//! Hipótese sintética: a cassette original ainda requer confronto independente.
#![cfg(unix)]
use anti_ies::atender_http;
use iesei::anexacoes;
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::documento_versoes::{self, DocumentoSimulado, Revisao};
use iesei::editor_documento::{self, Oferta};
use iesei::instalacao::Instalacao;
use iesei::observacao::{FaixaObservada, TabelaObservada, UnidadeExibida};
use iesei::processos::{self, ProcessoSimulado};
use native_tls::TlsAcceptor;
use rusqlite::Connection;
#[path = "support/falha.rs"]
mod falha_cli;
use falha_cli::{falha, falha_por};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::num::NonZeroU32;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[path = "aies_documento_criar_cli/fixture_tls.rs"]
mod fixture_tls;
use fixture_tls::{Pasta, identidade};

const ORIGEM: &str = "https://sei.test/sei";
const CENARIO: &str = "bdd-270-anexacao";
const PRINCIPAL: &str = "00001.000001/2026-01";
const ANEXADO: &str = "00001.000002/2026-01";

fn aceitar(escuta: &TcpListener, parar: &AtomicBool) -> Option<TcpStream> {
    loop {
        match escuta.accept() {
            Ok((cliente, peer)) => {
                assert!(peer.ip().is_loopback());
                cliente
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                cliente
                    .set_write_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                return Some(cliente);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if parar.load(Ordering::SeqCst) {
                    return None;
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => panic!("aceitar: {e}"),
        }
    }
}

fn pedido_http(fonte: &mut impl Read) -> (Vec<u8>, String) {
    let mut cabecalho = Vec::new();
    let mut byte = [0u8];
    while !cabecalho.ends_with(b"\r\n\r\n") {
        assert!(cabecalho.len() < 16_384);
        fonte.read_exact(&mut byte).unwrap();
        cabecalho.push(byte[0]);
    }
    let texto = std::str::from_utf8(&cabecalho).unwrap();
    let primeira = texto.lines().next().unwrap();
    let metodo = primeira.split_whitespace().next().unwrap();
    let acao = primeira
        .split_once('?')
        .and_then(|(_, q)| {
            q.split('&').find_map(|par| {
                par.strip_prefix("acao=")
                    .or_else(|| par.strip_prefix("acao_ajax="))
            })
        })
        .unwrap_or("sem_acao");
    let tamanho = texto
        .lines()
        .find_map(|linha| {
            let (chave, valor) = linha.split_once(':')?;
            chave
                .eq_ignore_ascii_case("content-length")
                .then(|| valor.trim().parse::<usize>().unwrap())
        })
        .unwrap_or(0);
    assert!(tamanho < 64_000);
    let mut corpo = vec![0; tamanho];
    fonte.read_exact(&mut corpo).unwrap();
    let evento = format!("{metodo} {acao}");
    cabecalho.extend(corpo);
    (cabecalho, evento)
}

fn estado() -> (Connection, Instalacao) {
    let mut banco = Connection::open_in_memory().unwrap();
    banco_cassettes::iniciar(&banco).unwrap();
    let origem = Instalacao::interpretar(ORIGEM).unwrap();
    banco_cassettes::guardar(
        &mut banco,
        CENARIO,
        &ControleResumido {
            instalacao: origem.clone(),
            unidade: UnidadeExibida::nova("TESTE").unwrap(),
            recebidos: TabelaObservada::<Linha>::nova(
                Some(2),
                FaixaObservada::nova(1, 2),
                vec![("10", PRINCIPAL), ("22", ANEXADO)]
                    .into_iter()
                    .map(|(id, numero)| Linha {
                        id: id.into(),
                        numero: numero.into(),
                        visualizado: false,
                        sigiloso: false,
                    })
                    .collect(),
            )
            .unwrap(),
            gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
        },
    )
    .unwrap();
    for (id, numero) in [("10", PRINCIPAL), ("22", ANEXADO)] {
        processos::guardar(
            &mut banco,
            &origem,
            CENARIO,
            &ProcessoSimulado {
                id: id.into(),
                numero: numero.into(),
                unidade_origem: "TESTE".into(),
                aberto_em: vec!["TESTE".into()],
                anotacao_origem: None,
                destinos: vec![],
            },
        )
        .unwrap();
    }
    (banco, origem)
}

fn cli(pasta: &Pasta, arquivos: [&std::path::Path; 4], proxy: u16, args: &[&str]) -> Output {
    let [controle, cookie, origem, certificado] = arquivos;
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(args)
        .env("IES_URL_CONTROLE_FILE", controle)
        .env("IES_COOKIE_FILE", cookie)
        .env("IES_ORIGEM_FILE", origem)
        .env("SSL_CERT_FILE", certificado)
        .env("https_proxy", format!("http://127.0.0.1:{proxy}"))
        .env("HOME", &pasta.0)
        .env_remove("HTTPS_PROXY")
        .env_remove("all_proxy")
        .env_remove("ALL_PROXY")
        .env_remove("NO_PROXY")
        .env_remove("no_proxy")
        .env_remove("IES_CONFIG_FILE")
        .env_remove("IES_CREDENCIAIS_FILE")
        .output()
        .unwrap()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Modo {
    Aceite,
    PdfAposAnexacao,
    RespostaIncerta,
    Previa,
    PreviaPendencia,
    PreviaAssinada,
    PreviaSemObservacao,
    JaAnexado,
}

/// Estado inicial sintético para uma das hipóteses de anexação.
fn preparar_estado(modo: Modo, banco: &mut Connection, instalacao: &Instalacao) {
    if matches!(
        modo,
        Modo::PreviaPendencia | Modo::PreviaAssinada | Modo::PreviaSemObservacao
    ) {
        documento_versoes::guardar(
            banco,
            CENARIO,
            &DocumentoSimulado {
                instalacao: instalacao.clone(),
                id_documento: "77".into(),
                id_procedimento: "22".into(),
                numero_processo: ANEXADO.into(),
                numero: "770000".into(),
                tipo: "Despacho".into(),
                revisoes: vec![Revisao {
                    versao: NonZeroU32::new(1).unwrap(),
                    usuario: "TESTE".into(),
                    unidade: "TESTE".into(),
                    modificacao: "10/11/2026".into(),
                    html_corpo: "<p>Documento com observação de assinatura</p>".into(),
                }],
            },
        )
        .unwrap();
        if !matches!(modo, Modo::PreviaSemObservacao) {
            editor_documento::registrar_oferta(
                banco,
                instalacao,
                CENARIO,
                "77",
                Oferta {
                    editavel: true,
                    assinaturas: u32::from(matches!(modo, Modo::PreviaAssinada)),
                },
            )
            .unwrap();
        }
    }
    if matches!(modo, Modo::PdfAposAnexacao) {
        pdf_apos_anexacao::preparar(banco, instalacao);
    }
    if matches!(modo, Modo::JaAnexado) {
        anexacoes::anexar(banco, instalacao, CENARIO, "10", "22").unwrap();
    }
}

fn conferir_aviso_pendencia(
    modo: Modo,
    pasta: &Pasta,
    arquivos: [&std::path::Path; 4],
    porta: u16,
) {
    if !matches!(modo, Modo::PreviaPendencia) {
        return;
    }
    let aviso = cli(
        pasta,
        arquivos,
        porta,
        &["processo", "anexar", ANEXADO, "--a", PRINCIPAL],
    );
    assert_eq!(aviso.status.code(), Some(0));
    assert!(
        String::from_utf8_lossy(&aviso.stdout).contains("há documento sem assinatura"),
        "aviso de prévia ausente: {}",
        String::from_utf8_lossy(&aviso.stdout)
    );
}

fn executar(modo: Modo) {
    let pasta = Pasta::nova();
    let (cert, tls_id) = identidade(&pasta.0, "sei.test");
    let origem = pasta.privado("origem", ORIGEM);
    let controle = pasta.privado("url", &format!(
        "{ORIGEM}/controlador.php?acao=procedimento_controlar&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={}", "a".repeat(64)
    ));
    let cookie = pasta.privado("cookie", "PHPSESSID=FICTICIO");
    let arquivos = [
        controle.as_path(),
        cookie.as_path(),
        origem.as_path(),
        cert.as_path(),
    ];
    let backend = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    backend.set_nonblocking(true).unwrap();
    let endereco = backend.local_addr().unwrap();
    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    proxy.set_nonblocking(true).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let parar = Arc::new(AtomicBool::new(false));
    let sinal = Arc::clone(&parar);
    let servidor = thread::spawn(move || {
        let (mut banco, instalacao) = estado();
        preparar_estado(modo, &mut banco, &instalacao);
        while let Some(mut cliente) = aceitar(&backend, &sinal) {
            atender_http(&mut cliente, Some((&banco, &instalacao, CENARIO))).unwrap();
        }
        banco
            .query_row("SELECT COUNT(*) FROM sim_anexacao", [], |r| {
                r.get::<_, i64>(0)
            })
            .unwrap()
    });
    let eventos = Arc::new(Mutex::new(Vec::<String>::new()));
    let pedidos = Arc::clone(&eventos);
    let sinal = Arc::clone(&parar);
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(tls_id).unwrap();
        while let Some(mut tcp) = aceitar(&proxy, &sinal) {
            let (connect, _) = pedido_http(&mut tcp);
            assert!(connect.starts_with(b"CONNECT"));
            tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .unwrap();
            let mut canal = tls.accept(tcp).unwrap();
            let (req, evento) = pedido_http(&mut canal);
            let mut destino = TcpStream::connect(endereco).unwrap();
            destino.write_all(&req).unwrap();
            destino.shutdown(Shutdown::Write).unwrap();
            let mut resposta = Vec::new();
            destino.read_to_end(&mut resposta).unwrap();
            if matches!(modo, Modo::RespostaIncerta) && evento == "POST procedimento_anexar" {
                // SQLite já recebeu a escrita, mas a resposta não prova aceite.
                resposta = b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec();
            }
            pedidos.lock().unwrap().push(evento);
            canal.write_all(&resposta).unwrap();
            canal.flush().unwrap();
        }
    });
    let pdf_antes = matches!(modo, Modo::PdfAposAnexacao)
        .then(|| pdf_apos_anexacao::baixar(&pasta, arquivos, porta, "antes"));
    let processo_antes =
        matches!(modo, Modo::PdfAposAnexacao).then(|| ler_processo(&pasta, arquivos, porta));
    if matches!(modo, Modo::PdfAposAnexacao) {
        pdf_apos_anexacao::recusar_nup_alheio(&pasta, arquivos, porta, "antes");
    }
    let comando: &[&str] = match modo {
        Modo::Aceite | Modo::PdfAposAnexacao | Modo::RespostaIncerta => &[
            "--json",
            "processo",
            "anexar",
            ANEXADO,
            "--a",
            PRINCIPAL,
            "--confirmar",
        ],
        Modo::Previa
        | Modo::PreviaPendencia
        | Modo::PreviaAssinada
        | Modo::PreviaSemObservacao
        | Modo::JaAnexado => &["--json", "processo", "anexar", ANEXADO, "--a", PRINCIPAL],
    };
    let saida = cli(&pasta, arquivos, porta, comando);
    conferir_aviso_pendencia(modo, &pasta, arquivos, porta);
    let posterior = ler_processo(&pasta, arquivos, porta);
    if let Some(bytes_antes) = pdf_antes {
        let bytes_depois = pdf_apos_anexacao::baixar(&pasta, arquivos, porta, "depois");
        pdf_apos_anexacao::conferir(
            &processo_antes.unwrap(),
            &posterior,
            &bytes_antes,
            &bytes_depois,
        );
        pdf_apos_anexacao::recusar_nup_alheio(&pasta, arquivos, porta, "depois");
    }
    parar.store(true, Ordering::SeqCst);
    tunel.join().unwrap();
    let total = servidor.join().unwrap();
    let pedidos = eventos.lock().unwrap().clone();
    conferir_resultados(modo, &saida, &posterior, total, &pedidos);
}

fn ler_processo(pasta: &Pasta, arquivos: [&std::path::Path; 4], porta: u16) -> Output {
    cli(
        pasta,
        arquivos,
        porta,
        &[
            "--json",
            "processo",
            PRINCIPAL,
            "--sem-andamentos",
            "--sem-documentos",
            "--sem-propriedades",
            "--sem-marcadores",
        ],
    )
}

fn conferir_resultados(
    modo: Modo,
    saida: &Output,
    posterior: &Output,
    total: i64,
    pedidos: &[String],
) {
    let erro_esperado = match modo {
        Modo::RespostaIncerta => Some("NaoConfirmado"),
        Modo::JaAnexado => Some("Recusada"),
        Modo::PreviaSemObservacao => Some("ConsultaResposta"),
        Modo::Aceite
        | Modo::PdfAposAnexacao
        | Modo::Previa
        | Modo::PreviaPendencia
        | Modo::PreviaAssinada => None,
    };
    if modo == Modo::JaAnexado {
        falha_por(saida, "Recusada", "ProcessoJaAnexado");
    } else if let Some(erro) = erro_esperado {
        falha(saida, erro);
    } else {
        assert_eq!(
            saida.status.code(),
            Some(0),
            "stdout={} stderr={} pedidos={pedidos:?}",
            String::from_utf8_lossy(&saida.stdout),
            String::from_utf8_lossy(&saida.stderr)
        );
        let saida_json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
        match modo {
            Modo::Aceite | Modo::PdfAposAnexacao => {
                assert_eq!(saida_json["confirmado"], true);
                assert_eq!(saida_json["alterado"], true);
            }
            Modo::Previa | Modo::PreviaAssinada => {
                assert_eq!(saida_json["principal"], PRINCIPAL);
                assert_eq!(saida_json["anexado"], ANEXADO);
                assert_eq!(saida_json["documentos_assinados"], true);
                assert_eq!(saida_json["confirmado"], false);
                assert_eq!(saida_json["alterado"], false);
            }
            Modo::PreviaPendencia => {
                assert_eq!(saida_json["principal"], PRINCIPAL);
                assert_eq!(saida_json["anexado"], ANEXADO);
                assert_eq!(saida_json["documentos_assinados"], false);
                assert_eq!(saida_json["confirmado"], false);
                assert_eq!(saida_json["alterado"], false);
            }
            Modo::RespostaIncerta | Modo::JaAnexado | Modo::PreviaSemObservacao => {
                unreachable!("falhas conferidas acima")
            }
        }
    }
    assert_eq!(
        posterior.status.code(),
        Some(0),
        "leitura={}",
        String::from_utf8_lossy(&posterior.stdout)
    );
    let leitura: serde_json::Value = serde_json::from_slice(&posterior.stdout).unwrap();
    assert_eq!(leitura["numero"], PRINCIPAL);
    let tem_anexado = matches!(
        modo,
        Modo::Aceite | Modo::PdfAposAnexacao | Modo::RespostaIncerta | Modo::JaAnexado
    );
    if tem_anexado {
        assert_eq!(leitura["anexados"][0]["numero"], ANEXADO);
        assert_eq!(leitura["anexados"][0]["id_procedimento"], "22");
    } else {
        assert!(leitura["anexados"].as_array().is_none_or(Vec::is_empty));
    }
    assert_eq!(total, i64::from(tem_anexado));
    if modo == Modo::PdfAposAnexacao {
        assert_eq!(
            pedidos
                .iter()
                .filter(|e| e.as_str() == "GET documento_download_anexo")
                .count(),
            2,
            "somente as duas leituras autorizadas baixam o PDF: {pedidos:?}"
        );
    }
    assert_eq!(
        pedidos
            .iter()
            .filter(|e| *e == "POST procedimento_anexar")
            .count(),
        usize::from(matches!(
            modo,
            Modo::Aceite | Modo::PdfAposAnexacao | Modo::RespostaIncerta
        )),
        "prévia e duplicidade não enviam POST mutador; {pedidos:?}"
    );
}

#[test]
fn bdd_270_02_confirmacao_e_releitura_independente() {
    executar(Modo::Aceite);
}

#[test]
fn bdd_270_02_nao_confirmado_sem_retry_de_anexacao() {
    executar(Modo::RespostaIncerta);
}

#[test]
fn bdd_270_01_previa_nao_anexa_e_preserva_o_principal() {
    executar(Modo::Previa);
}

#[test]
fn bdd_270_01_previa_avisa_documento_sem_assinatura_e_nao_anexa() {
    executar(Modo::PreviaPendencia);
}

#[test]
fn consulta_com_assinatura_positiva_observada_preserva_previa() {
    executar(Modo::PreviaAssinada);
}

#[test]
fn consulta_sem_observacao_de_assinatura_nao_inventa_assinada() {
    executar(Modo::PreviaSemObservacao);
}

#[test]
fn bdd_270_03_duplicidade_existente_recusada_sem_nova_escrita() {
    executar(Modo::JaAnexado);
}

#[path = "aies_anexacao_cli/pdf_apos_anexacao.rs"]
mod pdf_apos_anexacao;
