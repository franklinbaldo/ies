//! Replay de conformidade do acervo público para `processo comentar`: nas
//! cassettes que gravaram `comentario_listar` ou `comentario_cadastrar`, as
//! respostas passam pelos **mesmos leitores** do comando, offline: a lista
//! (`ler_tela`), o link de Novo Comentário publicado nela
//! (`link_do_cadastro`), o 302 da lista vazia para o cadastro do mesmo
//! processo (`conferir_endereco_relativo`), o formulário de cadastro
//! (`ler_tela`), o 302 do POST (`id_do_aceite`) e a lista relida depois de
//! um aceite. Só contagens e formas, nunca URL, cookie, hash ou corpo.
//! `#[ignore]`: lê os ZIPs do cache local (`IES_ACERVO_CACHE`).
//!
//! O corpo do POST não é gravado no acervo: a releitura confere só que o id
//! aceito aparece exatamente uma vez. O conteúdo do efeito fica `NOT_RUN`.
use super::{id_do_aceite, link_do_cadastro};
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::endereco::conferir_endereco_relativo;
use crate::cassette::acervo::{INSTALACOES, gravacao, percorrer, publicar};
use crate::cassette::html_da_resposta;
use crate::instalacao::Instalacao;
use crate::processo_comentarios::{ComentariosLidos, TelaDosComentarios, ler_tela};
use crate::tela_html::HtmlServido;
use std::collections::BTreeMap;
use std::fmt::Write as _;

