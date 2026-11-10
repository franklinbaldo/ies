//! Replay da busca do destino (Refs #173): nas cassettes que gravaram a
//! tela "Enviar Processo" ou a consulta `unidade_auto_completar_envio_processo`
//! (de qualquer tela), offline, pelos mesmos leitores: as opções do
//! `selOrgao` e quantas o filtro pelo prefixo da sigla escolheria
//! (`orgao_da_sigla`); cada resposta, se veio depois de uma vazia na mesma
//! tela (a forma do termo alargado no GO), e se cada sigla com `/` da
//! resposta é escolhida só por ela (`escolher_destino`). O corpo das
//! consultas não foi gravado: o termo e o órgão de cada uma não são
//! conhecidos. Só contagens, nunca URL. `#[ignore]`: lê o cache local.
use super::destino::{escolher_destino, ler_resposta_da_consulta, orgao_da_sigla};
use super::tela::conferir_tela_de_envio;
use crate::cassette::acervo::{INSTALACOES, gravacao, percorrer, publicar};
use crate::cassette::html_da_resposta;
use crate::erro_publico::Rotulo as _;
use crate::formulario_web::RespostaDoPost;
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

fn faixa(n: usize) -> &'static str {
    match n {
        0 => "nenhum item",
        1 => "um item",
        2..=49 => "2 a 49 itens",
        _ => "50 ou mais itens",
    }
}

/// Quantas siglas com `/` a regra exata escolhe, de quantas.
fn com_barra(itens: &[(String, String)]) -> String {
    let siglas: Vec<(&str, &str)> = itens
        .iter()
        .filter_map(|(id, d)| Some((id.as_str(), d.split_once(" - ")?.0.trim())))
        .filter(|(_, s)| s.contains('/'))
        .collect();
    let escolhidas = siglas
        .iter()
        .filter(|(id, s)| escolher_destino(s, itens).is_ok_and(|d| d.id_unidade == *id))
        .count();
    format!(
        "siglas com `/` escolhidas pela exata: {escolhidas} de {}",
        siglas.len()
    )
}

#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE (padrão ~/.cache/ies); sem rede"]
fn replay_da_busca_do_destino_no_acervo() {
    let mut contagem = Contagem::new();
    let operacoes = [
        "procedimento_enviar",
        "unidade_auto_completar_envio_processo",
    ];
    let lidos = percorrer(&operacoes, |_, nome, json| {
        let mut anterior_vazia = false;
        for r in gravacao::respostas(json) {
            match (r.operacao, r.metodo) {
                ("procedimento_enviar", "GET") => {
                    anterior_vazia = false;
                    let id = gravacao::id_procedimento(r.item).unwrap_or_default();
                    let Some(h) = html_da_resposta(r.item).ok() else {
                        continue;
                    };
                    let Ok(tela) = conferir_tela_de_envio(&h, &id) else {
                        continue;
                    };
                    let unicos = tela
                        .orgaos
                        .iter()
                        .filter(|(_, rotulo)| {
                            orgao_da_sigla(&format!("{rotulo}/X"), &tela.orgaos).is_some()
                        })
                        .count();
                    let classe = format!(
                        "{} opções; {unicos} órgãos escolhíveis pelo prefixo; órgão da tela `{}`",
                        tela.orgaos.len(),
                        tela.orgao
                    );
                    somar(&mut contagem, "selOrgao", classe, nome);
                }
                ("unidade_auto_completar_envio_processo", "POST") => {
                    let corpo = gravacao::corpo(r.item);
                    let resposta = RespostaDoPost {
                        resposta: RespostaWeb {
                            status_code: i32::try_from(r.status).unwrap_or(0),
                            mime: gravacao::mime(r.item),
                            content_length: Some(corpo.len()),
                            localizacao: None,
                            corpo: Cursor::new(corpo),
                        },
                        localizacao: None,
                    };
                    let classe = match ler_resposta_da_consulta(resposta) {
                        Ok(itens) => {
                            let depois = if anterior_vazia {
                                "depois de uma vazia"
                            } else {
                                "primeira ou depois de não vazia"
                            };
                            let classe =
                                format!("{depois}: {}; {}", faixa(itens.len()), com_barra(&itens));
                            anterior_vazia = itens.is_empty();
                            classe
                        }
                        Err(falha) => format!("`{}`", falha.rotulo()),
                    };
                    somar(&mut contagem, "resposta da consulta", classe, nome);
                }
                _ => anterior_vazia = false,
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
