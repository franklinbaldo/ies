//! BDD-342-02: a CLI real recusa identidade contraditória entre Controle e árvore.
//! Bancada exclusivamente sintética: HTTPS em loopback, anti-IES e SQLite descartável.
#![cfg(unix)]
use anti_ies::atender_http;
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::documentos_simulados::{self, Processo as ProcessoDocumental, Tipo};
use iesei::instalacao::Instalacao;
use iesei::observacao::{FaixaObservada, TabelaObservada, UnidadeExibida};
use iesei::pastas_incompletas::{self, PastaPendente};
use iesei::processos::{self, ProcessoSimulado};
use native_tls::TlsAcceptor;
use rusqlite::Connection;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[path = "aies_documento_criar_cli/fixture_tls.rs"]
mod fixture_tls;
use fixture_tls::{Pasta, identidade};

const ORIGEM: &str = "https://sei.test/sei";
const CENARIO: &str = "bdd-342-02-identidade";
const PEDIDO: &str = "00001.000001/2026-01";
const ALHEIO: &str = "00001.000002/2026-01";

#[derive(Clone, Copy)]
enum Cena {
    IdentidadeDivergente,
    PastaPendente,
    DocumentosVisiveisComPastaPendente,
    ValidacaoExigida,
    ValidacaoComCredenciais,
    ValidacaoRecusada,
}

