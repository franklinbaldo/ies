//! Base comum dos replays de conformidade do acervo público (Refs #57 e
//! #59): percorre os cards que gravaram alguma das operações pedidas,
//! confere cada cassette (SHA-1 do ZIP, SHA-256 do membro, schema, host)
//! e entrega o JSON ao leitor do replay; classifica as leituras (lido,
//! recusado, não aplicável) e agrega a tabela leitor × instalação.
//!
//! Só existe em `cfg(test)`. Os ZIPs ficam em `IES_ACERVO_CACHE` (padrão
//! `~/.cache/ies`); nenhuma requisição de rede.
pub(crate) mod gravacao;

use super::{ZipDoAcervo, conferir_cassette_ate};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Instalações públicas, na ordem dos relatórios.
pub(crate) const INSTALACOES: [&str; 7] = ["unb", "ufrj", "go", "pen1", "pen2", "pen3", "pen4"];

/// Classe de uma leitura de resposta gravada.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Resultado {
    Lido,
    Recusado(String),
    NaoAplicavel(String),
}

fn cache() -> PathBuf {
    if let Some(dir) = std::env::var_os("IES_ACERVO_CACHE") {
        return PathBuf::from(dir);
    }
    let casa = std::env::var_os("HOME").expect("HOME ou IES_ACERVO_CACHE");
    Path::new(&casa).join(".cache/ies")
}

fn cards() -> Vec<PathBuf> {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("cassettes");
    let mut cards = Vec::new();
    for instalacao in std::fs::read_dir(&raiz).expect("cassettes/") {
        let pasta = instalacao.expect("entrada").path();
        if !pasta.is_dir() {
            continue;
        }
        for card in std::fs::read_dir(&pasta).expect("pasta da instalação") {
            let card = card.expect("card").path();
            if card.extension().is_some_and(|e| e == "md") {
                cards.push(card);
            }
        }
    }
    cards.sort();
    cards
}

/// Chama `ler(nome, instalação, json)` para cada card conferido que
/// registra alguma das `operacoes`; devolve quantos foram lidos. Falha
/// (pânico) se um ZIP faltar ou um card não conferir.
pub(crate) fn percorrer(
    operacoes: &[&str],
    mut ler: impl FnMut(&str, &str, &serde_json::Value),
) -> usize {
    let cache = cache();
    let mut zips: HashMap<String, ZipDoAcervo> = HashMap::new();
    let mut lidos = 0usize;
    for card in cards() {
        let texto = std::fs::read_to_string(&card).expect("card");
        if !operacoes
            .iter()
            .any(|op| texto.contains(&format!("- `{op}`")))
        {
            continue;
        }
        let nome_zip = super::archive_filename(&card).expect("origem do card");
        let zip = zips.entry(nome_zip.clone()).or_insert_with(|| {
            ZipDoAcervo::ler(&cache.join(&nome_zip))
                .unwrap_or_else(|e| panic!("{nome_zip}: {e} (baixe-o para {})", cache.display()))
        });
        let conferida =
            conferir_cassette_ate(&card, zip, 120_000_000).unwrap_or_else(|e| panic!("{e}"));
        let nome = card.file_stem().expect("nome").to_string_lossy();
        ler(&nome, &conferida.instalacao, &conferida.json);
        lidos += 1;
    }
    lidos
}

/// Grava o relatório em `IES_ACERVO_SAIDA` ou o imprime.
pub(crate) fn publicar(texto: &str) {
    match std::env::var_os("IES_ACERVO_SAIDA") {
        Some(saida) => std::fs::write(saida, texto).expect("saída"),
        None => println!("{texto}"),
    }
}

#[derive(Default)]
struct Linha {
    total: usize,
    lido: usize,
    recusado: BTreeMap<String, usize>,
    nao_aplicavel: BTreeMap<String, usize>,
}

impl Linha {
    fn somar(&mut self, resultado: &Resultado) {
        self.total += 1;
        match resultado {
            Resultado::Lido => self.lido += 1,
            Resultado::Recusado(s) => *self.recusado.entry(s.clone()).or_default() += 1,
            Resultado::NaoAplicavel(m) => *self.nao_aplicavel.entry(m.clone()).or_default() += 1,
        }
    }
}

fn detalhe(mapa: &BTreeMap<String, usize>) -> String {
    let total: usize = mapa.values().sum();
    if total == 0 {
        return "0".into();
    }
    let partes: Vec<String> = mapa.iter().map(|(k, v)| format!("`{k}` {v}")).collect();
    format!("{total} ({})", partes.join("; "))
}

/// Tabela leitor × instalação (com a linha **todas**). Cada leitura é
/// `(ordem do leitor, nome do leitor, instalação, resultado)`.
pub(crate) fn tabela<'a>(
    s: &mut String,
    leituras: impl IntoIterator<Item = (usize, &'static str, &'a str, &'a Resultado)>,
) {
    let mut linhas: BTreeMap<(usize, &'static str, usize), Linha> = BTreeMap::new();
    for (ordem, leitor, instalacao, resultado) in leituras {
        let i = INSTALACOES
            .iter()
            .position(|n| *n == instalacao)
            .unwrap_or(INSTALACOES.len());
        linhas
            .entry((ordem, leitor, i))
            .or_default()
            .somar(resultado);
        linhas
            .entry((ordem, leitor, INSTALACOES.len() + 1))
            .or_default()
            .somar(resultado);
    }
    s.push_str("| Leitor | Instalação | Total | Lido | Recusado | Não aplicável |\n");
    s.push_str("|---|---|---:|---:|---|---|\n");
    for ((_, leitor, i), l) in &linhas {
        let instalacao = INSTALACOES.get(*i).copied().unwrap_or("**todas**");
        let _ = writeln!(
            s,
            "| {leitor} | {instalacao} | {} | {} | {} | {} |",
            l.total,
            l.lido,
            detalhe(&l.recusado),
            detalhe(&l.nao_aplicavel)
        );
    }
    s.push('\n');
}
