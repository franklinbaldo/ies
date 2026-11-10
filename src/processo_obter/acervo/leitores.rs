//! A identidade pedida sai da gravação, nunca do nome do cenário: o id da
//! URL servida e o NUP declarado pelo nó `PROCESSO` de uma árvore do mesmo
//! id na mesma cassette. Sem os dois, a leitura é `nao_aplicavel`.
use super::{Leitor, Registro, Resultado};
use crate::arvore_processo::endereco::conferir_endereco_relativo;
use crate::arvore_processo::{ler_tela_do_processo, nup_declarado_na_arvore};
use crate::cassette::acervo::gravacao::{self, Resposta};
use crate::cassette::html_da_resposta;
use crate::erro_publico::Rotulo as _;
use crate::historico_processo::{TipoDoHistorico, ler_andamentos};
use crate::html_atributos::nos_com_id;
use crate::instalacao::Instalacao;
use crate::marcadores_processo::{TelaDosMarcadores, ler_marcadores};
use crate::propriedades_processo::{FonteDasPropriedades, ler_propriedades};
use std::collections::HashMap;

/// O que vale para todas as respostas de uma cassette.
pub(super) struct Cassette<'a> {
    pub(super) card: &'a str,
    pub(super) instalacao_nome: &'a str,
    pub(super) instalacao: Instalacao,
    /// id → NUP declarado pelas árvores; `None` se duas árvores divergem.
    pub(super) nups: HashMap<String, Option<String>>,
}

impl Cassette<'_> {
    pub(super) fn registro(&self, leitor: Leitor, passo: usize, resultado: Resultado) -> Registro {
        Registro {
            leitor,
            instalacao: self.instalacao_nome.to_owned(),
            card: self.card.to_owned(),
            passo,
            resultado,
            contagens: Vec::new(),
            formas: Vec::new(),
        }
    }

    pub(super) fn nup(&self, id: &str) -> Result<&str, &'static str> {
        match self.nups.get(id) {
            Some(Some(nup)) => Ok(nup),
            Some(None) => Err("nups_divergentes_nas_arvores_do_mesmo_id"),
            None => Err("sem_arvore_do_mesmo_id_na_cassette"),
        }
    }
}

pub(super) fn replay(card: &str, instalacao_nome: &str, json: &serde_json::Value) -> Vec<Registro> {
    let respostas = gravacao::respostas(json);
    let Some(instalacao) = gravacao::instalacao(json) else {
        return Vec::new();
    };
    let mut nups: HashMap<String, Option<String>> = HashMap::new();
    for r in respostas.iter().filter(|r| e_arvore(r)) {
        let (Some(id), Ok(html)) = (gravacao::id_procedimento(r.item), html_da_resposta(r.item))
        else {
            continue;
        };
        if let Some(nup) = nup_declarado_na_arvore(&html) {
            let entrada = nups.entry(id).or_insert_with(|| Some(nup.clone()));
            if entrada.as_deref() != Some(nup.as_str()) {
                *entrada = None;
            }
        }
    }
    let cassette = Cassette {
        card,
        instalacao_nome,
        instalacao,
        nups,
    };
    let mut registros = Vec::new();
    for r in &respostas {
        registros.extend(ler(&cassette, r));
    }
    registros
}

fn e_arvore(r: &Resposta<'_>) -> bool {
    r.operacao == "procedimento_visualizar" && r.metodo == "GET" && r.status == 200
}

fn ler(c: &Cassette<'_>, r: &Resposta<'_>) -> Vec<Registro> {
    let leitor = match (r.operacao, r.metodo, r.status) {
        ("procedimento_visualizar", "GET", 200) => Leitor::Arvore,
        ("procedimento_trabalhar", "GET", 200) => Leitor::Tela,
        ("procedimento_consultar_historico", "GET" | "POST", 200) => Leitor::Andamentos,
        ("procedimento_alterar" | "procedimento_consultar", "GET", 200) => Leitor::Propriedades,
        ("andamento_marcador_gerenciar", "GET", 200 | 302) => Leitor::Marcadores,
        _ => return Vec::new(),
    };
    let id = gravacao::id_procedimento(r.item).or_else(|| {
        (leitor == Leitor::Tela)
            .then(|| gravacao::id_da_pesquisa_rapida(r.item))
            .flatten()
    });
    let Some(id) = id else {
        let motivo = Resultado::NaoAplicavel("id_procedimento_nao_publicado_na_url_servida");
        return vec![c.registro(leitor, r.passo, motivo)];
    };
    if r.status == 302 {
        return vec![marcadores_302(c, r, &id)];
    }
    let html = match html_da_resposta(r.item) {
        Ok(html) => html,
        // O transporte da CLI recusa o mesmo corpo (`MAX_HTML_BYTES`).
        Err(_) if corpo_acima_do_limite(r.item) => {
            let recusa = Resultado::Recusado("html_acima_do_limite_do_transporte");
            return vec![c.registro(leitor, r.passo, recusa)];
        }
        Err(_) => {
            let motivo = Resultado::NaoAplicavel("corpo_nao_e_html_decodificavel");
            return vec![c.registro(leitor, r.passo, motivo)];
        }
    };
    match leitor {
        Leitor::Arvore => super::arvore::ler(c, r, &html, &id),
        Leitor::Tela => vec![tela(c, r, &html, &id)],
        _ => {
            let nup = match c.nup(&id) {
                Ok(nup) => nup,
                Err(motivo) => {
                    return vec![c.registro(leitor, r.passo, Resultado::NaoAplicavel(motivo))];
                }
            };
            vec![match leitor {
                Leitor::Andamentos => andamentos(c, r, &html, nup),
                Leitor::Propriedades => propriedades(c, r, &html, nup, &id),
                _ => marcadores(c, r, &html, nup, &id),
            }]
        }
    }
}

