//! Leitor estrutural da lupa de unidades (issue #244).
//! Não executa JavaScript nem presume que autocomplete enumere todos.
//! Evidência: Todos `src/todos/sei/lupa_unidades.py`, Goiás 5.0.3.
#[cfg(test)]
#[path = "unidade_lupa/acervo.rs"]
mod acervo;
mod fluxo;
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::valor_do_atributo;
use crate::indice_local::UnidadeGuardada;
pub(crate) use fluxo::{FalhaDaLupa, HttpsLupa, ler_integral_com_contador};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ErroLupa {
    /// O tokenizador de atributos recusou a tag.
    Atributo(crate::html_atributos::ErroAtributoHtml),
    HtmlInesperado,
    FormularioAusente,
    TabelaForaDaForma,
    TotalAusente,
    PaginaInvalida,
    LinhaSemIdentidade,
    PaginasDivergentes,
}

/// Dados conferidos de uma página da lupa, antes de qualquer persistência.
pub(crate) struct PaginaDaLupa {
    pub(crate) unidades: Vec<UnidadeGuardada>,
    pub(crate) total: usize,
    pub(crate) pagina: usize,
    pub(crate) paginas: usize,
}

fn atributo(tag: &tl::HTMLTag<'_>, nome: &'static str) -> Result<Option<String>, ErroLupa> {
    let bruto = valor_do_atributo(tag, nome).map_err(ErroLupa::Atributo)?;
    bruto
        .map(|x| decodificar_atributo(&x).ok_or(ErroLupa::HtmlInesperado))
        .transpose()
}
fn tags<'a>(
    raiz: &tl::HTMLTag<'_>,
    parser: &'a tl::Parser<'a>,
    seletor: &str,
) -> Vec<&'a tl::HTMLTag<'a>> {
    raiz.query_selector(parser, seletor)
        .into_iter()
        .flatten()
        .filter_map(|handle| handle.get(parser).and_then(tl::Node::as_tag))
        .collect()
}
fn campo_unico(
    form: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    nome: &str,
) -> Result<String, ErroLupa> {
    let encontrados = tags(form, parser, "input");
    let correspondentes: Vec<_> = encontrados
        .into_iter()
        .filter(|t| atributo(t, "name").ok().flatten().as_deref() == Some(nome))
        .collect();
    let [unico] = correspondentes.as_slice() else {
        return Err(ErroLupa::PaginaInvalida);
    };
    atributo(unico, "value")?.ok_or(ErroLupa::PaginaInvalida)
}
fn total_da_legenda(legenda: &str) -> Option<usize> {
    let resto = legenda.strip_prefix("Lista de Unidades (")?;
    let (numero, depois) = resto.split_once(' ')?;
    let sufixo = depois
        .strip_prefix("registros")
        .or_else(|| depois.strip_prefix("registro"))?;
    // A cassette GO 5.0.3 publica a faixa após o número de registros:
    // "Lista de Unidades (13671 registros - 1 a 50):".
    // Não a confundir com uma legenda de outra tabela.
    if !(sufixo.starts_with(')') || sufixo.starts_with(" - ")) {
        return None;
    }
    numero.parse().ok()
}

