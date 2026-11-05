//! Leitor offline da tabela de comentários (`tblComentarios`).
//! Oráculo de comportamento: Todos tests/test_comentarios_010.py @323eb01;
//! SEI 4.x: linha de dados + trComentario; SEI 5.0: coluna Texto.
//! Não usa rede, não executa JS e não infere ausência de tabela como lista vazia.
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::{atributos, nos_com_id};
use std::collections::HashSet;

type Tag<'p> = &'p tl::HTMLTag<'p>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComentarioLido {
    pub id: String,
    pub data_hora: String,
    pub unidade: String,
    pub usuario: String,
    pub texto: String,
    pub protocolo: Option<String>,
    pub tipo: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisaoDosComentarios {
    Processo,
    Todos,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComentariosLidos {
    pub visao: VisaoDosComentarios,
    pub total_declarado: usize,
    pub itens: Vec<ComentarioLido>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TelaDosComentarios {
    /// O GET `comentario_listar` redireciona a este cadastro quando a lista está vazia.
    CadastroSemComentarios,
    Lista(ComentariosLidos),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErroComentarios {
    HtmlNaoInterpretado,
    TelaNaoReconhecida,
    FormulariosDaListaRepetidos,
    VisaoNaoPublicada,
    TabelaAusente,
    TabelasRepetidas,
    LegendaInvalida,
    CabecalhoInvalido,
    LinhaInvalida,
    TextoDoComentarioNaoPublicado,
    IdDoComentarioRepetido,
    ListaIncompleta,
}

impl ErroComentarios {
    pub fn subcodigo(self) -> &'static str {
        match self {
            Self::HtmlNaoInterpretado => "html_dos_comentarios_nao_interpretado",
            Self::TelaNaoReconhecida => "tela_de_comentarios_nao_reconhecida",
            Self::FormulariosDaListaRepetidos => "formularios_de_comentarios_repetidos",
            Self::VisaoNaoPublicada => "visao_dos_comentarios_nao_publicada",
            Self::TabelaAusente => "tabela_de_comentarios_ausente",
            Self::TabelasRepetidas => "tabelas_de_comentarios_repetidas",
            Self::LegendaInvalida => "legenda_dos_comentarios_invalida",
            Self::CabecalhoInvalido => "cabecalho_dos_comentarios_invalido",
            Self::LinhaInvalida => "linha_do_comentario_invalida",
            Self::TextoDoComentarioNaoPublicado => "texto_do_comentario_nao_publicado",
            Self::IdDoComentarioRepetido => "id_do_comentario_repetido",
            Self::ListaIncompleta => "lista_de_comentarios_incompleta",
        }
    }
}

fn descendentes<'p>(raiz: Tag<'_>, parser: &'p tl::Parser<'p>, tag: &str) -> Vec<Tag<'p>> {
    raiz.query_selector(parser, tag)
        .into_iter()
        .flatten()
        .filter_map(|n| n.get(parser).and_then(tl::Node::as_tag))
        .collect()
}

fn filhos<'p>(pai: Tag<'_>, parser: &'p tl::Parser<'p>, nome: &str) -> Vec<Tag<'p>> {
    pai.children()
        .top()
        .iter()
        .filter_map(|h| h.get(parser).and_then(tl::Node::as_tag))
        .filter(|tag| tag.name() == nome)
        .collect()
}

fn atributo(tag: Tag<'_>, nome: &'static str) -> Result<Option<String>, ErroComentarios> {
    atributos(tag)
        .and_then(|a| a.valor(nome))
        .map_err(|_| ErroComentarios::HtmlNaoInterpretado)?
        .map(|bruto| decodificar_atributo(bruto).ok_or(ErroComentarios::HtmlNaoInterpretado))
        .transpose()
}

fn texto(tag: Tag<'_>, parser: &tl::Parser<'_>) -> Result<String, ErroComentarios> {
    let bruto = tag.inner_text(parser).replace("&nbsp;", "\u{a0}");
    decodificar_atributo(&bruto)
        .map(|s| s.trim().to_owned())
        .ok_or(ErroComentarios::HtmlNaoInterpretado)
}

