//! Replay manual das sete cassettes publicas de Retorno Programado (#253).
//! So usa GETs registrados e o mesmo extrator estrutural da CLI.
//! O ZIP e seu membro JSON sao verificados antes da leitura (SHA-1/SHA-256).
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
        || PathBuf::from(std::env::var_os("HOME").expect("HOME"))
            .join(".cache/ies"),
        PathBuf::from,
    )
}

/// O replay exige os ZIPs originais em cache, tal como os outros testes
/// acervo. Nunca faz rede nem imprime URL assinada ou cookie.
#[test]
#[ignore = "replay manual #253: requer ZIPs do acervo verificados em IES_ACERVO_CACHE"]
fn replay_retornos_no_acervo() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("cassettes");
    let cache = pasta_do_acervo();
    let mut total_meses = 0usize;
    let mut total_lidos = 0usize;
    let mut total_recusados = 0usize;
    for (instalacao, revisao) in CARDS {
        let card = raiz
            .join(instalacao)
            .join(format!("unidade-listar-retornos-{instalacao}--{revisao}.md"));
        let arquivo = crate::cassette::archive_filename(&card).expect("card OKF");
        let zip = ZipDoAcervo::ler(&cache.join(&arquivo))
            .unwrap_or_else(|e| panic!("{arquivo}: {e}"));
        let conferida = conferir_cassette_ate(&card, &zip, 120_000_000)
            .expect("integridade SHA-1/SHA-256, schema e instalacao");
        assert_eq!(conferida.instalacao, instalacao);
        let mut meses = 0;
        let mut lidos = 0;
        let mut recusados = 0;
        for resposta in gravacao::respostas(&conferida.json) {
            if resposta.operacao != "retorno_programado_listar"
                || resposta.metodo != "GET"
                || resposta.status != 200
                || gravacao::parametro(resposta.item, "data_inicial").is_empty()
            {
                continue;
            }
            meses += 1;
            match html_da_resposta(resposta.item) {
                Ok(html) => match extrair_retornos(&html) {
                    Ok(itens) => {
                        lidos += 1;
                        println!("{instalacao}: mes gravado lido ({} retorno(s))", itens.len());
                    }
                    Err(erro) => {
                        recusados += 1;
                        println!(
                            "{instalacao}: passo {} recusado ({})",
                            resposta.passo,
                            erro.subcodigo()
                        );
                    }
                },
                Err(_) => {
                    recusados += 1;
                    println!("{instalacao}: passo {} sem HTML legivel", resposta.passo);
                }
            }
        }
        println!("{instalacao}: mensais={meses} lidos={lidos} recusados={recusados}");
        assert!(meses > 0, "{instalacao}: cassette sem listagem mensal GET");
        total_meses += meses;
        total_lidos += lidos;
        total_recusados += recusados;
    }
    println!("TOTAL: mensais={total_meses} lidos={total_lidos} recusados={total_recusados}");
    assert_eq!(
        total_recusados, 0,
        "replay mensal nao demonstrou paridade em todas as instalacoes"
    );
}
