//! Confronto offline da tela de anexação com as cassettes UnB/GO.
//! Não atribui paridade ao POST 503 não capturado.
//! As identidades sintéticas complementares são andaimes para a projeção HTTP.
use crate::cassette::acervo::gravacao;
use crate::cassette::{ZipDoAcervo, archive_filename, conferir_cassette_ate, html_da_resposta};
use iesei::processos::{self, ProcessoSimulado};
use iesei::{anexacoes, banco_cassettes};
use std::io::{Read as _, Write as _};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::{Path, PathBuf};

const CENARIO: &str = "replay-anexacao-formulario";
const UNIDADE: &str = "ACERVO";

fn processo(id: &str, numero: String) -> ProcessoSimulado {
    ProcessoSimulado {
        id: id.to_owned(),
        numero,
        unidade_origem: UNIDADE.to_owned(),
        aberto_em: vec![UNIDADE.to_owned()],
        anotacao_origem: None,
        destinos: vec![],
    }
}

/// Consulta o anti-IES por HTTP local, sem usar a saída do IES como oráculo.
fn formulario_simulado(
    banco: rusqlite::Connection,
    origem: iesei::instalacao::Instalacao,
    id: &str,
) -> String {
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let (mut fluxo, _) = escuta.accept().unwrap();
        anti_ies::atender_http(&mut fluxo, Some((&banco, &origem, CENARIO))).unwrap();
    });
    let mut cliente = TcpStream::connect(endereco).unwrap();
    write!(
        cliente,
        "GET /sei/controlador.php?acao=procedimento_anexar&id_procedimento={id} HTTP/1.1\r\nHost: localhost\r\n\r\n"
    )
    .unwrap();
    cliente.shutdown(Shutdown::Write).unwrap();
    let mut resposta = String::new();
    cliente.read_to_string(&mut resposta).unwrap();
    servidor.join().unwrap();
    let (cabecalho, corpo) = resposta.split_once("\r\n\r\n").unwrap();
    assert!(cabecalho.starts_with("HTTP/1.1 200 OK"), "anti-IES não ofereceu formulário de anexação");
    corpo.to_owned()
}

fn confrontar(instalacao: &str, json: &serde_json::Value) -> (usize, usize) {
    let origem = gravacao::instalacao(json).expect("origem validada por card");
    let mut lidos = 0;
    let mut bilaterais = 0;
    for resposta in gravacao::respostas(json) {
        if resposta.operacao != "procedimento_anexar"
            || resposta.metodo != "GET"
            || resposta.status != 200
        {
            continue;
        }
        let html = html_da_resposta(resposta.interacao).expect("GET gravado com HTML legível");
        if !html.contains("frmProcedimentoAnexar") {
            continue; // outros GETs da mesma operação não oferecem o formulário
        }
        let observado = super::ler_tela(&html).expect("leitor Rust da tela original de anexação");
        lidos += 1;

        let Some(id) = gravacao::id_procedimento(resposta.interacao) else {
            continue; // sem identidade de URL não se pode confrontar o contexto HTTP
        };
        // O ID principal e os anexados são fatos da resposta observada.
        // Os NUPs abaixo NÃO foram extraídos desta tela: só permitem criar
        // o contexto relacional do SQLite sem fingir origem externa.
        let mut banco = rusqlite::Connection::open_in_memory().unwrap();
        banco_cassettes::iniciar(&banco).unwrap();
        processos::guardar(
            &mut banco,
            &origem,
            CENARIO,
            &processo(&id, "NUP-PRINCIPAL-NAO-OBSERVADO".to_owned()),
        )
        .unwrap();
        for (indice, anexado) in observado.anexados.iter().enumerate() {
            assert_ne!(anexado, &id, "processo anexado igual ao principal em {instalacao}");
            processos::guardar(
                &mut banco,
                &origem,
                CENARIO,
                &processo(anexado, format!("NUP-NAO-OBSERVADO-{indice}")),
            )
            .unwrap();
            anexacoes::anexar(&banco, &origem, CENARIO, &id, anexado).unwrap();
        }
        let simulado = formulario_simulado(banco, origem.clone(), &id);
        let projetado = super::ler_tela(&simulado).expect("leitor Rust da projeção do anti-IES");
        let mut ids_originais = observado.anexados;
        let mut ids_simulados = projetado.anexados;
        ids_originais.sort_unstable();
        ids_simulados.sort_unstable();
        assert_eq!(ids_simulados, ids_originais, "IDs de anexados divergentes em {instalacao}");
        bilaterais += 1;
    }
    (lidos, bilaterais)
}

/// Executado pelo job `scripts/replay acervo` no GitHub Actions.
/// Confere ZIP SHA-1 e membro SHA-256 antes de ler corpos originais.
/// Não valida POST, HTTP 503 pós-escrita nem efeito real no SEI.
#[test]
#[ignore = "cassette anexação: ZIPs públicos conferidos no job de replay oficial"]
fn replay_de_processo_anexar_no_acervo() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("cassettes");
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || PathBuf::from(std::env::var_os("HOME").expect("HOME")).join(".cache/ies"),
        PathBuf::from,
    );
    let mut lidos = 0;
    let mut bilaterais = 0;
    for (instalacao, arquivo) in [
        ("unb", "processo-anexar-unb--031b1681716a.md"),
        ("go", "processo-anexar-go--7dec4ec7e6f4.md"),
    ] {
        let card = raiz.join(instalacao).join(arquivo);
        let zip = archive_filename(&card).expect("origem do card");
        let zip = ZipDoAcervo::ler(&cache.join(zip)).expect("ZIP original no cache verificado");
        let observacao = conferir_cassette_ate(&card, &zip, 120_000_000)
            .expect("ZIP, JSON, schema, origem e completude");
        assert_eq!(observacao.instalacao, instalacao);
        let (leitor, confronto) = confrontar(instalacao, &observacao.json);
        assert!(leitor > 0, "nenhum GET original de formulário legível em {instalacao}");
        lidos += leitor;
        bilaterais += confronto;
    }
    assert!(bilaterais > 0, "nenhum GET de anexação confrontado com anti-IES");
    println!("anexação GET original: {lidos} leitores IES, {bilaterais} comparações com HTTP anti-IES; POST 503: NOT_RUN externo");
}
