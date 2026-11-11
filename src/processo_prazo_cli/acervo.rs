//! Replay de conformidade do acervo público para `processo prazo` (definir e
//! excluir): nas cassettes que gravaram `controle_prazo_definir` ou
//! `controle_prazo_excluir`, as respostas passam pelos **mesmos leitores** do
//! comando, offline: a tela do prazo (`ler_tela`), o 302 de confirmação
//! (`aceite_confirmacao`), o link da lista na tela do controle
//! (`link_da_lista`), a lista (`id_na_lista`), a ação publicada
//! (`action_de_exclusao`) e, depois do 302 de exclusão, a lista relida. O
//! predicado do 302 de exclusão é inline em `escrever_e_verificar`; aqui ele é
//! replicado. Só contagens e formas, nunca URL, cookie, hash ou corpo.
//! `#[ignore]`: lê os ZIPs do cache local (`IES_ACERVO_CACHE`).
use super::aceite_confirmacao;
use super::exclusao::{action_de_exclusao, id_na_lista, link_da_lista};
use crate::arvore_processo::nup_declarado_na_arvore;
use crate::cassette::acervo::{INSTALACOES, gravacao, percorrer, publicar};
use crate::cassette::html_da_resposta;
use crate::processo_prazo::{EstadoDoPrazo, ler_tela};
use crate::tela_html::HtmlServido;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt::Write as _;

type Contagem = BTreeMap<(&'static str, String), BTreeMap<String, u64>>;

const TELA: &str = "1. tela do prazo (`ler_tela`)";
const CONFIRMACAO: &str = "2. 302 de `controle_prazo_definir` (`aceite_confirmacao`)";
const DEFINIR: &str = "3. resposta do POST `controle_prazo_definir`";
const LINK: &str = "4. link da lista na tela do controle (`link_da_lista`)";
const LISTA: &str = "5. lista de prazos (`id_na_lista`)";
const ACAO: &str = "6. ação de exclusão publicada (`action_de_exclusao`)";
const EXCLUSAO: &str = "7. 302 de `controle_prazo_excluir` (predicado inline)";

const EXCLUIDO: &str = "linha do NUP ainda presente (após exclusão aceita)";
const ANTES: &str = "linha do NUP (antes da exclusão)";
const SEM_LINHA_ANTES: &str = "sem linha do NUP (antes da exclusão)";
const SEM_LINHA_DEPOIS: &str = "sem linha do NUP (após exclusão aceita)";

/// As classes que a gravação comprova; qualquer outra é um achado.
fn esperada(leitor: &str, classe: &str) -> bool {
    match leitor {
        TELA => classe.starts_with("lida ("),
        CONFIRMACAO | EXCLUSAO => classe == "302 aceito",
        // HTTP 500 não é escrita confirmada: o transporte já a recusa.
        DEFINIR => classe == "HTML 200 (releitura pelo GET)" || classe == "status 500 (recusada)",
        LINK => classe == "link da lista publicado",
        LISTA => classe == ANTES || classe == SEM_LINHA_DEPOIS,
        ACAO => classe == "ação de exclusão publicada",
        _ => false,
    }
}

fn rotulo(estado: &EstadoDoPrazo) -> &'static str {
    match estado {
        EstadoDoPrazo::SemPrazo => "lida (sem prazo)",
        EstadoDoPrazo::Data(_) => "lida (data)",
        EstadoDoPrazo::Dias(_) => "lida (em dias)",
        EstadoDoPrazo::Concluido => "lida (concluído)",
    }
}

/// O que a gravação mostrou até o passo.
#[derive(Default)]
struct Estado {
    /// O NUP declarado na árvore do processo da gravação.
    nup: Option<String>,
    /// Verdadeiro depois de um 302 de exclusão aceito.
    apos_exclusao: bool,
}

