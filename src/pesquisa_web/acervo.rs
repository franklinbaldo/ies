//! Replay de conformidade do acervo público (Refs #58, #61): toda resposta
//! gravada da tela de Pesquisa (`protocolo_pesquisar`) e do Controle que a
//! precede passa pelas **mesmas peças** de `ies processo buscar`, offline:
//! link do menu e unidade do Controle, derivação do POST pela tela servida,
//! envio interpretado (302 conferido, lista, "sem resultado", validação) e o
//! plano da página 2; e, pelas peças de `ies documento buscar`, o POST em
//! documentos, o envio, o plano e as páginas seguintes. Cada leitura é lida, recusada (erro tipado) ou
//! não aplicável (a gravação não estabelece o que o leitor precisa).
//! Decisão em docs/decisoes/processo-buscar/replay-do-acervo.md; relatório
//! em docs/validacao-replay/2026-11-04-processo-buscar-acervo.md.
//!
//! Só existe em `cfg(test)`. O teste do acervo inteiro é `#[ignore]`
//! (precisa dos ZIPs no cache local); o CI roda os testes sintéticos.
mod leitores;
mod rapida;
mod relatorio;
mod tests;

pub(super) use crate::cassette::acervo::Resultado;
use crate::cassette::acervo::{percorrer, publicar};

/// Qual peça de produção leu a resposta.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum Leitor {
    Controle,
    Tela,
    Envio,
    PlanoDaPagina,
    PostDaTelaDeResultado,
    TelaEmDocumentos,
    EnvioEmDocumentos,
    PlanoEmDocumentos,
    PaginaEmDocumentos,
    PesquisaRapida,
}

impl Leitor {
    pub(super) fn nome(self) -> &'static str {
        match self {
            Self::Controle => "Controle (unidade e link)",
            Self::Tela => "tela da pesquisa (POST derivado)",
            Self::Envio => "resposta do envio",
            Self::PlanoDaPagina => "plano da página 2",
            Self::PostDaTelaDeResultado => "POST da tela de resultado",
            Self::TelaEmDocumentos => "tela da pesquisa (POST em documentos)",
            Self::EnvioEmDocumentos => "resposta do envio em documentos",
            Self::PlanoEmDocumentos => "plano da página 2 em documentos",
            Self::PaginaEmDocumentos => "página seguinte em documentos",
            Self::PesquisaRapida => "302 da pesquisa rápida",
        }
    }
}

/// Uma leitura de uma resposta gravada: só contagens e formas, nunca URL,
/// cookie, `infra_hash`, NUP ou id.
#[derive(Clone, Debug)]
pub(super) struct Registro {
    pub(super) leitor: Leitor,
    pub(super) instalacao: String,
    pub(super) card: String,
    pub(super) passo: usize,
    pub(super) resultado: Resultado,
    pub(super) forma: Option<&'static str>,
    pub(super) total_declarado: Option<u64>,
    pub(super) faixa: Option<(u64, u64)>,
    pub(super) linhas_ausentes: u64,
}

/// Só os cards que gravaram a tela de Pesquisa.
const OPERACOES: [&str; 2] = ["protocolo_pesquisar", "protocolo_pesquisa_rapida"];

#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE (padrão ~/.cache/ies); sem rede"]
fn replay_do_acervo_inteiro() {
    let mut registros = Vec::new();
    let lidos = percorrer(&OPERACOES, |nome, instalacao, json| {
        registros.extend(rapida::replay(nome, instalacao, json));
        if gravacoes_da_pesquisa_avancada(json) {
            registros.extend(leitores::replay(nome, instalacao, json));
        }
    });
    assert!(registros.iter().any(|r| r.leitor == Leitor::PesquisaRapida));
    publicar(&relatorio::markdown(lidos, &registros));
}

fn gravacoes_da_pesquisa_avancada(json: &serde_json::Value) -> bool {
    crate::cassette::acervo::gravacao::respostas(json)
        .iter()
        .any(|r| r.operacao == "protocolo_pesquisar")
}
