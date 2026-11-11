//! Leitor offline da tabela de comentários (`tblComentarios`).
//! Oráculo de comportamento: Todos `tests/test_comentarios_010.py` @323eb01;
//! SEI 4.x: linha de dados + trComentario; SEI 5.0: coluna Texto.
//! Não usa rede, não executa JS e não infere ausência de tabela como lista vazia.
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::{atributos, nos_com_id, valor_decodificado};
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

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroComentarios {
    VisaoNaoPublicada,
    TextoDoComentarioNaoPublicado,
    ListaIncompleta,
    CabecalhoDosComentariosInvalido,
    FormulariosDeComentariosRepetidos,
    HtmlDosComentariosNaoInterpretado,
    ParseDoHtmlDosComentariosFalhou(#[source] tl::ParseError),
    TotalDaLegendaDosComentariosIlegivel(#[source] std::num::ParseIntError),
    IdDoComentarioRepetido,
    LegendaDosComentariosInvalida,
    LinhaDoComentarioInvalida,
    TabelaDeComentariosAusente,
    TabelasDeComentariosRepetidas,
    TelaDeComentariosNaoReconhecida,
    AtributoDosComentariosIlegivel(#[from] crate::html_atributos::ErroAtributoHtml),
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

fn texto(tag: Tag<'_>, parser: &tl::Parser<'_>) -> Result<String, ErroComentarios> {
    let bruto = tag.inner_text(parser).replace("&nbsp;", "\u{a0}");
    decodificar_atributo(&bruto).ok_or(ErroComentarios::HtmlDosComentariosNaoInterpretado)
}

fn tabela_unica<'p>(dom: &'p tl::VDom<'p>, id: &str) -> Result<Option<Tag<'p>>, ErroComentarios> {
    let nos = nos_com_id(dom, id).map_err(ErroComentarios::AtributoDosComentariosIlegivel)?;
    match nos.as_slice() {
        [] => Ok(None),
        [um] => um
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .map(Some)
            .ok_or(ErroComentarios::HtmlDosComentariosNaoInterpretado),
        _ => Err(ErroComentarios::TabelasDeComentariosRepetidas),
    }
}

/// URL assinada que o SEI publicou na barra do processo.
/// Nunca revela a consulta nem o hash ao formatar para diagnóstico.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct EnderecoDosComentarios(String);

impl EnderecoDosComentarios {
    pub(crate) fn novo(url: String) -> Self {
        Self(url)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for EnderecoDosComentarios {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str("EnderecoDosComentarios(<redigido>)")
    }
}

/// Falhas verificáveis ao procurar a ação publicada na barra do processo.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroLinkDosComentarios {
    Barra(#[from] crate::arvore_processo::ErroBarraDoProcesso),
    EnderecoDeOutroProcesso,
    EnderecoDosComentariosNaoReconhecido,
    LinksDosComentariosRepetidosNaBarra,
}

/// Posicionamento das colunas observadas na tela, sem ordem pressuposta.
struct Colunas {
    data: usize,
    unidade: usize,
    usuario: usize,
    texto: Option<usize>,
    protocolo: Option<usize>,
    tipo: Option<usize>,
    total: usize,
}

/// Recusa quando qualquer coluna mandatória está ausente ou repetida.
fn ler_colunas(tabela: Tag<'_>, parser: &tl::Parser<'_>) -> Result<Colunas, ErroComentarios> {
    let cabecalho: Vec<String> = descendentes(tabela, parser, "th")
        .iter()
        .map(|th| texto(th, parser))
        .collect::<Result<_, _>>()?;
    let campo = |nome: &str| -> Result<usize, ErroComentarios> {
        let indices: Vec<usize> = cabecalho
            .iter()
            .enumerate()
            .filter_map(|(i, valor)| (valor == nome).then_some(i))
            .collect();
        match indices.as_slice() {
            [i] => Ok(*i),
            _ => Err(ErroComentarios::CabecalhoDosComentariosInvalido),
        }
    };
    let campo_opcional = |nome: &str| -> Result<Option<usize>, ErroComentarios> {
        match cabecalho.iter().filter(|c| *c == nome).count() {
            0 => Ok(None),
            1 => campo(nome).map(Some),
            _ => Err(ErroComentarios::CabecalhoDosComentariosInvalido),
        }
    };
    let (data, unidade, usuario) = (campo("Data")?, campo("Unidade")?, campo("Usuário")?);
    let (texto, protocolo, tipo) = (
        campo_opcional("Texto")?,
        campo_opcional("Protocolo")?,
        campo_opcional("Tipo")?,
    );
    if cabecalho.is_empty() || protocolo.is_some() != tipo.is_some() {
        return Err(ErroComentarios::CabecalhoDosComentariosInvalido);
    }
    Ok(Colunas {
        data,
        unidade,
        usuario,
        texto,
        protocolo,
        tipo,
        total: cabecalho.len(),
    })
}

/// Cada checkbox inicia uma linha; no 4.x exige a linha de texto seguinte.
fn ler_itens(
    tabela: Tag<'_>,
    parser: &tl::Parser<'_>,
    colunas: &Colunas,
    total: usize,
) -> Result<Vec<ComentarioLido>, ErroComentarios> {
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
            .filter(|c| valor_decodificado(c, "type").ok().flatten().as_deref() == Some("checkbox"))
            .collect();
        if caixas.is_empty() {
            let classe = atributos(tr)
                .map_err(ErroComentarios::AtributoDosComentariosIlegivel)?
                .tem_classe("trComentario")
                .map_err(ErroComentarios::AtributoDosComentariosIlegivel)?;
            if let (true, true, [celula]) = (aguardando_texto, classe, celulas.as_slice()) {
                let ultimo = itens
                    .last_mut()
                    .ok_or(ErroComentarios::LinhaDoComentarioInvalida)?;
                ultimo.texto = texto(celula, parser)?;
                aguardando_texto = false;
            } else if !(total == 0 && texto(tr, parser)?.contains("Nenhum registro")) {
                return Err(ErroComentarios::LinhaDoComentarioInvalida);
            }
            continue;
        }
        let ([caixa], false, true) = (
            caixas.as_slice(),
            aguardando_texto,
            celulas.len() == colunas.total,
        ) else {
            return Err(ErroComentarios::LinhaDoComentarioInvalida);
        };
        let id = valor_decodificado(caixa, "value")?
            .ok_or(ErroComentarios::LinhaDoComentarioInvalida)?;
        if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
            return Err(ErroComentarios::LinhaDoComentarioInvalida);
        }
        if !ids.insert(id.clone()) {
            return Err(ErroComentarios::IdDoComentarioRepetido);
        }
        let valor = |i: usize| -> Result<String, ErroComentarios> {
            celulas
                .get(i)
                .map(|td| texto(td, parser))
                .ok_or(ErroComentarios::LinhaDoComentarioInvalida)?
        };
        itens.push(ComentarioLido {
            id,
            data_hora: valor(colunas.data)?,
            unidade: valor(colunas.unidade)?,
            usuario: valor(colunas.usuario)?,
            texto: colunas.texto.map(valor).transpose()?.unwrap_or_default(),
            protocolo: colunas.protocolo.map(valor).transpose()?,
            tipo: colunas.tipo.map(valor).transpose()?,
        });
        aguardando_texto = colunas.texto.is_none();
    }
    if aguardando_texto {
        return Err(ErroComentarios::TextoDoComentarioNaoPublicado);
    }
    if itens.len() != total {
        return Err(ErroComentarios::ListaIncompleta);
    }
    Ok(itens)
}

/// Analisa a lista, sem confundir o número da legenda com a página lida.
fn ler_lista<'p>(
    dom: &'p tl::VDom<'p>,
    form: Tag<'p>,
) -> Result<TelaDosComentarios, ErroComentarios> {
    let parser = dom.parser();
    let visoes: Vec<_> = descendentes(form, parser, "input")
        .into_iter()
        .filter(|tag| {
            valor_decodificado(tag, "name").ok().flatten().as_deref() == Some("hdnVisualizacao")
        })
        .collect();
    let visao = match visoes.as_slice() {
        [unico] => match valor_decodificado(unico, "value")?.as_deref() {
            Some("P") => VisaoDosComentarios::Processo,
            Some("T") => VisaoDosComentarios::Todos,
            _ => return Err(ErroComentarios::VisaoNaoPublicada),
        },
        _ => return Err(ErroComentarios::VisaoNaoPublicada),
    };
    let tabela =
        tabela_unica(dom, "tblComentarios")?.ok_or(ErroComentarios::TabelaDeComentariosAusente)?;
    if tabela.name() != "table" {
        return Err(ErroComentarios::TabelaDeComentariosAusente);
    }
    let [caption] = descendentes(tabela, parser, "caption")[..] else {
        return Err(ErroComentarios::LegendaDosComentariosInvalida);
    };
    let legenda = texto(caption, parser)?;
    let (prefixo, resto) = legenda
        .rsplit_once('(')
        .ok_or(ErroComentarios::LegendaDosComentariosInvalida)?;
    if !prefixo.to_lowercase().contains("coment") {
        return Err(ErroComentarios::LegendaDosComentariosInvalida);
    }
    let resto = resto
        .strip_suffix("):")
        .or_else(|| resto.strip_suffix(')'))
        .ok_or(ErroComentarios::LegendaDosComentariosInvalida)?;
    let numero = resto
        .strip_suffix(" registros")
        .or_else(|| resto.strip_suffix(" registro"))
        .ok_or(ErroComentarios::LegendaDosComentariosInvalida)?;
    if numero.is_empty() || !numero.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ErroComentarios::LegendaDosComentariosInvalida);
    }
    let total_declarado = numero
        .parse::<usize>()
        .map_err(ErroComentarios::TotalDaLegendaDosComentariosIlegivel)?;
    let colunas = ler_colunas(tabela, parser)?;
    let itens = ler_itens(tabela, parser, &colunas, total_declarado)?;
    Ok(TelaDosComentarios::Lista(ComentariosLidos {
        visao,
        total_declarado,
        itens,
    }))
}