fn corpo_acima_do_limite(item: &serde_json::Value) -> bool {
    use base64::Engine as _;
    item["response"]["body_base64"]
        .as_str()
        .and_then(|b| base64::engine::general_purpose::STANDARD.decode(b).ok())
        .is_some_and(|b| b.len() > crate::MAX_HTML_BYTES)
}

fn tela(c: &Cassette<'_>, r: &Resposta<'_>, html: &str, id: &str) -> Registro {
    let resultado = match ler_tela_do_processo(&c.instalacao, html, id, None) {
        Ok(_) => Resultado::Lido,
        Err(erro) => Resultado::Recusado(erro.rotulo()),
    };
    c.registro(Leitor::Tela, r.passo, resultado)
}

/// A `div#divInfraAreaPaginacaoSuperior` (por `html_atributos`): vazia,
/// com algum conteúdo, ausente ou repetida.
fn paginador(html: &str) -> &'static str {
    let Ok(dom) = tl::parse(html, tl::ParserOptions::default()) else {
        return "html_nao_interpretado";
    };
    let parser = dom.parser();
    let Ok(nos) = nos_com_id(&dom, "divInfraAreaPaginacaoSuperior") else {
        return "html_nao_interpretado";
    };
    match nos.as_slice() {
        [] => "paginador_ausente",
        [no] => {
            let vazio = no
                .get(parser)
                .and_then(tl::Node::as_tag)
                .is_some_and(|div| {
                    let filhos = div.children();
                    filhos.all(parser).iter().all(|n| n.as_tag().is_none())
                        && div.inner_text(parser).trim().is_empty()
                });
            if vazio {
                "paginador_vazio"
            } else {
                "paginador_com_conteudo"
            }
        }
        _ => "paginadores_repetidos",
    }
}

fn andamentos(c: &Cassette<'_>, r: &Resposta<'_>, html: &str, nup: &str) -> Registro {
    let mut registro = c.registro(Leitor::Andamentos, r.passo, Resultado::Lido);
    registro.formas.push(paginador(html).to_owned());
    match ler_andamentos(html, nup) {
        Ok(lidos) => {
            let linhas = lidos.itens.len() as u64;
            registro.contagens = vec![
                ("linhas", linhas),
                ("total_declarado", lidos.total_declarado),
            ];
            registro.formas.push(format!(
                "{}_{}",
                r.metodo.to_ascii_lowercase(),
                match lidos.tipo {
                    TipoDoHistorico::Resumido => "resumido",
                    TipoDoHistorico::Completo => "completo",
                }
            ));
            if linhas < lidos.total_declarado {
                registro.formas.push("linhas_abaixo_do_total".into());
            }
        }
        Err(erro) => registro.resultado = Resultado::Recusado(erro.rotulo()),
    }
    registro
}

fn propriedades(c: &Cassette<'_>, r: &Resposta<'_>, html: &str, nup: &str, id: &str) -> Registro {
    let fonte = if r.operacao == "procedimento_consultar" {
        FonteDasPropriedades::Consultar
    } else {
        FonteDasPropriedades::Alterar
    };
    let mut registro = c.registro(Leitor::Propriedades, r.passo, Resultado::Lido);
    match ler_propriedades(html, nup, id, fonte) {
        Ok(lidas) => {
            registro.contagens = vec![
                ("interessados", lidas.interessados.len() as u64),
                ("assuntos", lidas.assuntos.len() as u64),
            ];
            registro
                .formas
                .push(lidas.nivel_de_acesso.como_texto().into());
            registro.formas.push(fonte.acao().into());
        }
        Err(erro) => registro.resultado = Resultado::Recusado(erro.rotulo()),
    }
    registro
}

fn marcadores(c: &Cassette<'_>, r: &Resposta<'_>, html: &str, nup: &str, id: &str) -> Registro {
    let mut registro = c.registro(Leitor::Marcadores, r.passo, Resultado::Lido);
    match ler_marcadores(html, nup, id) {
        Ok(TelaDosMarcadores::Lidos(lidos)) => {
            registro.contagens = vec![
                ("itens", lidos.itens.len() as u64),
                ("total_declarado", lidos.total_declarado as u64),
            ];
            registro.formas.push("tabela".into());
        }
        Ok(TelaDosMarcadores::SemTabela) => {
            registro.resultado =
                Resultado::NaoAplicavel("tabela_de_marcadores_nao_publicada_na_tela");
        }
        Err(erro) => registro.resultado = Resultado::Recusado(erro.rotulo()),
    }
    registro
}

/// O 302 de zero marcadores: `andamento_marcador_cadastrar` do mesmo id,
/// a mesma regra do fluxo.
fn marcadores_302(c: &Cassette<'_>, r: &Resposta<'_>, id: &str) -> Registro {
    let base = format!("{}/", c.instalacao.url_base());
    let resultado = match gravacao::localizacao(r.item) {
        Some(local) => {
            let relativo = local.strip_prefix(base.as_str()).unwrap_or(local);
            match conferir_endereco_relativo(
                &c.instalacao,
                relativo,
                "andamento_marcador_cadastrar",
                id,
            ) {
                Ok(_) => Resultado::Lido,
                Err(_) => Resultado::Recusado("redirecionamento_nao_reconhecido"),
            }
        }
        None => Resultado::NaoAplicavel("location_nao_gravado"),
    };
    let mut registro = c.registro(Leitor::Marcadores, r.passo, resultado);
    if registro.resultado == Resultado::Lido {
        registro.formas.push("302_sem_marcadores".into());
    }
    registro
}
