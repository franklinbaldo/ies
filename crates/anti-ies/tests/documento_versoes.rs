//! BDD-88-01/03: versões são estado do anti-IES, não HTML fixo.
use anti_ies::Estado;
use anti_ies::documento_versoes::responder;
use iesei::banco_cassettes::iniciar;
use iesei::documento_versoes::{DocumentoSimulado, Revisao, guardar, ler};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::num::NonZeroU32;

fn origem() -> Instalacao {
    Instalacao::interpretar("https://sei.exemplo/sei").unwrap()
}

fn documento() -> DocumentoSimulado {
    DocumentoSimulado {
        instalacao: origem(),
        id_documento: "42".into(),
        id_procedimento: "7".into(),
        numero_processo: "00001.000001/2026-01".into(),
        numero: "0011519".into(),
        tipo: "Ofício".into(),
        revisoes: vec![
            Revisao {
                versao: NonZeroU32::new(1).unwrap(),
                usuario: "Ana".into(),
                unidade: "PPI".into(),
                modificacao: "01/11/2026".into(),
                html_corpo: "<p>Texto histórico</p>".into(),
            },
            Revisao {
                versao: NonZeroU32::new(3).unwrap(),
                usuario: "Bia".into(),
                unidade: "PPI".into(),
                modificacao: "03/11/2026".into(),
                html_corpo: "<p>Texto atual</p>".into(),
            },
        ],
    }
}

#[test]
fn bdd_88_01_versao_nao_contigua_tem_conteudo_proprio() {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    guardar(&mut banco, "ensaio", &documento()).unwrap();
    let lista = responder(
        "GET",
        "/sei/controlador.php?acao=documento_versao_listar&id_documento=42",
        &banco,
        &origem(),
        "ensaio",
    );
    assert_eq!(lista.estado, Estado::Ok);
    assert!(lista.corpo.contains("Lista de Versões (2 registros)"));
    let antiga = responder(
        "GET",
        "/sei/controlador.php?acao=documento_visualizar&id_documento=42&versao=1",
        &banco,
        &origem(),
        "ensaio",
    );
    assert_eq!(antiga.estado, Estado::Ok);
    assert!(antiga.corpo.contains("Texto histórico"));
    assert!(!antiga.corpo.contains("Texto atual"));
    assert!(antiga.corpo.contains("versão 1 por"));
    let atual = responder(
        "GET",
        "/sei/controlador.php?acao=documento_visualizar&id_documento=42",
        &banco,
        &origem(),
        "ensaio",
    );
    assert!(atual.corpo.contains("Texto atual"));
    assert!(!atual.corpo.contains("Texto histórico"));
    assert!(atual.corpo.contains("versão 3 por"));
    assert_eq!(
        responder(
            "GET",
            "/sei/controlador.php?acao=documento_visualizar&id_documento=42&versao=2",
            &banco,
            &origem(),
            "ensaio"
        )
        .estado,
        Estado::NaoEncontrado
    );
}

#[test]
fn bdd_88_03_ausencia_nao_e_versao_zero_nem_lista_vazia() {
    let banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let alvo = "/sei/controlador.php?acao=documento_versao_listar&id_documento=42";
    assert_eq!(
        responder("GET", alvo, &banco, &origem(), "ensaio").estado,
        Estado::DadosInsuficientes
    );
    assert_eq!(
        responder("POST", alvo, &banco, &origem(), "ensaio").estado,
        Estado::MetodoNaoPermitido
    );
    assert_eq!(
        responder(
            "GET",
            "/sei/controlador.php?acao=documento_visualizar&id_documento=42&versao=0",
            &banco,
            &origem(),
            "ensaio"
        )
        .estado,
        Estado::RequisicaoInvalida
    );
}

