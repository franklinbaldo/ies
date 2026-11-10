//! Replay de conformidade do acervo público para `documento assinar`
//! (Refs #174): nas cassettes que gravaram `documento_assinar`, as
//! respostas passam pelos **mesmos leitores** do comando, offline: a ação
//! e a caneta de cada árvore, o link de assinar de cada tela do documento,
//! a tela "Assinatura de Documento" (conferência e derivação do POST com
//! cargo e senha sintéticos) e a resposta do POST, e a regra de verificação
//! (nenhuma caneta antes, a do assinante com o cargo da tela depois). Só
//! contagens e formas, nunca URL. `#[ignore]`: lê os ZIPs do cache local
//! (`IES_ACERVO_CACHE`).
use super::tela::{
    ASSINAR_DOCUMENTO, CARGO, FORMA_DE_AUTENTICACAO, ORGAO, PELA_SENHA, SENHA, TelaDeAssinatura,
    USUARIO, assinatura_realizada, conferir_tela_de_assinatura,
};
use crate::arvore_processo::{
    Assinante, ler_assinatura_na_arvore, ler_link_de_assinar, nup_declarado_na_arvore,
    tela_do_documento,
};
use crate::cassette::acervo::{INSTALACOES, gravacao, percorrer, publicar};
use crate::cassette::html_da_resposta;
use crate::erro_publico::Rotulo as _;
use crate::formulario_web::{CodificacaoDaTela, TelaServida, derivar_submissao};
use crate::instalacao::Instalacao;
use std::collections::BTreeMap;
use std::fmt::Write as _;

