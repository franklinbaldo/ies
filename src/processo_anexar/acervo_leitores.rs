//! Replay de conformidade do acervo público para `processo anexar`: nas
//! cassettes que gravaram `procedimento_anexar` ou
//! `procedimento_anexacao_verificar`, as respostas passam pelos **mesmos
//! leitores** do comando, offline: a tela `frmProcedimentoAnexar`
//! (`ler_tela` e as consultas AJAX declaradas nela), o XML de
//! `protocolo_RI1023` (`ler_destino`), o de `procedimento_anexacao_verificar`
//! (`ler_confirmacao`), a decisão (`decidir`), o 302 do POST (`aceite`) e,
//! depois de um 302 aceito, a lista relida na tela seguinte. Só contagens e
//! formas, nunca URL, cookie, hash ou corpo. `#[ignore]`: lê os ZIPs do
//! cache local (`IES_ACERVO_CACHE`).
use super::fluxo::escrita::aceite;
use super::fluxo::url_da_consulta;
use super::{Destino, decidir, ler_confirmacao, ler_destino, ler_tela};
use crate::arvore_processo::nup_declarado_na_arvore;
use crate::cassette::acervo::{INSTALACOES, gravacao, percorrer, publicar};
use crate::cassette::html_da_resposta;
use crate::transporte::{RespostaWeb, decodificar_resposta_xml};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::io::Cursor;

