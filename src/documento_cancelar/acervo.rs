//! Confronto offline #273: a tela original do acervo e a projeção de
//! anti-IES são examinadas separadamente. Nunca POST no SEI.
use crate::cassette::acervo::gravacao;
use crate::cassette::{ZipDoAcervo, archive_filename, conferir_cassette_ate, html_da_resposta};
use crate::formulario_web::TelaServida;
use crate::html_atributos::{nos_com_id, valor_do_atributo};
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use iesei::banco_cassettes::iniciar;
use iesei::documento_versoes::{DocumentoSimulado, Revisao, guardar};
use rusqlite::Connection;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

/// Uma testemunha de forma calculada fora do leitor de cancelamento IES.
/// Os atributos da ação (com identidade) são conferidos separadamente pelo
/// leitor real; aqui só interessa a oferta visível dos controles de envio.
#[derive(Debug, PartialEq, Eq)]
struct Forma {
    post: bool,
    motivos: usize,
    botoes: usize,
    ocultos: usize,
}

fn forma(html: &str) -> Option<Forma> {
    let dom = tl::parse(html, tl::ParserOptions::default()).ok()?;
    let forms = nos_com_id(&dom, "frmDocumentoCancelar").ok()?;
    let [form] = forms.as_slice() else {
        return None;
    };
    let tag = form.get(dom.parser())?.as_tag()?;
    let post = valor_do_atributo(tag, "method").ok()?.is_some_and(|m| m.eq_ignore_ascii_case("post"));
    let contar = |elemento: &str, nome: &str| {
        dom.query_selector(elemento)
            .into_iter()
            .flatten()
            .filter_map(|no| no.get(dom.parser()).and_then(tl::Node::as_tag))
            .filter(|tag| valor_do_atributo(tag, "name").ok().flatten().as_deref() == Some(nome))
            .count()
    };
    Some(Forma {
        post,
        motivos: contar("textarea", "txaMotivo"),
        botoes: contar("button", "sbmSalvar"),
        ocultos: contar("input", "hdnInfraTipoPagina"),
    })
}

fn id_unico(item: &serde_json::Value, nome: &str) -> Option<String> {
    let valores = gravacao::parametro(item, nome);
    let [valor] = valores.as_slice() else {
        return None;
    };
    (!valor.is_empty() && valor.bytes().all(|b| b.is_ascii_digit())).then(|| (*valor).to_owned())
}

/// Campos de identidade vieram da URL de resposta gravada; as demais
/// colunas são dados mínimos da bancada e NÃO afirmam origem na cassette.
fn servir_no_anti_ies(base: &str, id_proc: &str, id_doc: &str) -> String {
    let origem = Instalacao::interpretar(base).expect("instalação da URL gravada");
    let mut banco = Connection::open_in_memory().expect("SQLite efêmero");
    iniciar(&banco).expect("esquema do banco derivado");
    guardar(
        &mut banco,
        "replay-273",
        &DocumentoSimulado {
            instalacao: origem.clone(),
            id_documento: id_doc.to_owned(),
            id_procedimento: id_proc.to_owned(),
            numero_processo: "00000.000001/2026-01".into(),
            numero: "0000001".into(),
            tipo: "Despacho".into(),
            revisoes: vec![Revisao {
                versao: NonZeroU32::new(1).expect("versão"),
                usuario: "Pessoa fictícia".into(),
                unidade: "TESTE".into(),
                modificacao: "01/11/2026".into(),
                html_corpo: "<p>documento sintético</p>".into(),
            }],
        },
    )
    .expect("estado mínimo para a oferta de cancelamento");
    let caminho = base.strip_prefix("https://").expect("HTTPS")
        .split_once('/')
        .map_or(String::new(), |(_, caminho)| format!("/{caminho}"));
    let alvo = format!("{caminho}/controlador.php?acao=documento_cancelar&id_procedimento={id_proc}&id_documento={id_doc}");
    let resposta = anti_ies::documento_cancelar::responder(
        "GET", &alvo, "", &banco, &origem, "replay-273",
    );
    assert_eq!(resposta.estado, anti_ies::Estado::Ok);
    resposta.corpo
}

