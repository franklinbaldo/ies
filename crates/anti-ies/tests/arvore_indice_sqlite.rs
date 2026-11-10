//! BDD-60-05 (#496): a árvore mista usa índices derivados de estado SQLite.
//! Ensaio sintético de HTTP local, não prova de cassette ou treinamento.
use iesei::banco_cassettes::iniciar;
use iesei::instalacao::Instalacao;
use iesei::pastas_incompletas::{self, PastaPendente};
use iesei::pdf_externo::{self, Processo, TipoExterno};
use iesei::processos::{self, ProcessoSimulado};
use rusqlite::Connection;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};

fn pedir(endereco: SocketAddr, metodo: &str, alvo: &str) -> String {
    let mut fluxo = TcpStream::connect(endereco).unwrap();
    write!(
        fluxo,
        "{metodo} {alvo} HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\n\r\n"
    )
    .unwrap();
    fluxo.shutdown(Shutdown::Write).unwrap();
    let mut resposta = String::new();
    fluxo.read_to_string(&mut resposta).unwrap();
    resposta
}

fn cadastrar_pdf(banco: &Connection, origem: &Instalacao, nome: &str) {
    let temporario =
        pdf_externo::receber_conteudo(banco, origem, "indices", "7", nome, b"%PDF-1.7\ncorpo")
            .unwrap();
    pdf_externo::cadastrar(banco, origem, "indices", "7", "293", temporario.id).unwrap();
}

#[test]
fn bdd_60_05_nos_pdf_apos_pasta_dependem_do_sqlite_e_nao_da_contagem_html() {
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let mut banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
        pdf_externo::semear(
            &banco,
            &origem,
            "indices",
            &Processo {
                id: "7".into(),
                nup: "00001.000001/2026-01".into(),
                unidade: "TESTE".into(),
            },
            &TipoExterno {
                id: "293".into(),
                nome: "Anexo".into(),
                limite_bytes: 100_000,
            },
        )
        .unwrap();
        processos::guardar(
            &mut banco,
            &origem,
            "indices",
            &ProcessoSimulado {
                id: "7".into(),
                numero: "00001.000001/2026-01".into(),
                unidade_origem: "TESTE".into(),
                aberto_em: vec!["TESTE".into()],
                anotacao_origem: None,
                destinos: vec![],
            },
        )
        .unwrap();
        pastas_incompletas::guardar(
            &banco,
            &origem,
            "indices",
            "7",
            &PastaPendente {
                indice: 1,
                protocolo_declarado: "42".into(),
            },
        )
        .unwrap();
        cadastrar_pdf(&banco, &origem, "primeiro.pdf");
        for passo in 0..4 {
            let (mut fluxo, _) = escuta.accept().unwrap();
            anti_ies::atender_http(&mut fluxo, Some((&banco, &origem, "indices"))).unwrap();
            if passo == 0 {
                // A segunda leitura HTTP usa o mesmo banco, após a transição.
                cadastrar_pdf(&banco, &origem, "segundo.pdf");
            }
        }
        assert_eq!(
            pdf_externo::documentos(&banco, &origem, "indices", "7")
                .unwrap()
                .len(),
            2,
            "GET e POST recusado não removem o registro"
        );
    });
    let alvo = "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=7";
    let antes = pedir(endereco, "GET", alvo);
    let depois = pedir(endereco, "GET", alvo);
    let negado = pedir(endereco, "POST", alvo);
    let releitura = pedir(endereco, "GET", alvo);
    servidor.join().unwrap();

    assert!(antes.starts_with("HTTP/1.1 200"));
    assert!(antes.contains("Nos[1] = new infraArvoreNo(\"PASTA\""));
    assert!(antes.contains("Nos[2] = new infraArvoreNo(\"AGUARDE\""));
    assert!(antes.contains("Nos[3] = new infraArvoreNo(\"DOCUMENTO\""));
    assert!(!antes.contains("Nos[4] = new infraArvoreNo(\"DOCUMENTO\""));
    assert!(depois.starts_with("HTTP/1.1 200"));
    assert!(depois.contains("Nos[3] = new infraArvoreNo(\"DOCUMENTO\""));
    assert!(depois.contains("Nos[4] = new infraArvoreNo(\"DOCUMENTO\""));
    assert!(negado.starts_with("HTTP/1.1 405"));
    assert_eq!(depois, releitura);
}

