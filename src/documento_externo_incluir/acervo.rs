//! Replay de conformidade do acervo público para o documento externo (#332):
//! nas cassettes que gravaram `documento_escolher_tipo`, `documento_receber`
//! e `documento_upload_anexo`, as respostas passam pelos **mesmos leitores**
//! do cadastro, offline: a lista de escolha (`escolher_externo`, onde a #782
//! classifica o link `documento_receber` por `ConsultaDoControlador`), o 302
//! que aceita o destino (`conferir_destino`), os tipos oferecidos no cadastro
//! (`identificar_tipo`) e o plano de upload com a URL conferida
//! (`ler_plano` e `conferir_upload`). Só contagens e formas, nunca URL, cookie,
//! `infra_hash` ou corpo. `#[ignore]`: lê os ZIPs do cache local
//! (`IES_ACERVO_CACHE`).
use super::anexo::ler_plano;
use super::escolha::{EscolhaExterna, escolher_externo, identificar_tipo};
use super::fluxo::conferir_destino;
use super::fluxo_confirmar::conferir_upload;
use crate::cassette::acervo::{INSTALACOES, gravacao, percorrer, publicar};
use crate::cassette::html_da_resposta;
use crate::instalacao::Instalacao;
use crate::tela_html::HtmlServido;
use std::collections::BTreeMap;
use std::fmt::Write as _;

type Contagem = BTreeMap<(&'static str, String), BTreeMap<String, u64>>;

const ESCOLHA: &str = "1. lista de escolha do externo (`escolher_externo`)";
const DESTINO: &str = "2. 302 do escolher tipo para o externo (`conferir_destino`)";
const TIPO: &str = "3. tipos oferecidos no cadastro (`identificar_tipo`)";
const UPLOAD: &str = "4. plano de upload e URL conferida (`ler_plano`, `conferir_upload`)";

/// Uma leitura aceita tem classe sem crase; um erro do leitor vem entre crases.
fn aceita(classe: &str) -> bool {
    !classe.starts_with('`')
}

fn html(interacao: &serde_json::Value) -> Result<String, String> {
    html_da_resposta(interacao).map_err(|erro| format!("HTML ilegível: `{erro}`"))
}

/// Os textos das opções habilitadas e com valor do `selSerie`, como o usuário as veria.
fn tipos_oferecidos(html: &str) -> Vec<String> {
    let servido = HtmlServido::de_teste(html);
    let dom = crate::tela_html::interpretar(&servido);
    let parser = dom.parser();
    let selects = dom.query_selector("select").into_iter().flatten();
    selects
        .filter_map(|no| no.get(parser).and_then(tl::Node::as_tag))
        .filter(|tag| {
            crate::html_atributos::valor_do_atributo(tag, "id")
                .expect("id legível")
                .as_deref()
                == Some("selSerie")
        })
        .flat_map(|select| {
            select
                .children()
                .all(parser)
                .iter()
                .filter_map(tl::Node::as_tag)
                .collect::<Vec<_>>()
        })
        .filter(|tag| {
            tag.name().as_utf8_str() == "option"
                && !crate::html_atributos::atributo_presente(tag, "disabled")
                    .expect("disabled legível")
        })
        .filter(|tag| {
            crate::html_atributos::valor_do_atributo(tag, "value")
                .expect("value legível")
                .is_some_and(|valor| !valor.is_empty())
        })
        .map(|tag| {
            html_escape::decode_html_entities(&tag.inner_text(parser))
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|texto| !texto.is_empty())
        .collect()
}

fn escolha(r: &gravacao::Resposta<'_>) -> String {
    match html(r.interacao) {
        Err(classe) => classe,
        Ok(lista) => match escolher_externo(&HtmlServido::de_teste(&lista)) {
            Ok(EscolhaExterna::Url(_)) => "escolha por link (`documento_receber`)".to_owned(),
            Ok(EscolhaExterna::Formulario) => "escolha por formulário (`escolher(-1)`)".to_owned(),
            Err(erro) => format!("`{erro:?}`"),
        },
    }
}

/// O 302 só é lido como destino externo quando leva a `documento_receber`;
/// o 302 para `documento_gerar` é outro tipo de documento, não um achado.
fn destino(instalacao: &Instalacao, r: &gravacao::Resposta<'_>, processo: &str) -> String {
    let Some(local) = gravacao::localizacao(r.interacao) else {
        return "`302 sem Location`".to_owned();
    };
    if !local.contains("acao=documento_receber&") {
        return "não aplicável: 302 para outro tipo de documento".to_owned();
    }
    match conferir_destino(instalacao, local, processo) {
        Ok(_) => "302 aceito".to_owned(),
        Err(erro) => format!("`{erro:?}`"),
    }
}

fn tipos(r: &gravacao::Resposta<'_>) -> Vec<String> {
    let Ok(cadastro) = html(r.interacao) else {
        return vec!["HTML ilegível".to_owned()];
    };
    let oferecidos = tipos_oferecidos(&cadastro);
    if oferecidos.is_empty() {
        return vec!["nenhum tipo oferecido".to_owned()];
    }
    oferecidos
        .iter()
        .map(
            |tipo| match identificar_tipo(&HtmlServido::de_teste(&cadastro), tipo) {
                Ok(_) => "tipo oferecido lido".to_owned(),
                Err(erro) => format!("`{erro:?}`"),
            },
        )
        .collect()
}

fn upload(instalacao: &Instalacao, r: &gravacao::Resposta<'_>, processo: &str) -> String {
    let Ok(cadastro) = html(r.interacao) else {
        return "HTML ilegível".to_owned();
    };
    match ler_plano(&HtmlServido::de_teste(&cadastro)) {
        Err(erro) => format!("`{erro:?}`"),
        Ok(plano) => match conferir_upload(instalacao, plano.url_upload(), processo) {
            Ok(_) => "upload conferido".to_owned(),
            Err(erro) => format!("`{erro:?}`"),
        },
    }
}

fn replay(json: &serde_json::Value) -> Vec<(&'static str, String)> {
    let instalacao = gravacao::instalacao(json).expect("instalação da gravação");
    let mut leituras = Vec::new();
    for r in gravacao::respostas(json) {
        let processo = gravacao::id_procedimento(r.interacao).unwrap_or_default();
        match (r.operacao, r.metodo, r.status) {
            ("documento_escolher_tipo", "GET", 200) => {
                leituras.push((ESCOLHA, escolha(&r)));
            }
            ("documento_escolher_tipo", "POST", 302) => {
                leituras.push((DESTINO, destino(&instalacao, &r, &processo)));
            }
            ("documento_receber", "GET", 200) => {
                leituras.extend(tipos(&r).into_iter().map(|c| (TIPO, c)));
            }
            ("documento_receber", "POST", 200) => {
                leituras.push((UPLOAD, upload(&instalacao, &r, &processo)));
            }
            _ => {}
        }
    }
    leituras
}

#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE (padrão ~/.cache/ies); sem rede"]
fn replay_de_documento_externo_no_acervo() {
    let mut contagem = Contagem::new();
    let operacoes = ["documento_escolher_tipo", "documento_receber"];
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
    let aceitos: u64 = contagem
        .get(&(DESTINO, "302 aceito".to_owned()))
        .map_or(0, |por| por.values().sum());
    assert!(lidos > 0 && total(ESCOLHA) > 0 && total(TIPO) > 0 && total(UPLOAD) > 0);
    assert!(aceitos > 0, "nenhum 302 de destino externo foi lido");
}
