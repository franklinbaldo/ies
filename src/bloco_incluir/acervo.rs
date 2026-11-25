//! Replay de conformidade do acervo público para `bloco incluir`: nas
//! cassettes que gravaram `bloco_escolher`, as respostas passam pelos
//! **mesmos leitores** do comando, offline: a tela `frmBlocoEscolher`
//! (`ler_tela_de_escolha`, com os atributos lidos por `valor_decodificado`,
//! alterados pela PR #782), o link `bloco_escolher` da barra do documento
//! (`ler_bloco_escolher_da_barra`), o 302 do POST (`redirecionamento_da_inclusao`,
//! o aceite que a PR #782 alterou), o detalhe do bloco (`ler_detalhe`) e,
//! depois de um 302 aceito, o detalhe relido com o documento. Só contagens
//! e formas, nunca URL, cookie, hash ou corpo. `#[ignore]`: lê os ZIPs do
//! cache local (`IES_ACERVO_CACHE`).
use super::{ler_tela_de_escolha, redirecionamento_da_inclusao};
use crate::arvore_processo::ler_bloco_escolher_da_barra;
use crate::bloco::TipoBloco;
use crate::bloco_detalhe::ler_detalhe;
use crate::bloco_estado::EstadoDoBloco;
use crate::cassette::acervo::{INSTALACOES, gravacao, percorrer, publicar};
use crate::cassette::html_da_resposta;
use crate::instalacao::Instalacao;
use crate::tela_html::HtmlServido;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;

type Contagem = BTreeMap<(&'static str, String), BTreeMap<String, u64>>;

const TELA: &str = "1. tela de escolha do bloco (`ler_tela_de_escolha`)";
const BARRA: &str = "2. link da barra do documento (`ler_bloco_escolher_da_barra`)";
const ESCRITA: &str = "3. resposta do POST `bloco_escolher` (`redirecionamento_da_inclusao`)";
const DETALHE: &str = "4. detalhe do bloco (`ler_detalhe`)";
const RELEITURA: &str = "5. detalhe relido após o 302 aceito (`ler_detalhe`)";

/// As classes que a gravação comprova; qualquer outra é um achado.
fn esperada(leitor: &str, classe: &str) -> bool {
    match leitor {
        TELA | DETALHE => classe == "lida" || classe == "lido",
        BARRA => classe == "oferece" || classe == "não oferece",
        ESCRITA => classe == "302 aceito",
        RELEITURA => classe == "contém o documento",
        _ => false,
    }
}

/// O primeiro valor do parâmetro `nome`, se todos os repetidos forem iguais.
fn unico<'a>(interacao: &'a serde_json::Value, nome: &str) -> Option<&'a str> {
    let valores = gravacao::parametro(interacao, nome);
    let primeiro = *valores.first()?;
    valores.iter().all(|v| *v == primeiro).then_some(primeiro)
}

fn tela(interacao: &serde_json::Value, numeros: &mut HashMap<String, String>) -> String {
    let Ok(html) = html_da_resposta(interacao) else {
        return "HTML ilegível".into();
    };
    match ler_tela_de_escolha(&HtmlServido::de_teste(&html)) {
        Ok(tela) => {
            for documento in tela.documentos {
                numeros.insert(documento.id, documento.numero);
            }
            "lida".into()
        }
        Err(erro) => format!("`{erro:?}`"),
    }
}

fn barra(
    instalacao: Option<&Instalacao>,
    interacao: &serde_json::Value,
    id_documento: &str,
    id_procedimento: &str,
) -> String {
    let (Some(instalacao), Ok(html)) = (instalacao, html_da_resposta(interacao)) else {
        return "instalação ou HTML ilegível".into();
    };
    match ler_bloco_escolher_da_barra(instalacao, &html, id_documento, id_procedimento) {
        Ok(Some(_)) => "oferece".into(),
        Ok(None) => "não oferece".into(),
        Err(erro) => format!("`{erro:?}`"),
    }
}