type Contagem = BTreeMap<(&'static str, String), BTreeMap<String, u64>>;

const LISTA: &str = "1. lista de comentários (`ler_tela`)";
const LINK: &str = "2. link de Novo Comentário na lista (`link_do_cadastro`)";
const VAZIA: &str = "3. 302 da lista vazia (`conferir_endereco_relativo`)";
const CADASTRO: &str = "4. formulário de cadastro (`ler_tela`)";
const ESCRITA: &str = "5. resposta do POST `comentario_cadastrar` (`id_do_aceite`)";
const RELEITURA: &str = "6. lista relida após o 302 aceito (id do aceito)";

/// As classes que a gravação comprova; qualquer outra é um achado.
fn esperada(leitor: &str, classe: &str) -> bool {
    match leitor {
        LISTA => classe.starts_with("lida") || classe == "cadastro sem comentários",
        LINK => classe == "conferido",
        VAZIA => classe == "302 de lista vazia",
        CADASTRO => classe == "lido (sem comentários)",
        ESCRITA => classe == "302 aceito",
        RELEITURA => classe == "id do aceito uma vez (conteúdo NOT_RUN)",
        _ => false,
    }
}

/// O que a gravação mostrou até o passo: o id do último 302 aceito, à
/// espera da releitura da lista.
#[derive(Default)]
struct Estado {
    aceito_pendente: Option<String>,
}

fn html_servido(interacao: &serde_json::Value) -> Result<String, String> {
    html_da_resposta(interacao).map_err(|erro| format!("HTML ilegível: `{erro}`"))
}

/// Sem o corpo do POST no acervo, só a identidade é verificável: o id aceito
/// aparece exatamente uma vez na lista relida.
fn releitura(lidos: &ComentariosLidos, aceito: &str) -> String {
    match lidos.itens.iter().filter(|item| item.id == aceito).count() {
        0 => "ausente na releitura".to_owned(),
        1 => "id do aceito uma vez (conteúdo NOT_RUN)".to_owned(),
        n => format!("id do aceito {n} vezes"),
    }
}

/// O 302 de `comentario_listar` só é lista vazia se leva ao cadastro
/// assinado do mesmo processo, como na produção.
fn lista_vazia(json: &serde_json::Value, interacao: &serde_json::Value, id: &str) -> String {
    let (Some(instalacao), Some(local)) =
        (gravacao::instalacao(json), gravacao::localizacao(interacao))
    else {
        return "302 sem Location".to_owned();
    };
    let base = format!("{}/", instalacao.url_base());
    let relativo = local.strip_prefix(base.as_str()).unwrap_or(local);
    match conferir_endereco_relativo(&instalacao, relativo, AcaoDoSei::ComentarioCadastrar, id) {
        Ok(_) => "302 de lista vazia".to_owned(),
        Err(recusa) => format!("302 fora da forma: `{recusa:?}`"),
    }
}

impl Estado {
    fn lista(
        &mut self,
        interacao: &serde_json::Value,
        id: &str,
        instalacao: &Instalacao,
    ) -> Vec<(&'static str, String)> {
        let html = HtmlServido::de_teste(&match html_servido(interacao) {
            Ok(html) => html,
            Err(erro) => return vec![(LISTA, erro)],
        });
        let mut leituras = Vec::new();
        // A primeira lista depois de um aceite é a releitura, lida ou não.
        let pendente = self.aceito_pendente.take();
        match ler_tela(&html) {
            Ok(TelaDosComentarios::Lista(lidos)) => {
                let n = lidos.itens.len();
                leituras.push((LISTA, format!("lida ({n} comentário(s))")));
                if let Some(aceito) = pendente {
                    leituras.push((RELEITURA, releitura(&lidos, &aceito)));
                }
            }
            Ok(TelaDosComentarios::CadastroSemComentarios) => {
                leituras.push((LISTA, "cadastro sem comentários".to_owned()));
                if pendente.is_some() {
                    leituras.push((RELEITURA, "cadastro no lugar da lista".to_owned()));
                }
            }
            Err(erro) => {
                leituras.push((LISTA, format!("`{erro:?}`")));
                if pendente.is_some() {
                    leituras.push((RELEITURA, format!("`{erro:?}`")));
                }
            }
        }
        let link = match link_do_cadastro(instalacao, &html, id) {
            Ok(_) => "conferido".to_owned(),
            Err(erro) => format!("`{erro:?}`"),
        };
        leituras.push((LINK, link));
        leituras
    }

    fn cadastro(interacao: &serde_json::Value) -> (&'static str, String) {
        let classe = match html_servido(interacao) {
            Err(erro) => erro,
            Ok(html) => match ler_tela(&HtmlServido::de_teste(&html)) {
                Ok(TelaDosComentarios::CadastroSemComentarios) => {
                    "lido (sem comentários)".to_owned()
                }
                Ok(TelaDosComentarios::Lista(_)) => "lista (fora da forma)".to_owned(),
                Err(erro) => format!("`{erro:?}`"),
            },
        };
        (CADASTRO, classe)
    }

    fn escrita(
        &mut self,
        json: &serde_json::Value,
        r: &gravacao::Resposta<'_>,
        id: &str,
    ) -> (&'static str, String) {
        let url = gravacao::url_sem_consulta(json, r.interacao);
        let aceito = gravacao::localizacao(r.interacao)
            .and_then(|local| id_do_aceite(local, &url, id))
            .map(str::to_owned);
        let classe = if aceito.is_some() {
            "302 aceito"
        } else {
            "302 não aceito"
        };
        self.aceito_pendente = aceito;
        (ESCRITA, classe.to_owned())
    }
}

fn replay(json: &serde_json::Value) -> Vec<(&'static str, String)> {
    let Some(instalacao) = gravacao::instalacao(json) else {
        return vec![(LISTA, "instalação ilegível".to_owned())];
    };
    let mut estado = Estado::default();
    let mut leituras = Vec::new();
    for r in gravacao::respostas(json) {
        let id = gravacao::id_procedimento(r.interacao).unwrap_or_default();
        match (r.operacao, r.metodo, r.status) {
            ("comentario_listar", "GET" | "POST", 200) => {
                leituras.extend(estado.lista(r.interacao, &id, &instalacao));
            }
            ("comentario_listar", "GET", 302) => {
                leituras.push((VAZIA, lista_vazia(json, r.interacao, &id)));
            }
            ("comentario_cadastrar", "GET", 200) => leituras.push(Estado::cadastro(r.interacao)),
            ("comentario_cadastrar", "POST", 302) => leituras.push(estado.escrita(json, &r, &id)),
            ("comentario_cadastrar", metodo, status) => {
                leituras.push((ESCRITA, format!("{metodo} {status} fora da forma")));
            }
            _ => {}
        }
    }
    leituras
}

#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE (padrão ~/.cache/ies); sem rede"]
fn replay_de_processo_comentar_no_acervo() {
    let mut contagem = Contagem::new();
    let operacoes = ["comentario_listar", "comentario_cadastrar"];
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
    let aceitos = total(ESCRITA);
    assert!(lidos > 0 && total(LISTA) > 0 && total(LINK) > 0 && total(CADASTRO) > 0);
    assert!(aceitos > 0, "nenhum 302 do POST gravado foi lido");
    assert_eq!(total(RELEITURA), aceitos, "toda escrita aceita é relida");
}