/// Distingue cadastro vazio de lista com zero registros. O HTML de cadastro
/// só vale quando contém formulário e controles observados, sem tabela.
///
/// # Errors
///
/// Recusa telas sem forma observada, controles repetidos, tabelas ausentes,
/// dados incompletos ou ids de comentários não inequívocos.
pub(crate) fn ler_tela(html: &str) -> Result<TelaDosComentarios, ErroComentarios> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(ErroComentarios::ParseDoHtmlDosComentariosFalhou)?;
    let parser = dom.parser();
    let cadastros = nos_com_id(&dom, "frmComentarioCadastro")
        .map_err(ErroComentarios::AtributoDosComentariosIlegivel)?;
    let listas = nos_com_id(&dom, "frmComentarioLista")
        .map_err(ErroComentarios::AtributoDosComentariosIlegivel)?;
    if let ([cadastro], []) = (cadastros.as_slice(), listas.as_slice()) {
        let form = cadastro
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroComentarios::TelaDeComentariosNaoReconhecida)?;
        if form.name() == "form" {
            let areas = descendentes(form, parser, "textarea");
            let botoes = descendentes(form, parser, "input");
            let area_ok = areas
                .iter()
                .filter(|t| {
                    valor_decodificado(t, "name").ok().flatten().as_deref() == Some("txaDescricao")
                })
                .count()
                == 1;
            let botao_ok = botoes
                .iter()
                .filter(|t| {
                    valor_decodificado(t, "name").ok().flatten().as_deref()
                        == Some("sbmCadastrarComentario")
                })
                .count()
                == 1;
            if area_ok && botao_ok && tabela_unica(&dom, "tblComentarios")?.is_none() {
                return Ok(TelaDosComentarios::CadastroSemComentarios);
            }
        }
        return Err(ErroComentarios::TelaDeComentariosNaoReconhecida);
    }
    let [formulario] = listas.as_slice() else {
        return Err(if listas.is_empty() {
            ErroComentarios::TelaDeComentariosNaoReconhecida
        } else {
            ErroComentarios::FormulariosDeComentariosRepetidos
        });
    };
    let form = formulario
        .get(parser)
        .and_then(tl::Node::as_tag)
        .ok_or(ErroComentarios::TelaDeComentariosNaoReconhecida)?;
    if form.name() != "form" || !cadastros.is_empty() {
        return Err(ErroComentarios::TelaDeComentariosNaoReconhecida);
    }
    ler_lista(&dom, form)
}

#[cfg(test)]
#[path = "processo_comentarios/tests.rs"]
mod tests;