fn tabela_unica<'p>(dom: &'p tl::VDom<'p>, id: &str) -> Result<Option<Tag<'p>>, ErroComentarios> {
    let nos = nos_com_id(dom, id).map_err(|_| ErroComentarios::HtmlNaoInterpretado)?;
    match nos.as_slice() {
        [] => Ok(None),
        [um] => um
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .map(Some)
            .ok_or(ErroComentarios::HtmlNaoInterpretado),
        _ => Err(ErroComentarios::TabelasRepetidas),
    }
}

/// Distingue cadastro vazio de lista com zero registros. O HTML de cadastro
/// só vale quando contém formulário e controles observados, sem tabela.
pub fn ler_tela(html: &str) -> Result<TelaDosComentarios, ErroComentarios> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroComentarios::HtmlNaoInterpretado)?;
    let parser = dom.parser();
    let cadastros = nos_com_id(&dom, "frmComentarioCadastro")
        .map_err(|_| ErroComentarios::HtmlNaoInterpretado)?;
    let listas =
        nos_com_id(&dom, "frmComentarioLista").map_err(|_| ErroComentarios::HtmlNaoInterpretado)?;
    if cadastros.len() == 1 && listas.is_empty() {
        let form = cadastros[0]
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroComentarios::TelaNaoReconhecida)?;
        if form.name() == "form" {
            let areas = descendentes(form, parser, "textarea");
            let botoes = descendentes(form, parser, "input");
            let area_ok = areas
                .iter()
                .filter(|t| atributo(t, "name").ok().flatten().as_deref() == Some("txaDescricao"))
                .count()
                == 1;
            let botao_ok = botoes
                .iter()
                .filter(|t| {
                    atributo(t, "name").ok().flatten().as_deref() == Some("sbmCadastrarComentario")
                })
                .count()
                == 1;
            if area_ok && botao_ok && tabela_unica(&dom, "tblComentarios")?.is_none() {
                return Ok(TelaDosComentarios::CadastroSemComentarios);
            }
        }
        return Err(ErroComentarios::TelaNaoReconhecida);
    }
    let [formulario] = listas.as_slice() else {
        return Err(if listas.is_empty() {
            ErroComentarios::TelaNaoReconhecida
        } else {
            ErroComentarios::FormulariosDaListaRepetidos
        });
    };
    let form = formulario
        .get(parser)
        .and_then(tl::Node::as_tag)
        .ok_or(ErroComentarios::TelaNaoReconhecida)?;
    if form.name() != "form" || !cadastros.is_empty() {
        return Err(ErroComentarios::TelaNaoReconhecida);
    }
    let visoes: Vec<_> = descendentes(form, parser, "input")
        .into_iter()
        .filter(|tag| atributo(tag, "name").ok().flatten().as_deref() == Some("hdnVisualizacao"))
        .collect();
    let visao = match visoes.as_slice() {
        [unico] => match atributo(unico, "value")?.as_deref() {
            Some("P") => VisaoDosComentarios::Processo,
            Some("T") => VisaoDosComentarios::Todos,
            _ => return Err(ErroComentarios::VisaoNaoPublicada),
        },
        _ => return Err(ErroComentarios::VisaoNaoPublicada),
    };
    let tabela = tabela_unica(&dom, "tblComentarios")?.ok_or(ErroComentarios::TabelaAusente)?;
    if tabela.name() != "table" {
        return Err(ErroComentarios::TabelaAusente);
    }
    let [caption] = descendentes(tabela, parser, "caption")[..] else {
        return Err(ErroComentarios::LegendaInvalida);
    };
    let legenda = texto(caption, parser)?;
    let (prefixo, resto) = legenda
        .rsplit_once('(')
        .ok_or(ErroComentarios::LegendaInvalida)?;
    if !prefixo.to_lowercase().contains("coment") {
        return Err(ErroComentarios::LegendaInvalida);
    }
    let resto = resto
        .strip_suffix("):")
        .or_else(|| resto.strip_suffix(')'))
        .ok_or(ErroComentarios::LegendaInvalida)?;
    let numero = resto
        .strip_suffix(" registros")
        .or_else(|| resto.strip_suffix(" registro"))
        .ok_or(ErroComentarios::LegendaInvalida)?;
    if numero.is_empty() || numero.len() > 9 || !numero.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ErroComentarios::LegendaInvalida);
    }
    let total_declarado = numero
        .parse::<usize>()
        .map_err(|_| ErroComentarios::LegendaInvalida)?;
    let cabecalho: Vec<_> = descendentes(tabela, parser, "th")
        .iter()
        .map(|t| texto(t, parser))
        .collect::<Result<_, _>>()?;
    let campo = |nome: &str| -> Result<usize, ErroComentarios> {
        let indices: Vec<usize> = cabecalho
            .iter()
            .enumerate()
            .filter_map(|(i, c)| (c == nome).then_some(i))
            .collect();
        match indices.as_slice() {
            [indice] => Ok(*indice),
            _ => Err(ErroComentarios::CabecalhoInvalido),
        }
    };
    let (data, unidade, usuario) = (campo("Data")?, campo("Unidade")?, campo("Usuário")?);
    let texto_idx = cabecalho.iter().position(|h| h == "Texto");
    let protocolo = cabecalho.iter().position(|h| h == "Protocolo");
    let tipo = cabecalho.iter().position(|h| h == "Tipo");
    if cabecalho.is_empty()
        || (texto_idx.is_some() && cabecalho.iter().filter(|c| *c == "Texto").count() != 1)
        || (protocolo.is_some() != tipo.is_some())
    {
        return Err(ErroComentarios::CabecalhoInvalido);
    }
    let mut itens: Vec<ComentarioLido> = Vec::new();
    let mut ids = HashSet::new();
    let mut aguardando_texto = false;
    for tr in descendentes(tabela, parser, "tr") {
        let celulas = filhos(tr, parser, "td");
        if celulas.is_empty() {
            continue;
        }
        let caixas: Vec<_> = descendentes(tr, parser, "input")
            .into_iter()
            .filter(|c| atributo(c, "type").ok().flatten().as_deref() == Some("checkbox"))
            .collect();
        if caixas.is_empty() {
            let classe = atributos(tr)
                .map_err(|_| ErroComentarios::HtmlNaoInterpretado)?
                .tem_classe("trComentario")
                .map_err(|_| ErroComentarios::HtmlNaoInterpretado)?;
            if aguardando_texto && classe && celulas.len() == 1 {
                let ultimo = itens.last_mut().ok_or(ErroComentarios::LinhaInvalida)?;
                ultimo.texto = texto(celulas[0], parser)?;
                aguardando_texto = false;
            } else if !(total_declarado == 0 && texto(tr, parser)?.contains("Nenhum registro")) {
                return Err(ErroComentarios::LinhaInvalida);
            }
            continue;
        }
        if caixas.len() != 1 || aguardando_texto || celulas.len() != cabecalho.len() {
            return Err(ErroComentarios::LinhaInvalida);
        }
        let id = atributo(caixas[0], "value")?.ok_or(ErroComentarios::LinhaInvalida)?;
        if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
            return Err(ErroComentarios::LinhaInvalida);
        }
        if !ids.insert(id.clone()) {
            return Err(ErroComentarios::IdDoComentarioRepetido);
        }
        let valor = |i: usize| -> Result<String, ErroComentarios> {
            celulas
                .get(i)
                .map(|td| texto(td, parser))
                .ok_or(ErroComentarios::LinhaInvalida)?
        };
        let texto_lido = texto_idx.map(&valor).transpose()?.unwrap_or_default();
        itens.push(ComentarioLido {
            id,
            data_hora: valor(data)?,
            unidade: valor(unidade)?,
            usuario: valor(usuario)?,
            texto: texto_lido,
            protocolo: protocolo.map(&valor).transpose()?,
            tipo: tipo.map(&valor).transpose()?,
        });
        aguardando_texto = texto_idx.is_none();
    }
    if aguardando_texto {
        return Err(ErroComentarios::TextoDoComentarioNaoPublicado);
    }
    if itens.len() != total_declarado {
        return Err(ErroComentarios::ListaIncompleta);
    }
    Ok(TelaDosComentarios::Lista(ComentariosLidos {
        visao,
        total_declarado,
        itens,
    }))
}

#[cfg(test)]
#[path = "processo_comentarios/tests.rs"]
mod tests;