/// A resposta da lupa deve ser uma página de dados identificada pelo formulário,
/// pela tabela, pelo cabeçalho, pelo total e pelo paginador.
#[allow(clippy::too_many_lines)]
pub(crate) fn ler_pagina(html: &str) -> Result<PaginaDaLupa, ErroLupa> {
    let dom =
        tl::parse(html, tl::ParserOptions::default()).map_err(|_| ErroLupa::HtmlInesperado)?;
    let parser = dom.parser();
    let form = crate::formulario_web::localizar_formulario(&dom, "frmUnidadeLista")
        .map_err(|_| ErroLupa::FormularioAusente)?;
    let pagina: usize = campo_unico(form, parser, "hdnInfraPaginaAtual")?
        .parse()
        .map_err(|_| ErroLupa::PaginaInvalida)?;
    let seletores: Vec<_> = tags(form, parser, "select")
        .into_iter()
        .filter(|s| {
            atributo(s, "name").ok().flatten().as_deref() == Some("selInfraPaginacaoSuperior")
        })
        .collect();
    let paginas = match seletores.as_slice() {
        [] => 1,
        [seletor] => {
            // A lupa GO publica índices 0..N-1 nos valores das opções.
            // Contar opções sem conferir seus índices permitiria tratar
            // páginas saltadas ou duplicadas como um snapshot completo.
            let opcoes = tags(seletor, parser, "option");
            if opcoes.is_empty() {
                return Err(ErroLupa::PaginaInvalida);
            }
            for (indice, opcao) in opcoes.iter().enumerate() {
                let valor = atributo(opcao, "value")?.ok_or(ErroLupa::PaginaInvalida)?;
                if valor.parse::<usize>().ok() != Some(indice) {
                    return Err(ErroLupa::PaginaInvalida);
                }
            }
            opcoes.len()
        }
        _ => return Err(ErroLupa::PaginaInvalida),
    };
    let tabelas = tags(form, parser, "table");
    let mut candidatas: Vec<_> = Vec::new();
    let mut existe_tabela_infra = false;
    for tabela in tabelas {
        let classe = atributo(tabela, "class")?.unwrap_or_default();
        if !classe.split_whitespace().any(|c| c == "infraTable") {
            continue;
        }
        existe_tabela_infra = true;
        let legendas = tags(tabela, parser, "caption");
        if legendas.len() == 1
            && texto_da_tag(legendas[0], parser).starts_with("Lista de Unidades (")
        {
            candidatas.push(tabela);
        }
    }
    let tabela = match candidatas.as_slice() {
        [tabela] => *tabela,
        [] if !existe_tabela_infra
            && pagina == 0
            && paginas == 1
            && texto_da_tag(form, parser).ends_with("Nenhum registro encontrado.") =>
        {
            // GO, cassette unidade-listar-destinos-go, POST passo 12:
            // a lupa mantém o formulário, mas omite infraTable e declara
            // a ausência de resultados. Não confundir com uma tabela quebrada.
            return Ok(PaginaDaLupa {
                unidades: Vec::new(),
                total: 0,
                pagina,
                paginas,
            });
        }
        _ => return Err(ErroLupa::TabelaForaDaForma),
    };
    let legenda = texto_da_tag(tags(tabela, parser, "caption")[0], parser);
    let total = total_da_legenda(&legenda).ok_or(ErroLupa::TotalAusente)?;
    if pagina >= paginas {
        return Err(ErroLupa::PaginaInvalida);
    }
    let trs = tags(tabela, parser, "tr");
    let mut indices: Option<(usize, usize, usize, usize)> = None;
    let mut unidades = Vec::new();
    for tr in trs {
        let ths = tags(tr, parser, "th");
        if !ths.is_empty() {
            if indices.is_some() {
                continue;
            }
            let nomes: Vec<_> = ths.iter().map(|c| texto_da_tag(c, parser)).collect();
            let posicao = |alvo: &str| -> Result<usize, ErroLupa> {
                let achados: Vec<_> = nomes
                    .iter()
                    .enumerate()
                    .filter(|(_, nome)| nome == &alvo)
                    .map(|(i, _)| i)
                    .collect();
                let [unico] = achados.as_slice() else {
                    return Err(ErroLupa::TabelaForaDaForma);
                };
                Ok(*unico)
            };
            // Primeira célula da linha contém checkbox; os cabeçalhos dão
            // a posição de cada texto, independentemente da ordem.
            indices = Some((
                nomes.len(),
                posicao("Sigla")?,
                posicao("Descrição")?,
                posicao("Órgão")?,
            ));
            continue;
        }
        let tds = tags(tr, parser, "td");
        if tds.is_empty() {
            continue;
        }
        let Some((tamanho, sigla, descricao, orgao)) = indices else {
            return Err(ErroLupa::TabelaForaDaForma);
        };
        if tds.len() == 1 && atributo(tds[0], "colspan")?.is_some() {
            // O único aviso permitido é a lista realmente vazia.
            if total == 0 && unidades.is_empty() {
                continue;
            }
            return Err(ErroLupa::TabelaForaDaForma);
        }
        if tds.len() != tamanho {
            return Err(ErroLupa::TabelaForaDaForma);
        }
        let checkbox: Vec<_> = tags(tr, parser, "input")
            .into_iter()
            .filter(|t| atributo(t, "type").ok().flatten().as_deref() == Some("checkbox"))
            .collect();
        let [unico] = checkbox.as_slice() else {
            return Err(ErroLupa::LinhaSemIdentidade);
        };
        let id = atributo(unico, "value")?.ok_or(ErroLupa::LinhaSemIdentidade)?;
        if id.is_empty() {
            return Err(ErroLupa::LinhaSemIdentidade);
        }
        let sigla = texto_da_tag(tds[sigla], parser);
        if sigla.is_empty() {
            return Err(ErroLupa::TabelaForaDaForma);
        }
        unidades.push(UnidadeGuardada {
            id,
            sigla,
            descricao: texto_da_tag(tds[descricao], parser),
            orgao: texto_da_tag(tds[orgao], parser),
        });
    }
    if indices.is_none() {
        return Err(ErroLupa::TabelaForaDaForma);
    }
    if unidades.len() > total {
        return Err(ErroLupa::PaginasDivergentes);
    }
    Ok(PaginaDaLupa {
        unidades,
        total,
        pagina,
        paginas,
    })
}
fn texto_da_tag(tag: &tl::HTMLTag<'_>, parser: &tl::Parser<'_>) -> String {
    html_escape::decode_html_entities(&tag.inner_text(parser))
        .into_owned()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Confere que a resposta POST do SEI é a lista *integral*, não uma
/// lista menor (mas coerente) por filtros lembrados na sessão.
///
/// O valor "null" do órgão não é "Todos": só o valor vazio é universal
/// na lupa observada no GO pelo Todos @323eb01.
pub(crate) fn conferir_filtros_de_lista_integral(html: &str) -> Result<(), ErroLupa> {
    let dom =
        tl::parse(html, tl::ParserOptions::default()).map_err(|_| ErroLupa::HtmlInesperado)?;
    let parser = dom.parser();
    let form = crate::formulario_web::localizar_formulario(&dom, "frmUnidadeLista")
        .map_err(|_| ErroLupa::FormularioAusente)?;
    let inputs = tags(form, parser, "input");
    for nome in ["txtSiglaUnidade", "txtDescricaoUnidade"] {
        let mut encontrados = 0;
        for campo in &inputs {
            if atributo(campo, "name")?.as_deref() != Some(nome) {
                continue;
            }
            encontrados += 1;
            if !atributo(campo, "value")?.unwrap_or_default().is_empty() {
                return Err(ErroLupa::PaginaInvalida);
            }
        }
        if encontrados != 1 {
            return Err(ErroLupa::PaginaInvalida);
        }
    }

    let mut selects = 0;
    for seletor in tags(form, parser, "select") {
        if atributo(seletor, "name")?.as_deref() != Some("selOrgaoUnidade") {
            continue;
        }
        selects += 1;
        let opcoes = tags(seletor, parser, "option");
        let mut selecionadas = Vec::new();
        for (i, opcao) in opcoes.iter().enumerate() {
            if atributo(opcao, "selected")?.is_some() {
                selecionadas.push(i);
            }
        }
        // Sem selected, o navegador escolhe a primeira opção. Quando
        // essa opção tem valor "null", a resposta não é lista integral.
        let indice = match selecionadas.as_slice() {
            [i] => *i,
            [] => 0,
            _ => return Err(ErroLupa::PaginaInvalida),
        };
        let opcao = opcoes.get(indice).ok_or(ErroLupa::PaginaInvalida)?;
        if atributo(opcao, "value")?.as_deref() != Some("")
            || texto_da_tag(opcao, parser) != "Todos"
        {
            return Err(ErroLupa::PaginaInvalida);
        }
    }
    if selects != 1 {
        return Err(ErroLupa::PaginaInvalida);
    }
    Ok(())
}

/// Verifica a ordem, os totais e as identidades antes de abrir transação.
pub(crate) fn reunir_paginas(paginas: Vec<PaginaDaLupa>) -> Result<Vec<UnidadeGuardada>, ErroLupa> {
    let Some(primeira) = paginas.first() else {
        return Err(ErroLupa::PaginasDivergentes);
    };
    if paginas.len() != primeira.paginas {
        return Err(ErroLupa::PaginasDivergentes);
    }
    let total = primeira.total;
    let quantidade = primeira.paginas;
    let mut unidades = Vec::new();
    let mut ids = std::collections::HashSet::new();
    for (i, pagina) in paginas.into_iter().enumerate() {
        if pagina.pagina != i || pagina.paginas != quantidade || pagina.total != total {
            return Err(ErroLupa::PaginasDivergentes);
        }
        for unidade in pagina.unidades {
            if !ids.insert(unidade.id.clone()) {
                return Err(ErroLupa::PaginasDivergentes);
            }
            unidades.push(unidade);
        }
    }
    if unidades.len() != total {
        return Err(ErroLupa::PaginasDivergentes);
    }
    Ok(unidades)
}

#[cfg(test)]
#[path = "unidade_lupa/testes.rs"]
mod tests;
