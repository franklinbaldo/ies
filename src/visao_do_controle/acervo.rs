//! Replay de conformidade do Controle de Processos no acervo público
//! (Refs #57): toda resposta gravada de `procedimento_controlar` (GET e
//! POST) passa pelos **mesmos leitores** de `ies unidade
//! listar-processos|recebidos|gerados`, offline: transporte, visão,
//! unidade, lista R e listas da categoria, plano de página de cada tabela,
//! visão D com colunas e a posição da D, e as linhas que a colheita lê.
//! Cada leitura é lida, recusada (erro tipado) ou não aplicável
//! (motivo). Decisão em docs/decisoes/unidade-listar-processos/replay-do-acervo.md;
//! relatório em docs/validacao-replay/2026-11-04-unidade-listar-processos-acervo.md.
//!
//! Só existe em `cfg(test)`. O teste do acervo inteiro é `#[ignore]`
//! (precisa dos ZIPs no cache local); o CI roda os testes sintéticos.
mod continuidade;
mod leitores;
mod paginacao;
mod relatorio;
#[cfg(test)]
mod tests;

pub(super) use crate::cassette::acervo::Resultado;
use crate::cassette::acervo::{percorrer, publicar};

/// Qual leitor de produção leu a resposta.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum Leitor {
    Transporte,
    Visao,
    Unidade,
    UnidadeSolicitada,
    Resumida,
    Recebidos,
    Gerados,
    PaginaRecebidos,
    PaginaGerados,
    Detalhada,
    PaginaDetalhada,
    LinhasComLink,
    TelaDeLogin,
}

impl Leitor {
    pub(super) fn nome(self) -> &'static str {
        match self {
            Self::Transporte => "transporte (GET/POST)",
            Self::Visao => "visão (hdnTipoVisualizacao)",
            Self::Unidade => "unidade (lnkInfraUnidade)",
            Self::UnidadeSolicitada => "--unidade da tela anterior",
            Self::Resumida => "lista R (juntar)",
            Self::Recebidos => "recebidos",
            Self::Gerados => "gerados",
            Self::PaginaRecebidos => "página de recebidos",
            Self::PaginaGerados => "página de gerados",
            Self::Detalhada => "lista D (colunas)",
            Self::PaginaDetalhada => "página da D",
            Self::LinhasComLink => "linhas com link (colheita)",
            Self::TelaDeLogin => "tela de login (sip/login.php)",
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
    pub(super) fn contagem(&self, nome: &str) -> Option<u64> {
        self.contagens
            .iter()
            .find(|(n, _)| *n == nome)
            .map(|(_, v)| *v)
    }
}

/// Tabela lida numa página: a base da continuidade entre páginas.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Lista {
    Recebidos,
    Gerados,
    Detalhada,
}

/// O que uma página lida publicou de uma lista (ids opacos, só comparados).
#[derive(Clone, Debug)]
pub(super) struct PaginaLida {
    pub(super) card: String,
    pub(super) instalacao: String,
    pub(super) passo: usize,
    pub(super) lista: Lista,
    pub(super) unidade: String,
    /// `hdn{T}HashCriterios` servido; só comparado, nunca publicado.
    pub(super) criterio: Option<String>,
    pub(super) total: usize,
    pub(super) faixa: Option<(usize, usize)>,
    pub(super) ids: Vec<String>,
}

/// Leituras e páginas de uma cassette.
#[derive(Default)]
pub(super) struct Replay {
    pub(super) registros: Vec<Registro>,
    pub(super) paginas: Vec<PaginaLida>,
}

#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE (padrão ~/.cache/ies); sem rede"]
fn replay_do_controle_inteiro() {
    let mut todos = Replay::default();
    let lidos = percorrer(&["procedimento_controlar"], |nome, instalacao, json| {
        let lido = leitores::replay(nome, instalacao, json);
        todos.registros.extend(lido.registros);
        todos.paginas.extend(lido.paginas);
    });
    publicar(&relatorio::markdown(lidos, &todos));
}
