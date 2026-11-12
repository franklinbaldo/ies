//! Leitor estrutural da lupa de unidades (issue #244).
//! Não executa JavaScript nem presume que autocomplete enumere todos.
//! Evidência: Todos `src/todos/sei/lupa_unidades.py`, Goiás 5.0.3.
#[cfg(test)]
#[path = "unidade_lupa/acervo.rs"]
mod acervo;
mod fluxo;
use crate::indice_local::UnidadeGuardada;
use crate::legenda::ler_legenda;
pub(crate) use fluxo::{FalhaDaLupa, HttpsLupa, ler_integral_com_contador};

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroLupa {
    Legenda(#[from] crate::legenda::ErroLegenda),
    /// O tokenizador de atributos recusou a tag.
    Atributo(#[from] crate::html_atributos::ErroAtributoHtml),
    FormularioAusente(#[source] crate::formulario_web::ErroSubmissaoFormulario),
    TabelaForaDaForma,
    TotalAusente,
    PaginaInvalida,
    /// O número da página atual não é um inteiro decimal.
    NumeroDaPaginaIlegivel(#[source] std::num::ParseIntError),
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

fn tags<'a>(
    raiz: crate::tela_html::Elemento<'a>,
    seletor: &str,
) -> Vec<crate::tela_html::Elemento<'a>> {
    raiz.elementos(seletor).collect()
}
fn campo_unico(form: crate::tela_html::Elemento<'_>, nome: &str) -> Result<String, ErroLupa> {
    let encontrados = tags(form, "input");
    let correspondentes: Vec<_> = encontrados
        .into_iter()
        .filter(|t| t.atributo("name") == Some(nome))
        .collect();
    let [unico] = correspondentes.as_slice() else {
        return Err(ErroLupa::PaginaInvalida);
    };
    unico
        .atributo("value")
        .map(str::to_owned)
        .ok_or(ErroLupa::PaginaInvalida)
}
fn total_da_legenda(legenda: &str) -> Result<usize, ErroLupa> {
    if !legenda.starts_with("Lista de Unidades (") {
        return Err(ErroLupa::TotalAusente);
    }
    Ok(ler_legenda(legenda)?.total)
}

fn ler_unidades(
    tabela: crate::tela_html::Elemento<'_>,
    total: usize,
) -> Result<Vec<UnidadeGuardada>, ErroLupa> {
    let trs = tags(tabela, "tr");
    let mut indices: Option<(usize, usize, usize, usize)> = None;
    let mut unidades = Vec::new();
    for tr in trs {
        let ths = tags(tr, "th");
        if !ths.is_empty() {
            if indices.is_some() {
                continue;
            }
            let nomes: Vec<_> = ths.iter().copied().map(texto_da_tag).collect();
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
        let tds = tags(tr, "td");
        if tds.is_empty() {
            continue;
        }
        let Some((tamanho, sigla, descricao, orgao)) = indices else {
            return Err(ErroLupa::TabelaForaDaForma);
        };
        if let [aviso] = tds.as_slice()
            && aviso.atributo("colspan").map(str::to_owned).is_some()
        {
            // O único aviso permitido é a lista realmente vazia.
            if total == 0 && unidades.is_empty() {
                continue;
            }
            return Err(ErroLupa::TabelaForaDaForma);
        }
        if tds.len() != tamanho {
            return Err(ErroLupa::TabelaForaDaForma);
        }
        let checkbox: Vec<_> = tags(tr, "input")
            .into_iter()
            .filter(|t| t.atributo("type") == Some("checkbox"))
            .collect();
        let [unico] = checkbox.as_slice() else {
            return Err(ErroLupa::LinhaSemIdentidade);
        };
        let id = unico
            .atributo("value")
            .map(str::to_owned)
            .ok_or(ErroLupa::LinhaSemIdentidade)?;
        if id.is_empty() {
            return Err(ErroLupa::LinhaSemIdentidade);
        }
        let celula = |i: usize| {
            tds.get(i)
                .copied()
                .map(texto_da_tag)
                .ok_or(ErroLupa::TabelaForaDaForma)
        };
        let sigla = celula(sigla)?;
        if sigla.is_empty() {
            return Err(ErroLupa::TabelaForaDaForma);
        }
        unidades.push(UnidadeGuardada {
            id,
            sigla,
            descricao: celula(descricao)?,
            orgao: celula(orgao)?,
        });
    }
    if indices.is_none() {
        return Err(ErroLupa::TabelaForaDaForma);
    }
    if unidades.len() > total {
        return Err(ErroLupa::PaginasDivergentes);
    }
    Ok(unidades)
}

/// A resposta da lupa deve ser uma página de dados identificada pelo formulário,
/// pela tabela, pelo cabeçalho, pelo total e pelo paginador.
pub(crate) fn ler_pagina(html: &crate::tela_html::HtmlServido) -> Result<PaginaDaLupa, ErroLupa> {
    let dom = crate::tela_html::arvore(html);
    let form = crate::formulario_web::localizar_formulario(&dom, "frmUnidadeLista")
        .map_err(ErroLupa::FormularioAusente)?;
    let pagina: usize = campo_unico(form, "hdnInfraPaginaAtual")?
        .parse()
        .map_err(ErroLupa::NumeroDaPaginaIlegivel)?;
    let seletores: Vec<_> = tags(form, "select")
        .into_iter()
        .filter(|s| s.atributo("name") == Some("selInfraPaginacaoSuperior"))
        .collect();
    let paginas = match seletores.as_slice() {
        [] => 1,
        [seletor] => {
            // A lupa GO publica índices 0..N-1 nos valores das opções.
            // Contar opções sem conferir seus índices permitiria tratar
            // páginas saltadas ou duplicadas como um snapshot completo.
            let opcoes = tags(*seletor, "option");
            if opcoes.is_empty() {
                return Err(ErroLupa::PaginaInvalida);
            }
            for (indice, opcao) in opcoes.iter().enumerate() {
                let valor = opcao
                    .atributo("value")
                    .map(str::to_owned)
                    .ok_or(ErroLupa::PaginaInvalida)?;
                if valor.parse::<usize>().ok() != Some(indice) {
                    return Err(ErroLupa::PaginaInvalida);
                }
            }
            opcoes.len()
        }
        _ => return Err(ErroLupa::PaginaInvalida),
    };
    let tabelas = tags(form, "table");
    let mut candidatas: Vec<_> = Vec::new();
    let mut existe_tabela_infra = false;
    for tabela in tabelas {
        let classe = tabela
            .atributo("class")
            .map(str::to_owned)
            .unwrap_or_default();
        if !classe.split_whitespace().any(|c| c == "infraTable") {
            continue;
        }
        existe_tabela_infra = true;
        let legendas = tags(tabela, "caption");
        if let [legenda] = legendas.as_slice()
            && texto_da_tag(*legenda).starts_with("Lista de Unidades (")
        {
            candidatas.push(tabela);
        }
    }
    let tabela = match candidatas.as_slice() {
        [tabela] => *tabela,
        [] if !existe_tabela_infra
            && pagina == 0
            && paginas == 1
            && texto_da_tag(form).ends_with("Nenhum registro encontrado.") =>
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
    let legenda = tags(tabela, "caption")
        .first()
        .copied()
        .map(texto_da_tag)
        .ok_or(ErroLupa::TabelaForaDaForma)?;
    let total = total_da_legenda(&legenda)?;
    if pagina >= paginas {
        return Err(ErroLupa::PaginaInvalida);
    }
    let unidades = ler_unidades(tabela, total)?;
    Ok(PaginaDaLupa {
        unidades,
        total,
        pagina,
        paginas,
    })
}
fn texto_da_tag(tag: crate::tela_html::Elemento<'_>) -> String {
    tag.texto().split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Confere que a resposta POST do SEI é a lista *integral*, não uma
/// lista menor (mas coerente) por filtros lembrados na sessão.
///
/// O valor "null" do órgão não é "Todos": só o valor vazio é universal
/// na lupa observada no GO pelo Todos @323eb01.
pub(crate) fn conferir_filtros_de_lista_integral(
    html: &crate::tela_html::HtmlServido,
) -> Result<(), ErroLupa> {
    let dom = crate::tela_html::arvore(html);
    let form = crate::formulario_web::localizar_formulario(&dom, "frmUnidadeLista")
        .map_err(ErroLupa::FormularioAusente)?;
    let inputs = tags(form, "input");
    for nome in ["txtSiglaUnidade", "txtDescricaoUnidade"] {
        let mut encontrados = 0;
        for campo in &inputs {
            if campo.atributo("name") != Some(nome) {
                continue;
            }
            encontrados += 1;
            if !campo
                .atributo("value")
                .map(str::to_owned)
                .unwrap_or_default()
                .is_empty()
            {
                return Err(ErroLupa::PaginaInvalida);
            }
        }
        if encontrados != 1 {
            return Err(ErroLupa::PaginaInvalida);
        }
    }

    let mut selects = 0;
    for seletor in tags(form, "select") {
        if seletor.atributo("name") != Some("selOrgaoUnidade") {
            continue;
        }
        selects += 1;
        let opcoes = tags(seletor, "option");
        let mut selecionadas = Vec::new();
        for (i, opcao) in opcoes.iter().enumerate() {
            if opcao.atributo("selected").map(str::to_owned).is_some() {
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
        if opcao.atributo("value") != Some("") || texto_da_tag(*opcao) != "Todos" {
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
