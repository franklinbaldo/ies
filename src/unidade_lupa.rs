//! Leitor estrutural da lupa de unidades (issue #244).
//! Não executa JavaScript nem presume que autocomplete enumere todos.
//! Evidência: Todos `src/todos/sei/lupa_unidades.py`, Goiás 5.0.3.
#[cfg(test)]
#[path = "unidade_lupa/acervo.rs"]
mod acervo;
mod fluxo;
use crate::entidades_html::decodificar_texto;
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::valor_do_atributo;
use crate::indice_local::UnidadeGuardada;
pub(crate) use fluxo::{FalhaDaLupa, HttpsLupa, ler_integral};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ErroLupa {
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
    let bruto = valor_do_atributo(tag, nome).map_err(|_| ErroLupa::HtmlInesperado)?;
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
    if !(depois.starts_with("registro)") || depois.starts_with("registros)")) {
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
            let n = tags(seletor, parser, "option").len();
            if n == 0 {
                return Err(ErroLupa::PaginaInvalida);
            }
            n
        }
        _ => return Err(ErroLupa::PaginaInvalida),
    };
    let tabelas = tags(form, parser, "table");
    let mut candidatas: Vec<_> = Vec::new();
    for tabela in tabelas {
        let classe = atributo(tabela, "class")?.unwrap_or_default();
        if !classe.split_whitespace().any(|c| c == "infraTable") {
            continue;
        }
        let legendas = tags(tabela, parser, "caption");
        if legendas.len() == 1
            && texto_da_tag(legendas[0], parser).starts_with("Lista de Unidades (")
        {
            candidatas.push(tabela);
        }
    }
    let [tabela] = candidatas.as_slice() else {
        return Err(ErroLupa::TabelaForaDaForma);
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
    decodificar_texto(&tag.inner_text(parser))
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
mod tests {
    use super::*;
    fn pagina(n: usize, paginas: usize, total: usize, ids: &[&str]) -> String {
        let options = (0..paginas).fold(String::new(), |mut html, i| {
            use std::fmt::Write as _;
            write!(html, "<option value='{i}'>{}</option>", i + 1).unwrap();
            html
        });
        let rows = ids.iter().fold(String::new(), |mut html, id| {
            use std::fmt::Write as _;
            write!(html, "<tr><td><input type='checkbox' value='{id}'></td><td>ABC/{id}</td><td>Descrição</td><td>Órgão</td></tr>").unwrap();
            html
        });
        format!(
            "<form id='frmUnidadeLista' method='post'><input type='hidden' name='hdnInfraPaginaAtual' value='{n}'/><select name='selInfraPaginacaoSuperior'>{options}</select><table class='infraTable'><caption>Lista de Unidades ({total} registros):</caption><tr><th></th><th>Sigla</th><th>Descrição</th><th>Órgão</th></tr>{rows}</table></form>"
        )
    }
    #[test]
    fn paginas_inteiras_sem_repeticao() {
        let p0 = ler_pagina(&pagina(0, 2, 3, &["1", "2"])).unwrap();
        let p1 = ler_pagina(&pagina(1, 2, 3, &["3"])).unwrap();
        assert_eq!(reunir_paginas(vec![p0, p1]).unwrap().len(), 3);
    }
    #[test]
    fn repeticao_e_total_errado_sao_recusados_antes_do_indice() {
        let p0 = ler_pagina(&pagina(0, 2, 3, &["1", "2"])).unwrap();
        let p1 = ler_pagina(&pagina(1, 2, 3, &["1"])).unwrap();
        assert_eq!(
            reunir_paginas(vec![p0, p1]),
            Err(ErroLupa::PaginasDivergentes)
        );
        let p0 = ler_pagina(&pagina(0, 2, 3, &["1", "2"])).unwrap();
        let p1 = ler_pagina(&pagina(1, 2, 3, &[])).unwrap();
        assert_eq!(
            reunir_paginas(vec![p0, p1]),
            Err(ErroLupa::PaginasDivergentes)
        );
    }
    #[test]
    fn cabecalho_ausente_nao_pode_virar_lista_vazia() {
        assert!(ler_pagina("<form id='frmUnidadeLista'></form>").is_err());
    }

    #[test]
    fn filtros_publicados_precisam_provar_lista_integral() {
        // Cenários de filtro e órgão do Todos tests/test_lupa_unidades_010.py @323eb01.
        let html = pagina(0, 1, 1, &["7"]).replace(
            "<table class='infraTable'>",
            "<select name='selOrgaoUnidade'>
                <option value='null'>Nenhum órgão</option>
                <option value='' selected='selected'>Todos</option>
             </select>
             <input name='txtSiglaUnidade' value=''>
             <input name='txtDescricaoUnidade' value=''>
             <table class='infraTable'>",
        );
        assert_eq!(conferir_filtros_de_lista_integral(&html), Ok(()));

        // Uma lista de nove resultados filtrados pode ser internamente
        // consistente, mas não substitui a lista integral de destinos.
        for alterada in [
            html.replace(
                "name='txtSiglaUnidade' value=''",
                "name='txtSiglaUnidade' value='CDE'",
            ),
            html.replace(
                "name='txtDescricaoUnidade' value=''",
                "name='txtDescricaoUnidade' value='gabinete'",
            ),
            html.replace("value='' selected='selected'>Todos", "value=''>Todos"),
            html.replace(
                "value='' selected='selected'>Todos",
                "value='null' selected='selected'>Todos",
            ),
            html.replace(
                "name='txtSiglaUnidade' value=''",
                "name='txtSiglaUnidade' value=''><input name='txtSiglaUnidade' value=''",
            ),
        ] {
            assert_eq!(
                conferir_filtros_de_lista_integral(&alterada),
                Err(ErroLupa::PaginaInvalida),
                "não publicar um snapshot quando o filtro não é integral"
            );
        }
    }
}
