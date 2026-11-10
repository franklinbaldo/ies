//! Replay de conformidade do acervo público para `processo enviar` (Refs
//! #173): nas cassettes que gravaram `procedimento_enviar`, as respostas
//! passam pelos **mesmos leitores** do comando, offline: o link de enviar
//! da barra de cada árvore (cruzado com a unidade da sessão em
//! `aberto_em`), a tela "Enviar Processo" (conferência e a consulta
//! derivada), a resposta XML de cada consulta feita a partir dela, o 302
//! do POST e, por escrita, a regra de verificação: a unidade nova da
//! árvore relida é um interacao da última consulta (escolhido pela sigla), e o
//! POST derivado da tela com ele; a origem fica ou sai. Só contagens e
//! formas, nunca URL. `#[ignore]`: lê os ZIPs do cache local
//! (`IES_ACERVO_CACHE`).
use super::destino::{escolher_destino, ler_resposta_da_consulta};
use super::tela::{
    AUTOCOMPLETAR_UNIDADE, ENVIAR_PROCESSO, TelaDeEnvio, campos_da_consulta, campos_do_envio,
    conferir_tela_de_envio, redirecionamento_esperado,
};
use crate::arvore_processo::{
    AbertoEm, ler_arvore, ler_enviar_da_barra, ler_tela_do_processo, nup_declarado_na_arvore,
};
use crate::cassette::acervo::{INSTALACOES, gravacao, percorrer, publicar};
use crate::cassette::html_da_resposta;
use crate::erro_publico::Rotulo as _;
use crate::formulario_web::{
    CodificacaoDaTela, RespostaDoPost, TelaServida, derivar_consulta_ajax, derivar_submissao,
};
use crate::instalacao::Instalacao;
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

fn barra(
    inst: &Instalacao,
    html: &str,
    id: &str,
    unidade: Option<&String>,
    lida: Option<&Vec<String>>,
) -> String {
    let oferece = match ler_enviar_da_barra(inst, html, id) {
        Ok(link) => link.is_some(),
        Err(erro) => return format!("`{}`", erro.rotulo()),
    };
    let (Some(u), Some(l)) = (unidade, lida) else {
        return format!(
            "{}; unidade ou situação não lida",
            if oferece { "oferece" } else { "não oferece" }
        );
    };
    match (oferece, l.contains(u)) {
        (true, true) => "oferece; unidade em `aberto_em` (segue)".into(),
        (false, false) => "não oferece; unidade fora de `aberto_em` (recusa)".into(),
        (true, false) => "oferece; unidade fora de `aberto_em` (recusa)".into(),
        (false, true) => "não oferece; unidade em `aberto_em` (recusa)".into(),
    }
}

/// A tela conferida e a URL servida, para a consulta e o POST.
struct Tela {
    id: String,
    url: String,
    html: String,
    conferida: TelaDeEnvio,
}

fn tela_servida(tela: &Tela) -> TelaServida<'_> {
    TelaServida {
        url: &tela.url,
        html: &tela.html,
        codificacao: CodificacaoDaTela::Iso88591,
    }
}

fn resposta_gravada(interacao: &serde_json::Value, status: u64) -> RespostaDoPost<Cursor<Vec<u8>>> {
    let corpo = gravacao::corpo(interacao);
    RespostaDoPost {
        resposta: RespostaWeb {
            status_code: i32::try_from(status).unwrap_or(0),
            mime: gravacao::mime(interacao),
            content_length: Some(corpo.len()),
            localizacao: None,
            corpo: Cursor::new(corpo),
        },
        localizacao: None,
    }
}

/// O que se sabe antes do POST: id, unidade, `aberto_em` e os itens.
struct Escrita {
    id: String,
    unidade: Option<String>,
    antes: Option<Vec<String>>,
    itens: Vec<(String, String)>,
}

/// A regra de verificação do contrato, com o destino tirado da leitura.
fn verificacao(
    inst: &Instalacao,
    escrita: &Escrita,
    tela: Option<&Tela>,
    depois: Option<&Vec<String>>,
) -> String {
    let (Some(unidade), Some(antes), Some(depois)) = (&escrita.unidade, &escrita.antes, depois)
    else {
        return "unidade ou situação não lida".into();
    };
    let novas: Vec<&String> = depois.iter().filter(|s| !antes.contains(s)).collect();
    let [nova] = novas.as_slice() else {
        return format!("{} unidades novas em `aberto_em`", novas.len());
    };
    let Ok(destino) = escolher_destino(nova, &escrita.itens) else {
        return "unidade nova fora dos itens da consulta".into();
    };
    let origem = if depois.contains(unidade) {
        "origem mantida"
    } else {
        "origem fechada"
    };
    let derivado = tela.map_or("sem tela", |tela| {
        let manter = depois.contains(unidade);
        let campos = campos_do_envio(&tela.conferida, &destino, manter, false);
        let pares: Vec<(&str, &str)> = campos.iter().map(|(n, v)| (*n, v.as_str())).collect();
        match derivar_submissao(&ENVIAR_PROCESSO, inst, &tela_servida(tela), "", &pares) {
            Ok(_) => "POST derivado",
            Err(_) => "POST não derivado",
        }
    });
    format!("verificada: destino da consulta aberto, {origem}; {derivado}")
}