fn aceitar(escuta: &TcpListener, parar: &AtomicBool) -> Option<TcpStream> {
    loop {
        match escuta.accept() {
            Ok((cliente, endereco)) => {
                assert!(endereco.ip().is_loopback());
                cliente
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                cliente
                    .set_write_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                return Some(cliente);
            }
            Err(erro) if erro.kind() == std::io::ErrorKind::WouldBlock => {
                if parar.load(Ordering::SeqCst) {
                    return None;
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(erro) => panic!("aceitar: {erro}"),
        }
    }
}

fn pedido_http(fonte: &mut impl Read) -> (Vec<u8>, String) {
    let mut bytes = Vec::new();
    let mut byte = [0u8];
    while !bytes.ends_with(b"\r\n\r\n") {
        assert!(bytes.len() < 16_384);
        fonte.read_exact(&mut byte).unwrap();
        bytes.push(byte[0]);
    }
    let texto = std::str::from_utf8(&bytes).unwrap();
    let primeira = texto.lines().next().unwrap();
    let evento = primeira.split_whitespace().next().unwrap().to_owned();
    let acao = primeira
        .split_once('?')
        .and_then(|(_, consulta)| {
            consulta
                .split('&')
                .find_map(|parte| parte.strip_prefix("acao="))
        })
        .unwrap_or("sem_acao")
        .to_owned();
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
    bytes.extend(corpo);
    (bytes, format!("{evento} {acao}"))
}

fn guardar_controle_parcial(banco: &mut Connection, instalacao: &Instalacao) {
    // O Controle observa PEDIDO, porém apenas uma página de duas linhas.
    // A projeção de processos não pode substituir a identidade observada
    // numa página parcial pela identidade divergente da árvore.
    banco_cassettes::guardar(
        banco,
        CENARIO,
        &ControleResumido {
            instalacao: instalacao.clone(),
            unidade: UnidadeExibida::nova("TESTE").unwrap(),
            recebidos: TabelaObservada::<Linha>::nova(
                Some(2),
                FaixaObservada::nova(1, 1),
                vec![Linha {
                    id: "10".into(),
                    numero: PEDIDO.into(),
                    visualizado: false,
                    sigiloso: false,
                }],
            )
            .unwrap(),
            gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
        },
    )
    .unwrap();
}

fn estado(cena: Cena) -> (Connection, Instalacao) {
    let mut banco = Connection::open_in_memory().unwrap();
    banco_cassettes::iniciar(&banco).unwrap();
    let instalacao = Instalacao::interpretar(ORIGEM).unwrap();

    guardar_controle_parcial(&mut banco, &instalacao);
    processos::guardar(
        &mut banco,
        &instalacao,
        CENARIO,
        &ProcessoSimulado {
            id: "10".into(),
            numero: (if matches!(cena, Cena::IdentidadeDivergente) {
                ALHEIO
            } else {
                PEDIDO
            })
            .into(),
            unidade_origem: "TESTE".into(),
            aberto_em: vec!["TESTE".into()],
            anotacao_origem: None,
            destinos: vec![],
        },
    )
    .unwrap();
    if matches!(
        cena,
        Cena::PastaPendente | Cena::DocumentosVisiveisComPastaPendente
    ) {
        // Estado de falha inventado somente onde os cards UnB não cobrem
        // a transição; a resposta HTML vem do modelo tipado no SQLite.
        pastas_incompletas::guardar(
            &banco,
            &instalacao,
            CENARIO,
            "10",
            &PastaPendente {
                indice: 1,
                protocolo_declarado: "900".into(),
            },
        )
        .unwrap();
    }
    if matches!(
        cena,
        Cena::ValidacaoExigida | Cena::ValidacaoComCredenciais | Cena::ValidacaoRecusada
    ) {
        iesei::validacao_acesso::guardar(
            &banco,
            &instalacao,
            CENARIO,
            "10",
            if matches!(cena, Cena::ValidacaoRecusada) {
                iesei::validacao_acesso::Validacao::Recusada
            } else {
                iesei::validacao_acesso::Validacao::Exigida
            },
        )
        .unwrap();
    }
    if matches!(cena, Cena::DocumentosVisiveisComPastaPendente) {
        // O corpus registra documentos visíveis e pastas fechadas, mas não
        // a falha em que a pasta nunca carrega. Só essa falha é hipotética.
        documentos_simulados::registrar_processo(
            &banco,
            &instalacao,
            CENARIO,
            &ProcessoDocumental {
                id: "10".into(),
                nup: PEDIDO.into(),
                unidade: "TESTE".into(),
            },
        )
        .unwrap();
        documentos_simulados::registrar_tipo(
            &banco,
            &instalacao,
            CENARIO,
            "10",
            &Tipo {
                id: "291".into(),
                nome: "Despacho".into(),
            },
        )
        .unwrap();
        for indice in 1..=19 {
            documentos_simulados::criar(
                &banco,
                &instalacao,
                CENARIO,
                "10",
                "291",
                &format!("Documento {indice}"),
            )
            .unwrap();
        }
        assert_eq!(
            documentos_simulados::documentos(&banco, &instalacao, CENARIO, "10")
                .unwrap()
                .len(),
            19
        );
    }
    (banco, instalacao)
}

fn identidade_no_estado(
    cena: Cena,
    banco: &Connection,
    instalacao: &Instalacao,
) -> (String, String) {
    let controle = banco_cassettes::ler(banco, instalacao, CENARIO)
        .unwrap()
        .unwrap();
    let processo = processos::ler(banco, instalacao, CENARIO, "10")
        .unwrap()
        .unwrap();
    if matches!(
        cena,
        Cena::ValidacaoExigida | Cena::ValidacaoComCredenciais | Cena::ValidacaoRecusada
    ) {
        assert_eq!(
            iesei::validacao_acesso::ler(banco, instalacao, CENARIO, "10").unwrap(),
            Some(match cena {
                Cena::ValidacaoExigida => iesei::validacao_acesso::Validacao::Exigida,
                Cena::ValidacaoComCredenciais => iesei::validacao_acesso::Validacao::Realizada,
                Cena::ValidacaoRecusada => iesei::validacao_acesso::Validacao::Recusada,
                _ => unreachable!(),
            }),
        );
    }
    (controle.recebidos.linhas[0].numero.clone(), processo.numero)
}

fn executar(cena: Cena) -> (Output, Vec<String>, (String, String)) {
    let pasta = Pasta::nova();
    let (cert, identidade_tls) = identidade(&pasta.0, "sei.test");
    let origem = pasta.privado("origem", ORIGEM);
    let chave = ["infra", "hash"].join("_");
    let controle = pasta.privado(
        "url",
        &format!(
            "{ORIGEM}/controlador.php?acao=procedimento_controlar&infra_sistema=100000100&infra_unidade_atual=110000001&{chave}={}",
            "a".repeat(64)
        ),
    );
    let cookie = pasta.privado("cookie", "PHPSESSID=FICTICIO");
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    escuta.set_nonblocking(true).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    proxy.set_nonblocking(true).unwrap();
    let porta = proxy.local_addr().unwrap().port();
    let parar = Arc::new(AtomicBool::new(false));
    let parar_servidor = Arc::clone(&parar);
    let servidor = thread::spawn(move || {
        let (banco, instalacao) = estado(cena);
        while let Some(mut cliente) = aceitar(&escuta, &parar_servidor) {
            atender_http(&mut cliente, Some((&banco, &instalacao, CENARIO))).unwrap();
        }
        identidade_no_estado(cena, &banco, &instalacao)
    });
    let trilha = Arc::new(Mutex::new(Vec::<String>::new()));
    let registro = Arc::clone(&trilha);
    let parar_proxy = Arc::clone(&parar);
    let tunel = thread::spawn(move || {
        let tls = TlsAcceptor::new(identidade_tls).unwrap();
        while let Some(mut tcp) = aceitar(&proxy, &parar_proxy) {
            let (connect, _) = pedido_http(&mut tcp);
            assert!(connect.starts_with(b"CONNECT"));
            tcp.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .unwrap();
            let mut canal = tls.accept(tcp).unwrap();
            let (bytes, evento) = pedido_http(&mut canal);
            let mut backend = TcpStream::connect(endereco).unwrap();
            backend.write_all(&bytes).unwrap();
            backend.shutdown(Shutdown::Write).unwrap();
            let mut resposta = Vec::new();
            backend.read_to_end(&mut resposta).unwrap();
            registro.lock().unwrap().push(evento);
            canal.write_all(&resposta).unwrap();
            canal.flush().unwrap();
        }
    });
    let credenciais = matches!(
        cena,
        Cena::ValidacaoComCredenciais | Cena::ValidacaoRecusada
    )
    .then(|| {
        pasta.privado(
            "credenciais",
            "usuario=teste\nsenha=SENHA_APENAS_SINTETICA\norgao=TESTE\n",
        )
    });
    let mut comando = Command::new(env!("CARGO_BIN_EXE_ies"));
    comando
        .args([
            "--json",
            "processo",
            PEDIDO,
            "--sem-andamentos",
            "--sem-propriedades",
            "--sem-marcadores",
        ])
        .env("IES_URL_CONTROLE_FILE", controle)
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
        .env_remove("IES_CREDENCIAIS_FILE");
    if let Some(caminho) = credenciais {
        comando.env("IES_CREDENCIAIS_FILE", caminho);
    }
    let saida = comando.output().unwrap();
    parar.store(true, Ordering::SeqCst);
    tunel.join().unwrap();
    let identidade_final = servidor.join().unwrap();
    let eventos = trilha.lock().unwrap().clone();
    (saida, eventos, identidade_final)
}

#[test]
fn bdd_342_02_cli_recusa_identidade_da_arvore_divergente_do_controle() {
    let (saida, pedidos, (controle, arvore)) = executar(Cena::IdentidadeDivergente);
    assert_eq!((controle.as_str(), arvore.as_str()), (PEDIDO, ALHEIO));
    assert_eq!(
        saida.status.code(),
        Some(1),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr)
    );
    assert!(
        pedidos
            .iter()
            .any(|evento| evento == "GET procedimento_controlar"),
        "a CLI deve consultar o Controle real do simulador: {pedidos:?}"
    );
    assert!(
        pedidos
            .iter()
            .any(|evento| evento == "GET procedimento_visualizar"),
        "a identidade divergente precisa ser encontrada na árvore: {pedidos:?}"
    );
    assert!(
        pedidos.iter().all(|evento| evento.starts_with("GET ")),
        "uma recusa de leitura não deve escrever: {pedidos:?}"
    );
    let saida_json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_ne!(saida_json["numero"], ALHEIO);
    assert_ne!(saida_json["documentos_incluidos"], false);
}

/// BDD-342-01: o processo e seu NUP são comprovados, mas a lista documental não.
/// PASTA+AGUARDE sem ação `ABRIR_PASTAS` é hipótese sintética de falha,
/// distinta das capturas `UnB` em que abrir as pastas teve sucesso.
#[test]
fn bdd_342_01_cli_informa_metadados_sem_inventar_documentos() {
    let (saida, pedidos, (controle, arvore)) = executar(Cena::PastaPendente);
    assert_eq!((controle.as_str(), arvore.as_str()), (PEDIDO, PEDIDO));
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr)
    );
    let valor: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(valor["numero"], PEDIDO);
    assert_eq!(valor["id_procedimento"], "10");
    assert_eq!(valor["documentos_incluidos"], false);
    assert_eq!(valor["documentos"], serde_json::json!([]));
    assert_eq!(valor["documentos_movidos"], serde_json::json!([]));
    assert_eq!(valor["pastas"], serde_json::json!([]));
    assert!(
        valor["aviso_leitura_parcial"]
            .as_str()
            .is_some_and(|aviso| aviso.contains("leitura parcial")),
        "a saída deve distinguir lista desconhecida de lista comprovada vazia: {valor}"
    );
    assert!(
        pedidos
            .iter()
            .any(|evento| evento == "GET procedimento_visualizar"),
        "a CLI precisa consultar a árvore do anti-IES: {pedidos:?}"
    );
    assert!(
        pedidos.iter().all(|evento| evento.starts_with("GET ")),
        "consulta informativa nunca executa POST: {pedidos:?}"
    );
}

