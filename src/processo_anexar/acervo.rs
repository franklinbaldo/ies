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

// Extrator independente dos fatos observáveis. Usa apenas o DOM e os
// atributos da resposta original, jamais `ler_tela` nem a saída do IES.
#[derive(Debug, PartialEq, Eq)]
struct FatosDoFormulario {
    controles: Vec<String>,
    anexados: Vec<String>,
}

fn atributo_da_fonte(tag: &tl::HTMLTag<'_>, nome: &str) -> Option<String> {
    // Leitura crua e restrita do atributo na fonte, sem usar o leitor do IES.
    // O DOM localiza as tags; os bytes da tag oferecem o oráculo separado.
    let fonte = std::str::from_utf8(tag.raw().as_bytes()).expect("tag UTF-8 na fonte");
    for (posicao, _) in fonte.match_indices(nome) {
        if posicao == 0 || !fonte.as_bytes()[posicao - 1].is_ascii_whitespace() {
            continue;
        }
        let Some(restante) = fonte[posicao + nome.len()..]
            .trim_start()
            .strip_prefix('=')
        else {
            continue;
        };
        let restante = restante.trim_start();
        let valor = if let Some(texto) = restante.strip_prefix('"') {
            texto.split('"').next()
        } else if let Some(texto) = restante.strip_prefix('\'') {
            texto.split('\'').next()
        } else {
            restante.split(|c: char| c.is_ascii_whitespace() || c == '>' || c == '/').next()
        };
        if let Some(valor) = valor {
            return Some(valor.to_owned());
        }
    }
    None
}

fn fatos_da_fonte(html: &str) -> FatosDoFormulario {
    let dom = tl::parse(html, tl::ParserOptions::default()).expect("DOM da fonte");
    let parser = dom.parser();
    let formulario = dom
        .query_selector("form")
        .into_iter()
        .flatten()
        .filter_map(|no| no.get(parser).and_then(tl::Node::as_tag))
        .find(|tag| atributo_da_fonte(tag, "id").as_deref() == Some("frmProcedimentoAnexar"))
        .expect("formulário oferecido na fonte");
    assert_eq!(
        atributo_da_fonte(formulario, "method")
            .unwrap_or_default()
            .to_ascii_lowercase(),
        "post",
        "método da oferta original"
    );
    assert!(
        atributo_da_fonte(formulario, "action")
            .is_some_and(|v| v.contains("acao=procedimento_anexar")),
        "ação do formulário da fonte"
    );
    let exigidos = [
        "hdnAnexacao",
        "hdnIdProtocolo",
        "sbmAnexar",
        "txtIdentificacaoProtocolo",
        "txtProtocolo",
    ];
    let mut controles: Vec<String> = formulario
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
        .filter(|tag| matches!(tag.name().as_utf8_str().as_ref(), "input" | "button"))
        .filter_map(|tag| atributo_da_fonte(tag, "name"))
        .filter(|nome| exigidos.contains(&nome.as_str()))
        .collect();
    controles.sort_unstable();
    assert_eq!(controles, exigidos, "campos da oferta original");

    let mut anexados = Vec::new();
    let mut tabelas = 0;
    for tabela in dom.query_selector("table").into_iter().flatten() {
        let Some(tag) = tabela.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        if atributo_da_fonte(tag, "summary").as_deref() != Some("Lista de Processos Anexados") {
            continue;
        }
        tabelas += 1;
        for filho in tag.children().all(parser) {
            let Some(input) = filho.as_tag() else {
                continue;
            };
            if input.name().as_utf8_str().eq_ignore_ascii_case("input")
                && atributo_da_fonte(input, "type")
                    .is_some_and(|v| v.eq_ignore_ascii_case("checkbox"))
            {
                let id = atributo_da_fonte(input, "value").expect("ID original de anexado");
                assert!(
                    !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()),
                    "ID original precisa ser decimal"
                );
                anexados.push(id);
            }
        }
    }
    assert!(tabelas <= 1, "lista original ambígua");
    anexados.sort_unstable();
    assert!(
        anexados.windows(2).all(|par| par[0] != par[1]),
        "ID original duplicado"
    );
    FatosDoFormulario {
        controles,
        anexados,
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
    assert!(
        cabecalho.starts_with("HTTP/1.1 200 OK"),
        "anti-IES não ofereceu formulário de anexação"
    );
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
        let fonte = fatos_da_fonte(&html);
        let observado = super::ler_tela(&html).expect("leitor Rust da tela original de anexação");
        let mut ids_ies = observado.anexados;
        ids_ies.sort_unstable();
        assert_eq!(
            ids_ies, fonte.anexados,
            "IES divergiu dos IDs extraídos da fonte em {instalacao}"
        );
        lidos += 1;

        let Some(id) = gravacao::id_procedimento(resposta.interacao) else {
            continue; // sem identidade de URL não se pode confrontar o contexto HTTP
        };
        // IDs vêm do extrator da resposta original, nunca do leitor IES.
        // Os NUPs NÃO constam da tela: apenas permitem a projeção SQLite.
        let mut banco = rusqlite::Connection::open_in_memory().unwrap();
        banco_cassettes::iniciar(&banco).unwrap();
        processos::guardar(
            &mut banco,
            &origem,
            CENARIO,
            &processo(&id, "NUP-PRINCIPAL-NAO-OBSERVADO".to_owned()),
        )
        .unwrap();
        for (indice, anexado) in fonte.anexados.iter().enumerate() {
            assert_ne!(
                anexado, &id,
                "processo anexado igual ao principal em {instalacao}"
            );
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
        let fonte_simulada = fatos_da_fonte(&simulado);
        assert_eq!(
            fonte_simulada, fonte,
            "oferta/IDs do anti-IES divergem da fonte em {instalacao}"
        );
        let projetado = super::ler_tela(&simulado).expect("leitor Rust da projeção do anti-IES");
        let mut ids_simulados = projetado.anexados;
        ids_simulados.sort_unstable();
        assert_eq!(
            ids_simulados, fonte.anexados,
            "leitor do HTTP simulado diverge da fonte em {instalacao}"
        );
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
        assert!(
            leitor > 0,
            "nenhum GET original de formulário legível em {instalacao}"
        );
        assert!(
            confronto > 0,
            "nenhum confronto independente anti-IES em {instalacao}"
        );
        lidos += leitor;
        bilaterais += confronto;
    }
    assert!(
        bilaterais > 0,
        "nenhum GET de anexação confrontado com anti-IES"
    );
    println!(
        "anexação GET original: {lidos} leitores IES, {bilaterais} comparações com HTTP anti-IES; POST 503: NOT_RUN externo"
    );
}