impl Estado {
    fn aceite_exclusao(&mut self, r: &gravacao::Resposta<'_>) -> &'static str {
        let aceito = gravacao::localizacao(r.interacao).is_some_and(|local| {
            local.contains("acao=controle_prazo_listar")
                && local.contains("acao_origem=controle_prazo_excluir")
        });
        if aceito {
            self.apos_exclusao = true;
            "302 aceito"
        } else {
            "302 não aceito"
        }
    }

    fn lista(&self, interacao: &Value) -> Vec<(&'static str, String)> {
        let html = match html_da_resposta(interacao) {
            Ok(html) => html,
            Err(erro) => return vec![(LISTA, format!("HTML ilegível: `{erro}`"))],
        };
        let Some(nup) = self.nup.as_deref() else {
            return vec![(LISTA, "NUP não lido na árvore".to_owned())];
        };
        let classe = match (
            id_na_lista(&HtmlServido::de_teste(&html), nup),
            self.apos_exclusao,
        ) {
            (Err(erro), _) => format!("`{erro:?}`"),
            (Ok(Some(_)), false) => ANTES.to_owned(),
            (Ok(None), false) => SEM_LINHA_ANTES.to_owned(),
            (Ok(Some(_)), true) => EXCLUIDO.to_owned(),
            (Ok(None), true) => SEM_LINHA_DEPOIS.to_owned(),
        };
        let mut leituras = vec![(LISTA, classe)];
        if !self.apos_exclusao {
            let acao = match action_de_exclusao(&html) {
                Ok(_) => "ação de exclusão publicada".to_owned(),
                Err(erro) => format!("`{erro:?}`"),
            };
            leituras.push((ACAO, acao));
        }
        leituras
    }
}

fn origem(json: &Value) -> Option<String> {
    let raiz = json["sei_root"].as_str()?;
    let caminho = json["interactions"]
        .as_array()?
        .iter()
        .filter_map(|it| it["request"]["pathname"].as_str())
        .find(|c| c.ends_with("/controlador.php"))?;
    let base = caminho.strip_suffix("/controlador.php")?;
    Some(format!("{raiz}{base}"))
}

fn replay(json: &Value) -> Vec<(&'static str, String)> {
    let mut estado = Estado::default();
    let mut leituras = Vec::new();
    let origem = origem(json).unwrap_or_default();
    let respostas = gravacao::respostas(json);
    let excluiu = respostas
        .iter()
        .any(|r| r.operacao == "controle_prazo_excluir");
    for r in &respostas {
        let id = gravacao::id_procedimento(r.interacao).unwrap_or_default();
        match (r.operacao, r.metodo, r.status) {
            ("procedimento_visualizar", "GET", 200) => {
                if estado.nup.is_none() {
                    estado.nup = html_da_resposta(r.interacao)
                        .ok()
                        .and_then(|h| nup_declarado_na_arvore(&h));
                }
            }
            ("controle_prazo_definir", "GET", 200) => {
                leituras.push((TELA, tela(r.interacao, &id)));
            }
            ("controle_prazo_definir", "POST", 302) => {
                let aceito = gravacao::localizacao(r.interacao).is_some_and(|local| {
                    let url = gravacao::url_sem_consulta(json, r.interacao);
                    aceite_confirmacao(local, &url, &id)
                });
                let classe = if aceito {
                    "302 aceito"
                } else {
                    "302 não aceito"
                };
                leituras.push((CONFIRMACAO, classe.to_owned()));
            }
            ("controle_prazo_definir", "POST", 200) => {
                leituras.push((DEFINIR, "HTML 200 (releitura pelo GET)".to_owned()));
            }
            ("controle_prazo_definir", "POST", status) => {
                leituras.push((DEFINIR, format!("status {status} (recusada)")));
            }
            ("procedimento_controlar", "GET", 200) if excluiu => {
                let classe = match html_da_resposta(r.interacao) {
                    Ok(html) => match link_da_lista(&HtmlServido::de_teste(&html), &origem) {
                        Ok(_) => "link da lista publicado".to_owned(),
                        Err(erro) => format!("`{erro:?}`"),
                    },
                    Err(erro) => format!("HTML ilegível: `{erro}`"),
                };
                leituras.push((LINK, classe));
            }
            ("controle_prazo_listar", "GET", 200) => leituras.extend(estado.lista(r.interacao)),
            ("controle_prazo_excluir", "POST", 302) => {
                leituras.push((EXCLUSAO, estado.aceite_exclusao(r).to_owned()));
            }
            ("controle_prazo_excluir", "POST", status) => {
                leituras.push((EXCLUSAO, format!("status {status} (não confirmada)")));
            }
            _ => {}
        }
    }
    leituras
}

fn tela(interacao: &Value, id: &str) -> String {
    match html_da_resposta(interacao) {
        Ok(html) => match ler_tela(&HtmlServido::de_teste(&html), id) {
            Ok(tela) => rotulo(&tela.estado).to_owned(),
            Err(erro) => format!("`{erro:?}`"),
        },
        Err(erro) => format!("HTML ilegível: `{erro}`"),
    }
}

#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE (padrão ~/.cache/ies); sem rede"]
fn replay_de_processo_prazo_no_acervo() {
    let mut contagem = Contagem::new();
    let operacoes = ["controle_prazo_definir", "controle_prazo_excluir"];
    let lidos = percorrer(&operacoes, |_, nome, json| {
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
    let total = |leitor: &str| -> u64 {
        contagem
            .iter()
            .filter(|((l, _), _)| *l == leitor)
            .flat_map(|(_, por)| por.values())
            .sum()
    };
    assert!(lidos > 0 && total(TELA) > 0 && total(LINK) > 0 && total(LISTA) > 0);
    assert!(total(ACAO) > 0 && total(EXCLUSAO) > 0 && total(DEFINIR) > 0);
    let depois = contagem
        .get(&(LISTA, SEM_LINHA_DEPOIS.to_owned()))
        .map_or(0, |por| por.values().sum::<u64>());
    assert!(depois >= total(EXCLUSAO), "toda exclusão aceita é relida");
}
