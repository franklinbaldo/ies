//! Replay de conformidade do acervo público para `processo andamento
//! registrar`: nas cassettes que gravaram `procedimento_atualizar_andamento`,
//! as respostas passam pelos **mesmos leitores** do comando, offline: o
//! formulário servido (`conferir_formulario`), o 302 do POST
//! (`localizacao_esperada`, o aceite que a PR #782 alterou) e o histórico
//! lido pelo `ler_historico` antes e depois de um 302 aceito. Só contagens
//! e formas, nunca URL, cookie, hash ou corpo. `#[ignore]`: lê os ZIPs do
//! cache local (`IES_ACERVO_CACHE`).
use super::{conferir_formulario, localizacao_esperada};
use crate::arvore_processo::nup_declarado_na_arvore;
use crate::cassette::acervo::{INSTALACOES, gravacao, percorrer, publicar};
use crate::cassette::html_da_resposta;
use crate::historico::ler_historico;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;

type Contagem = BTreeMap<(&'static str, String), BTreeMap<String, u64>>;

const FORMULARIO: &str = "1. formulário de andamento (`conferir_formulario`)";
const ESCRITA: &str =
    "2. resposta do POST `procedimento_atualizar_andamento` (`localizacao_esperada`)";
const HISTORICO: &str = "3. histórico lido (`ler_historico`)";
const RELEITURA: &str = "4. histórico relido após o 302 aceito (`ler_historico`)";

/// As classes que a gravação comprova; qualquer outra é um achado. Um 200
/// no POST não é recusa: o fluxo segue para a releitura, que decide.
fn esperada(leitor: &str, classe: &str) -> bool {
    match leitor {
        FORMULARIO => classe == "conferido",
        ESCRITA => classe == "302 aceito" || classe == "HTML 200 (releitura decide)",
        HISTORICO | RELEITURA => classe == "lido",
        _ => false,
    }
}

fn formulario(interacao: &serde_json::Value, id: &str) -> String {
    let Ok(html) = html_da_resposta(interacao) else {
        return "HTML ilegível".into();
    };
    match conferir_formulario(&html, id) {
        Ok(()) => "conferido".into(),
        Err(erro) => format!("`{erro:?}`"),
    }
}

/// Classifica o POST e diz se o 302 foi aceito (e, logo, há releitura).
fn escrita(json: &serde_json::Value, r: &gravacao::Resposta<'_>, id: &str) -> (String, bool) {
    match r.status {
        302 => {
            let url = gravacao::url_sem_consulta(json, r.interacao);
            let aceito = localizacao_esperada(gravacao::localizacao(r.interacao), &url, id);
            let classe = if aceito {
                "302 aceito"
            } else {
                "302 não aceito"
            };
            (classe.to_owned(), aceito)
        }
        200 => ("HTML 200 (releitura decide)".into(), false),
        outro => (format!("status {outro}"), false),
    }
}

fn historico(interacao: &serde_json::Value, nup: Option<&String>) -> String {
    let Some(nup) = nup else {
        return "NUP não lido na árvore".into();
    };
    let Ok(html) = html_da_resposta(interacao) else {
        return "HTML ilegível".into();
    };
    match ler_historico(&html, nup) {
        Ok(_) => "lido".into(),
        Err(erro) => format!("`{erro:?}`"),
    }
}

/// Uma passagem pelas respostas: o histórico logo após um 302 aceito é a releitura.
fn replay(json: &serde_json::Value) -> Vec<(&'static str, String)> {
    let mut leituras = Vec::new();
    let mut nups: HashMap<String, String> = HashMap::new();
    let mut releitura = false;
    for r in gravacao::respostas(json) {
        let id = gravacao::id_procedimento(r.interacao).unwrap_or_default();
        match (r.operacao, r.metodo, r.status) {
            ("procedimento_visualizar", "GET", 200) => {
                let nup = html_da_resposta(r.interacao)
                    .ok()
                    .and_then(|h| nup_declarado_na_arvore(&h));
                if let Some(nup) = nup {
                    nups.insert(id, nup);
                }
            }
            ("procedimento_atualizar_andamento", "GET", 200) => {
                leituras.push((FORMULARIO, formulario(r.interacao, &id)));
            }
            ("procedimento_atualizar_andamento", "POST", _) => {
                let (classe, aceito) = escrita(json, &r, &id);
                leituras.push((ESCRITA, classe));
                releitura = aceito;
            }
            ("procedimento_consultar_historico", "GET", 200) => {
                let leitor = if releitura { RELEITURA } else { HISTORICO };
                leituras.push((leitor, historico(r.interacao, nups.get(&id))));
                releitura = false;
            }
            _ => {}
        }
    }
    leituras
}

fn total(contagem: &Contagem, leitor: &str) -> u64 {
    contagem
        .iter()
        .filter(|((l, _), _)| *l == leitor)
        .flat_map(|(_, por)| por.values())
        .sum()
}

#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE (padrão ~/.cache/ies); sem rede"]
fn replay_de_andamento_registrar_no_acervo() {
    let mut contagem = Contagem::new();
    let lidos = percorrer(&["procedimento_atualizar_andamento"], |_, nome, json| {
        for (leitor, classe) in replay(json) {
            *contagem
                .entry((leitor, classe))
                .or_default()
                .entry(nome.to_owned())
                .or_default() += 1;
        }
    });
    let mut s = format!("{lidos} cassettes lidas.\n\n| Leitor | Classe |");
    for i in INSTALACOES {
        let _ = write!(s, " {i} |");
    }
    s.push_str("\n| --- | --- |");
    s.push_str(&" --- |".repeat(INSTALACOES.len()));
    for ((leitor, classe), por) in &contagem {
        let _ = write!(s, "\n| {leitor} | {classe} |");
        for i in INSTALACOES {
            let _ = write!(s, " {} |", por.get(i).copied().unwrap_or(0));
        }
    }
    s.push('\n');
    publicar(&s);

    let achados: Vec<_> = contagem
        .keys()
        .filter(|(leitor, classe)| !esperada(leitor, classe))
        .collect();
    assert!(
        achados.is_empty(),
        "leitores recusaram a gravação: {achados:?}"
    );
    let aceitos = contagem
        .get(&(ESCRITA, "302 aceito".to_owned()))
        .map_or(0, |por| por.values().sum());
    assert!(
        lidos > 0 && total(&contagem, FORMULARIO) > 0,
        "nenhum formulário lido"
    );
    assert!(aceitos > 0, "nenhum 302 do POST foi lido");
    assert_eq!(
        total(&contagem, RELEITURA),
        aceitos,
        "todo andamento aceito é relido"
    );
}
