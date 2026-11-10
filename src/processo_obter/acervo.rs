//! Replay de conformidade do acervo público (Refs #59): toda resposta
//! gravada de tela do processo, árvore, histórico, propriedades e
//! marcadores passa pelos **mesmos leitores** de `ies processo NUP`,
//! offline, e cada leitura é classificada como lida, recusada (erro
//! tipado) ou não aplicável (identidade não estabelecida pela gravação).
//! Decisão em docs/decisoes/processo-obter/replay-do-acervo.md; relatório
//! em docs/validacao-replay/2026-11-04-processo-obter-acervo.md.
//!
//! Só existe em `cfg(test)`: nenhuma superfície pública na CLI. O teste
//! do acervo inteiro é `#[ignore]` (precisa dos ZIPs no cache local); o CI
//! roda só os testes sintéticos da classificação.
mod arvore;
mod leitores;
mod relatorio;
mod tests;

pub(super) use crate::cassette::acervo::Resultado;
use crate::cassette::acervo::{percorrer, publicar};

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

#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE (padrão ~/.cache/ies); sem rede"]
fn replay_do_acervo_inteiro() {
    let mut registros = Vec::new();
    let lidos = percorrer(&OPERACOES, |nome, instalacao, json| {
        registros.extend(leitores::replay(nome, instalacao, json));
    });
    publicar(&relatorio::markdown(lidos, &registros));
}
