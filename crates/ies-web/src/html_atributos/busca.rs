//! Busca por `id` e por classe sem o índice do `tl`, que é montado com a
//! chave corrompida pelo defeito do 0.7.8. A estrutura (ordem do documento,
//! descendentes, nome do elemento) continua vindo do `tl`.
use super::{ErroAtributoHtml, atributos};

/// `id` exatamente igual; repetir `id` na mesma tag é recusado.
fn tem_id(tag: &tl::HTMLTag<'_>, id: &str) -> Result<bool, ErroAtributoHtml> {
    Ok(atributos(tag)?.valor("id")? == Some(id))
}

fn filtrar_por_id(
    parser: &tl::Parser<'_>,
    handles: impl Iterator<Item = tl::NodeHandle>,
    id: &str,
) -> Result<Vec<tl::NodeHandle>, ErroAtributoHtml> {
    let mut achados = Vec::new();
    for handle in handles {
        if let Some(tag) = handle.get(parser).and_then(tl::Node::as_tag)
            && tem_id(tag, id)?
        {
            achados.push(handle);
        }
    }
    Ok(achados)
}

fn handle(indice: usize) -> tl::NodeHandle {
    // O `tl` endereça nós por `u32`; o limite de 8 MB por resposta mantém
    // o índice muito abaixo de `u32::MAX`.
    tl::NodeHandle::new(u32::try_from(indice).unwrap_or(u32::MAX))
}

/// Todos os elementos do documento cujo `id` é exatamente `id`, em ordem.
/// Uma tag ilegível em qualquer ponto impede afirmar unicidade: erro.
/// # Errors
/// Retorna erro tipado quando a marcação HTML não pode ser interpretada com segurança.
pub fn nos_com_id(dom: &tl::VDom<'_>, id: &str) -> Result<Vec<tl::NodeHandle>, ErroAtributoHtml> {
    filtrar_por_id(dom.parser(), (0..dom.nodes().len()).map(handle), id)
}

/// Descendentes de `tag` cujo `id` é exatamente `id`, em ordem.
/// # Errors
/// Retorna erro tipado quando a marcação HTML não pode ser interpretada com segurança.
pub fn nos_com_id_em(
    tag: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    id: &str,
) -> Result<Vec<tl::NodeHandle>, ErroAtributoHtml> {
    let Some((inicio, fim)) = tag.children().boundaries(parser) else {
        return Ok(Vec::new());
    };
    filtrar_por_id(parser, (inicio..=fim).map(tl::NodeHandle::new), id)
}

/// Elementos `elemento` com a classe `classe`, no documento (`raiz` `None`)
/// ou entre os descendentes de `raiz`. A seleção por nome é a do `tl`.
/// # Errors
/// Retorna erro tipado quando a marcação HTML não pode ser interpretada com segurança.
pub fn nos_por_elemento_e_classe(
    dom: &tl::VDom<'_>,
    raiz: Option<&tl::HTMLTag<'_>>,
    elemento: &str,
    classe: &str,
) -> Result<Vec<tl::NodeHandle>, ErroAtributoHtml> {
    filtrar_por_classe(dom, raiz, elemento, classe, Repeticao::Recusada)
}

/// Como `nos_por_elemento_e_classe`, aceitando `class` repetido com valores
/// idênticos (link da pesquisa da `UnB` 4.0.9); valores diferentes são erro.
/// # Errors
/// Retorna erro tipado quando a marcação HTML não pode ser interpretada com segurança.
pub fn nos_por_elemento_e_classe_com_repeticao_identica(
    dom: &tl::VDom<'_>,
    raiz: Option<&tl::HTMLTag<'_>>,
    elemento: &str,
    classe: &str,
) -> Result<Vec<tl::NodeHandle>, ErroAtributoHtml> {
    filtrar_por_classe(dom, raiz, elemento, classe, Repeticao::Identica)
}

/// Como `nos_por_elemento_e_classe`, aceitando `class` repetido quando
/// **todas** as ocorrências contêm `classe` (PEN3 5.0.4 serve
/// `class="protocoloNormal processoVisitado" … class="protocoloNormal"` no
/// link do processo já visitado); uma ocorrência sem ela é erro.
/// # Errors
/// Retorna erro tipado quando a marcação HTML não pode ser interpretada com segurança.
pub fn nos_por_elemento_e_classe_em_todas_as_ocorrencias(
    dom: &tl::VDom<'_>,
    raiz: Option<&tl::HTMLTag<'_>>,
    elemento: &str,
    classe: &str,
) -> Result<Vec<tl::NodeHandle>, ErroAtributoHtml> {
    filtrar_por_classe(dom, raiz, elemento, classe, Repeticao::ContendoAClasse)
}

/// O que fazer com o atributo `class` repetido na mesma tag.
#[derive(Clone, Copy)]
enum Repeticao {
    Recusada,
    Identica,
    ContendoAClasse,
}

fn filtrar_por_classe(
    dom: &tl::VDom<'_>,
    raiz: Option<&tl::HTMLTag<'_>>,
    elemento: &str,
    classe: &str,
    repeticao: Repeticao,
) -> Result<Vec<tl::NodeHandle>, ErroAtributoHtml> {
    let parser = dom.parser();
    let candidatos: Vec<tl::NodeHandle> = match raiz {
        Some(raiz) => raiz
            .query_selector(parser, elemento)
            .into_iter()
            .flatten()
            .collect(),
        None => dom.query_selector(elemento).into_iter().flatten().collect(),
    };
    let mut achados = Vec::new();
    for candidato in candidatos {
        let Some(tag) = candidato.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let lidos = atributos(tag)?;
        let tem = match repeticao {
            Repeticao::Recusada => lidos.tem_classe(classe)?,
            Repeticao::Identica => lidos.tem_classe_com_repeticao_identica(classe)?,
            Repeticao::ContendoAClasse => lidos.tem_classe_em_todas_as_ocorrencias(classe)?,
        };
        if tem {
            achados.push(candidato);
        }
    }
    Ok(achados)
}
