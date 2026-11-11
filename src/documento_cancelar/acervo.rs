//! Confronto offline #273: a tela original do acervo e a projeção de
//! anti-IES são examinadas separadamente. Nunca POST no SEI.
use crate::cassette::acervo::gravacao;
use crate::cassette::{ZipDoAcervo, archive_filename, conferir_cassette_ate, html_da_resposta};
use crate::formulario_web::TelaServida;
use crate::html_atributos::{nos_com_id, valor_do_atributo};
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::tela_html::HtmlServido;
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
    let servido = HtmlServido::de_teste(html);
    let dom = crate::tela_html::interpretar(&servido);
    let forms = nos_com_id(&dom, "frmDocumentoCancelar").ok()?;
    let [form] = forms.as_slice() else {
        return None;
    };
    let tag = form.get(dom.parser())?.as_tag()?;
    let post = valor_do_atributo(tag, "method")
        .ok()?
        .is_some_and(|m| m.eq_ignore_ascii_case("post"));
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

fn id_unico(interacao: &serde_json::Value, nome: &str) -> Option<String> {
    let valores = gravacao::parametro(interacao, nome);
    let [valor] = valores.as_slice() else {
        return None;
    };
    (!valor.is_empty() && valor.bytes().all(|b| b.is_ascii_digit())).then(|| (*valor).to_owned())
}

/// Campos de identidade vieram da URL de resposta gravada; as demais
/// colunas são dados mínimos da bancada e NÃO afirmam origem na cassette.
/// A bancada isola a identidade observada da cassette e os demais dados hipotéticos.
fn preparar_estado_anti_ies(
    base: &str,
    id_proc: &str,
    id_doc: &str,
) -> (Connection, Instalacao, String) {
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
    let caminho = base
        .strip_prefix("https://")
        .expect("HTTPS")
        .split_once('/')
        .map_or(String::new(), |(_, caminho)| format!("/{caminho}"));
    let alvo = format!(
        "{caminho}/controlador.php?acao=documento_cancelar&id_procedimento={id_proc}&id_documento={id_doc}"
    );
    (banco, origem, alvo)
}

fn servir_no_anti_ies(base: &str, id_proc: &str, id_doc: &str) -> String {
    let (banco, origem, alvo) = preparar_estado_anti_ies(base, id_proc, id_doc);
    let resposta =
        anti_ies::documento_cancelar::responder("GET", &alvo, "", &banco, &origem, "replay-273");
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
        let anteriores = (forms_reais, comparados, sem_identidade, posts_historicos);
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
                gravacao::id_procedimento(resposta.interacao),
                id_unico(resposta.interacao, "id_documento"),
                resposta.interacao["response"]["url"].as_str(),
                html_da_resposta(resposta.interacao),
            ) else {
                sem_identidade += 1;
                continue;
            };
            let Some((base, _)) = url.split_once("/controlador.php") else {
                sem_identidade += 1;
                continue;
            };
            let Some(codificacao) = codificacao_da_tela(&gravacao::mime(resposta.interacao)) else {
                sem_identidade += 1;
                continue;
            };
            let origem = Instalacao::interpretar(base).expect("base do controlador");
            let tela = TelaServida {
                url,
                html: &HtmlServido::de_teste(&html),
                codificacao,
            };
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
                html: &HtmlServido::de_teste(&simulado),
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
        assert_eq!(
            (
                forms_reais - anteriores.0,
                comparados - anteriores.1,
                sem_identidade - anteriores.2,
                posts_historicos - anteriores.3,
            ),
            (2, 2, 0, 1),
            "cobertura do cancelamento incompleta no card {nome}"
        );
    }
    println!(
        "documento cancelar: GET reais={forms_reais}; pares confrontados={comparados}; sem identidade/codificação={sem_identidade}; POST históricos={posts_historicos} (não provam efeito)"
    );
    assert!(forms_reais > 0, "nenhum GET de cancelamento registrado");
    assert!(comparados > 0, "nenhuma tela pôde ser confrontada");
}

