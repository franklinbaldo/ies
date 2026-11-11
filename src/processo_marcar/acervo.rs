//! Replay de conformidade do acervo público para `processo marcar`: nas
//! cassettes que gravaram `andamento_marcador_cadastrar`,
//! `andamento_marcador_gerenciar` ou `andamento_marcador_remover`, as
//! respostas passam pelos **mesmos leitores** do comando, offline: a tela
//! de gerenciamento (`ler_marcadores`, `action_de_remocao`,
//! `link_de_adicionar`), o 302 que leva ao cadastro (`conferir_endereco_relativo`),
//! as opções do formulário de cadastro (`rotulo_oferecido`), os 302 dos POSTs
//! (`aceite_da_atribuicao`, `aceite_da_remocao`) e a lista relida depois de
//! um aceite. Só contagens e formas, nunca URL, cookie, hash ou corpo.
//! `#[ignore]`: lê os ZIPs do cache local (`IES_ACERVO_CACHE`).
use super::Rotulo;
use super::catalogo::{aceite_da_atribuicao, link_de_adicionar, rotulo_oferecido};
use super::escrita::{aceite_da_remocao, action_de_remocao};
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::endereco::conferir_endereco_relativo;
use crate::arvore_processo::nup_declarado_na_arvore;
use crate::cassette::acervo::{INSTALACOES, gravacao, percorrer, publicar};
use crate::cassette::html_da_resposta;
use crate::formulario_web::{Redirecionamento302, decodificar_atributo};
use crate::html_atributos::{nos_com_id, valor_do_atributo};
use crate::marcadores_processo::{TelaDosMarcadores, ler_marcadores};
use crate::processo_obter::marcadores::sem_marcadores;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;

