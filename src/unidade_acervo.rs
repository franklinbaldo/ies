//! Primeiro núcleo de leitura de acervo histórico (#878).
//! Forma derivada do Todos estatisticas.py; confronto original ainda pendente.
use crate::legenda::{ErroLegenda, Legenda, ler_legenda};
use crate::tela_html::{Elemento, HtmlServido, arvore};

#[derive(Debug, PartialEq, Eq)]
struct ProcessoDoAcervo {
    processo: String,
    tipo: String,
    orgao: String,
    unidade: String,
    mes: String,
    ano: String,
}

#[derive(Debug, PartialEq, Eq)]
struct Pagina {
    legenda: Legenda,
    processos: Vec<ProcessoDoAcervo>,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
enum ErroAcervo {
    TelaDesconhecida,
    Legenda(#[from] ErroLegenda),
    FaixaIncompleta,
}

fn texto(no: Elemento<'_>) -> String {
    no.texto().split_whitespace().collect::<Vec<_>>().join(" ")
}

fn detalhe(html: &HtmlServido) -> Result<Pagina, ErroAcervo> {
    let dom = arvore(html);
    let tabelas: Vec<_> = dom
        .elementos("table")
        .filter(|t| {
            t.elementos("caption")
                .any(|c| texto(c).starts_with("Lista de Registros de Detalhamento"))
        })
        .collect();
    let [tabela] = tabelas.as_slice() else {
        return Err(ErroAcervo::TelaDesconhecida);
    };
    let legendas: Vec<_> = tabela.elementos("caption").collect();
    let [legenda] = legendas.as_slice() else {
        return Err(ErroAcervo::TelaDesconhecida);
    };
    let legenda = ler_legenda(&texto(*legenda))?;
    let colunas: Vec<_> = tabela.elementos("th").map(texto).collect();
    let indice = |nome| {
        let mut posicoes = colunas
            .iter()
            .enumerate()
            .filter(|(_, valor)| valor.as_str() == nome);
        let (i, _) = posicoes.next().ok_or(ErroAcervo::TelaDesconhecida)?;
        if posicoes.next().is_some() {
            return Err(ErroAcervo::TelaDesconhecida);
        }
        Ok(i)
    };
    let [orgao, unidade, processo, tipo, mes, ano] = [
        indice("Órgão")?,
        indice("Unidade")?,
        indice("Processo")?,
        indice("Tipo")?,
        indice("Mês")?,
        indice("Ano")?,
    ];
    let mut processos = Vec::new();
    for linha in tabela.elementos("tr") {
        let celulas: Vec<_> = linha.filhos().filter(|c| c.nome() == "td").collect();
        if celulas.is_empty() {
            continue;
        }
        if celulas.len() != colunas.len() {
            return Err(ErroAcervo::TelaDesconhecida);
        }
        let nup = texto(celulas[processo]);
        if nup.is_empty() {
            return Err(ErroAcervo::TelaDesconhecida);
        }
        processos.push(ProcessoDoAcervo {
            processo: nup,
            tipo: texto(celulas[tipo]),
            orgao: texto(celulas[orgao]),
            unidade: texto(celulas[unidade]),
            mes: texto(celulas[mes]),
            ano: texto(celulas[ano]),
        });
    }
    let linhas = processos.len();
    let coerente = match &legenda.faixa {
        Some(f) => {
            f.inicio > 0
                && f.inicio <= f.fim
                && f.fim <= legenda.total
                && f.fim - f.inicio + 1 == linhas
        }
        None => legenda.total == linhas,
    };
    if !coerente {
        return Err(ErroAcervo::FaixaIncompleta);
    }
    Ok(Pagina { legenda, processos })
}

#[test]
fn detalhe_preserva_identidade_e_faixa() {
    let tela = HtmlServido::de_teste(
        "<table><caption>Lista de Registros de Detalhamento (3 registros - 1 a 2):</caption>\
         <tr><th></th><th>Órgão</th><th>Unidade</th><th>Processo</th><th>Tipo</th><th>Mês</th><th>Ano</th></tr>\
         <tr><td></td><td>O</td><td>U</td><td>0001.000001/2026-11</td><td>T</td><td>10</td><td>2024</td></tr>\
         <tr><td></td><td>O</td><td>U</td><td>0001.000001/2026-11</td><td>T</td><td>10</td><td>2024</td></tr></table>",
    );
    let lida = detalhe(&tela).unwrap();
    assert_eq!(lida.legenda.total, 3);
    assert_eq!(lida.processos.len(), 2);
    assert_eq!(lida.processos[0].processo, "0001.000001/2026-11");
    assert_eq!(lida.processos[0].ano, "2024");
    assert_eq!(lida.processos[0], lida.processos[1]);
    let faixa_errada = tela.as_str().replace("1 a 2", "1 a 3");
    assert_eq!(
        detalhe(&HtmlServido::de_teste(&faixa_errada)),
        Err(ErroAcervo::FaixaIncompleta)
    );
}

#[test]
fn tela_ausente_nao_e_zero() {
    assert_eq!(
        detalhe(&HtmlServido::de_teste("<p>Outra tela</p>")),
        Err(ErroAcervo::TelaDesconhecida)
    );
}

/// Somente diagnóstico do leitor de detalhe, não execução da CLI nem prova
/// de que todas as respostas pertencem à mesma fotografia do acervo.
#[test]
#[ignore = "acervo original: exige ZIPs verificados no cache"]
fn acervo_detalhe_original() {
    use crate::cassette::acervo::gravacao;
    use crate::cassette::{ZipDoAcervo, conferir_cassette_ate, html_da_resposta};
    use std::path::{Path, PathBuf};

    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("cassettes");
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || PathBuf::from(std::env::var_os("HOME").expect("HOME")).join(".cache/ies"),
        PathBuf::from,
    );
    for (instalacao, revisao) in [
        ("unb", "74b641c4a8a1"),
        ("pen1", "5c557e93a6ce"),
        ("pen2", "c64c576f9a9a"),
    ] {
        let card = raiz.join(instalacao).join(format!(
            "unidade-estatisticas-acervo-{instalacao}--{revisao}.md"
        ));
        let arquivo = crate::cassette::archive_filename(&card).expect("card OKF");
        let zip = ZipDoAcervo::ler(&cache.join(&arquivo)).expect("ZIP original");
        let conferida =
            conferir_cassette_ate(&card, &zip, 120_000_000).expect("integridade do original");
        assert_eq!(conferida.instalacao, instalacao);
        let mut lidos = 0;
        let mut recusados = 0;
        for resposta in gravacao::respostas(&conferida.json) {
            if resposta.operacao != "estatisticas_detalhar_unidade" || resposta.status != 200 {
                continue;
            }
            let html = html_da_resposta(resposta.interacao).expect("HTML original");
            match detalhe(&HtmlServido::de_teste(&html)) {
                Ok(p) => {
                    lidos += 1;
                    println!(
                        "{instalacao}: detalhe passo={} total={} linhas={} faixa={:?}",
                        resposta.passo,
                        p.legenda.total,
                        p.processos.len(),
                        p.legenda.faixa
                    );
                }
                Err(_) => {
                    recusados += 1;
                    println!("{instalacao}: detalhe passo={} recusado", resposta.passo);
                }
            }
        }
        assert!(lidos > 0, "{instalacao}: nenhum detalhe reconhecido");
        assert_eq!(recusados, 0, "{instalacao}: detalhes recusados");
    }
}