#[test]
fn estado_sqlite_compartilhado_idempotente_e_isolado() {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let original = documento();
    guardar(&mut banco, "ensaio", &original).unwrap();
    guardar(&mut banco, "ensaio", &original).unwrap();
    assert_eq!(
        ler(&banco, &origem(), "ensaio", "42").unwrap(),
        Some(original.clone())
    );
    assert!(ler(&banco, &origem(), "outro", "42").unwrap().is_none());
    let outra = Instalacao::interpretar("https://outra.exemplo/sei").unwrap();
    assert!(ler(&banco, &outra, "ensaio", "42").unwrap().is_none());
    let count: i64 = banco
        .query_row("SELECT count(*) FROM documento_revisao", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 2);
    let mut modificado = original;
    modificado.revisoes.pop();
    guardar(&mut banco, "ensaio", &modificado).unwrap();
    assert_eq!(
        ler(&banco, &origem(), "ensaio", "42").unwrap(),
        Some(modificado)
    );
    let count: i64 = banco
        .query_row("SELECT count(*) FROM documento_revisao", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
    let vazio = Connection::open_in_memory().unwrap();
    iniciar(&vazio).unwrap();
    assert!(ler(&vazio, &origem(), "ensaio", "42").unwrap().is_none());
}

#[test]
fn estado_invalido_nao_grava_meia_revisao() {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let mut invalido = documento();
    invalido.revisoes[1].versao = invalido.revisoes[0].versao;
    assert!(guardar(&mut banco, "ensaio", &invalido).is_err());
    assert!(ler(&banco, &origem(), "ensaio", "42").unwrap().is_none());
}

#[test]
fn duas_rotas_http_reais_compartilham_a_mesma_versao_no_banco() {
    use std::io::{Read, Write};
    use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};

    fn pedir(endereco: SocketAddr, alvo: &str) -> String {
        let mut conexao = TcpStream::connect(endereco).unwrap();
        write!(conexao, "GET {alvo} HTTP/1.1\r\nHost: localhost\r\n\r\n").unwrap();
        conexao.shutdown(Shutdown::Write).unwrap();
        let mut resposta = String::new();
        conexao.read_to_string(&mut resposta).unwrap();
        resposta
    }

    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let mut banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        guardar(&mut banco, "ensaio", &documento()).unwrap();
        let origem = origem();
        for _ in 0..2 {
            let (mut conexao, _) = escuta.accept().unwrap();
            anti_ies::atender_http(&mut conexao, Some((&banco, &origem, "ensaio"))).unwrap();
        }
    });

    let lista = pedir(
        endereco,
        "/sei/controlador.php?acao=documento_versao_listar&id_documento=42",
    );
    assert!(lista.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(lista.contains("Lista de Versões (2 registros)"));
    let antiga = pedir(
        endereco,
        "/sei/controlador.php?acao=documento_visualizar&id_documento=42&versao=1",
    );
    assert!(antiga.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(antiga.contains("Texto histórico"));
    assert!(!antiga.contains("Texto atual"));
    servidor.join().unwrap();
}

#[test]
fn id_de_processo_explicito_nao_pode_selecionar_conteudo_de_outro_processo() {
    use std::io::{Read, Write};
    use std::net::{Shutdown, TcpListener, TcpStream};

    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let mut banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        let primeiro = documento();
        guardar(&mut banco, "ensaio", &primeiro).unwrap();
        let mut segundo = primeiro.clone();
        segundo.id_documento = "43".into();
        segundo.id_procedimento = "8".into();
        segundo.numero_processo = "00001.000002/2026-02".into();
        segundo.numero = "0011520".into();
        segundo.revisoes[1].html_corpo = "<p>Segundo processo</p>".into();
        guardar(&mut banco, "ensaio", &segundo).unwrap();
        for _ in 0..5 {
            let (mut conexao, _) = escuta.accept().unwrap();
            anti_ies::atender_http(&mut conexao, Some((&banco, &origem(), "ensaio"))).unwrap();
        }
    });
    let pedir = |metodo: &str, documento: &str, processo: &str| -> String {
        let mut conexao = TcpStream::connect(endereco).unwrap();
        write!(
            conexao,
            "{metodo} /sei/controlador.php?acao=documento_visualizar&id_documento={documento}&id_procedimento={processo} HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\n\r\n"
        )
        .unwrap();
        conexao.shutdown(Shutdown::Write).unwrap();
        let mut resposta = String::new();
        conexao.read_to_string(&mut resposta).unwrap();
        resposta
    };
    let trocado_a = pedir("GET", "42", "8");
    let trocado_b = pedir("GET", "43", "7");
    assert!(trocado_a.starts_with("HTTP/1.1 404"), "{trocado_a}");
    assert!(trocado_b.starts_with("HTTP/1.1 404"), "{trocado_b}");
    assert!(!trocado_a.contains("Texto atual"));
    assert!(!trocado_b.contains("Segundo processo"));
    let certo_a = pedir("GET", "42", "7");
    let certo_b = pedir("GET", "43", "8");
    assert!(certo_a.starts_with("HTTP/1.1 200"), "{certo_a}");
    assert!(certo_a.contains("Texto atual"));
    assert!(certo_b.starts_with("HTTP/1.1 200"), "{certo_b}");
    assert!(certo_b.contains("Segundo processo"));
    assert!(pedir("POST", "42", "7").starts_with("HTTP/1.1 405"));
    servidor.join().unwrap();
}

/// #60 / BDD-60-05 (teste técnico): a rota não escolhe o primeiro valor
/// se a identidade ou a versão vier repetida na própria requisição HTTP.
/// A leitura válida posterior continua a usar o mesmo SQLite.
#[test]
fn parametros_repetidos_na_visualizacao_recusam_sem_entregar_documento() {
    use std::io::{Read, Write};
    use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};

    fn pedir(endereco: SocketAddr, metodo: &str, alvo: &str) -> String {
        let mut tcp = TcpStream::connect(endereco).unwrap();
        write!(
            tcp,
            "{metodo} {alvo} HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        tcp.shutdown(Shutdown::Write).unwrap();
        let mut resposta = String::new();
        tcp.read_to_string(&mut resposta).unwrap();
        resposta
    }

    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let mut banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        guardar(&mut banco, "ensaio", &documento()).unwrap();
        for _ in 0..5 {
            let (mut tcp, _) = escuta.accept().unwrap();
            anti_ies::atender_http(&mut tcp, Some((&banco, &origem(), "ensaio"))).unwrap();
        }
    });

    for alvo in [
        "/sei/controlador.php?acao=documento_visualizar&id_documento=42&id_documento=43",
        "/sei/controlador.php?acao=documento_visualizar&id_documento=42&versao=1&versao=1",
        "/sei/controlador.php?acao=documento_visualizar&id_documento=42&id_procedimento=7&id_procedimento=7",
    ] {
        let resposta = pedir(endereco, "GET", alvo);
        assert!(resposta.starts_with("HTTP/1.1 400"), "{resposta}");
        assert!(!resposta.contains("Texto atual"), "{resposta}");
    }
    let alvo = "/sei/controlador.php?acao=documento_visualizar&id_documento=42";
    assert!(pedir(endereco, "POST", alvo).starts_with("HTTP/1.1 405"));
    // A URL de histórico publicada repete id_documento com o mesmo valor.
    // Ela continua legítima, enquanto a repetição divergente acima é recusada.
    let link_historico = "/sei/controlador.php?acao=documento_visualizar&id_documento=42&id_procedimento=7&id_documento=42";
    let valido = pedir(endereco, "GET", link_historico);
    assert!(valido.starts_with("HTTP/1.1 200"), "{valido}");
    assert!(valido.contains("Texto atual"));
    servidor.join().unwrap();
}
