//! Replay manual das sete cassettes publicas de Retorno Programado (#253).
//! So usa GETs registrados e o mesmo extrator estrutural da CLI.
//! Os ZIPs e membros JSON sao conferidos por SHA-1/SHA-256 antes da leitura.
use super::extrair_retornos;
use crate::cassette::acervo::gravacao;
use crate::cassette::{ZipDoAcervo, conferir_cassette_ate, html_da_resposta};
use crate::tela_html::HtmlServido;
use std::path::{Path, PathBuf};

const CARDS: [(&str, &str); 7] = [
    ("unb", "61ae9a77b5df"),
    ("ufrj", "3a6152f7d1bc"),
    ("go", "645ae924663b"),
    ("pen1", "8163e3fcb3db"),
    ("pen2", "8bbd8541acd6"),
    ("pen3", "2d16437b3e19"),
    ("pen4", "87e2e8e70f44"),
];

fn pasta_do_acervo() -> PathBuf {
    std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || PathBuf::from(std::env::var_os("HOME").expect("HOME")).join(".cache/ies"),
        PathBuf::from,
    )
}

/// Conta somente elementos e nomes de parametros de links gravados.
/// Nunca publica href, corpo HTML, `infra_hash`, URL assinada ou cookie.
fn diagnosticar_calendario(html: &str, instalacao: &str, passo: usize) {
    let servido = HtmlServido::de_teste(html);
    let dom = crate::tela_html::interpretar(&servido);
    let parser = dom.parser();
    let mut links = 0usize;
    let mut links_de_mes = 0usize;
    let mut hrefs_mensais = std::collections::HashSet::new();
    let mut datas_mensais = std::collections::HashSet::new();
    let mut tamanhos_hash = std::collections::HashSet::new();
    let mut hrefs_operacao = 0usize;
    let mut absolutos_https = 0usize;
    let mut links_de_dia = 0usize;
    let mut acoes_js = 0usize;
    let mut hoje = 0usize;
    for no in dom.query_selector("a").into_iter().flatten() {
        if let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) {
            links += 1;
            if let Ok(Some(href)) = crate::html_atributos::valor_do_atributo(tag, "href") {
                if href.contains("retorno_programado_listar") {
                    hrefs_operacao += 1;
                    absolutos_https += usize::from(href.starts_with("https://"));
                    if let Some(valor) = href.split("infra_hash=").nth(1) {
                        tamanhos_hash.insert(valor.split('&').next().unwrap_or("").len());
                    }
                }
                if href.contains("data_inicial=") && href.contains("data_final=") {
                    links_de_mes += 1;
                    hrefs_mensais.insert(href.clone());
                    if let Some(data) = href
                        .split("data_inicial=")
                        .nth(1)
                        .and_then(|parte| parte.split('&').next())
                    {
                        // Parametro de data isolado, nunca o restante da URL assinada.
                        datas_mensais.insert(data.to_owned());
                    }
                }
                if href.contains("data_atual=") {
                    links_de_dia += 1;
                }
            }
            if let Ok(Some(acao)) = crate::html_atributos::valor_do_atributo(tag, "onclick")
                && acao.contains("retorno_programado")
            {
                acoes_js += 1;
            }
        }
    }
    for no in dom.query_selector("td").into_iter().flatten() {
        if let Some(tag) = no.get(parser).and_then(tl::Node::as_tag)
            && let Ok(Some(classe)) = crate::html_atributos::valor_do_atributo(tag, "class")
            && classe.split_whitespace().any(|classe| classe == "diaHoje")
        {
            hoje += 1;
        }
    }
    println!(
        "{instalacao}: calendario passo={passo} ancoras={links} links_mes={links_de_mes} urls_mensais_distintas={} datas_mensais_distintas={} links_dia={links_de_dia} js_retornos={acoes_js} dia_hoje={hoje} hrefs_operacao={hrefs_operacao} absolutos_https={absolutos_https} tamanhos_hash={tamanhos_hash:?}",
        hrefs_mensais.len(),
        datas_mensais.len()
    );
}

/// Exercita o parser da CLI nas respostas originais, sem rede ou logs de URLs.
fn conferir_calendario_real(
    html: &str,
    resposta: &gravacao::Resposta<'_>,
    origem: &crate::instalacao::Instalacao,
    nome: &str,
) -> (bool, bool) {
    let url_servida = resposta.interacao["response"]["url"]
        .as_str()
        .expect("URL original da resposta gravada");
    crate::unidade_retornos_web::conferir_calendario_gravado(html, url_servida, origem)
        .unwrap_or_else(|erro| {
            panic!(
                "{nome}: calendario passo {} recusado: {erro:?}",
                resposta.passo
            )
        })
}