type Contagem = BTreeMap<(&'static str, String), BTreeMap<String, u64>>;

const TELA: &str = "1. tela de anexação (`ler_tela`)";
const CONSULTAS: &str = "2. consultas declaradas na tela (`url_da_consulta`)";
const DESTINO: &str = "3. destino `protocolo_RI1023` (`ler_destino`)";
const CONFIRMACAO: &str = "4. assinaturas `procedimento_anexacao_verificar` (`ler_confirmacao`)";
const DECISAO: &str = "5. decisão (`decidir`)";
const ESCRITA: &str = "6. resposta do POST `procedimento_anexar` (`aceite`)";
const LISTA: &str = "7. lista relida após o 302 aceito";

/// As declarações das duas consultas na tela, como `prever` as procura.
const DECLARACOES: [&str; 2] = [
    "new infraAjaxComplementar('txtProtocolo','",
    "new infraAjaxComplementar(null,'",
];

/// As classes que a gravação comprova; qualquer outra é um achado.
fn esperada(leitor: &str, classe: &str) -> bool {
    match leitor {
        TELA | DESTINO => classe.starts_with("lida"),
        CONSULTAS => classe == "ambas declaradas",
        CONFIRMACAO => classe == "`S`" || classe == "`N`",
        // `ProcessoJaAnexado` só sai quando a tela lista o destino.
        DECISAO => classe == "Ok(true)" || classe == "`Err(ProcessoJaAnexado)` (listado na tela)",
        ESCRITA => classe == "302 aceito" || classe == "HTML 200 (escrita não confirmada)",
        LISTA => classe == "contém o anexado e os anteriores",
        _ => false,
    }
}

/// O que a gravação mostrou até o passo.
#[derive(Default)]
struct Estado {
    /// O NUP declarado na árvore, por id do processo.
    nups: HashMap<String, String>,
    /// O id do principal e os anexados da última tela lida.
    tela: Option<(String, Vec<String>)>,
    destino: Option<Destino>,
    /// O destino e os anexados de antes de um 302 aceito.
    depois_do_post: Option<(String, Vec<String>)>,
}

fn xml(interacao: &serde_json::Value, status: u64) -> Result<String, String> {
    let corpo = gravacao::corpo(interacao);
    decodificar_resposta_xml(RespostaWeb {
        status_code: i32::try_from(status).unwrap_or(0),
        mime: gravacao::mime(interacao),
        content_length: Some(corpo.len()),
        localizacao: None,
        corpo: Cursor::new(corpo),
    })
    .map_err(|falha| format!("XML não decodificado: `{falha:?}`"))
}

impl Estado {
    fn tela(&mut self, interacao: &serde_json::Value, id: &str) -> Vec<(&'static str, String)> {
        let html = match html_da_resposta(interacao) {
            Ok(html) => html,
            Err(erro) => return vec![(TELA, format!("HTML ilegível: `{erro}`"))],
        };
        let mut leituras = Vec::new();
        let consultas: Vec<String> = DECLARACOES
            .iter()
            .filter_map(|d| url_da_consulta(&html, d).err())
            .map(|erro| format!("`{erro:?}`"))
            .collect();
        let consultas = if consultas.is_empty() {
            "ambas declaradas".to_owned()
        } else {
            consultas.join(", ")
        };
        match ler_tela(&html) {
            Ok(tela) => {
                let n = tela.anexados.len();
                leituras.push((TELA, format!("lida ({n} anexado(s))")));
                if let Some((destino, anteriores)) = self.depois_do_post.take() {
                    let contem = tela.anexados.contains(&destino)
                        && anteriores.iter().all(|a| tela.anexados.contains(a));
                    let classe = if contem {
                        "contém o anexado e os anteriores"
                    } else {
                        "não contém o anexado ou um anterior"
                    };
                    leituras.push((LISTA, classe.to_owned()));
                }
                self.tela = Some((id.to_owned(), tela.anexados));
            }
            Err(erro) => {
                leituras.push((TELA, format!("`{erro:?}`")));
                self.tela = None;
            }
        }
        leituras.push((CONSULTAS, consultas));
        self.destino = None;
        leituras
    }

    fn destino(&mut self, xml: Result<String, String>) -> Vec<(&'static str, String)> {
        let lido = match xml.map(|x| ler_destino(&x)) {
            Err(classe) => return vec![(DESTINO, classe)],
            Ok(Err(erro)) => return vec![(DESTINO, format!("`{erro:?}`"))],
            Ok(Ok(destino)) => destino,
        };
        let forma = if lido.tipo.is_empty() {
            "lida (sem tipo)"
        } else {
            "lida (com tipo)"
        };
        let decisao = match &self.tela {
            None => "sem tela lida antes".to_owned(),
            Some((id, anexados)) => match self.nups.get(id) {
                None => "NUP do principal não lido na árvore".to_owned(),
                Some(nup) => match decidir(nup, id, &lido, anexados) {
                    Ok(v) => format!("Ok({v})"),
                    Err(erro) if anexados.contains(&lido.id) => {
                        format!("`Err({erro:?})` (listado na tela)")
                    }
                    Err(erro) => format!("`Err({erro:?})`"),
                },
            },
        };
        self.destino = Some(lido);
        vec![(DESTINO, forma.to_owned()), (DECISAO, decisao)]
    }

    fn escrita(&mut self, json: &serde_json::Value, r: &gravacao::Resposta<'_>) -> String {
        match r.status {
            302 => {
                let principal = self.tela.as_ref().map_or("", |(id, _)| id.as_str());
                let url = gravacao::url_sem_consulta(json, r.interacao);
                let aceito = gravacao::localizacao(r.interacao)
                    .is_some_and(|loc| aceite(loc, &url, principal));
                if !aceito {
                    return "302 não aceito".into();
                }
                self.depois_do_post = self.destino.as_ref().map(|d| {
                    let anteriores = self.tela.as_ref().map(|(_, a)| a.clone());
                    (d.id.clone(), anteriores.unwrap_or_default())
                });
                "302 aceito".into()
            }
            200 => "HTML 200 (escrita não confirmada)".into(),
            outro => format!("status {outro}"),
        }
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
            ("procedimento_anexar", "GET", 200) => {
                leituras.extend(estado.tela(r.interacao, &id));
            }
            ("protocolo_RI1023", "POST", status) => {
                leituras.extend(estado.destino(xml(r.interacao, status)));
            }
            ("procedimento_anexacao_verificar", "POST", status) => {
                let classe = match xml(r.interacao, status).map(|x| ler_confirmacao(&x)) {
                    Err(classe) => classe,
                    Ok(Ok(true)) => "`S`".to_owned(),
                    Ok(Ok(false)) => "`N`".to_owned(),
                    Ok(Err(erro)) => format!("`{erro:?}`"),
                };
                leituras.push((CONFIRMACAO, classe));
            }
            ("procedimento_anexar", "POST", _) => {
                leituras.push((ESCRITA, estado.escrita(json, &r)));
            }
            ("procedimento_anexar", metodo, status) => {
                leituras.push((TELA, format!("{metodo} {status} sem tela")));
            }
            _ => {}
        }
    }
    leituras
}

#[test]
#[ignore = "lê os ZIPs do acervo em IES_ACERVO_CACHE (padrão ~/.cache/ies); sem rede"]
fn replay_de_processo_anexar_no_acervo() {
    let mut contagem = Contagem::new();
    let operacoes = ["procedimento_anexar", "procedimento_anexacao_verificar"];
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
    let aceitos: u64 = contagem
        .get(&(ESCRITA, "302 aceito".to_owned()))
        .map_or(0, |por| por.values().sum());
    assert!(lidos > 0 && total(TELA) > 0 && total(DESTINO) > 0 && total(CONFIRMACAO) > 0);
    assert!(aceitos > 0, "nenhum 302 pós-POST gravado foi lido");
    assert_eq!(total(LISTA), aceitos, "toda anexação aceita é relida");
}