/// Regressão do BDD-342-01: 19 documentos conhecidos no banco não podem
/// escapar para a saída informativa quando ainda há pasta não carregada.
/// Não é transcrição do caso 19/20 dentro da mesma pasta em uma cassette.
#[test]
fn bdd_342_01_documentos_visiveis_nao_vazam_com_pasta_pendente() {
    let (saida, pedidos, (controle, arvore)) = executar(Cena::DocumentosVisiveisComPastaPendente);
    assert_eq!((controle.as_str(), arvore.as_str()), (PEDIDO, PEDIDO));
    assert_eq!(
        saida.status.code(),
        Some(0),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr)
    );
    let valor: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(valor["numero"], PEDIDO);
    assert_eq!(valor["documentos_incluidos"], false);
    assert_eq!(valor["documentos"], serde_json::json!([]));
    assert_eq!(valor["documentos_movidos"], serde_json::json!([]));
    assert_eq!(valor["pastas"], serde_json::json!([]));
    assert!(
        valor["aviso_leitura_parcial"]
            .as_str()
            .is_some_and(|aviso| aviso.contains("leitura parcial")),
        "não publicar os 19 documentos como lista comprovada: {valor}"
    );
    assert!(
        pedidos
            .iter()
            .any(|evento| evento == "GET procedimento_visualizar"),
        "a árvore deve ser realmente consultada: {pedidos:?}"
    );
    assert!(
        pedidos.iter().all(|evento| evento.starts_with("GET ")),
        "leitura não deve produzir escrita: {pedidos:?}"
    );
}

#[path = "aies_processo_parcial_cli/bdd_633.rs"]
mod bdd_633;
