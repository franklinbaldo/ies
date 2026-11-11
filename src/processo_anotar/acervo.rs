//! Replay de conformidade do acervo público para `processo anotar`: nas
//! cassettes que gravaram `anotacao_registrar`, as respostas passam pelos
//! **mesmos leitores** do comando, offline: a tela de anotação
//! (`texto_anotacao`), a presença do botão de salvar (`salvar_oferecido`),
//! o 302 do POST (`redirecionamento_esperado`, o aceite que a PR #782
//! alterou) e, depois de um 302 aceito, a tela relida. Só contagens e
//! formas, nunca URL, cookie, hash ou corpo. `#[ignore]`: lê os ZIPs do
//! cache local (`IES_ACERVO_CACHE`).
use super::redirecionamento_esperado;
use crate::cassette::acervo::{INSTALACOES, gravacao, percorrer, publicar};
use crate::cassette::html_da_resposta;
use crate::processo_enviar::{salvar_oferecido, texto_anotacao};
use std::collections::BTreeMap;
use std::fmt::Write as _;

type Contagem = BTreeMap<(&'static str, String), BTreeMap<String, u64>>;

const TELA: &str = "1. tela de anotação (`texto_anotacao`)";
const SALVAR: &str = "2. botão de salvar (`salvar_oferecido`)";
const ESCRITA: &str = "3. resposta do POST `anotacao_registrar` (`redirecionamento_esperado`)";
const RELEITURA: &str = "4. tela relida após o 302 aceito (`texto_anotacao`)";

/// As classes que a gravação comprova; qualquer outra é um achado.
fn esperada(leitor: &str, classe: &str) -> bool {
    match leitor {
        TELA => classe.starts_with("lida"),
        SALVAR => classe == "oferecido" || classe == "não oferecido",
        ESCRITA => classe == "302 aceito",
        RELEITURA => classe == "lida (com anotação)",
        _ => false,
    }
}

fn texto(interacao: &serde_json::Value, id: &str) -> String {
    match html_da_resposta(interacao) {
        Ok(html) => match texto_anotacao(&html, id) {
            Ok(texto) if texto.trim().is_empty() => "lida (sem anotação)".into(),
            Ok(_) => "lida (com anotação)".into(),
            Err(erro) => format!("`{erro:?}`"),
        },
        Err(erro) => format!("HTML ilegível: `{erro}`"),
    }
}

fn salvar(interacao: &serde_json::Value, id: &str) -> String {
    let Ok(html) = html_da_resposta(interacao) else {
        return "HTML ilegível".into();
    };
    match salvar_oferecido(&html) {
        Ok(true) => "oferecido".into(),
        Ok(false) => "não oferecido".into(),
        Err(erro) => format!("`{erro:?}` (processo {id})"),
    }
}

/// Uma passagem pelas respostas: a tela logo após um 302 aceito é a releitura.
fn replay(json: &serde_json::Value) -> Vec<(&'static str, String)> {
    let mut leituras = Vec::new();
    let mut releitura = false;
    for r in gravacao::respostas(json) {
        let id = gravacao::id_procedimento(r.interacao).unwrap_or_default();
        match (r.operacao, r.metodo, r.status) {
            ("anotacao_registrar", "GET", 200) => {
                let leitor = if releitura { RELEITURA } else { TELA };
                leituras.push((leitor, texto(r.interacao, &id)));
                leituras.push((SALVAR, salvar(r.interacao, &id)));
                releitura = false;
            }
            ("anotacao_registrar", "POST", 302) => {
                let url = gravacao::url_sem_consulta(json, r.interacao);
                let aceito =
                    redirecionamento_esperado(gravacao::localizacao(r.interacao), &url, &id);
                let classe = if aceito {
                    "302 aceito"
                } else {
                    "302 não aceito"
                };
                leituras.push((ESCRITA, classe.to_owned()));
                releitura = aceito;
            }
            ("anotacao_registrar", metodo, status) => {
                leituras.push((ESCRITA, format!("{metodo} {status} não lido")));
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
fn replay_de_processo_anotar_no_acervo() {
    let mut contagem = Contagem::new();
    let lidos = percorrer(&["anotacao_registrar"], |_, nome, json| {
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
    assert!(lidos > 0 && total(&contagem, TELA) > 0, "nenhuma tela lida");
    assert!(aceitos > 0, "nenhum 302 do POST foi lido");
    assert_eq!(
        total(&contagem, RELEITURA),
        aceitos,
        "toda anotação aceita é relida"
    );
}
