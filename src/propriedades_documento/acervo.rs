//! Replay de conformidade do acervo público para as propriedades do documento
//! (`documento_alterar`, `documento_consultar` e as variantes `_recebido`): nas
//! cassettes que gravaram essas operações, as respostas passam pelos **mesmos
//! leitores** de produção, offline: a barra da árvore (`ler_link`, onde a #782
//! trocou a leitura de `acao=` por `ConsultaDoControlador`), o formulário do
//! cadastro (`ler_formulario`, atributos por `valor_decodificado`) e o anexo
//! publicado (`ler_anexo_publicado`). Só contagens e formas, nunca URL, cookie,
//! `infra_hash` ou corpo. `#[ignore]`: lê os ZIPs do cache local
//! (`IES_ACERVO_CACHE`).
use super::{ler_anexo_publicado, ler_formulario, ler_link};
use crate::cassette::acervo::{INSTALACOES, gravacao, percorrer, publicar};
use crate::cassette::html_da_resposta;
use std::collections::BTreeMap;
use std::fmt::Write as _;

type Contagem = BTreeMap<(&'static str, String), BTreeMap<String, u64>>;

const BARRA: &str = "1. link de cadastro na barra (`ler_link`)";
const FORMULARIO: &str = "2. formulário do cadastro (`ler_formulario`)";
const ANEXO: &str = "3. anexo publicado no cadastro recebido (`ler_anexo_publicado`)";

/// Uma leitura aceita tem classe sem crase; um erro do leitor vem entre crases.
fn aceita(classe: &str) -> bool {
    !classe.starts_with('`')
}

/// O valor único do parâmetro `nome` na URL servida, ou vazio se ausente ou repetido.
fn valor_unico(interacao: &serde_json::Value, nome: &str) -> String {
    let valores = gravacao::parametro(interacao, nome);
    match valores.first() {
        Some(primeiro) if valores.iter().all(|v| v == primeiro) => (*primeiro).to_owned(),
        _ => String::new(),
    }
}

fn html(interacao: &serde_json::Value) -> Result<String, String> {
    html_da_resposta(interacao).map_err(|erro| format!("HTML ilegível: `{erro}`"))
}

fn barra(instalacao: &crate::instalacao::Instalacao, r: &gravacao::Resposta<'_>) -> String {
    let arvore = match html(r.interacao) {
        Ok(arvore) => arvore,
        Err(classe) => return classe,
    };
    let documento = valor_unico(r.interacao, "id_documento");
    let processo = gravacao::id_procedimento(r.interacao).unwrap_or_default();
    match ler_link(instalacao, &arvore, &documento, &processo) {
        Ok(None) => "sem link de cadastro na barra".to_owned(),
        Ok(Some(link)) => format!("link `{:?}`", link.fonte),
        Err(erro) => format!("`{erro:?}`"),
    }
}

/// Lê o formulário e, nos `_recebido`, o anexo publicado; cada um é uma leitura.
fn cadastro(
    instalacao: &crate::instalacao::Instalacao,
    r: &gravacao::Resposta<'_>,
) -> Vec<(&'static str, String)> {
    let corpo = match html(r.interacao) {
        Ok(corpo) => corpo,
        Err(classe) => return vec![(FORMULARIO, classe)],
    };
    let documento = valor_unico(r.interacao, "id_documento");
    let processo = gravacao::id_procedimento(r.interacao).unwrap_or_default();
    let mut leituras = vec![(
        FORMULARIO,
        match ler_formulario(&corpo, &documento, &processo) {
            Ok(_) => "lido".to_owned(),
            Err(erro) => format!("`{erro:?}`"),
        },
    )];
    if r.operacao.ends_with("_recebido") {
        let anexo = match ler_anexo_publicado(instalacao, &corpo, &documento, &processo) {
            Ok(Some(_)) => "anexo publicado lido".to_owned(),
            Ok(None) => "sem anexo publicado".to_owned(),
            Err(erro) => format!("`{erro:?}`"),
        };
        leituras.push((ANEXO, anexo));
    }
    leituras
}

fn replay(json: &serde_json::Value) -> Vec<(&'static str, String)> {
    let instalacao = gravacao::instalacao(json).expect("instalação da gravação");
    let mut leituras = Vec::new();
    for r in gravacao::respostas(json) {
        if r.metodo != "GET" || r.status != 200 {
            continue;
        }
        match r.operacao {
            "procedimento_visualizar" => leituras.push((BARRA, barra(&instalacao, &r))),
            "documento_alterar"
            | "documento_consultar"
            | "documento_alterar_recebido"
            | "documento_consultar_recebido" => leituras.extend(cadastro(&instalacao, &r)),
            _ => {}
        }
    }
    leituras
}

#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE (padrão ~/.cache/ies); sem rede"]
fn replay_de_propriedades_no_acervo() {
    let mut contagem = Contagem::new();
    let operacoes = [
        "procedimento_visualizar",
        "documento_alterar",
        "documento_consultar",
        "documento_alterar_recebido",
        "documento_consultar_recebido",
    ];
    let lidos = percorrer(&operacoes, |_, instalacao, json| {
        for (leitor, classe) in replay(json) {
            *contagem
                .entry((leitor, classe))
                .or_default()
                .entry(instalacao.to_owned())
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

    let recusas: Vec<_> = contagem
        .keys()
        .filter(|(_, classe)| !aceita(classe))
        .collect();
    assert!(
        recusas.is_empty(),
        "leitores recusaram a gravação: {recusas:?}"
    );
    let total = |leitor: &str| -> u64 {
        contagem
            .iter()
            .filter(|((l, _), _)| *l == leitor)
            .flat_map(|(_, por)| por.values())
            .sum()
    };
    let links: u64 = contagem
        .iter()
        .filter(|((l, c), _)| *l == BARRA && c.starts_with("link"))
        .flat_map(|(_, por)| por.values())
        .sum();
    assert!(total(BARRA) > 0 && total(FORMULARIO) > 0 && total(ANEXO) > 0);
    assert!(links > 0, "nenhum link de cadastro lido na barra");
}