/// Classifica o POST; o número do documento vem da tela de escolha lida antes.
fn escrita(
    json: &serde_json::Value,
    r: &gravacao::Resposta<'_>,
    id_documento: Option<&str>,
    id_procedimento: &str,
    numeros: &HashMap<String, String>,
) -> (String, Option<String>) {
    let (302, Some(documento)) = (r.status, id_documento) else {
        return (format!("status {}", r.status), None);
    };
    let url = gravacao::url_sem_consulta(json, r.interacao);
    let aceito = redirecionamento_da_inclusao(
        gravacao::localizacao(r.interacao),
        &url,
        id_procedimento,
        documento,
    );
    if aceito {
        ("302 aceito".into(), numeros.get(documento).cloned())
    } else {
        ("302 não aceito".into(), None)
    }
}

/// O detalhe do bloco; com `documento`, diz se ele aparece na lista.
fn detalhe(interacao: &serde_json::Value, documento: Option<&str>) -> String {
    let Some(bloco) = unico(interacao, "id_bloco") else {
        return "bloco não identificado".into();
    };
    let Ok(html) = html_da_resposta(interacao) else {
        return "HTML ilegível".into();
    };
    // O estado só ecoa na saída; este replay confere os itens.
    match ler_detalhe(
        &HtmlServido::de_teste(&html),
        bloco,
        TipoBloco::Assinatura,
        EstadoDoBloco::Gerado,
    ) {
        Err(erro) => format!("`{erro:?}`"),
        Ok(lido) => match documento {
            None => "lido".into(),
            Some(numero)
                if lido
                    .itens
                    .iter()
                    .any(|i| i.documento.as_deref() == Some(numero)) =>
            {
                "contém o documento".into()
            }
            Some(_) => "não contém o documento".into(),
        },
    }
}

/// Uma passagem pelas respostas: o detalhe logo após um 302 aceito é a releitura.
fn replay(json: &serde_json::Value) -> Vec<(&'static str, String)> {
    let instalacao = gravacao::instalacao(json);
    let mut leituras = Vec::new();
    let mut numeros = HashMap::new();
    let mut pendente: Option<String> = None;
    for r in gravacao::respostas(json) {
        let id_procedimento = gravacao::id_procedimento(r.interacao).unwrap_or_default();
        let id_documento = unico(r.interacao, "id_documento");
        match (r.operacao, r.metodo, r.status, id_documento) {
            ("arvore_visualizar", "GET", 200, Some(documento)) => {
                let leitura = barra(
                    instalacao.as_ref(),
                    r.interacao,
                    documento,
                    &id_procedimento,
                );
                leituras.push((BARRA, leitura));
            }
            ("bloco_escolher", "GET", 200, _) => {
                leituras.push((TELA, tela(r.interacao, &mut numeros)));
            }
            ("bloco_escolher", "POST", _, _) => {
                let (classe, numero) = escrita(json, &r, id_documento, &id_procedimento, &numeros);
                leituras.push((ESCRITA, classe));
                pendente = numero;
            }
            ("rel_bloco_protocolo_listar", "GET", 200, _) => match pendente.take() {
                Some(numero) => leituras.push((RELEITURA, detalhe(r.interacao, Some(&numero)))),
                None => leituras.push((DETALHE, detalhe(r.interacao, None))),
            },
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
fn replay_de_bloco_incluir_no_acervo() {
    let mut contagem = Contagem::new();
    let lidos = percorrer(&["bloco_escolher"], |_, nome, json| {
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
        lidos > 0 && total(&contagem, TELA) > 0,
        "nenhuma tela de escolha lida"
    );
    assert!(aceitos > 0, "nenhum 302 do POST foi lido");
    assert_eq!(
        total(&contagem, RELEITURA),
        aceitos,
        "toda inclusão aceita é relida"
    );
}
