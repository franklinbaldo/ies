//! Leitura de grupos de envio por identidade publicada na unidade atual.
//! Forma de tela: Todos @caabe699, `src/todos/grupos_envio.py`; PEN1 4.0.12.
//! Página observada; paginação Infra ainda é hipótese para grupos.
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::{ConsultaDoControlador, menciona_acao};
use crate::tela_html::{Elemento, HtmlServido, arvore};
use crate::transporte::sessao::ReferenciasDaSessao;
use crate::visao_do_controle::{EnvioDoControle, EnvioHttps};
use crate::{Cli, configuracao, controle_reconhecido, transporte, unidade_lupa};
use serde::Serialize;
use std::collections::HashSet;

mod paginacao;

#[derive(Debug, clap::Subcommand)]
pub(crate) enum Acao {
    /// Lista grupos da unidade.
    Listar {
        #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
        pagina: Option<u32>,
    },
    /// Consulta o grupo pelo ID publicado na página observada.
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
    GrupoNaoEncontradoNaPagina,
    PaginaNaoOferecida,
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
pub(crate) struct Membro {
    pub(crate) id: String,
    pub(crate) nome: String,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub(crate) struct Grupo {
    pub(crate) id: String,
    pub(crate) nome: String,
    pub(crate) descricao: String,
    pub(crate) unidades: Vec<Membro>,
}

fn texto(no: Elemento<'_>) -> String {
    no.texto().split_whitespace().collect::<Vec<_>>().join(" ")
}

fn ler_pagina(html: &HtmlServido) -> Result<(crate::legenda::Legenda, Vec<Linha>), ErroGrupo> {
    let dom = arvore(html);
    let forms: Vec<_> = dom.com_id("frmGrupoUnidadeLista").collect();
    let [form] = forms.as_slice() else {
        return Err(ErroGrupo::ListaInesperada);
    };
    if form.nome() != "form" {
        return Err(ErroGrupo::ListaInesperada);
    }
    let paginada = crate::paginacao_controle::controles_de_paginacao(html);
    let tabelas: Vec<_> = form
        .elementos("table")
        .filter(|t| t.elementos("th").any(|th| texto(th) == "Nome"))
        .collect();
    // Forma de vazio observada em outras listagens Infra (Todos @caabe699,
    // sei/menu.py). Para grupos, ainda é hipótese: exigir o formulário próprio.
    if tabelas.is_empty() && !paginada {
        let areas: Vec<_> = form
            .elementos("div")
            .filter(|n| n.atributo("id") == Some("divInfraAreaTabela"))
            .collect();
        if let [area] = areas.as_slice()
            && area.elementos("table").next().is_none()
        {
            let aviso = texto(*area);
            let contadores: Vec<_> = form
                .elementos("input")
                .filter(|n| n.atributo("name") == Some("hdnInfraNroItens"))
                .map(|n| n.atributo("value"))
                .collect();
            if (aviso == "Nenhum registro encontrado."
                && (contadores.is_empty() || contadores == [Some("0")]))
                || (aviso.is_empty() && contadores == [Some("0")])
            {
                return Ok((
                    crate::legenda::Legenda {
                        total: 0,
                        faixa: None,
                    },
                    Vec::new(),
                ));
            }
        }
    }
    let [tabela] = tabelas.as_slice() else {
        return Err(ErroGrupo::ListaInesperada);
    };
    let legendas: Vec<_> = tabela.elementos("caption").collect();
    let [legenda] = legendas.as_slice() else {
        return Err(ErroGrupo::ListaInesperada);
    };
    let contagem =
        crate::legenda::ler_legenda(&texto(*legenda)).map_err(|_| ErroGrupo::ListaInesperada)?;
    let linhas = ler_linhas(*tabela)?;
    let completa = match &contagem.faixa {
        Some(f) => {
            f.inicio > 0
                && f.inicio <= f.fim
                && f.fim <= contagem.total
                && linhas.len() == f.fim - f.inicio + 1
        }
        None => !paginada && linhas.len() == contagem.total,
    };
    if !completa {
        return Err(ErroGrupo::ListaIncompleta);
    }
    Ok((contagem, linhas))
}

#[cfg(test)]
fn ler_lista(html: &HtmlServido) -> Result<Vec<Linha>, ErroGrupo> {
    let (legenda, linhas) = ler_pagina(html)?;
    if linhas.len() != legenda.total {
        return Err(ErroGrupo::ListaIncompleta);
    }
    Ok(linhas)
}

fn ler_linhas(tabela: Elemento<'_>) -> Result<Vec<Linha>, ErroGrupo> {
    let mut colunas = None;
    let mut linhas = Vec::new();
    let mut ids = HashSet::new();
    for tr in tabela.elementos("tr") {
        let th: Vec<_> = tr
            .filhos()
            .filter(|n| n.nome() == "th")
            .map(texto)
            .collect();
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
            let achados: Vec<_> = headers
                .iter()
                .zip(&td)
                .filter(|(h, _)| h.as_str() == nome)
                .collect();
            match achados.as_slice() {
                [(_, no)] => Ok(texto(**no)),
                _ => Err(ErroGrupo::ListaInesperada),
            }
        };
        let checkboxes: Vec<_> = tr
            .elementos("input")
            .filter(|n| n.atributo("type") == Some("checkbox"))
            .collect();
        let [checkbox] = checkboxes.as_slice() else {
            return Err(ErroGrupo::ListaInesperada);
        };
        let id = checkbox
            .atributo("value")
            .filter(|id| !id.is_empty())
            .ok_or(ErroGrupo::ListaInesperada)?
            .to_owned();
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
        let consulta = tr
            .elementos("a")
            .filter_map(|a| a.atributo("href"))
            .filter(|href| menciona_acao(href, AcaoDoSei::GrupoUnidadeConsultar))
            .map(str::to_owned)
            .collect();
        linhas.push(Linha {
            item: Item {
                id,
                nome,
                descricao,
            },
            consulta,
        });
    }
    if colunas.is_none() {
        return Err(ErroGrupo::ListaInesperada);
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
        vec![href.clone()],
        url,
        origem,
        AcaoDoSei::GrupoUnidadeConsultar,
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
        let achados: Vec<_> = form
            .elementos(tag)
            .filter(|n| n.atributo("name") == Some(nome))
            .collect();
        match achados.as_slice() {
            [no] => Ok(*no),
            _ => Err(ErroGrupo::ConsultaInesperada),
        }
    };
    if campo("input", "hdnIdGrupoUnidade")?.atributo("value") != Some(id) {
        return Err(ErroGrupo::ConsultaInesperada);
    }
    let nome = campo("input", "txtNome")?
        .atributo("value")
        .filter(|n| !n.trim().is_empty())
        .ok_or(ErroGrupo::ConsultaInesperada)?
        .to_owned();
    let descricao = texto(campo("textarea", "txaDescricao")?);
    campo("input", "hdnUnidades")?;
    let select = campo("select", "selUnidades")?;
    let mut unidades = Vec::new();
    let mut ids = HashSet::new();
    for opcao in select.elementos("option") {
        let id = opcao
            .atributo("value")
            .ok_or(ErroGrupo::ConsultaInesperada)?;
        if id.is_empty() {
            continue;
        }
        let nome = texto(opcao);
        if nome.is_empty() || !ids.insert(id) {
            return Err(ErroGrupo::IdentidadeAmbigua);
        }
        unidades.push(Membro {
            id: id.to_owned(),
            nome,
        });
    }
    Ok(Grupo {
        id: id.to_owned(),
        nome,
        descricao,
        unidades,
    })
}

fn listar_via<V: EnvioDoControle>(
    sessao: &ReferenciasDaSessao,
    unidade_esperada: Option<&str>,
    pagina: Option<u32>,
    envio: &mut V,
) -> anyhow::Result<(String, String, crate::legenda::Legenda, Vec<Linha>)> {
    let controle = transporte::envio_post::ler_tela_via(
        sessao.url_do_controle(),
        sessao.cookie(),
        sessao.origem(),
        |url, cookie| envio.get(url, cookie),
    )?;
    let unidade = controle_reconhecido::unidade_do_controle(&controle.html)?;
    if unidade_esperada.is_some_and(|v| v != unidade) {
        return Err(ErroGrupo::UnidadeInesperada.into());
    }
    let url = unidade_lupa::destino(
        &controle.html,
        &controle.url,
        sessao.origem(),
        AcaoDoSei::GrupoUnidadeListar,
    )?;
    let cookie = sessao.cookie().valor()?;
    let resposta = envio.get(&url, cookie)?;
    let codificacao = crate::pagina_do_controle::codificacao_da_tela(&resposta.mime);
    let html = transporte::decodificar_resposta(resposta)?;
    let (url, legenda, linhas) = paginacao::ler_observada(
        html,
        url,
        codificacao,
        sessao,
        &controle.instalacao,
        pagina,
        envio,
    )?;
    Ok((unidade, url, legenda, linhas))
}

/// Consulta tipada pelo link publicado na página observada.
pub(crate) fn obter_via<V: EnvioDoControle>(
    sessao: &ReferenciasDaSessao,
    unidade_esperada: Option<&str>,
    id: &str,
    envio: &mut V,
) -> anyhow::Result<(String, Grupo)> {
    let (unidade, url, _, linhas) = listar_via(sessao, unidade_esperada, None, envio)?;
    let linha = linhas
        .iter()
        .find(|l| l.item.id == id)
        .ok_or(ErroGrupo::GrupoNaoEncontradoNaPagina)?;
    let consulta = consulta_da_linha(linha, &url, sessao.origem())?;
    let cookie = sessao.cookie().valor()?;
    let html = transporte::decodificar_resposta(envio.get(&consulta, cookie)?)?;
    Ok((unidade, ler_grupo(&html, id)?))
}

fn ler_via<V: EnvioDoControle>(
    sessao: &ReferenciasDaSessao,
    unidade_esperada: Option<&str>,
    acao: &Acao,
    envio: &mut V,
) -> anyhow::Result<serde_json::Value> {
    match acao {
        Acao::Listar { pagina } => {
            let (unidade, _, legenda, linhas) =
                listar_via(sessao, unidade_esperada, *pagina, envio)?;
            let itens: Vec<_> = linhas.into_iter().map(|linha| linha.item).collect();
            Ok(paginacao::saida(unidade, legenda, itens))
        }
        Acao::Obter { id } => {
            let (unidade, grupo) = obter_via(sessao, unidade_esperada, id, envio)?;
            Ok(serde_json::json!({"unidade": unidade, "origem": "web", "grupo": grupo}))
        }
    }
}

pub(crate) fn executar(cli: &Cli, grupo: &GrupoEnvio) -> anyhow::Result<()> {
    let arquivos = configuracao::arquivos_de_acesso()?;
    let sessao =
        ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem, &arquivos.cookie)?;
    let valor = ler_via(
        &sessao,
        cli.unidade.as_deref(),
        &grupo.acao,
        &mut EnvioHttps,
    )?;
    println!("{}", serde_json::to_string_pretty(&valor)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const GRUPO: &str = "<form id='frmGrupoCadastro'><input name='hdnIdGrupoUnidade' value='6'><input name='txtNome' value='Grupo'><textarea name='txaDescricao'>Descrição</textarea><input name='hdnUnidades' value=''><select name='selUnidades'><option value='11'>Mesmo nome</option><option value='12'>Mesmo nome</option></select></form>";

    #[test]
    fn consulta_preserva_identidade_dos_membros_homonimos() {
        let grupo = ler_grupo(&HtmlServido::de_teste(GRUPO), "6").unwrap();
        assert_eq!(
            grupo.unidades.into_iter().map(|u| u.id).collect::<Vec<_>>(),
            ["11", "12"]
        );
    }

    #[test]
    fn consulta_tipificada_preserva_contrato_json_e_ids_opacos() {
        let html = GRUPO.replace("value='11'", "value='0011'");
        let grupo = ler_grupo(&HtmlServido::de_teste(&html), "6").unwrap();
        assert_eq!(grupo.unidades[0].id, "0011");
        assert_eq!(grupo.unidades[1].id, "12");
        assert_eq!(
            serde_json::to_value(&grupo).unwrap(),
            serde_json::json!({
                "id": "6",
                "nome": "Grupo",
                "descricao": "Descrição",
                "unidades": [
                    {"id": "0011", "nome": "Mesmo nome"},
                    {"id": "12", "nome": "Mesmo nome"}
                ]
            })
        );
    }

    #[test]
    fn consulta_recusa_id_trocado_e_membro_duplicado() {
        assert_eq!(
            ler_grupo(&HtmlServido::de_teste(GRUPO), "7"),
            Err(ErroGrupo::ConsultaInesperada)
        );
        let repetido = GRUPO.replace("value='12'", "value='11'");
        assert_eq!(
            ler_grupo(&HtmlServido::de_teste(&repetido), "6"),
            Err(ErroGrupo::IdentidadeAmbigua)
        );
    }

    #[test]
    fn lista_vazia_exige_evidencia_no_formulario_proprio() {
        let aviso = "<form id='frmGrupoUnidadeLista'><div id='divInfraAreaTabela'><label>Nenhum registro encontrado.</label></div></form>";
        let contador = "<form id='frmGrupoUnidadeLista'><input name='hdnInfraNroItens' value='0'><div id='divInfraAreaTabela'></div></form>";
        for html in [aviso, contador] {
            assert!(ler_lista(&HtmlServido::de_teste(html)).unwrap().is_empty());
        }

        for html in [
            aviso.replace("frmGrupoUnidadeLista", "outro"),
            aviso.replace("Nenhum registro encontrado.", "Acesso negado"),
            aviso.replace("</label>", "</label><table></table>"),
            aviso.replace(
                "</form>",
                "<select id='selInfraPaginacaoSuperior'></select></form>",
            ),
            aviso.replace(
                "</form>",
                "<input name='hdnInfraNroItens' value='1'></form>",
            ),
            contador.replace("value='0'", "value='2'"),
            contador.replace(
                "value='0'",
                "value='0'><input name='hdnInfraNroItens' value='0'",
            ),
            contador.replace("<div id='divInfraAreaTabela'></div>", ""),
        ] {
            assert!(ler_lista(&HtmlServido::de_teste(&html)).is_err());
        }
    }

    #[test]
    fn lista_preserva_homonimos_e_recusa_contagem_incompleta() {
        let html = "<form id='frmGrupoUnidadeLista'><table><caption>Grupos de Envio (2 registros):</caption><tr><th></th><th>Nome</th></tr><tr><td><input type='checkbox' value='4'></td><td>Mesmo</td></tr><tr><td><input type='checkbox' value='6'></td><td>Mesmo</td></tr></table></form>";
        let linhas = ler_lista(&HtmlServido::de_teste(html)).unwrap();
        assert_eq!(
            linhas.into_iter().map(|l| l.item.id).collect::<Vec<_>>(),
            ["4", "6"]
        );
        let incompleta = html.replace("2 registros", "3 registros");
        assert!(matches!(
            ler_lista(&HtmlServido::de_teste(&incompleta)),
            Err(ErroGrupo::ListaIncompleta)
        ));
    }
}

#[cfg(test)]
#[test]
#[ignore = "exige ZIP original PEN1 verificado em IES_ACERVO_CACHE"]
fn cassette_pen1_grupos_envio_preservam_ids() {
    use crate::cassette::acervo::gravacao;
    use crate::cassette::{ZipDoAcervo, conferir_cassette_ate, html_da_resposta};
    use std::path::Path;

    let cache = std::env::var_os("IES_ACERVO_CACHE").expect("IES_ACERVO_CACHE");
    let zip = ZipDoAcervo::ler(&Path::new(&cache).join("todos-sei-cassettes-pen1.zip")).unwrap();
    let card = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("cassettes/pen1/grupo-envio-ciclo-pen1--cf5fac874b66.md");
    let gravada = conferir_cassette_ate(&card, &zip, 120_000_000).unwrap();

    // Ordinais e oráculo independente: Todos @caabe699 tests/test_grupo_envio_010.py.
    let tela = |acao: &str, ordem: usize| {
        let r = gravacao::respostas(&gravada.json)
            .into_iter()
            .filter(|r| r.operacao == acao && r.metodo == "GET")
            .nth(ordem)
            .expect("resposta esperada no original");
        assert_eq!(r.status, 200);
        HtmlServido::de_teste(&html_da_resposta(r.interacao).expect("HTML da cassette"))
    };
    let lista = ler_lista(&tela("grupo_unidade_listar", 2)).unwrap();
    let ids: Vec<_> = lista.iter().map(|l| l.item.id.as_str()).collect();
    assert_eq!(ids, ["4", "6"]);
    let grupo = ler_grupo(&tela("grupo_unidade_consultar", 0), "6").unwrap();
    assert_eq!(
        grupo
            .unidades
            .iter()
            .map(|u| u.id.as_str())
            .collect::<Vec<_>>(),
        ["110000002"]
    );
    let alterado = ler_grupo(&tela("grupo_unidade_consultar", 2), "6").unwrap();
    assert_eq!(
        alterado
            .unidades
            .iter()
            .map(|u| u.id.as_str())
            .collect::<Vec<_>>(),
        ["110000002", "110000003"]
    );
    let antes = ler_lista(&tela("grupo_unidade_listar", 0)).unwrap();
    assert_eq!(
        antes.iter().map(|l| l.item.id.as_str()).collect::<Vec<_>>(),
        ["4"]
    );
    let depois = ler_lista(&tela("grupo_unidade_listar", 13)).unwrap();
    // A exclusão do grupo 6 não deve apagar nem modificar o grupo preexistente.
    assert_eq!(
        depois.iter().map(|l| &l.item).collect::<Vec<_>>(),
        antes.iter().map(|l| &l.item).collect::<Vec<_>>()
    );
}
