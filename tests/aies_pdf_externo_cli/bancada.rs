use super::{CENARIO, NUP, ORIGEM};
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::instalacao::Instalacao;
use iesei::observacao::{TabelaObservada, UnidadeExibida};
use iesei::pdf_externo::{self, Processo, TipoExterno};
use iesei::processos::{self, ProcessoSimulado};
use rusqlite::Connection;
use std::io::{Cursor, Read};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

pub(super) fn pedido(canal: &mut impl Read) -> (Vec<u8>, String, String) {
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

pub(super) fn aceitar(escuta: &TcpListener, parar: &AtomicBool) -> Option<TcpStream> {
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

pub(super) fn preparar_banco() -> (Connection, Instalacao) {
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

// Intervenção sintética: troca apenas o nome do segundo membro, no
// cabeçalho local e no central; mantém comprimento HTTP e bytes do PDF.
pub(super) fn trocar_membro_zip(resposta: &mut [u8]) {
    let inicio = resposta
        .windows(4)
        .position(|janela| janela == b"\r\n\r\n")
        .unwrap()
        + 4;
    let corpo = &mut resposta[inicio..];
    let original = b"050000002.pdf";
    let alheio = b"999999999.pdf";
    let posicoes: Vec<_> = corpo
        .windows(original.len())
        .enumerate()
        .filter_map(|(i, janela)| (janela == original).then_some(i))
        .collect();
    assert_eq!(posicoes.len(), 2);
    for i in posicoes {
        corpo[i..i + original.len()].copy_from_slice(alheio);
    }
    let mut zip = zip::ZipArchive::new(Cursor::new(corpo)).unwrap();
    assert_eq!(zip.len(), 2);
    assert!(zip.by_name("050000002.pdf").is_err());
    let mut bytes = Vec::new();
    zip.by_name("999999999.pdf")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    assert_eq!(bytes, b"%PDF-1.4\n2 0 obj\n%%EOF\n");
}