/// Somente GET do calendario: nao reexecuta a operacao gravada.
fn verificar_resposta_de_calendario(
    resposta: &gravacao::Resposta<'_>,
    nome: &str,
    origem: &crate::instalacao::Instalacao,
) -> Option<(bool, bool)> {
    if resposta.operacao != "retorno_programado_listar"
        || resposta.metodo != "GET"
        || resposta.status != 200
        || !gravacao::parametro(resposta.interacao, "data_inicial").is_empty()
    {
        return None;
    }
    let html = html_da_resposta(resposta.interacao).expect("HTML do calendario gravado");
    diagnosticar_calendario(&html, nome, resposta.passo);
    Some(conferir_calendario_real(&html, resposta, origem, nome))
}

/// Sem acesso ao SEI; usa exclusivamente o acervo original em cache.
#[test]
#[ignore = "replay manual #253: requer ZIPs do acervo verificados em IES_ACERVO_CACHE"]
fn replay_retornos_no_acervo() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("cassettes");
    let cache = pasta_do_acervo();
    let mut total_meses = 0usize;
    let mut total_lidos = 0usize;
    let mut total_recusados = 0usize;
    let mut total_calendarios = 0usize;
    for (instalacao, revisao) in CARDS {
        let card = raiz.join(instalacao).join(format!(
            "unidade-listar-retornos-{instalacao}--{revisao}.md"
        ));
        let arquivo = crate::cassette::archive_filename(&card).expect("card OKF");
        let zip =
            ZipDoAcervo::ler(&cache.join(&arquivo)).unwrap_or_else(|e| panic!("{arquivo}: {e}"));
        let conferida = conferir_cassette_ate(&card, &zip, 120_000_000)
            .expect("integridade SHA-1/SHA-256, schema e instalacao");
        assert_eq!(conferida.instalacao, instalacao);
        let mut meses = 0;
        let mut lidos = 0;
        let mut recusados = 0;
        let mut calendarios = 0usize;
        let mut calendarios_com_hoje = 0usize;
        let mut calendarios_com_vizinho = 0usize;
        let origem =
            gravacao::instalacao(&conferida.json).expect("instalacao verificada no acervo");
        for resposta in gravacao::respostas(&conferida.json) {
            if let Some((hoje, vizinho)) =
                verificar_resposta_de_calendario(&resposta, instalacao, &origem)
            {
                calendarios += 1;
                calendarios_com_hoje += usize::from(hoje);
                calendarios_com_vizinho += usize::from(vizinho);
                continue;
            }
            if resposta.operacao != "retorno_programado_listar"
                || resposta.metodo != "GET"
                || resposta.status != 200
                || gravacao::parametro(resposta.interacao, "data_inicial").is_empty()
            {
                continue;
            }
            meses += 1;
            let Ok(html) = html_da_resposta(resposta.interacao) else {
                recusados += 1;
                println!("{instalacao}: passo {} sem HTML legivel", resposta.passo);
                continue;
            };
            match extrair_retornos(&HtmlServido::de_teste(&html)) {
                Ok(itens) => {
                    lidos += 1;
                    println!(
                        "{instalacao}: mes gravado lido ({} retorno(s))",
                        itens.len()
                    );
                }
                Err(erro) => {
                    recusados += 1;
                    println!("{instalacao}: passo {} recusado ({erro:?})", resposta.passo);
                }
            }
        }
        println!(
            "{instalacao}: mensais={meses} lidos={lidos} recusados={recusados} calendarios={calendarios} hoje={calendarios_com_hoje} vizinho={calendarios_com_vizinho}"
        );
        assert!(
            calendarios >= 3,
            "{instalacao}: falta replay das telas do calendario"
        );
        assert!(
            calendarios_com_hoje >= 1,
            "{instalacao}: diaHoje real nao foi reconhecido"
        );
        assert!(
            calendarios_com_vizinho >= 1,
            "{instalacao}: nenhuma navegacao vizinha observada"
        );
        total_calendarios += calendarios;
        // As sete capturas imutaveis continham exatamente dois GETs
        // mensais cada: uma regressao nao pode passar lendo so uma pagina.
        assert_eq!(meses, 2, "{instalacao}: mudou a cobertura mensal");
        assert_eq!(lidos, 2, "{instalacao}: perdeu uma listagem mensal");
        assert_eq!(recusados, 0, "{instalacao}: listagem mensal recusada");
        total_meses += meses;
        total_lidos += lidos;
        total_recusados += recusados;
    }
    println!(
        "TOTAL: mensais={total_meses} lidos={total_lidos} recusados={total_recusados} calendarios={total_calendarios}"
    );
    assert!(
        total_calendarios >= 21,
        "replay nao cobriu os calendarios reais das sete instalacoes"
    );
    assert_eq!(total_meses, 14, "acervo nao cobriu os 14 meses");
    assert_eq!(total_lidos, 14, "acervo nao leu os 14 meses");
    assert_eq!(
        total_recusados, 0,
        "replay mensal nao demonstrou paridade em todas as instalacoes"
    );
}