/// A captura registra o redirecionamento e uma leitura posterior, mas não
/// preserva o motivo enviado. O efeito real não pode ser inferido daí.
/// O simulador usa motivo sintético apenas para verificar seu próprio SQLite.
#[test]
#[ignore = "acervo #273: exige ZIPs GO/UFRJ conferidos, executar em runner com cache público"]
fn replay_independente_redirecionamento_cancelamento_go_ufrj() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || PathBuf::from(std::env::var_os("HOME").expect("HOME")).join(".cache/ies"),
        PathBuf::from,
    );
    let cards = [
        "cassettes/go/documento-mover-cancelar-go--a152c0306b45.md",
        "cassettes/ufrj/documento-mover-cancelar-ufrj--de1301eb5634.md",
    ];
    let mut confrontos = 0usize;
    for nome in cards {
        let card = raiz.join(nome);
        let zip = archive_filename(&card).expect("origem no card");
        let zip = ZipDoAcervo::ler(&cache.join(zip)).expect("ZIP do acervo");
        let conferida = conferir_cassette_ate(&card, &zip, 120_000_000)
            .expect("SHA-1 do ZIP, SHA-256 do membro, schema e host");
        let respostas = gravacao::respostas(&conferida.json);
        let posts: Vec<_> = respostas
            .iter()
            .enumerate()
            .filter(|(_, interacao)| {
                interacao.operacao == "documento_cancelar" && interacao.metodo == "POST"
            })
            .collect();
        let [(indice, original)] = posts.as_slice() else {
            panic!("esperado um POST de cancelamento no card {nome}");
        };
        assert_eq!(original.status, 302, "resposta original de {nome}");
        let id_proc = gravacao::id_procedimento(original.interacao).expect("processo original");
        let id_doc = id_unico(original.interacao, "id_documento").expect("documento original");
        let enviado = original.interacao["response"]["url"]
            .as_str()
            .expect("URL do POST gravado");
        let (base, _) = enviado.split_once("/controlador.php").expect("controlador");
        let loc_original = gravacao::localizacao(original.interacao).expect("Location gravado");
        assert!(
            super::fluxo::localizacao_aceita(loc_original, enviado, &id_proc, &id_doc),
            "leitor IES deve reconhecer o 302 original de {}",
            conferida.instalacao
        );
        let depois = respostas.get(indice + 1).expect("leitura após o POST");
        assert_eq!(
            (depois.metodo, depois.operacao, depois.status),
            ("GET", "procedimento_visualizar", 200),
            "a captura deve registrar leitura após o 302 em {nome}"
        );

        // A mesma identidade entra no modelo: o motivo NÃO veio do POST gravado.
        let (banco, origem, alvo) = preparar_estado_anti_ies(base, &id_proc, &id_doc);
        let formulario = "hdnInfraTipoPagina=2&sbmSalvar=Salvar&txaMotivo=hipotese%20sintetica";
        let simulado = anti_ies::documento_cancelar::responder(
            "POST",
            &alvo,
            formulario,
            &banco,
            &origem,
            "replay-273",
        );
        assert_eq!(u64::from(simulado.estado.codigo_http()), original.status);
        let loc_simulado = simulado.localizacao.expect("Location sintético");
        assert!(
            super::fluxo::localizacao_aceita(&loc_simulado, enviado, &id_proc, &id_doc),
            "leitor IES deve reconhecer separadamente o 302 do anti-IES"
        );
        assert!(matches!(
            iesei::cancelamento_documento::consultar(
                &banco,
                &origem,
                "replay-273",
                &id_doc,
                &id_proc
            )
            .expect("SQLite do anti-IES"),
            Some(iesei::cancelamento_documento::Situacao::Cancelado { .. })
        ));
        let repetido = anti_ies::documento_cancelar::responder(
            "POST",
            &alvo,
            formulario,
            &banco,
            &origem,
            "replay-273",
        );
        assert_eq!(repetido.estado, anti_ies::Estado::MetodoNaoPermitido);
        confrontos += 1;
    }
    assert_eq!(confrontos, 2);
    println!(
        "documento cancelar: POST/302 original e simulado reconhecidos independentemente em {confrontos} instalações; estado posterior do SEI NÃO comprovado"
    );
}
