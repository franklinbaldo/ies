//! #332 BDD-332-03/04: binário real → HTTPS local → anti-IES → SQLite → releitura.
//! Hipótese sintética; nunca comunica com uma instalação SEI.
#![cfg(unix)]

use anti_ies::atender_http;
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::instalacao::Instalacao;
use iesei::observacao::{TabelaObservada, UnidadeExibida};
use iesei::pdf_externo::{self, Processo, TipoExterno};
use iesei::processos::{self, ProcessoSimulado};
use native_tls::TlsAcceptor;
use rusqlite::Connection;
use serde_json::Value;
#[path = "support/falha.rs"]
mod falha_cli;
use falha_cli::falha;
use std::fs;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const ORIGEM: &str = "https://sei.test/sei";
const NUP: &str = "00001.000001/2026-01";
const CENARIO: &str = "bdd-332-binario";

#[path = "aies_documento_criar_cli/fixture_tls.rs"]
mod fixture_tls;
use fixture_tls::{Pasta, identidade};

fn aceitar(escuta: &TcpListener, parar: &AtomicBool) -> Option<TcpStream> {
    loop {
        match escuta.accept() {
            Ok((conexao, peer)) => {
                assert!(peer.ip().is_loopback(), "ensaio somente em loopback");
                conexao
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                conexao
                    .set_write_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                return Some(conexao);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if parar.load(Ordering::SeqCst) {
                    return None;
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => panic!("accept: {e}"),
        }
    }
}

fn pedido(canal: &mut impl Read) -> (Vec<u8>, String, String) {
    let mut cabecalho = Vec::new();
    let mut byte = [0];
    while !cabecalho.ends_with(b"\r\n\r\n") {
        assert!(cabecalho.len() < 16_384);
        canal.read_exact(&mut byte).unwrap();
        cabecalho.push(byte[0]);
    }
    let texto = std::str::from_utf8(&cabecalho).unwrap();
    let inicio = texto.lines().next().unwrap().to_owned();
    let tamanho = texto
        .lines()
        .find_map(|linha| {
            let (nome, valor) = linha.split_once(':')?;
            nome.eq_ignore_ascii_case("content-length")
                .then(|| valor.trim().parse::<usize>().unwrap())
        })
        .unwrap_or(0);
    assert!(tamanho < 3 * 1024 * 1024, "limite da bancada sintética");
    let mut corpo = vec![0; tamanho];
    canal.read_exact(&mut corpo).unwrap();
    cabecalho.extend(corpo);
    let acao = inicio
        .split_whitespace()
        .nth(1)
        .unwrap()
        .split_once('?')
        .and_then(|(_, query)| query.split('&').find_map(|par| par.strip_prefix("acao=")))
        .unwrap_or("desconhecida")
        .to_owned();
    let metodo = inicio.split_whitespace().next().unwrap().to_owned();
    (cabecalho, metodo, acao)
}

// Intervenção sintética na projeção HTTP, não no cadastro SQLite:
// o segundo nó passa a publicar a forma fechada observada no acervo.
// O gerador real do anti-IES continua disponível para o primeiro anexo.
fn fechar_segundo_no(resposta: &mut [u8]) -> bool {
    let prefixo = b"Nos[2].src = '";
    let Some(inicio) = resposta.windows(prefixo.len()).position(|w| w == prefixo) else {
        return false;
    };
    let resto = &mut resposta[inicio..];
    let fim = resto.iter().position(|&b| b == b'\n').unwrap();
    let linha = &mut resto[..fim];
    assert!(linha.starts_with(b"Nos[2].src = 'controlador.php?acao=documento_download_anexo"));
    let fechado = b"Nos[2].src = '';";
    // Preserva o Content-Length, sem tocar nos demais nós ou na barra.
    linha.fill(b' ');
    linha[..fechado.len()].copy_from_slice(fechado);
    true
}

fn preparar_banco() -> (Connection, Instalacao) {
    let mut banco = Connection::open_in_memory().unwrap();
    banco_cassettes::iniciar(&banco).unwrap();
    let origem = Instalacao::interpretar(ORIGEM).unwrap();
    banco_cassettes::guardar(
        &mut banco,
        CENARIO,
        &ControleResumido {
            instalacao: origem.clone(),
            unidade: UnidadeExibida::nova("TESTE").unwrap(),
            recebidos: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
            gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
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
            unidade_origem: "TESTE".into(),
            aberto_em: vec!["TESTE".into()],
            anotacao_origem: None,
            destinos: vec![],
        },
    )
    .unwrap();
    pdf_externo::semear(
        &banco,
        &origem,
        CENARIO,
        &Processo {
            id: "10".into(),
            nup: NUP.into(),
            unidade: "TESTE".into(),
        },
        &TipoExterno {
            id: "293".into(),
            nome: "Anexo".into(),
            limite_bytes: 10 * 1024 * 1024,
        },
    )
    .unwrap();
    (banco, origem)
}

type ResultadoBancada = (Vec<Output>, Vec<String>, usize, Vec<Option<Vec<u8>>>);

enum FalhaPrevia {
    LimiteMenor(usize),
    SemAnexos(usize),
    DocumentoFechado,
}

#[derive(Clone, Copy)]
enum Leitura {
    Nenhuma,
    Pdf,
    Zip,
    PdfProcesso,
    ZipSelecao(&'static [&'static str]),
}

fn executar(
    cadastro_incerto: Option<usize>,
    somente_previa: bool,
    leitura: Leitura,
    adicionais: &[&[u8]],
) -> ResultadoBancada {
    executar_com_falha_previa(cadastro_incerto, None, somente_previa, leitura, adicionais)
}

#[expect(
    clippy::too_many_lines,
    reason = "Bancada E2E reúne proxy TLS, servidor HTTP e processos reais"
)]
fn executar_com_falha_previa(
    cadastro_incerto: Option<usize>,
    falha_previa: Option<FalhaPrevia>,
    somente_previa: bool,
    leitura: Leitura,
    adicionais: &[&[u8]],
) -> ResultadoBancada {
    let pasta = Pasta::nova();
    let (cert, identidade_tls) = identidade(&pasta.0, "sei.test");
    let origem = pasta.privado("origem", ORIGEM);
    let chave = ["infra", "hash"].join("_");
    let controle = pasta.privado(
        "url",
        &format!(
            "{ORIGEM}/controlador.php?acao=procedimento_controlar&{chave}={}",
            "a".repeat(64)
        ),
    );
    let cookie = pasta.privado("cookie", "PHPSESSID=FICTICIO");
    let pdf = pasta.0.join("peticao.pdf");
    fs::write(&pdf, b"%PDF-1.4\n1 0 obj\n%%EOF\n").unwrap();
    let outros: Vec<_> = adicionais
        .iter()
        .enumerate()
        .map(|(i, bytes)| {
            let nome = ["segundo.pdf", "terceiro.pdf"][i];
            let caminho = if matches!(leitura, Leitura::ZipSelecao(_)) {
                let pasta_anexo = pasta.0.join(nome);
                fs::create_dir(&pasta_anexo).unwrap();
                pasta_anexo.join("peticao.pdf")
            } else {
                pasta.0.join(nome)
            };
            fs::write(&caminho, bytes).unwrap();
            caminho
        })
        .collect();

    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    escuta.set_nonblocking(true).unwrap();
    let destino = escuta.local_addr().unwrap();
    let parar = Arc::new(AtomicBool::new(false));
    let sinal = Arc::clone(&parar);
    let servidor = thread::spawn(move || {
        let (banco, instalacao) = preparar_banco();
        while let Some(mut cliente) = aceitar(&escuta, &sinal) {
            atender_http(&mut cliente, Some((&banco, &instalacao, CENARIO))).unwrap();
        }
        pdf_externo::documentos(&banco, &instalacao, CENARIO, "10")
            .unwrap()
            .len()
    });

    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    proxy.set_nonblocking(true).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let sinal_proxy = Arc::clone(&parar);
    let trilha = Arc::new(Mutex::new(Vec::<String>::new()));
    let registro = Arc::clone(&trilha);
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(identidade_tls).unwrap();
        let mut cadastros = 0;
        let mut previas = 0;
        while let Some(mut tcp) = aceitar(&proxy, &sinal_proxy) {
            let (connect, _, _) = pedido(&mut tcp);
            assert!(connect.starts_with(b"CONNECT sei.test:443 HTTP/1.1\r\n"));
            tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .unwrap();
            let mut canal = tls.accept(tcp).unwrap();
            let (bytes, metodo, acao) = pedido(&mut canal);
            let mut backend = TcpStream::connect(destino).unwrap();
            backend.write_all(&bytes).unwrap();
            backend.shutdown(Shutdown::Write).unwrap();
            let mut resposta = Vec::new();
            backend.read_to_end(&mut resposta).unwrap();
            let status = std::str::from_utf8(
                &resposta[..resposta.iter().position(|b| *b == b'\r').unwrap()],
            )
            .unwrap()
            .split_whitespace()
            .nth(1)
            .unwrap()
            .to_owned();
            if metodo == "POST" && acao == "documento_receber" && status == "200" {
                previas += 1;
                let troca = match falha_previa {
                    Some(FalhaPrevia::LimiteMenor(n)) if n == previas => {
                        Some(("tamanho : 10", "tamanho : 01"))
                    }
                    Some(FalhaPrevia::SemAnexos(n)) if n == previas => {
                        Some(("name='hdnAnexos'", "name='hdnAusent'"))
                    }
                    _ => None,
                };
                if let Some((original, alterado)) = troca {
                    // Mesmo comprimento: preserva o Content-Length da resposta.
                    assert_eq!(original.len(), alterado.len());
                    let texto = String::from_utf8(resposta).unwrap();
                    assert_eq!(texto.matches(original).count(), 1);
                    resposta = texto.replace(original, alterado).into_bytes();
                }
            }
            if matches!(falha_previa, Some(FalhaPrevia::DocumentoFechado))
                && cadastros == 2
                && acao == "procedimento_visualizar"
                && status == "200"
                && fechar_segundo_no(&mut resposta)
            {
                registro.lock().unwrap().push("arvore mista projetada".into());
            }
            registro
                .lock()
                .unwrap()
                .push(format!("{metodo} {acao} {status}"));
            if metodo == "POST" && acao == "documento_receber" && status == "302" {
                cadastros += 1;
                if cadastro_incerto == Some(cadastros) {
                    let texto = String::from_utf8(resposta).unwrap();
                    let (_, resto) = texto.split_once("id_documento=").unwrap();
                    let id: String = resto.chars().take_while(char::is_ascii_digit).collect();
                    assert_ne!(id, "");
                    resposta = texto
                        .replace(&format!("id_documento={id}"), "id_documento=8000000000999")
                        .into_bytes();
                }
            }
            canal.write_all(&resposta).unwrap();
            canal.flush().unwrap();
        }
    });
    let base = [
        "--json",
        "documento",
        "incluir-externo",
        "--processo",
        NUP,
        "--arquivo",
    ];
    let pdf_str = pdf.to_str().unwrap();
    let modo = if somente_previa {
        "--consultar-sei"
    } else {
        "--confirmar"
    };
    let mut incluir = [base.as_slice(), &[pdf_str, "--tipo", "Anexo", modo]].concat();
    if !matches!(leitura, Leitura::ZipSelecao(_)) {
        for caminho in &outros {
            incluir.extend(["--arquivo", caminho.to_str().unwrap()]);
        }
    }
    let consultar = [
        "--json",
        "processo",
        NUP,
        "--sem-andamentos",
        "--sem-propriedades",
        "--sem-marcadores",
    ];
    let destino_arquivo = pasta.0.join("saida");
    fs::create_dir(&destino_arquivo).unwrap();
    let destino_str = destino_arquivo.to_str().unwrap();
    let numeros: Vec<_> = if matches!(leitura, Leitura::Pdf) {
        (1..=1 + outros.len())
            .map(|i| format!("{:09}", 50_000_000 + i))
            .collect()
    } else {
        Vec::new()
    };
    let leituras: Vec<_> = numeros
        .iter()
        .map(|numero| {
            [
                "--json",
                "documento",
                numero.as_str(),
                "--processo",
                NUP,
                "--saida",
                destino_str,
                "--sem-cache",
            ]
        })
        .collect();
    let (formato, nome) = match leitura {
        Leitura::PdfProcesso => ("pdf", "processo.pdf"),
        _ => ("zip", "processo.zip"),
    };
    let exportado = destino_arquivo.join(nome);
    let exportar = [
        "--json",
        "processo",
        "exportar",
        NUP,
        "--formato",
        formato,
        "--saida",
        exportado.to_str().unwrap(),
    ];
    let mut exportar_selecao = exportar.to_vec();
    let mut comandos = vec![&incluir[..]];
    let incluir_outros: Vec<_> = if matches!(leitura, Leitura::ZipSelecao(_)) {
        outros
            .iter()
            .map(|caminho| {
                [
                    base.as_slice(),
                    &[caminho.to_str().unwrap(), "--tipo", "Anexo", modo],
                ]
                .concat()
            })
            .collect()
    } else {
        Vec::new()
    };
    comandos.extend(incluir_outros.iter().map(Vec::as_slice));
    if let Leitura::ZipSelecao(documentos) = leitura {
        for &documento in documentos {
            exportar_selecao.extend(["--documentos", documento]);
        }
    }
    comandos.push(&consultar);
    comandos.extend(leituras.iter().map(<[&str; 8]>::as_slice));
    match leitura {
        Leitura::Zip | Leitura::PdfProcesso => comandos.push(&exportar),
        Leitura::ZipSelecao(_) => comandos.push(&exportar_selecao),
        _ => {}
    }
    let saidas = comandos
        .iter()
        .map(|args| {
            Command::new(env!("CARGO_BIN_EXE_ies"))
                .args(*args)
                .env("IES_URL_CONTROLE_FILE", &controle)
                .env("IES_COOKIE_FILE", &cookie)
                .env("IES_ORIGEM_FILE", &origem)
                .env("SSL_CERT_FILE", &cert)
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
                .unwrap()
        })
        .collect();
    parar.store(true, Ordering::SeqCst);
    tunel.join().unwrap();
    let total = servidor.join().unwrap();
    let eventos = trilha.lock().unwrap().clone();
    let arquivos = match leitura {
        Leitura::Zip | Leitura::PdfProcesso | Leitura::ZipSelecao(_) => {
            vec![fs::read(exportado).ok()]
        }
        _ => numeros
            .iter()
            .map(|numero| fs::read(destino_arquivo.join(format!("{numero}.pdf"))).ok())
            .collect(),
    };
    (saidas, eventos, total, arquivos)
}