/// BDD-60-05 (#496): uma anexação observada no mesmo SQLite deve deslocar
/// o PDF na árvore, sem depender de contar linhas de JavaScript renderizado.
/// Variante sintética: os cards `UnB` de navegação não capturam esta transição.
#[test]
fn bdd_60_05_anexacao_desloca_pdf_sem_colisao_de_indice() {
    use iesei::anexacoes;

    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let mut banco = Connection::open_in_memory().unwrap();
        iniciar(&banco).unwrap();
        let origem = Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
        pdf_externo::semear(
            &banco,
            &origem,
            "indices",
            &Processo {
                id: "7".into(),
                nup: "00001.000001/2026-01".into(),
                unidade: "TESTE".into(),
            },
            &TipoExterno {
                id: "293".into(),
                nome: "Anexo".into(),
                limite_bytes: 100_000,
            },
        )
        .unwrap();
        for (id, nup) in [("7", "00001.000001/2026-01"), ("8", "00001.000002/2026-02")] {
            processos::guardar(
                &mut banco,
                &origem,
                "indices",
                &ProcessoSimulado {
                    id: id.into(),
                    numero: nup.into(),
                    unidade_origem: "TESTE".into(),
                    aberto_em: vec!["TESTE".into()],
                    anotacao_origem: None,
                    destinos: vec![],
                },
            )
            .unwrap();
        }
        pastas_incompletas::guardar(
            &banco,
            &origem,
            "indices",
            "7",
            &PastaPendente {
                indice: 1,
                protocolo_declarado: "42".into(),
            },
        )
        .unwrap();
        cadastrar_pdf(&banco, &origem, "com-anexacao.pdf");
        for passo in 0..4 {
            let (mut fluxo, _) = escuta.accept().unwrap();
            anti_ies::atender_http(&mut fluxo, Some((&banco, &origem, "indices"))).unwrap();
            if passo == 0 {
                // Muda a relação tipada, não o HTML devolvido ao cliente.
                anexacoes::anexar(&banco, &origem, "indices", "7", "8").unwrap();
            }
        }
        assert_eq!(
            anexacoes::listar(&banco, &origem, "indices", "7")
                .unwrap()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            pdf_externo::documentos(&banco, &origem, "indices", "7")
                .unwrap()
                .len(),
            1,
            "GET e POST 405 não modificam o documento"
        );
    });

    let alvo = "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=7";
    let antes = pedir(endereco, "GET", alvo);
    let depois = pedir(endereco, "GET", alvo);
    let recusado = pedir(endereco, "POST", alvo);
    let releitura = pedir(endereco, "GET", alvo);
    servidor.join().unwrap();

    assert!(antes.starts_with("HTTP/1.1 200"));
    assert!(antes.contains("Nos[3] = new infraArvoreNo(\"DOCUMENTO\""));
    assert!(!antes.contains("new infraArvoreNo(\"PROCESSO_ANEXADO\""));
    assert!(depois.starts_with("HTTP/1.1 200"));
    assert!(depois.contains("Nos[1] = new infraArvoreNo(\"PASTA\""));
    assert!(depois.contains("Nos[2] = new infraArvoreNo(\"AGUARDE\""));
    assert!(depois.contains("Nos[3] = new infraArvoreNo(\"PROCESSO_ANEXADO\""));
    assert!(depois.contains("Nos[4] = new infraArvoreNo(\"DOCUMENTO\""));
    assert!(depois.contains("00001.000002/2026-02"));
    assert!(recusado.starts_with("HTTP/1.1 405"));
    assert_eq!(depois, releitura, "a recusa preserva a projeção do SQLite");
}
