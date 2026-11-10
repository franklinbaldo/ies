//! BDD-255-02 (prévia) e 255-03: CLI real, HTTPS local, anti-IES e SQLite.
//! Hipótese sintética, não prova paridade com uma instalação SEI.
#![cfg(unix)]
use anti_ies::atender_http;
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::instalacao::Instalacao;
use iesei::marcadores::{self, Marcador, Observacao};
use iesei::observacao::{FaixaObservada, TabelaObservada, UnidadeExibida};
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
const CENARIO: &str = "bdd-255-marcador-idempotente";
const NUP: &str = "00001.000001/2026-01";

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
        .and_then(|(_, query)| query.split('&').find_map(|s| s.strip_prefix("acao=")))
        .unwrap_or("sem_acao");
    let evento = format!("{metodo} {acao}");
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
    cabecalho.extend(corpo);
    (cabecalho, evento)
}

fn estado(sem_marcadores: bool) -> (Connection, Instalacao) {
    let mut banco = Connection::open_in_memory().unwrap();
    banco_cassettes::iniciar(&banco).unwrap();
    let instalacao = Instalacao::interpretar(ORIGEM).unwrap();
    banco_cassettes::guardar(
        &mut banco,
        CENARIO,
        &ControleResumido {
            instalacao: instalacao.clone(),
            unidade: UnidadeExibida::nova("TESTE").unwrap(),
            recebidos: TabelaObservada::<Linha>::nova(
                Some(1),
                FaixaObservada::nova(1, 1),
                vec![Linha {
                    id: "10".into(),
                    numero: NUP.into(),
                    visualizado: false,
                    sigiloso: false,
                }],
            )
            .unwrap(),
            gerados: TabelaObservada::<Linha>::nova(Some(0), None, vec![]).unwrap(),
        },
    )
    .unwrap();
    processos::guardar(
        &mut banco,
        &instalacao,
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
    // Observação explícita: marcada ou vazia. A ausência de registro é desconhecida.
    let itens = if sem_marcadores {
        Vec::new()
    } else {
        vec![Marcador {
            id: "501".into(),
            nome: "Revisão".into(),
            cor: "azul".into(),
            texto: Some("Conferir".into()),
            usuario: Some("aluno1".into()),
            data_hora: "04/11/2026 09:00".into(),
        }]
    };
    let total_declarado = itens.len();
    marcadores::registrar(
        &mut banco,
        &instalacao,
        CENARIO,
        "10",
        "TESTE",
        &Observacao {
            total_declarado,
            itens,
        },
    )
    .unwrap();
    (banco, instalacao)
}

fn executar_cli(
    pasta: &Pasta,
    arquivos: [&std::path::Path; 4],
    porta_proxy: u16,
    argumentos: &[&str],
) -> Output {
    let [controle, cookie, origem, cert] = arquivos;
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(argumentos)
        .env("IES_URL_CONTROLE_FILE", controle)
        .env("IES_COOKIE_FILE", cookie)
        .env("IES_ORIGEM_FILE", origem)
        .env("SSL_CERT_FILE", cert)
        .env("https_proxy", format!("http://127.0.0.1:{porta_proxy}"))
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

/// Testa a mesma prévia na saída humana; não há POST de remoção.
fn conferir_previa_humana(pasta: &Pasta, arquivos: [&std::path::Path; 4], porta_proxy: u16) {
    let preview = executar_cli(
        pasta,
        arquivos,
        porta_proxy,
        &["processo", "marcar", NUP, "--retirar", "501#Revisão"],
    );
    assert_eq!(preview.status.code(), Some(0));
    let humano = String::from_utf8_lossy(&preview.stdout);
    assert!(humano.contains("Prévia: retirar marcador 501#Revisão"));
    assert!(humano.contains("Esta prévia não alterou o processo"));
    assert!(humano.contains("--confirmar em treinamento"));
    assert!(humano.contains("formulário conferido"));
    assert!(!humano.contains("A escrita ainda não está disponível"));
}

#[derive(Clone, Copy)]
enum Caso {
    AtribuirExistente,
    RetirarAusente,
    RetirarPresente,
    RetirarSemMarcadores,
    RetirarConfirmado,
    RetirarEfeitoIncerto,
}

fn pedido(caso: Caso) -> (&'static str, &'static str) {
    match caso {
        Caso::AtribuirExistente => ("--marcador", "501#Revisão"),
        Caso::RetirarAusente | Caso::RetirarSemMarcadores => ("--retirar", "502#Pendente"),
        Caso::RetirarPresente | Caso::RetirarConfirmado | Caso::RetirarEfeitoIncerto => {
            ("--retirar", "501#Revisão")
        }
    }
}

fn executar_caso_cli(
    pasta: &Pasta,
    arquivos: [&std::path::Path; 4],
    porta_proxy: u16,
    caso: Caso,
) -> Output {
    let (operacao, marcador) = pedido(caso);
    let mut argumentos = vec!["--json", "processo", "marcar", NUP, operacao, marcador];
    if matches!(caso, Caso::RetirarConfirmado | Caso::RetirarEfeitoIncerto) {
        argumentos.push("--confirmar");
    }
    executar_cli(pasta, arquivos, porta_proxy, &argumentos)
}

fn injetar_resposta_inconclusiva(resposta: &mut [u8], evento: &str, efeito_incerto: bool) {
    if efeito_incerto && evento == "POST andamento_marcador_remover" {
        assert!(resposta.starts_with(b"HTTP/1.1 302 "));
        // O anti-IES já aplicou o POST; a resposta recebida pela CLI é 403.
        // A Location anterior não autentica a escrita quando status é 403.
        resposta[9..12].copy_from_slice(b"403");
    }
}

fn conferir_controles(controles: &[(bool, bool, bool)], sem_marcadores: bool) {
    if sem_marcadores {
        assert!(
            controles.is_empty(),
            "zero marcadores usa 302 sem formulário de gerenciamento"
        );
    } else {
        assert!(!controles.is_empty(), "gerenciamento não foi servido");
        assert!(
            controles.iter().all(|vistos| *vistos == (true, true, true)),
            "GET de gerenciamento deve oferecer adicionar, remover e remoção individual"
        );
    }
}

fn executar(caso: Caso) {
    let sem_marcadores = matches!(caso, Caso::RetirarSemMarcadores);
    let efeito_incerto = matches!(caso, Caso::RetirarEfeitoIncerto);
    let pasta = Pasta::nova();
    let (cert, tls_id) = identidade(&pasta.0, "sei.test");
    let origem = pasta.privado("origem", ORIGEM);
    let hash_chave = ["infra", "hash"].join("_");
    let controle = pasta.privado("url", &format!(
        "{ORIGEM}/controlador.php?acao=procedimento_controlar&infra_sistema=100000100&infra_unidade_atual=110000001&{hash_chave}={}",
        "a".repeat(64)
    ));
    let cookie = pasta.privado("cookie", "PHPSESSID=FICTICIO");
    let backend = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    backend.set_nonblocking(true).unwrap();
    let endereco = backend.local_addr().unwrap();
    let proxy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    proxy.set_nonblocking(true).unwrap();
    let porta_proxy = proxy.local_addr().unwrap().port();
    let parar = Arc::new(AtomicBool::new(false));
    let sinal = Arc::clone(&parar);
    let servidor = thread::spawn(move || {
        let (banco, instalacao) = estado(sem_marcadores);
        while let Some(mut cliente) = aceitar(&backend, &sinal) {
            atender_http(&mut cliente, Some((&banco, &instalacao, CENARIO))).unwrap();
        }
        marcadores::ler(&banco, &instalacao, CENARIO, "10", "TESTE")
            .unwrap()
            .unwrap()
    });
    let trilha = Arc::new(Mutex::new(Vec::<String>::new()));
    let pedidos = Arc::clone(&trilha);
    // Inspecionar somente estrutura do GET, sem URLs ou corpos em logs.
    let controles = Arc::new(Mutex::new(Vec::<(bool, bool, bool)>::new()));
    let controles_no_tunel = Arc::clone(&controles);
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
            injetar_resposta_inconclusiva(&mut resposta, &evento, efeito_incerto);
            if evento == "GET andamento_marcador_gerenciar"
                && resposta.starts_with(b"HTTP/1.1 200 ")
            {
                let tela = String::from_utf8_lossy(&resposta);
                controles_no_tunel.lock().unwrap().push((
                    tela.contains("id='btnAdicionar'"),
                    tela.contains("id='btnRemover'"),
                    tela.contains("acaoRemover("),
                ));
            }
            pedidos.lock().unwrap().push(evento);
            canal.write_all(&resposta).unwrap();
            canal.flush().unwrap();
        }
    });
    let arquivos = [
        controle.as_path(),
        cookie.as_path(),
        origem.as_path(),
        cert.as_path(),
    ];
    let saida = executar_caso_cli(&pasta, arquivos, porta_proxy, caso);
    if matches!(caso, Caso::RetirarPresente) {
        conferir_previa_humana(&pasta, arquivos, porta_proxy);
    }
    // Segunda execução independente: identidade e ausência de alteração persistidas.
    let releitura = executar_cli(
        &pasta,
        arquivos,
        porta_proxy,
        &[
            "--json",
            "processo",
            NUP,
            "--sem-andamentos",
            "--sem-documentos",
            "--sem-propriedades",
        ],
    );
    parar.store(true, Ordering::SeqCst);
    tunel.join().unwrap();
    let persistido = servidor.join().unwrap();
    let pedidos = trilha.lock().unwrap().clone();
    let controles = controles.lock().unwrap().clone();
    conferir_controles(&controles, sem_marcadores);
    conferir(&saida, &releitura, &persistido, &pedidos, caso);
}
fn conferir(
    saida: &Output,
    releitura: &Output,
    persistido: &Observacao,
    pedidos: &[String],
    caso: Caso,
) {
    let sem_marcadores = matches!(caso, Caso::RetirarSemMarcadores);
    let retirada_presente = matches!(caso, Caso::RetirarPresente);
    let confirmado = matches!(caso, Caso::RetirarConfirmado | Caso::RetirarEfeitoIncerto);
    let efeito_incerto = matches!(caso, Caso::RetirarEfeitoIncerto);
    let json: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(
        saida.status.code(),
        Some(if efeito_incerto { 3 } else { 0 }),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&saida.stdout),
        String::from_utf8_lossy(&saida.stderr)
    );
    if efeito_incerto {
        assert_eq!(json["codigo"], "efeito_incerto");
        assert_eq!(json["retentavel"], false);
        assert_eq!(json["ausente_relido"], true);
        assert!(json["erro"].as_str().unwrap().contains("não repita"));
    } else {
        assert_eq!(json["nada_a_fazer"], !retirada_presente && !confirmado);
        assert_eq!(json["retirada_possivel"], confirmado);
        assert_eq!(json["confirmado"], confirmado);
        assert_eq!(json["alterado"], confirmado);
    }
    assert!(json["requisicoes"].as_u64().is_some());
    assert_eq!(
        releitura.status.code(),
        Some(0),
        "stdout={} stderr={} pedidos={pedidos:?}",
        String::from_utf8_lossy(&releitura.stdout),
        String::from_utf8_lossy(&releitura.stderr)
    );
    let processo: serde_json::Value = serde_json::from_slice(&releitura.stdout).unwrap();
    let total = usize::from(!sem_marcadores && !confirmado);
    assert_eq!(processo["marcadores_total_declarado"], total);
    assert_eq!(processo["marcadores"].as_array().unwrap().len(), total);
    assert_eq!(persistido.total_declarado, total);
    assert_eq!(persistido.itens.len(), total);
    if !sem_marcadores && !confirmado {
        assert_eq!(processo["marcadores"][0]["rotulo"], "501#Revisão");
        assert_eq!(processo["marcadores"][0]["texto"], "Conferir");
        assert_eq!(persistido.itens[0].id, "501");
    }
    assert_eq!(
        pedidos.iter().filter(|p| p.starts_with("POST ")).count(),
        usize::from(confirmado)
    );
    if confirmado {
        assert_eq!(
            pedidos
                .iter()
                .filter(|p| p.as_str() == "POST andamento_marcador_remover")
                .count(),
            1
        );
    }
    assert!(
        pedidos
            .iter()
            .any(|p| p == "GET andamento_marcador_gerenciar"),
        "a CLI não leu o gerenciamento de marcadores: {pedidos:?}"
    );
}
#[test]
fn bdd_255_03_atribuir_marcador_existente_e_idempotente() {
    executar(Caso::AtribuirExistente);
}
#[test]
fn bdd_255_03_retirar_marcador_ausente_e_idempotente() {
    executar(Caso::RetirarAusente);
}

#[test]
fn bdd_255_03_retirar_de_processo_sem_marcadores_nao_inventa_lista() {
    // Cassette UnB: a tela sem marcadores é 302 para cadastrar, sem POST.
    executar(Caso::RetirarSemMarcadores);
}

#[test]
fn bdd_255_02_previa_de_retirar_presente_mantem_marcador_na_releitura() {
    // Fase prévia apenas: status 0 não significa que remoção foi confirmada.
    executar(Caso::RetirarPresente);
}

#[test]
fn bdd_255_02_retirar_presente_confirmado_exige_post_unico_e_releitura() {
    executar(Caso::RetirarConfirmado);
}

#[test]
fn bdd_255_02_post_aplicado_com_403_e_efeito_incerto_sem_retry() {
    // Resposta 403 após o POST aplicado não é aceite causal, mesmo com releitura ausente.
    executar(Caso::RetirarEfeitoIncerto);
}
