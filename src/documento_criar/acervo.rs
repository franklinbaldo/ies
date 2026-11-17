//! Replay offline do cadastro de documento no acervo público de Goiás (#246).
//! ZIP, membro, origem e integridade são conferidos antes da classificação.
//! Só imprime contagens: nunca HTML, URL assinada ou cookie.
use super::{conferir_cadastro, tipos_oferecidos};
use crate::cassette::acervo::gravacao;
use crate::cassette::{ZipDoAcervo, conferir_cassette_ate, html_da_resposta};
use crate::tela_html::HtmlServido;
use std::path::Path;

const CARDS: [&str; 2] = [
    "documento-incluir-go--fa4845e16ba9.md",
    "documento-escritas-go--16f83179ecaa.md",
];

fn tipo_do_cadastro(html: &str) -> Option<String> {
    let servido = HtmlServido::de_teste(html);
    let dom = crate::tela_html::arvore(&servido);
    let rotulos = dom.com_id("lblSerieTitulo").collect::<Vec<_>>();
    let [rotulo] = rotulos.as_slice() else {
        return None;
    };
    if !rotulo.nome().eq_ignore_ascii_case("label") {
        return None;
    }
    let tipo = rotulo
        .texto()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    (!tipo.is_empty()).then_some(tipo)
}

#[test]
#[ignore = "wip #246: requer ZIP público conferido em IES_ACERVO_CACHE, sem rede"]
fn replay_documento_criar_go() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || {
            let home = std::env::var_os("HOME").expect("HOME ou IES_ACERVO_CACHE");
            Path::new(&home).join(".cache/ies")
        },
        std::path::PathBuf::from,
    );
    let zip = ZipDoAcervo::ler(&cache.join("todos-sei-cassettes-go.zip"))
        .expect("ZIP de Goiás não está no cache do acervo");
    let mut lidas = 0usize;
    let mut recusadas = 0usize;
    let mut nao_aplicaveis = 0usize;
    let mut respostas_alvo = 0usize;
    for card in CARDS {
        let caminho = raiz.join("cassettes/go").join(card);
        let cassette = conferir_cassette_ate(&caminho, &zip, 120_000_000)
            .expect("integridade ou origem do card inválida");
        assert_eq!(cassette.instalacao, "go");
        for resposta in gravacao::respostas(&cassette.json) {
            if !matches!(
                resposta.operacao,
                "documento_escolher_tipo" | "documento_gerar"
            ) {
                continue;
            }
            respostas_alvo += 1;
            if resposta.status != 200 {
                nao_aplicaveis += 1;
                continue;
            }
            let Ok(html) = html_da_resposta(resposta.interacao) else {
                nao_aplicaveis += 1;
                continue;
            };
            let resultado = match resposta.operacao {
                "documento_escolher_tipo" => {
                    tipos_oferecidos(&HtmlServido::de_teste(&html)).is_ok()
                }
                "documento_gerar" => tipo_do_cadastro(&html).is_some_and(|tipo| {
                    conferir_cadastro(&HtmlServido::de_teste(&html), &tipo, None, None).is_ok()
                }),
                _ => unreachable!(),
            };
            if resultado {
                lidas += 1;
            } else {
                recusadas += 1;
            }
        }
    }
    assert!(
        respostas_alvo > 0,
        "cards sem respostas de inclusão de documento"
    );
    assert!(lidas + recusadas > 0, "nenhum HTML candidato no replay");
    assert_eq!(respostas_alvo, lidas + recusadas + nao_aplicaveis);
    // Referência observada no replay GO imutável (run 42118476321):
    // sem estes números um leitor quebrado poderia recusar tudo e passar.
    // As 5 recusas antigas eram listas de tipos com "Nome&nbsp;(Formulário)",
    // lidas desde a correção do ensaio live de 10/11/2026 (PR #635).
    assert_eq!(respostas_alvo, 15, "mudou o conjunto de respostas-alvo");
    assert_eq!(lidas, 8, "regressão na leitura de inclusão/cadastro");
    assert_eq!(recusadas, 0, "mudou a classificação de recusas");
    assert_eq!(nao_aplicaveis, 7, "mudou a classe não aplicável");
    println!(
        "documento.criar GO: {respostas_alvo} respostas; {lidas} lidas, {recusadas} recusadas, {nao_aplicaveis} não aplicáveis"
    );
}
