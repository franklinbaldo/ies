//! Replay manual das sete cassettes publicas de Retorno Programado (#253).
//! So usa GETs registrados e o mesmo extrator estrutural da CLI.
//! Os ZIPs e membros JSON sao conferidos por SHA-1/SHA-256 antes da leitura.
use super::extrair_retornos;
use crate::cassette::acervo::gravacao;
use crate::cassette::{ZipDoAcervo, conferir_cassette_ate, html_da_resposta};
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
    let Ok(dom) = tl::parse(html, tl::ParserOptions::default()) else {
        println!("{instalacao}: calendario passo={passo} html_nao_estruturado");
        return;
    };
    let parser = dom.parser();
    let mut links = 0usize;
    let mut links_de_mes = 0usize;
    let mut hrefs_mensais = std::collections::HashSet::new();
    let mut datas_mensais = std::collections::HashSet::new();
    let mut links_de_dia = 0usize;
    let mut acoes_js = 0usize;
    let mut hoje = 0usize;
    for no in dom.query_selector("a").into_iter().flatten() {
        if let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) {
            links += 1;
            if let Ok(Some(href)) = crate::html_atributos::valor_do_atributo(tag, "href") {
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
        "{instalacao}: calendario passo={passo} ancoras={links} links_mes={links_de_mes} urls_mensais_distintas={} datas_mensais_distintas={} links_dia={links_de_dia} js_retornos={acoes_js} dia_hoje={hoje}",
        hrefs_mensais.len(),
        datas_mensais.len()
    );
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
        for resposta in gravacao::respostas(&conferida.json) {
            if resposta.operacao == "retorno_programado_listar"
                && resposta.metodo == "GET"
                && resposta.status == 200
                && gravacao::parametro(resposta.item, "data_inicial").is_empty()
            {
                if let Ok(html) = html_da_resposta(resposta.item) {
                    diagnosticar_calendario(&html, instalacao, resposta.passo);
                }
                continue;
            }
            if resposta.operacao != "retorno_programado_listar"
                || resposta.metodo != "GET"
                || resposta.status != 200
                || gravacao::parametro(resposta.item, "data_inicial").is_empty()
            {
                continue;
            }
            meses += 1;
            let Ok(html) = html_da_resposta(resposta.item) else {
                recusados += 1;
                println!("{instalacao}: passo {} sem HTML legivel", resposta.passo);
                continue;
            };
            match extrair_retornos(&html) {
                Ok(itens) => {
                    lidos += 1;
                    println!(
                        "{instalacao}: mes gravado lido ({} retorno(s))",
                        itens.len()
                    );
                }
                Err(erro) => {
                    recusados += 1;
                    println!(
                        "{instalacao}: passo {} recusado ({})",
                        resposta.passo,
                        erro.subcodigo()
                    );
                }
            }
        }
        println!("{instalacao}: mensais={meses} lidos={lidos} recusados={recusados}");
        // As sete capturas imutaveis continham exatamente dois GETs
        // mensais cada: uma regressao nao pode passar lendo so uma pagina.
        assert_eq!(meses, 2, "{instalacao}: mudou a cobertura mensal");
        assert_eq!(lidos, 2, "{instalacao}: perdeu uma listagem mensal");
        assert_eq!(recusados, 0, "{instalacao}: listagem mensal recusada");
        total_meses += meses;
        total_lidos += lidos;
        total_recusados += recusados;
    }
    println!("TOTAL: mensais={total_meses} lidos={total_lidos} recusados={total_recusados}");
    assert_eq!(total_meses, 14, "acervo nao cobriu os 14 meses");
    assert_eq!(total_lidos, 14, "acervo nao leu os 14 meses");
    assert_eq!(
        total_recusados, 0,
        "replay mensal nao demonstrou paridade em todas as instalacoes"
    );
}
