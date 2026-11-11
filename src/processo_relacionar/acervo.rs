//! Replay de conformidade do acervo público para `processo relacionar`: nas
//! cassettes que gravaram `procedimento_relacionar` ou
//! `procedimento_excluir_relacionamento`, as respostas passam pelos **mesmos
//! leitores** do comando, offline: a tela (`ler_linhas`), a consulta declarada
//! (`url_complementar`), a ação de remoção (`url_exclusao`), o XML do destino
//! (`xml_da_consulta` e `ler_destino`), o 302 de aceite (`aceite_da_remocao`) e
//! a tela relida depois dele. O predicado de releitura é replicado aqui, pois
//! está inline em `remover` e `adicionar`. Só contagens e formas, nunca URL,
//! cookie, hash ou corpo. `#[ignore]`: lê os ZIPs do cache local.
use super::escrita::{aceite_da_remocao, action_do_processo};
use super::tela::{
    DestinoConsultado, Relacionamento, ler_destino, ler_linhas, url_complementar, url_exclusao,
};
use super::{mesmo_protocolo, xml_da_consulta};
use crate::cassette::acervo::{INSTALACOES, gravacao, percorrer, publicar};
use crate::cassette::html_da_resposta;
use crate::transporte::RespostaWeb;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Cursor;

type Contagem = BTreeMap<(&'static str, String), BTreeMap<String, u64>>;

const TELA: &str = "1. tela de relacionamentos (`ler_linhas`)";
const CONSULTA: &str = "2. consulta declarada na tela (`url_complementar`)";
const ACAO: &str = "3. ação de remoção publicada (`url_exclusao`)";
const DESTINO: &str = "4. consulta do destino `protocolo_RI1023` (`ler_destino`)";
const ADICAO: &str = "5. 302 de `procedimento_relacionar` (`aceite`)";
const REMOCAO: &str = "6. 302 de `procedimento_excluir_relacionamento` (`aceite`)";
const RELIDA: &str = "7. tela relida após 302 aceito (`ler_linhas`)";

const ADICAO_OK: &str = "vínculo do destino confirmado";
const REMOCAO_OK: &str = "uma relação removida e as demais preservadas";

/// As classes que a gravação comprova; qualquer outra é um achado.
fn esperada(leitor: &str, classe: &str) -> bool {
    match leitor {
        TELA | DESTINO => classe.starts_with("lida ("),
        CONSULTA => classe == "consulta declarada",
        ACAO => classe == "ação de remoção publicada",
        ADICAO | REMOCAO => classe == "302 aceito",
        RELIDA => classe == ADICAO_OK || classe == REMOCAO_OK,
        _ => false,
    }
}

/// Um relacionamento lido: id do processo e NUP, o que a comparação usa.
type Par = (String, String);

fn presente(linhas: &[Relacionamento], id: &str, nup: &str) -> bool {
    linhas
        .iter()
        .any(|l| l.id == id && mesmo_protocolo(&l.nup, nup))
}

/// O que a releitura deve comprovar depois de um 302 aceito.
enum Pos {
    Adicao { destino: Par, antes: Vec<Par> },
    Remocao { antes: Vec<Par> },
}

impl Pos {
    fn verificar(&self, linhas: &[Relacionamento]) -> &'static str {
        match self {
            Pos::Adicao { destino, antes } => {
                let ok = presente(linhas, &destino.0, &destino.1)
                    && antes.iter().all(|(i, n)| presente(linhas, i, n));
                if ok {
                    ADICAO_OK
                } else {
                    "destino ausente ou anterior perdido"
                }
            }
            Pos::Remocao { antes } => {
                let removidos: Vec<&Par> = antes
                    .iter()
                    .filter(|(i, n)| !presente(linhas, i, n))
                    .collect();
                let ok = match removidos.as_slice() {
                    [(id, nup)] => {
                        !linhas
                            .iter()
                            .any(|l| l.id == *id || mesmo_protocolo(&l.nup, nup))
                            && antes
                                .iter()
                                .filter(|(i, _)| i != id)
                                .all(|(i, n)| presente(linhas, i, n))
                    }
                    _ => false,
                };
                if ok {
                    REMOCAO_OK
                } else {
                    "remoção não comprovada"
                }
            }
        }
    }
}

/// A última tela lida: o processo, os vínculos e a ação de remoção publicada.
struct TelaLida {
    id: String,
    pares: Vec<Par>,
    acao: Option<String>,
}

#[derive(Default)]
struct Estado {
    tela: Option<TelaLida>,
    destino: Option<DestinoConsultado>,
    pendente: Option<Pos>,
}

