//! Replay de conformidade do acervo público (Refs #59): toda resposta
//! gravada de tela do processo, árvore, histórico, propriedades e
//! marcadores passa pelos **mesmos leitores** de `ies processo NUP`,
//! offline, e cada leitura é classificada como lida, recusada (subcódigo
//! tipado) ou não aplicável (identidade não estabelecida pela gravação).
//! Decisão em docs/decisoes/processo-obter/replay-do-acervo.md; relatório
//! em docs/validacao-replay/2026-11-04-processo-obter-acervo.md.
//!
//! Só existe em `cfg(test)`: nenhuma superfície pública na CLI. O teste
//! do acervo inteiro é `#[ignore]` (precisa dos ZIPs no cache local); o CI
//! roda só os testes sintéticos da classificação.
mod arvore;
mod gravacao;
mod leitores;
mod relatorio;
mod tests;

use crate::cassette::{ZipDoAcervo, conferir_cassette_ate};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Qual leitor de produção leu a resposta.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum Leitor {
    Tela,
    Arvore,
    Pastas,
    Situacao,
    Natureza,
    LinkDoHistorico,
    LinkDasPropriedades,
    LinkDosMarcadores,
    Andamentos,
    Propriedades,
    Marcadores,
}

impl Leitor {
    pub(super) fn nome(self) -> &'static str {
        match self {
            Self::Tela => "tela do processo",
            Self::Arvore => "árvore",
            Self::Pastas => "pastas",
            Self::Situacao => "situação (aberto_em)",
            Self::Natureza => "natureza",
            Self::LinkDoHistorico => "link do histórico",
            Self::LinkDasPropriedades => "link das propriedades",
            Self::LinkDosMarcadores => "link dos marcadores",
            Self::Andamentos => "andamentos",
            Self::Propriedades => "propriedades",
            Self::Marcadores => "marcadores",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Resultado {
    Lido,
    Recusado(&'static str),
    NaoAplicavel(&'static str),
}

/// Uma leitura de uma resposta gravada. Nada aqui guarda URL, cookie nem
/// `infra_hash`: só contagens e formas.
#[derive(Clone, Debug)]
pub(super) struct Registro {
    pub(super) leitor: Leitor,
    pub(super) instalacao: String,
    pub(super) card: String,
    pub(super) passo: usize,
    pub(super) resultado: Resultado,
    pub(super) contagens: Vec<(&'static str, u64)>,
    pub(super) formas: Vec<String>,
}

impl Registro {
    pub(super) fn contagem(&self, nome: &str) -> u64 {
        self.contagens
            .iter()
            .find(|(n, _)| *n == nome)
            .map_or(0, |(_, v)| *v)
    }
}

/// Operações cujas respostas o replay lê.
const OPERACOES: [&str; 6] = [
    "procedimento_trabalhar",
    "procedimento_visualizar",
    "procedimento_consultar_historico",
    "procedimento_alterar",
    "procedimento_consultar",
    "andamento_marcador_gerenciar",
];

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

#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE (padrão ~/.cache/ies); sem rede"]
fn replay_do_acervo_inteiro() {
    let cache = cache();
    let mut zips: HashMap<String, ZipDoAcervo> = HashMap::new();
    let mut registros = Vec::new();
    let mut lidos = 0usize;
    for card in cards() {
        let texto = std::fs::read_to_string(&card).expect("card");
        if !OPERACOES
            .iter()
            .any(|op| texto.contains(&format!("- `{op}`")))
        {
            continue;
        }
        let nome_zip = crate::cassette::archive_filename(&card).expect("origem do card");
        let zip = zips.entry(nome_zip.clone()).or_insert_with(|| {
            ZipDoAcervo::ler(&cache.join(&nome_zip))
                .unwrap_or_else(|e| panic!("{nome_zip}: {e} (baixe-o para {})", cache.display()))
        });
        let conferida =
            conferir_cassette_ate(&card, zip, 120_000_000).unwrap_or_else(|e| panic!("{e}"));
        let nome = card
            .file_stem()
            .expect("nome")
            .to_string_lossy()
            .into_owned();
        registros.extend(leitores::replay(
            &nome,
            &conferida.instalacao,
            &conferida.json,
        ));
        lidos += 1;
    }
    let texto = relatorio::markdown(lidos, &registros);
    match std::env::var_os("IES_ACERVO_SAIDA") {
        Some(saida) => std::fs::write(saida, texto).expect("saída"),
        None => println!("{texto}"),
    }
}
