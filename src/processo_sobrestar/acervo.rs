//! #269: confronto estrutural offline do formulario de sobrestamento.
//! Nao confunde forma observada com comprovacao de efeito do POST.
//! O ZIP original e verificado por hash antes de interpretar qualquer HTML.
use super::tela;
use crate::cassette::acervo::gravacao;
use crate::cassette::{ZipDoAcervo, conferir_cassette_ate, html_da_resposta};
use crate::html_atributos::valor_do_atributo;
use iesei::banco_cassettes::{self, ControleResumido, Linha};
use iesei::instalacao::Instalacao;
use iesei::observacao::{FaixaObservada, TabelaObservada, UnidadeExibida};
use iesei::processos::{self, ProcessoSimulado};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::{Path, PathBuf};

const CENARIO: &str = "bdd-269-formulario-original";
const ORIGEM: &str = "https://sei.test/sei";
const NUP: &str = "00001.000001/2026-01";
const ID: &str = "10";

/// Campos enviaveis e botoes exibidos, sem valores de sessao nem links.
/// Preserva a ordem observada e os valores publicos das opcoes de formulario.
fn forma(html: &str) -> Vec<(String, String, Option<String>)> {
    let dom = tl::parse(html, tl::ParserOptions::default()).unwrap();
    let form = crate::formulario_web::localizar_formulario(&dom, "frmDesentranharDocumento")
        .expect("frmDesentranharDocumento presente");
    form.children()
        .all(dom.parser())
        .iter()
        .filter_map(tl::Node::as_tag)
        .filter_map(|tag| {
            let tipo = tag.name().as_utf8_str().to_ascii_lowercase();
            if !matches!(tipo.as_str(), "input" | "button" | "select" | "textarea") {
                return None;
            }
            let attr = |nome| valor_do_atributo(tag, nome).unwrap();
            let nome = attr("name").expect("controle com nome");
            let classe = attr("type").unwrap_or(tipo);
            let opcao = match nome.as_str() {
                "rdoSobrestar" | "sbmSalvar" | "hdnInfraTipoPagina" => attr("value"),
                _ => None,
            };
            Some((nome, classe, opcao))
        })
        .collect()
}

/// Serve o HTML a partir de estado SQLite real do anti-IES, sem template local.
fn forma_do_anti_ies() -> String {
    let mut banco = rusqlite::Connection::open_in_memory().unwrap();
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
                    id: ID.into(),
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
            id: ID.into(),
            numero: NUP.into(),
            unidade_origem: "TESTE".into(),
            aberto_em: vec!["TESTE".into()],
            anotacao_origem: None,
            destinos: vec![],
        },
    )
    .unwrap();

    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let servidor = std::thread::spawn(move || {
        let (mut fluxo, _) = escuta.accept().unwrap();
        anti_ies::atender_http(&mut fluxo, Some((&banco, &instalacao, CENARIO))).unwrap();
    });
    let mut cliente = TcpStream::connect(endereco).unwrap();
    write!(
        cliente,
        "GET /sei/controlador.php?acao=procedimento_sobrestar&id_procedimento={ID} HTTP/1.1\r\nHost: sei.test\r\n\r\n"
    )
    .unwrap();
    cliente.shutdown(Shutdown::Write).unwrap();
    let mut resposta = String::new();
    cliente.read_to_string(&mut resposta).unwrap();
    servidor.join().unwrap();
    let (cabecalho, corpo) = resposta.split_once("\r\n\r\n").unwrap();
    assert!(cabecalho.starts_with("HTTP/1.1 200 OK"), "{cabecalho}");
    corpo.to_owned()
}

fn cache() -> PathBuf {
    std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || PathBuf::from(std::env::var_os("HOME").unwrap()).join(".cache/ies"),
        PathBuf::from,
    )
}

/// Compara duas fontes independentes: cada HTML original e a resposta do
/// servidor anti-IES. O POST/releitura permanece coberto pelo E2E sintetico.
#[test]
#[ignore = "ZIPs publicos SHA-verificados em IES_ACERVO_CACHE; sem rede SEI"]
fn replay_do_formulario_de_sobrestamento_unb_go_contra_anti_ies() {
    let simulado = forma_do_anti_ies();
    tela::conferir_formulario(&simulado, ID).unwrap();
    let campos_simulados = forma(&simulado);
    for caminho in [
        "cassettes/unb/processo-sobrestar-retomar-unb--cde2a847ebd8.md",
        "cassettes/go/processo-sobrestar-retomar-go--4fb6c8fe18df.md",
    ] {
        let card = Path::new(env!("CARGO_MANIFEST_DIR")).join(caminho);
        let zip =
            ZipDoAcervo::ler(&cache().join(crate::cassette::archive_filename(&card).unwrap()))
                .unwrap();
        let captura = conferir_cassette_ate(&card, &zip, 120_000_000).unwrap();
        let telas: Vec<_> = gravacao::respostas(&captura.json)
            .into_iter()
            .filter(|r| {
                r.metodo == "GET" && r.operacao == "procedimento_sobrestar" && r.status == 200
            })
            .collect();
        assert_eq!(telas.len(), 1, "uma tela observada: {caminho}");
        for resposta in telas {
            let id = gravacao::id_procedimento(resposta.item)
                .expect("id_procedimento unico na resposta original");
            let original = html_da_resposta(resposta.item).unwrap();
            tela::conferir_formulario(&original, &id)
                .unwrap_or_else(|e| panic!("{caminho}: leitor IES recusou forma real: {e:?}"));
            assert_eq!(
                forma(&original),
                campos_simulados,
                "divergencia entre formulario original {caminho} e HTML do anti-IES"
            );
        }
    }
}