type Contagem = BTreeMap<(&'static str, String), BTreeMap<String, u64>>;

const TELA: &str = "1. gerenciamento de marcadores (`ler_marcadores`)";
const MARCADOR: &str = "2. ação de remoção por marcador (`action_de_remocao`)";
const CADASTRO: &str = "3. link de cadastro na tela (`link_de_adicionar`)";
const DESTINO: &str = "4. 302 do link de cadastro (`conferir_endereco_relativo`)";
const OPCOES: &str = "5. opções do formulário de cadastro (`rotulo_oferecido`)";
const ESCRITA_CADASTRO: &str = "6. 302 do POST de cadastro (`aceite_da_atribuicao`)";
const ESCRITA_REMOCAO: &str = "7. 302 do POST de remoção (`aceite_da_remocao`)";
const LISTA: &str = "8. gerenciamento relido após o 302 aceito (`ler_marcadores`)";

/// As classes que a gravação comprova; qualquer outra é um achado.
fn esperada(leitor: &str, classe: &str) -> bool {
    match leitor {
        TELA | LISTA => classe.starts_with("lida") || classe == "sem tabela",
        MARCADOR | CADASTRO | OPCOES => classe == "lido",
        DESTINO => classe == "conferido",
        ESCRITA_CADASTRO | ESCRITA_REMOCAO => classe == "302 aceito",
        _ => false,
    }
}

/// O que a gravação mostrou até o passo.
#[derive(Default)]
struct Estado {
    /// O NUP declarado na árvore, por id do processo.
    nups: HashMap<String, String>,
    /// Um 302 de escrita foi aceito e a próxima tela de gerenciamento é a releitura.
    aceito_pendente: bool,
}

/// As opções do `select#selMarcador` servido, como `Rotulo` (id e texto).
fn opcoes_do_select(html: &str) -> Vec<Rotulo> {
    let Ok(dom) = tl::parse(html, tl::ParserOptions::default()) else {
        return Vec::new();
    };
    let parser = dom.parser();
    let Ok(selects) = nos_com_id(&dom, "selMarcador") else {
        return Vec::new();
    };
    selects
        .iter()
        .filter_map(|h| h.get(parser).and_then(tl::Node::as_tag))
        .flat_map(|select| select.children().all(parser).iter())
        .filter_map(tl::Node::as_tag)
        .filter(|tag| tag.name().as_utf8_str() == "option")
        .filter_map(|tag| {
            let id = valor_do_atributo(tag, "value").ok()??;
            let nome = decodificar_atributo(&tag.inner_text(parser))?;
            Some(Rotulo { id, nome })
        })
        .collect()
}

fn html_servido(interacao: &serde_json::Value) -> Result<String, String> {
    html_da_resposta(interacao).map_err(|erro| format!("HTML ilegível: `{erro}`"))
}

fn redirecionamento(json: &serde_json::Value, r: &gravacao::Resposta<'_>) -> Redirecionamento302 {
    let url = gravacao::url_sem_consulta(json, r.interacao);
    Redirecionamento302::sintetico(gravacao::localizacao(r.interacao), &url)
}

fn resultado<T>(r: Result<T, impl std::fmt::Debug>, ok: &str) -> String {
    match r {
        Ok(_) => ok.to_owned(),
        Err(erro) => format!("`{erro:?}`"),
    }
}

impl Estado {
    fn gerenciar(
        &mut self,
        interacao: &serde_json::Value,
        id: &str,
    ) -> Vec<(&'static str, String)> {
        let html = match html_servido(interacao) {
            Ok(html) => html,
            Err(erro) => return vec![(TELA, erro)],
        };
        let mut leituras = Vec::new();
        let Some(nup) = self.nups.get(id) else {
            leituras.push((TELA, "NUP do principal não lido na árvore".to_owned()));
            return leituras;
        };
        let tela = ler_marcadores(&html, nup, id);
        match &tela {
            Ok(TelaDosMarcadores::Lidos(lidos)) => {
                let n = lidos.itens.len();
                leituras.push((TELA, format!("lida ({n} marcador(es))")));
                for marcador in &lidos.itens {
                    let classe = resultado(action_de_remocao(&html, &marcador.id), "lido");
                    leituras.push((MARCADOR, classe));
                }
            }
            Ok(TelaDosMarcadores::SemTabela) => leituras.push((TELA, "sem tabela".to_owned())),
            Err(erro) => leituras.push((TELA, format!("`{erro:?}`"))),
        }
        leituras.push((CADASTRO, resultado(link_de_adicionar(&html), "lido")));
        if std::mem::take(&mut self.aceito_pendente) {
            let classe = match tela {
                Ok(TelaDosMarcadores::Lidos(lidos)) => {
                    format!("lida ({} marcador(es))", lidos.itens.len())
                }
                Ok(TelaDosMarcadores::SemTabela) => "sem tabela".to_owned(),
                Err(erro) => format!("`{erro:?}`"),
            };
            leituras.push((LISTA, classe));
        }
        leituras
    }

    /// Com escrita aceita pendente, o 302 de gerenciamento é a releitura: a
    /// mesma regra de produção (`sem_marcadores`) diz se a lista é vazia.
    /// Uma escrita aceita gera uma linha de LISTA, conferida ou não.
    fn releitura_por_302(
        &mut self,
        json: &serde_json::Value,
        interacao: &serde_json::Value,
        id: &str,
    ) -> Option<(&'static str, String)> {
        if !self.aceito_pendente {
            return None;
        }
        let instalacao = gravacao::instalacao(json)?;
        let conferida = sem_marcadores(&instalacao, gravacao::localizacao(interacao), id);
        self.aceito_pendente = false;
        let classe = if conferida {
            "lida (0 marcador(es))"
        } else {
            "302 fora da forma"
        };
        Some((LISTA, classe.to_owned()))
    }

    fn destino(
        json: &serde_json::Value,
        interacao: &serde_json::Value,
        id: &str,
    ) -> Result<(&'static str, String), String> {
        let instalacao = gravacao::instalacao(json).ok_or("instalação ilegível")?;
        let loc = gravacao::localizacao(interacao).ok_or("302 sem Location")?;
        let relativo = loc
            .strip_prefix(&format!("{}/", instalacao.url_base()))
            .unwrap_or(loc);
        let classe = resultado(
            conferir_endereco_relativo(
                &instalacao,
                relativo,
                AcaoDoSei::AndamentoMarcadorCadastrar,
                id,
            ),
            "conferido",
        );
        Ok((DESTINO, classe))
    }

    fn opcoes(interacao: &serde_json::Value) -> Vec<(&'static str, String)> {
        let html = match html_servido(interacao) {
            Ok(html) => html,
            Err(erro) => return vec![(OPCOES, erro)],
        };
        opcoes_do_select(&html)
            .iter()
            .map(|rotulo| (OPCOES, resultado(rotulo_oferecido(&html, rotulo), "lido")))
            .collect()
    }

    fn escrita(
        &mut self,
        json: &serde_json::Value,
        r: &gravacao::Resposta<'_>,
        id: &str,
    ) -> Vec<(&'static str, String)> {
        let redir = redirecionamento(json, r);
        let (leitor, aceito) = if r.operacao == "andamento_marcador_cadastrar" {
            (ESCRITA_CADASTRO, aceite_da_atribuicao(&redir, id))
        } else {
            (ESCRITA_REMOCAO, aceite_da_remocao(&redir, id))
        };
        let classe = if aceito {
            "302 aceito"
        } else {
            "302 não aceito"
        };
        self.aceito_pendente |= aceito;
        vec![(leitor, classe.to_owned())]
    }
}

fn replay(json: &serde_json::Value) -> Vec<(&'static str, String)> {
    let mut estado = Estado::default();
    let mut leituras = Vec::new();
    for r in gravacao::respostas(json) {
        let id = gravacao::id_procedimento(r.interacao).unwrap_or_default();
        match (r.operacao, r.metodo, r.status) {
            ("procedimento_visualizar", "GET", 200) => {
                if let Some(nup) = html_da_resposta(r.interacao)
                    .ok()
                    .and_then(|h| nup_declarado_na_arvore(&h))
                {
                    estado.nups.insert(id, nup);
                }
            }
            ("andamento_marcador_gerenciar", "GET", 200) => {
                leituras.extend(estado.gerenciar(r.interacao, &id));
            }
            ("andamento_marcador_gerenciar", "GET", 302) => {
                leituras.push(
                    estado
                        .releitura_por_302(json, r.interacao, &id)
                        .unwrap_or_else(|| {
                            Estado::destino(json, r.interacao, &id)
                                .unwrap_or_else(|erro| (DESTINO, erro))
                        }),
                );
            }
            ("andamento_marcador_cadastrar", "GET", 200) => {
                leituras.extend(Estado::opcoes(r.interacao));
            }
            ("andamento_marcador_cadastrar" | "andamento_marcador_remover", "POST", 302) => {
                leituras.extend(estado.escrita(json, &r, &id));
            }
            ("andamento_marcador_cadastrar" | "andamento_marcador_remover", metodo, status) => {
                leituras.push((ESCRITA_CADASTRO, format!("{metodo} {status} fora da forma")));
            }
            _ => {}
        }
    }
    leituras
}

#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE (padrão ~/.cache/ies); sem rede"]
fn replay_de_processo_marcar_no_acervo() {
    let mut contagem = Contagem::new();
    let operacoes = [
        "andamento_marcador_cadastrar",
        "andamento_marcador_gerenciar",
        "andamento_marcador_remover",
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
    let aceitos = total(ESCRITA_CADASTRO) + total(ESCRITA_REMOCAO);
    assert!(lidos > 0 && total(TELA) > 0 && total(DESTINO) > 0 && total(OPCOES) > 0);
    assert!(total(MARCADOR) > 0 && total(CADASTRO) > 0 && total(ESCRITA_REMOCAO) > 0);
    assert!(aceitos > 0, "nenhum 302 de escrita gravado foi lido");
    assert_eq!(total(LISTA), aceitos, "toda escrita aceita é relida");
}