impl Estado {
    fn tela(&mut self, interacao: &Value) -> Vec<(&'static str, String)> {
        let html = match html_da_resposta(interacao) {
            Ok(html) => html,
            Err(erro) => return vec![(TELA, format!("HTML ilegível: `{erro}`"))],
        };
        let consulta = if url_complementar(&html).is_some() {
            "consulta declarada"
        } else {
            "consulta não declarada"
        };
        let mut leituras = vec![(CONSULTA, consulta.to_owned())];
        match ler_linhas(&html) {
            Ok(linhas) => {
                let forma = if linhas.is_empty() {
                    "lida (sem relacionamentos)"
                } else {
                    "lida (com relacionamentos)"
                };
                leituras.push((TELA, forma.to_owned()));
                if let Some(pos) = self.pendente.take() {
                    leituras.push((RELIDA, pos.verificar(&linhas).to_owned()));
                }
                self.tela = Some(TelaLida {
                    id: gravacao::id_procedimento(interacao).unwrap_or_default(),
                    pares: linhas
                        .iter()
                        .map(|l| (l.id.clone(), l.nup.clone()))
                        .collect(),
                    acao: url_exclusao(&html),
                });
            }
            Err(erro) => {
                leituras.push((TELA, format!("`{erro:?}`")));
                self.tela = None;
            }
        }
        leituras
    }

    fn consulta(&mut self, interacao: &Value) -> (&'static str, String) {
        let corpo = gravacao::corpo(interacao);
        let resposta = RespostaWeb {
            status_code: 200,
            mime: gravacao::mime(interacao),
            content_length: Some(corpo.len()),
            localizacao: None,
            corpo: Cursor::new(corpo),
        };
        let classe = match xml_da_consulta(resposta).map(|x| ler_destino(&x)) {
            Err(erro) => format!("`{erro:?}`"),
            Ok(Err(erro)) => format!("`{erro:?}`"),
            Ok(Ok(destino)) => {
                let forma = if destino.tipo.is_some() {
                    "lida (com tipo)"
                } else {
                    "lida (sem tipo)"
                };
                self.destino = Some(destino);
                forma.to_owned()
            }
        };
        (DESTINO, classe)
    }

    /// A ação publicada na tela deve apontar para o processo aberto.
    fn acao_de_remocao(&self) -> &'static str {
        match self
            .tela
            .as_ref()
            .and_then(|t| t.acao.as_deref().map(|a| (a, &t.id)))
        {
            Some((url, id)) if action_do_processo(url, id) => "ação de remoção publicada",
            _ => "ação de remoção ausente ou de outro processo",
        }
    }

    fn aceite(&mut self, json: &Value, interacao: &Value, remocao: bool) -> String {
        let (id, antes) = match &self.tela {
            Some(t) => (t.id.clone(), t.pares.clone()),
            None => return "sem tela lida antes".to_owned(),
        };
        let url = gravacao::url_sem_consulta(json, interacao);
        let aceito =
            gravacao::localizacao(interacao).is_some_and(|loc| aceite_da_remocao(loc, &url, &id));
        if !aceito {
            return "302 não aceito".to_owned();
        }
        self.pendente = Some(if remocao {
            Pos::Remocao { antes }
        } else {
            match self.destino.as_ref() {
                Some(d) => Pos::Adicao {
                    destino: (d.id.clone(), d.nup.clone()),
                    antes,
                },
                None => return "destino não lido".to_owned(),
            }
        });
        "302 aceito".to_owned()
    }
}

fn replay(json: &Value) -> Vec<(&'static str, String)> {
    let mut estado = Estado::default();
    let mut leituras = Vec::new();
    for r in gravacao::respostas(json) {
        match (r.operacao, r.metodo, r.status) {
            ("procedimento_relacionar", "GET", 200) => leituras.extend(estado.tela(r.interacao)),
            ("protocolo_RI1023", "POST", 200) => leituras.push(estado.consulta(r.interacao)),
            ("procedimento_relacionar", "POST", 302) => {
                let classe = estado.aceite(json, r.interacao, false);
                leituras.push((ADICAO, classe));
            }
            ("procedimento_excluir_relacionamento", "POST", 302) => {
                leituras.push((ACAO, estado.acao_de_remocao().to_owned()));
                let classe = estado.aceite(json, r.interacao, true);
                leituras.push((REMOCAO, classe));
            }
            ("procedimento_relacionar", "POST", status) => {
                leituras.push((ADICAO, format!("status {status} (não confirmada)")));
            }
            ("procedimento_excluir_relacionamento", "POST", status) => {
                leituras.push((REMOCAO, format!("status {status} (não confirmada)")));
            }
            _ => {}
        }
    }
    leituras
}

#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE (padrão ~/.cache/ies); sem rede"]
fn replay_de_processo_relacionar_no_acervo() {
    let mut contagem = Contagem::new();
    let operacoes = [
        "procedimento_relacionar",
        "procedimento_excluir_relacionamento",
    ];
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
    let aceitos = |leitor: &'static str| -> u64 {
        contagem
            .get(&(leitor, "302 aceito".to_owned()))
            .map_or(0, |por| por.values().sum())
    };
    assert!(lidos > 0 && total(TELA) > 0 && total(CONSULTA) > 0 && total(DESTINO) > 0);
    assert!(aceitos(ADICAO) > 0, "nenhuma adição aceita foi lida");
    assert!(aceitos(REMOCAO) > 0, "nenhuma remoção aceita foi lida");
    assert_eq!(
        total(RELIDA),
        aceitos(ADICAO) + aceitos(REMOCAO),
        "toda escrita aceita é relida"
    );
}