#[test]
#[allow(clippy::too_many_lines)] // Um laço por gravação, como os replays de concluir e reabrir.
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE (padrão ~/.cache/ies); sem rede"]
fn replay_de_processo_enviar_no_acervo() {
    let mut contagem = Contagem::new();
    let lidos = percorrer(&["procedimento_enviar"], |_, nome, json| {
        let Some(inst) = gravacao::instalacao(json) else {
            return;
        };
        let mut unidade: Option<String> = None;
        let mut aberto_em: BTreeMap<String, Option<Vec<String>>> = BTreeMap::new();
        let mut tela: Option<Tela> = None;
        let mut itens: Vec<(String, String)> = Vec::new();
        let mut escrita: Option<Escrita> = None;
        let mut ultima: &str = "";
        for r in gravacao::respostas(json) {
            let id = gravacao::id_procedimento(r.interacao)
                .or_else(|| gravacao::id_da_pesquisa_rapida(r.interacao))
                .unwrap_or_default();
            let html = || html_da_resposta(r.interacao).ok();
            match (r.operacao, r.metodo, r.status) {
                ("procedimento_trabalhar", "GET", 200) => {
                    if let Some(t) =
                        html().and_then(|h| ler_tela_do_processo(&inst, &h, &id, None).ok())
                    {
                        unidade = Some(t.unidade);
                    }
                }
                ("procedimento_visualizar", "GET", 200) => {
                    let Some(h) = html() else { continue };
                    let lida = nup_declarado_na_arvore(&h)
                        .and_then(|nup| ler_arvore(&h, &nup, &id).ok())
                        .and_then(|a| siglas(&a.aberto_em));
                    let classe = barra(&inst, &h, &id, unidade.as_ref(), lida.as_ref());
                    somar(&mut contagem, "link na barra", classe, nome);
                    if escrita.as_ref().is_some_and(|e| e.id == id)
                        && let Some(e) = escrita.take()
                    {
                        let v = verificacao(
                            &inst,
                            &e,
                            tela.as_ref().filter(|t| t.id == id),
                            lida.as_ref(),
                        );
                        somar(&mut contagem, "verificação", v, nome);
                    }
                    aberto_em.insert(id.clone(), lida);
                }
                ("procedimento_enviar", "GET", 200) => {
                    let Some(h) = html() else { continue };
                    let url = r.interacao["response"]["url"]
                        .as_str()
                        .unwrap_or("")
                        .to_owned();
                    let classe = match conferir_tela_de_envio(&h, &id) {
                        Ok(conferida) => {
                            let forma = if conferida.unidade_atual {
                                "4.1/5.0"
                            } else {
                                "4.0"
                            };
                            let nova = Tela {
                                id: id.clone(),
                                url,
                                html: h,
                                conferida,
                            };
                            let campos = campos_da_consulta(&nova.conferida, "SIGLA");
                            let consulta = derivar_consulta_ajax(
                                &AUTOCOMPLETAR_UNIDADE,
                                &inst,
                                &tela_servida(&nova),
                                &nova.conferida.autocompletar,
                                &campos,
                            );
                            let c = if consulta.is_ok() {
                                "consulta derivada"
                            } else {
                                "consulta não derivada"
                            };
                            let classe = format!(
                                "conferida (forma {forma}, órgão `{}`); {c}",
                                nova.conferida.orgao
                            );
                            tela = Some(nova);
                            classe
                        }
                        Err(erro) => format!("`{}`", erro.rotulo()),
                    };
                    somar(&mut contagem, "tela de envio", classe, nome);
                }
                ("unidade_auto_completar_envio_processo", "POST", status) => {
                    let de = if ultima == "procedimento_enviar" {
                        "da tela de envio"
                    } else {
                        "de outra tela"
                    };
                    let classe =
                        match ler_resposta_da_consulta(resposta_gravada(r.interacao, status)) {
                            Ok(lidos) => {
                                let n = match lidos.len() {
                                    0 => "nenhum interacao",
                                    1 => "um interacao",
                                    _ => "vários itens",
                                };
                                if ultima == "procedimento_enviar" {
                                    itens = lidos;
                                }
                                format!("{de}: {n}")
                            }
                            Err(falha) => format!("{de}: `{}`", falha.rotulo()),
                        };
                    somar(&mut contagem, "resposta da consulta", classe, nome);
                }
                ("procedimento_enviar", "POST", status) => {
                    let classe = match status {
                        302 if redirecionamento_esperado(
                            gravacao::localizacao(r.interacao),
                            &gravacao::url_sem_consulta(json, r.interacao),
                            &id,
                        ) =>
                        {
                            "302 esperado".to_owned()
                        }
                        302 => "302 não reconhecido".to_owned(),
                        outro => format!("status {outro}"),
                    };
                    somar(&mut contagem, "resposta do envio", classe, nome);
                    escrita = Some(Escrita {
                        id: id.clone(),
                        unidade: unidade.clone(),
                        antes: aberto_em.get(&id).cloned().flatten(),
                        itens: std::mem::take(&mut itens),
                    });
                }
                _ => {}
            }
            if r.operacao != "unidade_auto_completar_envio_processo" {
                ultima = r.operacao;
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