#[test]
#[ignore = "acervo #273: exige ZIPs GO/UFRJ conferidos, executar em runner com cache público"]
fn replay_independente_formulario_documento_cancelar_go_ufrj() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || PathBuf::from(std::env::var_os("HOME").expect("HOME")).join(".cache/ies"),
        PathBuf::from,
    );
    let cards = [
        "cassettes/go/documento-mover-cancelar-go--a152c0306b45.md",
        "cassettes/ufrj/documento-mover-cancelar-ufrj--de1301eb5634.md",
    ];
    let mut forms_reais = 0usize;
    let mut comparados = 0usize;
    let mut sem_identidade = 0usize;
    let mut posts_historicos = 0usize;
    for nome in cards {
        let card = raiz.join(nome);
        let zip = archive_filename(&card).expect("origem no card");
        let zip = ZipDoAcervo::ler(&cache.join(zip)).expect("ZIP do acervo");
        let conferida = conferir_cassette_ate(&card, &zip, 120_000_000)
            .expect("SHA-1 do ZIP, SHA-256 do membro, schema e host");
        for resposta in gravacao::respostas(&conferida.json) {
            if resposta.operacao != "documento_cancelar" {
                continue;
            }
            if resposta.metodo == "POST" {
                posts_historicos += 1;
                continue; // POST isolado não comprova cancelamento nem motivo.
            }
            if resposta.metodo != "GET" || resposta.status != 200 {
                continue;
            }
            forms_reais += 1;
            let (Some(id_proc), Some(id_doc), Some(url), Ok(html)) = (
                gravacao::id_procedimento(resposta.item),
                id_unico(resposta.item, "id_documento"),
                resposta.item["response"]["url"].as_str(),
                html_da_resposta(resposta.item),
            ) else {
                sem_identidade += 1;
                continue;
            };
            let Some((base, _)) = url.split_once("/controlador.php") else {
                sem_identidade += 1;
                continue;
            };
            let Some(codificacao) = codificacao_da_tela(&gravacao::mime(resposta.item)) else {
                sem_identidade += 1;
                continue;
            };
            let origem = Instalacao::interpretar(base).expect("base do controlador");
            let tela = TelaServida { url, html: &html, codificacao };
            assert!(
                super::fluxo::conferir_formulario(&origem, &tela, &id_proc, &id_doc).is_ok(),
                "o leitor IES recusou formulário real na instalação {}",
                conferida.instalacao
            );
            let simulado = servir_no_anti_ies(base, &id_proc, &id_doc);
            let url_simulado = format!(
                "{base}/controlador.php?acao=documento_cancelar&id_procedimento={id_proc}&id_documento={id_doc}"
            );
            let sim_tela = TelaServida {
                url: &url_simulado,
                html: &simulado,
                codificacao: codificacao_da_tela("text/html; charset=utf-8").expect("UTF-8"),
            };
            assert!(
                super::fluxo::conferir_formulario(&origem, &sim_tela, &id_proc, &id_doc).is_ok(),
                "o leitor IES recusou o formulário servido pelo anti-IES"
            );
            assert_eq!(
                forma(&html),
                forma(&simulado),
                "diferença de controles entre a observação e o anti-IES na instalação {}",
                conferida.instalacao
            );
            comparados += 1;
        }
    }
    println!(
        "documento cancelar: GET reais={forms_reais}; pares confrontados={comparados}; sem identidade/codificação={sem_identidade}; POST históricos={posts_historicos} (não provam efeito)"
    );
    assert!(forms_reais > 0, "nenhum GET de cancelamento registrado");
    assert!(comparados > 0, "nenhuma tela pôde ser confrontada");
}
