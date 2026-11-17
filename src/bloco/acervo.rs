//! Replay de conformidade do acervo público para a lista de blocos
//! (`bloco_assinatura_listar`, `bloco_interno_listar`, `bloco_reuniao_listar`):
//! passa as respostas gravadas pelo leitor de produção `ler_lista`, sem
//! afrouxar asserts. Refs #797.
use super::{TipoBloco, ler_lista};
use crate::cassette::acervo::{INSTALACOES, gravacao, percorrer, publicar};
use crate::cassette::html_da_resposta;
use crate::tela_html::HtmlServido;
use std::collections::BTreeMap;
use std::fmt::Write as _;

type Contagem = BTreeMap<(&'static str, String), BTreeMap<String, u64>>;

fn tipo_da_operacao(operacao: &str) -> Option<TipoBloco> {
    match operacao {
        "bloco_assinatura_listar" => Some(TipoBloco::Assinatura),
        "bloco_interno_listar" => Some(TipoBloco::Interno),
        "bloco_reuniao_listar" => Some(TipoBloco::Reuniao),
        _ => None,
    }
}

fn replay(json: &serde_json::Value) -> Vec<(&'static str, String)> {
    let mut leituras = Vec::new();
    for r in gravacao::respostas(json) {
        if r.status != 200 {
            continue;
        }
        let Some(tipo) = tipo_da_operacao(r.operacao) else {
            continue;
        };
        let classe = match html_da_resposta(r.interacao) {
            Ok(html) => match ler_lista(&HtmlServido::de_teste(&html), tipo) {
                Ok(lista) if lista.lista.itens.is_empty() => "vazia".to_owned(),
                Ok(_) => "lida".to_owned(),
                Err(erro) => format!("`{erro:?}`"),
            },
            Err(erro) => format!("HTML ilegível: `{erro}`"),
        };
        leituras.push((tipo.rotulo(), classe));
    }
    leituras
}

#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE (padrão ~/.cache/ies); sem rede"]
fn replay_da_lista_de_blocos_no_acervo() {
    let mut contagem = Contagem::new();
    let operacoes = [
        "bloco_assinatura_listar",
        "bloco_interno_listar",
        "bloco_reuniao_listar",
    ];
    let lidos = percorrer(&operacoes, |_, instalacao, json| {
        for (tipo, classe) in replay(json) {
            *contagem
                .entry((tipo, classe))
                .or_default()
                .entry(instalacao.to_owned())
                .or_default() += 1;
        }
    });
    let mut s = format!("{lidos} cassettes lidas.\n\n| Tipo | Classe |");
    for i in INSTALACOES {
        let _ = write!(s, " {i} |");
    }
    s.push_str("\n| --- | --- |");
    s.push_str(&" --- |".repeat(INSTALACOES.len()));
    for ((tipo, classe), por) in &contagem {
        let _ = write!(s, "\n| {tipo} | {classe} |");
        for i in INSTALACOES {
            let _ = write!(s, " {} |", por.get(i).copied().unwrap_or(0));
        }
    }
    s.push('\n');
    publicar(&s);

    let recusas: Vec<_> = contagem
        .keys()
        .filter(|(_, classe)| classe.starts_with('`') || classe.starts_with("HTML"))
        .collect();
    assert!(
        recusas.is_empty(),
        "leitores recusaram a gravação: {recusas:?}"
    );
    let total: u64 = contagem.values().flat_map(|por| por.values()).sum();
    assert!(total > 0, "nenhuma lista de blocos lida");
}