fn json(saida: &Output) -> Value {
    serde_json::from_slice(&saida.stdout).unwrap_or_else(|erro| {
        panic!(
            "saída inválida {erro}: status={:?}, stderr={}",
            saida.status.code(),
            String::from_utf8_lossy(&saida.stderr)
        )
    })
}

#[path = "aies_pdf_externo_cli/singular.rs"]
mod singular;

#[path = "aies_pdf_externo_cli/lote.rs"]
mod lote;

#[path = "aies_pdf_externo_cli/exportacao.rs"]
mod exportacao;

// Mesmo estado sintético e intervenção nas três execuções; apenas a
// seleção muda. A positiva precisa passar pelo gerador e baixar o ZIP.
// O replay original UnB/UFRJ da forma fechada é um teste separado.
#[test]
fn exportacao_seletiva_recusa_no_fechado_antes_do_gerador() {
    use std::io::{Cursor, Read as _};

    for (pedidos, valido) in [
        (&["050000001"][..], true),
        (&["050000002"][..], false),
        (&["050000001", "050000002"][..], false),
    ] {
        let (saidas, eventos, documentos, arquivos) = executar_com_falha_previa(
            None,
            Some(FalhaPrevia::DocumentoFechado),
            false,
            Leitura::ZipSelecao(pedidos),
            &[b"%PDF-1.4\n2 0 obj\n%%EOF\n"],
        );
        assert_eq!(documentos, 2);
        for saida in &saidas[..3] {
            assert!(saida.status.success(), "HTTP={eventos:?}");
        }
        let arvore = json(&saidas[2]);
        let docs = arvore["documentos"].as_array().unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0]["numero_sei"], "050000001");
        assert_eq!(docs[1]["numero_sei"], "050000002");
        assert!(eventos.iter().any(|e| e == "arvore mista projetada"));
        if valido {
            assert!(saidas[3].status.success(), "HTTP={eventos:?}");
            let bytes = arquivos.into_iter().next().flatten().unwrap();
            let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
            assert_eq!(zip.len(), 1);
            let mut membro = zip.by_name("050000001.pdf").unwrap();
            let mut conteudo = Vec::new();
            membro.read_to_end(&mut conteudo).unwrap();
            assert_eq!(conteudo, b"%PDF-1.4\n1 0 obj\n%%EOF\n");
        } else {
            falha(&saidas[3], "DocumentoSelecionadoInacessivel");
            assert_eq!(arquivos, vec![None]);
        }
        for prefixo in [
            "GET procedimento_gerar_zip",
            "POST procedimento_gerar_zip",
            "GET exibir_arquivo",
        ] {
            assert_eq!(
                eventos.iter().filter(|e| e.starts_with(prefixo)).count(),
                usize::from(valido),
                "{prefixo}: {eventos:?}"
            );
        }
    }
}
