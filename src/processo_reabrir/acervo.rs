//! Replay de conformidade do acervo público para `processo reabrir` (Refs
//! #172): nas cassettes que gravaram `procedimento_concluir` ou
//! `procedimento_reabrir`, as respostas passam pelos **mesmos leitores**
//! do comando, offline: a ação de reabrir da barra de cada árvore,
//! cruzada com a regra de decisão (a unidade da tela do processo em
//! `aberto_em`); a URL de reabrir em cada tela do nó; a resposta do GET de
//! reabrir; e, por escrita, a verificação (unidade fora de `aberto_em`
//! antes e dentro dele depois). Só contagens e formas, nunca URL.
//! `#[ignore]`: lê os ZIPs do cache local (`IES_ACERVO_CACHE`).
use crate::arvore_processo::{
    AbertoEm, ler_arvore, ler_endereco_de_reabrir, ler_reabrir_da_barra, ler_tela_do_processo,
    nup_declarado_na_arvore,
};
use crate::cassette::acervo::{INSTALACOES, gravacao, percorrer, publicar};
use crate::cassette::html_da_resposta;
use crate::formulario_web::RespostaDoPost;
use crate::instalacao::Instalacao;
use crate::processo_concluir::FalhaDoEnvio;
use crate::processo_concluir::escrita::resposta_da_funcao;
use crate::tela_html::HtmlServido;
use crate::transporte::RespostaWeb;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Cursor;

type Contagem = BTreeMap<(&'static str, String), BTreeMap<String, u64>>;

fn somar(contagem: &mut Contagem, leitor: &'static str, classe: String, instalacao: &str) {
    *contagem
        .entry((leitor, classe))
        .or_default()
        .entry(instalacao.to_owned())
        .or_default() += 1;
}

fn siglas(aberto_em: &AbertoEm) -> Option<Vec<String>> {
    match aberto_em {
        AbertoEm::Unidades(u) => Some(u.iter().map(|u| u.sigla.clone()).collect()),
        _ => None,
    }
}

/// A ação da barra cruzada com a regra de decisão do contrato.
fn barra(
    inst: &Instalacao,
    html: &str,
    id: &str,
    unidade: Option<&String>,
    lida: Option<&Vec<String>>,
) -> String {
    let oferece = match ler_reabrir_da_barra(inst, html, id) {
        Ok(oferece) => oferece.is_some(),
        Err(erro) => return format!("`{erro:?}`"),
    };
    let (Some(u), Some(l)) = (unidade, lida) else {
        let barra = if oferece { "oferece" } else { "não oferece" };
        return format!("{barra}; unidade ou situação não lida");
    };
    let na_unidade = l.contains(u);
    match (oferece, na_unidade) {
        (true, false) => "oferece; unidade fora de `aberto_em` (reabrir)".into(),
        (false, true) => "não oferece; unidade em `aberto_em` (nada a fazer)".into(),
        (true, true) => "oferece; unidade em `aberto_em` (recusa)".into(),
        (false, false) => "não oferece; unidade fora de `aberto_em` (recusa)".into(),
    }
}

fn resposta(interacao: &serde_json::Value, status: u64) -> String {
    let corpo = gravacao::corpo(interacao);
    let resposta = RespostaDoPost {
        resposta: RespostaWeb {
            status_code: i32::try_from(status).unwrap_or(0),
            mime: gravacao::mime(interacao),
            content_length: Some(corpo.len()),
            localizacao: None,
            corpo: Cursor::new(corpo),
        },
        localizacao: gravacao::localizacao(interacao).map(str::to_owned),
    };
    match resposta_da_funcao(resposta, FalhaDoEnvio::RedirecionamentoNaReabertura) {
        None => "HTML 200 esperado".into(),
        Some(falha) => format!("`{falha:?}`"),
    }
}

/// O id e a unidade da sessão de antes da última escrita, e se a unidade
/// estava fora de `aberto_em`.
type Escrita = (String, Option<String>, Option<bool>);

#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE (padrão ~/.cache/ies); sem rede"]
fn replay_de_processo_reabrir_no_acervo() {
    let mut contagem = Contagem::new();
    let operacoes = ["procedimento_concluir", "procedimento_reabrir"];
    let lidos = percorrer(&operacoes, |_, nome, json| {
        let Some(inst) = gravacao::instalacao(json) else {
            return;
        };
        let mut unidade: Option<String> = None;
        let mut aberto_em: BTreeMap<String, Option<Vec<String>>> = BTreeMap::new();
        let mut escrita: Option<Escrita> = None;
        for r in gravacao::respostas(json) {
            let id = gravacao::id_procedimento(r.interacao)
                .or_else(|| gravacao::id_da_pesquisa_rapida(r.interacao))
                .unwrap_or_default();
            let html = || html_da_resposta(r.interacao).ok();
            match (r.operacao, r.metodo, r.status) {
                ("procedimento_trabalhar", "GET", 200) => {
                    if let Some(tela) = html().and_then(|h| {
                        ler_tela_do_processo(&inst, &HtmlServido::de_teste(&h), &id, None).ok()
                    }) {
                        unidade = Some(tela.unidade);
                    }
                }
                ("procedimento_visualizar", "GET", 200) => {
                    let Some(h) = html() else { continue };
                    let lida = nup_declarado_na_arvore(&h)
                        .and_then(|nup| ler_arvore(&h, &nup, &id).ok())
                        .and_then(|a| siglas(&a.aberto_em));
                    let classe = barra(&inst, &h, &id, unidade.as_ref(), lida.as_ref());
                    somar(&mut contagem, "ação de reabrir na barra", classe, nome);
                    if escrita.as_ref().is_some_and(|(de, ..)| *de == id)
                        && let Some((_, antes_unidade, fora_antes)) = escrita.take()
                    {
                        let classe = match (antes_unidade, fora_antes, lida.as_ref()) {
                            (Some(u), Some(true), Some(depois)) if depois.contains(&u) => {
                                "verificada (unidade da sessão aberta depois)"
                            }
                            (Some(_), Some(true), Some(_)) => "unidade ainda fora depois",
                            (_, Some(false), _) => "unidade já em `aberto_em` antes",
                            _ => "unidade ou situação não lida",
                        };
                        somar(&mut contagem, "verificação", classe.into(), nome);
                    }
                    aberto_em.insert(id.clone(), lida);
                }
                ("arvore_visualizar", "GET", 200) => {
                    let Some(h) = html() else { continue };
                    let classe = match ler_endereco_de_reabrir(&inst, &h, &id) {
                        Ok((_, forma)) => format!("`{}`", forma.como_texto()),
                        Err(erro) => format!("`{erro:?}`"),
                    };
                    somar(&mut contagem, "URL na tela do nó", classe, nome);
                }
                ("procedimento_reabrir", "GET", status) => {
                    somar(
                        &mut contagem,
                        "resposta da escrita",
                        resposta(r.interacao, status),
                        nome,
                    );
                    let antes = aberto_em.get(&id).cloned().flatten();
                    let fora = unidade
                        .as_ref()
                        .zip(antes.as_ref())
                        .map(|(u, a)| !a.contains(u));
                    escrita = Some((id.clone(), unidade.clone(), fora));
                }
                _ => {}
            }
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
    publicar(&s);
}