type Contagem = BTreeMap<(&'static str, String), BTreeMap<String, u64>>;

fn somar(contagem: &mut Contagem, leitor: &'static str, classe: String, instalacao: &str) {
    *contagem
        .entry((leitor, classe))
        .or_default()
        .entry(instalacao.to_owned())
        .or_default() += 1;
}

/// O parâmetro `chave` da URL da resposta gravada.
fn parametro(interacao: &serde_json::Value, chave: &str) -> Option<String> {
    let url = interacao["response"]["url"].as_str()?;
    let (_, consulta) = url.split_once('?')?;
    consulta
        .split('&')
        .filter_map(|p| p.split_once('='))
        .find(|(k, _)| *k == chave)
        .map(|(_, v)| v.to_owned())
}

fn tela(
    inst: &Instalacao,
    html: &str,
    interacao: &serde_json::Value,
    doc: &str,
) -> (String, Option<TelaDeAssinatura>) {
    let lida = match conferir_tela_de_assinatura(html, doc) {
        Ok(lida) => lida,
        Err(erro) => return (format!("`{}`", erro.rotulo()), None),
    };
    let Some(cargo) = lida.cargo_da_tela.clone() else {
        return ("conferida, sem cargo sugerido".into(), Some(lida));
    };
    let url = interacao["response"]["url"].as_str().unwrap_or("");
    let servida = TelaServida {
        url,
        html,
        codificacao: CodificacaoDaTela::Iso88591,
    };
    let campos = [
        (FORMA_DE_AUTENTICACAO, PELA_SENHA),
        (ORGAO, lida.orgao.as_str()),
        (USUARIO, lida.assinante.as_str()),
        (CARGO, cargo.as_str()),
        (SENHA, "senha-sintetica"),
    ];
    let classe = match derivar_submissao(&ASSINAR_DOCUMENTO, inst, &servida, "", &campos) {
        Ok(s) => format!(
            "conferida e derivada ({} campos)",
            s.nomes_dos_campos().len()
        ),
        Err(erro) => format!("`{}`", erro.rotulo()),
    };
    (classe, Some(lida))
}

/// A última tela de assinatura lida e a caneta de antes, por documento.
#[derive(Default)]
struct Estado {
    tela: BTreeMap<String, TelaDeAssinatura>,
    antes: BTreeMap<String, Vec<Assinante>>,
    escrita: Option<String>,
}

/// A ação e a caneta de cada documento já visto numa tela de assinatura, e
/// a verificação da última escrita.
fn arvore(
    contagem: &mut Contagem,
    estado: &mut Estado,
    inst: &Instalacao,
    h: &str,
    processo: &str,
    nome: &str,
) {
    // A árvore de cada documento já visto numa tela de assinatura.
    let docs: Vec<String> = estado.tela.keys().cloned().collect();
    for d in docs {
        let lida = ler_assinatura_na_arvore(h, &d);
        let classe = match &lida {
            Ok(a) if a.publicada => {
                format!("publicada, {} assinante(s)", a.assinantes.len())
            }
            Ok(a) => format!("não publicada, {} assinante(s)", a.assinantes.len()),
            Err(erro) => format!("`{}`", erro.onde),
        };
        somar(contagem, "árvore (ação e caneta)", classe, nome);
        if tela_do_documento(inst, h, &d, processo).is_some() {
            somar(contagem, "tela do documento (a3)", "conferida".into(), nome);
        }
        let Ok(lida) = lida else { continue };
        if estado.escrita.as_deref() == Some(d.as_str()) {
            estado.escrita = None;
            let t = &estado.tela[&d];
            let antes = estado.antes.get(&d).cloned().unwrap_or_default();
            let classe = match lida.assinantes.as_slice() {
                [a] if antes.is_empty()
                    && a.nome == t.assinante
                    && Some(&a.cargo) == t.cargo_da_tela.as_ref() =>
                {
                    "verificada (caneta do assinante com o cargo)".to_owned()
                }
                [] => "sem caneta depois".to_owned(),
                _ => "caneta diferente da esperada".to_owned(),
            };
            somar(contagem, "verificação", classe, nome);
        } else {
            estado.antes.insert(d, lida.assinantes);
        }
    }
}

#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE (padrão ~/.cache/ies); sem rede"]
fn replay_de_documento_assinar_no_acervo() {
    let mut contagem = Contagem::new();
    let lidos = percorrer(&["documento_assinar"], |_, nome, json| {
        let Some(inst) = gravacao::instalacao(json) else {
            return;
        };
        let mut estado = Estado::default();
        for r in gravacao::respostas(json) {
            let html = || html_da_resposta(r.interacao).ok();
            let doc = parametro(r.interacao, "id_documento").unwrap_or_default();
            let processo = gravacao::id_procedimento(r.interacao).unwrap_or_default();
            match (r.operacao, r.metodo, r.status) {
                ("procedimento_visualizar", "GET", 200) => {
                    let Some(h) = html() else { continue };
                    if nup_declarado_na_arvore(&h).is_none() {
                        continue;
                    }
                    arvore(&mut contagem, &mut estado, &inst, &h, &processo, nome);
                }
                ("arvore_visualizar", "GET", 200) if !doc.is_empty() => {
                    let Some(h) = html() else { continue };
                    if !h.contains("linkAssinarDocumento")
                        && !h.contains("function assinarDocumento")
                    {
                        continue;
                    }
                    let classe = match ler_link_de_assinar(&inst, &h, &doc, &processo) {
                        Ok(_) => "link lido e conferido".to_owned(),
                        Err(erro) => format!("`{}`", erro.rotulo()),
                    };
                    somar(&mut contagem, "link na tela do documento", classe, nome);
                }
                ("documento_assinar", "GET", 200) => {
                    let Some(h) = html() else { continue };
                    let (classe, lida) = tela(&inst, &h, r.interacao, &doc);
                    somar(&mut contagem, "tela de assinatura", classe, nome);
                    if let Some(lida) = lida {
                        estado.tela.insert(doc.clone(), lida);
                    }
                }
                ("documento_assinar", "POST", status) => {
                    let classe = match (status, html()) {
                        (200, Some(h)) if assinatura_realizada(&h) => {
                            "HTML 200 com finalizar()".to_owned()
                        }
                        (200, Some(_)) => "HTML 200 sem finalizar()".to_owned(),
                        (outro, _) => format!("status {outro}"),
                    };
                    somar(&mut contagem, "resposta da escrita", classe, nome);
                    estado.escrita = Some(doc.clone());
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
