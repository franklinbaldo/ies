//! Leitura de grupos de envio por identidade publicada na unidade atual.
//! Forma de tela: Todos @caabe699, src/todos/grupos_envio.py; PEN1 4.0.12.
//! Primeira fatia: lista completa de uma página; paginação ainda não suportada.
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::{ConsultaDoControlador, menciona_acao};
use crate::tela_html::{Elemento, HtmlServido, arvore};
use crate::transporte::sessao::ReferenciasDaSessao;
use crate::visao_do_controle::{EnvioDoControle, EnvioHttps};
use crate::{Cli, configuracao, controle_reconhecido, transporte, unidade_lupa};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, clap::Subcommand)]
pub(crate) enum Acao {
    /// Lista grupos da unidade; recusa lista paginada nesta fatia.
    Listar,
    /// Consulta o grupo pelo ID publicado na lista.
    Obter { id: String },
}

#[derive(Debug, clap::Args)]
pub(crate) struct GrupoEnvio {
    #[command(subcommand)]
    acao: Acao,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
enum ErroGrupo {
    ListaInesperada,
    ListaIncompleta,
    IdentidadeAmbigua,
    GrupoAusente,
    ConsultaNaoOferecida,
    ConsultaInesperada,
    UnidadeInesperada,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct Item {
    id: String,
    nome: String,
    descricao: String,
}

// URLs assinadas não aparecem na saída, nem em Debug.
struct Linha {
    item: Item,
    consulta: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct Membro {
    id: String,
    nome: String,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct Grupo {
    id: String,
    nome: String,
    descricao: String,
    unidades: Vec<Membro>,
}

fn texto(no: Elemento<'_>) -> String {
    no.texto().split_whitespace().collect::<Vec<_>>().join(" ")
}

fn ler_lista(html: &HtmlServido) -> Result<Vec<Linha>, ErroGrupo> {
    let dom = arvore(html);
    let forms: Vec<_> = dom.com_id("frmGrupoUnidadeLista").collect();
    let [form] = forms.as_slice() else {
        return Err(ErroGrupo::ListaInesperada);
    };
    if form.nome() != "form" {
        return Err(ErroGrupo::ListaInesperada);
    }
    if crate::paginacao_controle::controles_de_paginacao(html) {
        return Err(ErroGrupo::ListaIncompleta);
    }
    let tabelas: Vec<_> = form
        .elementos("table")
        .filter(|t| t.elementos("th").any(|th| texto(th) == "Nome"))
        .collect();
    let [tabela] = tabelas.as_slice() else {
        return Err(ErroGrupo::ListaInesperada);
    };
    let legendas: Vec<_> = tabela.elementos("caption").collect();
    let [legenda] = legendas.as_slice() else {
        return Err(ErroGrupo::ListaInesperada);
    };
    let contagem = crate::legenda::ler_legenda(&texto(*legenda))
        .map_err(|_| ErroGrupo::ListaInesperada)?;
    let mut colunas = None;
    let mut linhas = Vec::new();
    let mut ids = HashSet::new();
    for tr in tabela.elementos("tr") {
        let th: Vec<_> = tr.filhos().filter(|n| n.nome() == "th").map(texto).collect();
        if !th.is_empty() {
            if colunas.is_some() {
                return Err(ErroGrupo::ListaInesperada);
            }
            colunas = Some(th);
            continue;
        }
        let td: Vec<_> = tr.filhos().filter(|n| n.nome() == "td").collect();
        if td.is_empty() {
            continue;
        }
        let headers = colunas.as_ref().ok_or(ErroGrupo::ListaInesperada)?;
        if td.len() != headers.len() {
            return Err(ErroGrupo::ListaInesperada);
        }
        let celula = |nome: &str| -> Result<String, ErroGrupo> {
            let achados: Vec<_> = headers.iter().zip(&td).filter(|(h, _)| h.as_str() == nome).collect();
            match achados.as_slice() {
                [(_, no)] => Ok(texto(**no)),
                _ => Err(ErroGrupo::ListaInesperada),
            }
        };
        let checkboxes: Vec<_> = tr.elementos("input")
            .filter(|n| n.atributo("type") == Some("checkbox")).collect();
        let [checkbox] = checkboxes.as_slice() else {
            return Err(ErroGrupo::ListaInesperada);
        };
        let id = checkbox.atributo("value").filter(|id| !id.is_empty())
            .ok_or(ErroGrupo::ListaInesperada)?.to_owned();
        if !ids.insert(id.clone()) {
            return Err(ErroGrupo::IdentidadeAmbigua);
        }
        let nome = celula("Nome")?;
        if nome.is_empty() {
            return Err(ErroGrupo::ListaInesperada);
        }
        let descricao = if headers.iter().any(|h| h == "Descrição") {
            celula("Descrição")?
        } else {
            String::new()
        };
        let consulta = tr.elementos("a").filter_map(|a| a.atributo("href"))
            .filter(|href| menciona_acao(href, AcaoDoSei::GrupoUnidadeConsultar))
            .map(str::to_owned).collect();
        linhas.push(Linha { item: Item { id, nome, descricao }, consulta });
    }
    if colunas.is_none() {
        return Err(ErroGrupo::ListaInesperada);
    }
    if linhas.len() != contagem.total || contagem.faixa.is_some_and(|f| f.inicio != 1 || f.fim != contagem.total) {
        return Err(ErroGrupo::ListaIncompleta);
    }
    Ok(linhas)
}

fn consulta_da_linha(linha: &Linha, url: &str, origem: &str) -> anyhow::Result<String> {
    let [href] = linha.consulta.as_slice() else {
        return Err(ErroGrupo::ConsultaNaoOferecida.into());
    };
    let (_, query) = href.split_once('?').ok_or(ErroGrupo::ConsultaInesperada)?;
    let query = ConsultaDoControlador::ler(query)?;
    if query.unico("id_grupo_unidade")? != linha.item.id {
        return Err(ErroGrupo::ConsultaInesperada.into());
    }
    Ok(unidade_lupa::destino_de_candidatos(
        vec![href.clone()], url, origem, AcaoDoSei::GrupoUnidadeConsultar,
    )?)
}

fn ler_grupo(html: &HtmlServido, id: &str) -> Result<Grupo, ErroGrupo> {
    let dom = arvore(html);
    let forms: Vec<_> = dom.com_id("frmGrupoCadastro").collect();
    let [form] = forms.as_slice() else {
        return Err(ErroGrupo::ConsultaInesperada);
    };
    if form.nome() != "form" {
        return Err(ErroGrupo::ConsultaInesperada);
    }
    let campo = |tag, nome| {
        let achados: Vec<_> = form.elementos(tag).filter(|n| n.atributo("name") == Some(nome)).collect();
        match achados.as_slice() {
            [no] => Ok(*no),
            _ => Err(ErroGrupo::ConsultaInesperada),
        }
    };
    if campo("input", "hdnIdGrupoUnidade")?.atributo("value") != Some(id) {
        return Err(ErroGrupo::ConsultaInesperada);
    }
    let nome = campo("input", "txtNome")?.atributo("value")
        .filter(|n| !n.trim().is_empty()).ok_or(ErroGrupo::ConsultaInesperada)?.to_owned();
    let descricao = texto(campo("textarea", "txaDescricao")?);
    campo("input", "hdnUnidades")?;
    let select = campo("select", "selUnidades")?;
    let mut unidades = Vec::new();
    let mut ids = HashSet::new();
    for opcao in select.elementos("option") {
        let id = opcao.atributo("value").ok_or(ErroGrupo::ConsultaInesperada)?;
        if id.is_empty() {
            continue;
        }
        let nome = texto(opcao);
        if nome.is_empty() || !ids.insert(id) {
            return Err(ErroGrupo::IdentidadeAmbigua);
        }
        unidades.push(Membro { id: id.to_owned(), nome });
    }
    Ok(Grupo { id: id.to_owned(), nome, descricao, unidades })
}

fn ler_via<V: EnvioDoControle>(
    sessao: &ReferenciasDaSessao,
    unidade_esperada: Option<&str>,
    acao: &Acao,
    envio: &mut V,
) -> anyhow::Result<serde_json::Value> {
    let controle = transporte::envio_post::ler_tela_via(
        sessao.url_do_controle(), sessao.cookie(), sessao.origem(),
        |url, cookie| envio.get(url, cookie),
    )?;
    let unidade = controle_reconhecido::unidade_do_controle(&controle.html)?;
    if unidade_esperada.is_some_and(|v| v != unidade) {
        return Err(ErroGrupo::UnidadeInesperada.into());
    }
    let url = unidade_lupa::destino(
        &controle.html, &controle.url, sessao.origem(), AcaoDoSei::GrupoUnidadeListar,
    )?;
    let cookie = sessao.cookie().valor()?;
    let html = transporte::decodificar_resposta(envio.get(&url, cookie)?)?;
    let linhas = ler_lista(&html)?;
    match acao {
        Acao::Listar => {
            let itens: Vec<_> = linhas.into_iter().map(|linha| linha.item).collect();
            Ok(serde_json::json!({"unidade": unidade, "origem": "web", "total": itens.len(), "itens": itens}))
        }
        Acao::Obter { id } => {
            let linha = linhas.iter().find(|l| l.item.id == *id).ok_or(ErroGrupo::GrupoAusente)?;
            let consulta = consulta_da_linha(linha, &url, sessao.origem())?;
            let html = transporte::decodificar_resposta(envio.get(&consulta, cookie)?)?;
            let grupo = ler_grupo(&html, id)?;
            Ok(serde_json::json!({"unidade": unidade, "origem": "web", "grupo": grupo}))
        }
    }
}

pub(crate) fn executar(cli: &Cli, grupo: &GrupoEnvio) -> anyhow::Result<()> {
    let arquivos = configuracao::arquivos_de_acesso()?;
    let sessao = ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem, &arquivos.cookie)?;
    let valor = ler_via(&sessao, cli.unidade.as_deref(), &grupo.acao, &mut EnvioHttps)?;
    if cli.json {
        println!("{}", serde_json::to_string_pretty(&valor)?);
    } else {
        // O mesmo modelo tipado serializado, sem URL ou cookie.
        println!("{}", serde_json::to_string_pretty(&valor)?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const GRUPO: &str = "<form id='frmGrupoCadastro'><input name='hdnIdGrupoUnidade' value='6'><input name='txtNome' value='Grupo'><textarea name='txaDescricao'>Descrição</textarea><input name='hdnUnidades' value=''><select name='selUnidades'><option value='11'>Mesmo nome</option><option value='12'>Mesmo nome</option></select></form>";

    #[test]
    fn consulta_preserva_identidade_dos_membros_homonimos() {
        let grupo = ler_grupo(&HtmlServido::de_teste(GRUPO), "6").unwrap();
        assert_eq!(grupo.unidades.into_iter().map(|u| u.id).collect::<Vec<_>>(), ["11", "12"]);
    }

    #[test]
    fn consulta_recusa_id_trocado_e_membro_duplicado() {
        assert_eq!(ler_grupo(&HtmlServido::de_teste(GRUPO), "7"), Err(ErroGrupo::ConsultaInesperada));
        let repetido = GRUPO.replace("value='12'", "value='11'");
        assert_eq!(ler_grupo(&HtmlServido::de_teste(&repetido), "6"), Err(ErroGrupo::IdentidadeAmbigua));
    }

    #[test]
    fn lista_preserva_homonimos_e_recusa_contagem_incompleta() {
        let html = "<form id='frmGrupoUnidadeLista'><table><caption>Grupos de Envio (2 registros):</caption><tr><th></th><th>Nome</th></tr><tr><td><input type='checkbox' value='4'></td><td>Mesmo</td></tr><tr><td><input type='checkbox' value='6'></td><td>Mesmo</td></tr></table></form>";
        let linhas = ler_lista(&HtmlServido::de_teste(html)).unwrap();
        assert_eq!(linhas.into_iter().map(|l| l.item.id).collect::<Vec<_>>(), ["4", "6"]);
        let incompleta = html.replace("2 registros", "3 registros");
        assert!(matches!(ler_lista(&HtmlServido::de_teste(&incompleta)), Err(ErroGrupo::ListaIncompleta)));
    }
}
