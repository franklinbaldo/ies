//! Projeção tipada de processos nas respostas HTML de pesquisa do SEI Web.
//! Parser puro: não executa links nem presume permissão/resultado live.
use crate::acao_do_sei::AcaoDoSei;
use crate::html_atributos::{
    nos_por_elemento_e_classe, nos_por_elemento_e_classe_em_todas_as_ocorrencias, valor_do_atributo,
};
use crate::processo::{NumeroProcesso, NumeroProcessoInvalido};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResultadoPesquisa {
    pub numero: NumeroProcesso,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub enum ErroResultadoPesquisa {
    /// O tokenizador de atributos recusou a tag.
    Atributo(#[source] crate::html_atributos::ErroAtributoHtml),
    LinhaDePesquisaSemLinkDoProcesso,
    LinkDeProcessoSemClasseReconhecida,
    /// `a.arvore` da linha com `href` diferente do `a.protocoloNormal`.
    LinkDaArvoreComHrefDiferenteDoLinkDoProcesso,
    LinkDePesquisaNaoAbreProcesso,
    LinkDePesquisaSemIdentificadorDoProcesso,
    LinkDePesquisaComChaveNaoCanonica,
    LinkDePesquisaComAcaoSemIgual,
    LinkDePesquisaComIdentificadorSemIgual,
    LinkDePesquisaComIdentificadorDoProcessoNaoDecimal,
    NumeroDePesquisa(#[from] NumeroProcessoInvalido),
    /// `a.protocoloNormal` de `documento_visualizar` na linha cujo link do
    /// processo não leva o mesmo `id_documento` (Goiás, acerto em documento).
    LinkDoDocumentoDaLinhaDiferenteDoProcesso,
    HtmlDePesquisaNaoInterpretado,
    /// `href` de um `a.protocoloNormal` da linha ilegível.
    HrefDoLinkDaLinhaIlegivel(#[source] crate::html_atributos::ErroAtributoHtml),
    IdentificadorDoProcessoRepetidoNaPesquisa,
    LinhaDePesquisaComMaisDeUmLinkDaArvore,
    LinhaDePesquisaComMaisDeUmLinkDoProcesso,
    LinkDePesquisaComIdentificadoresRepetidos,
    LinkDeProcessoForaDaLinhaDePesquisa,
    NumeroDePesquisaRepetido,
    TabelaDePesquisaAusente,
    TabelasDePesquisaRepetidas,
}

// O controlador PHP decodifica nomes de query e normaliza pontos/espacos
// para '_'. %5F, '.' ou '+' podem sobrescrever acao/id_procedimento.
// Os nomes devem ser literais ASCII, inclusive os parametros auxiliares.
fn validar_nomes_dos_parametros(campos: &[&str]) -> Result<(), ErroResultadoPesquisa> {
    for campo in campos {
        // Para o controlador PHP, `acao` e `id_procedimento` sem `=`
        // são campos vazios e podem substituir os valores anteriores.
        // Não deixar uma chave obrigatória sem valor parecer parâmetro auxiliar.
        let chave = if let Some((nome, _)) = campo.split_once('=') {
            nome
        } else {
            match *campo {
                "acao" => return Err(ErroResultadoPesquisa::LinkDePesquisaComAcaoSemIgual),
                "id_procedimento" => {
                    return Err(ErroResultadoPesquisa::LinkDePesquisaComIdentificadorSemIgual);
                }
                _ => campo,
            }
        };
        if chave.is_empty()
            || !chave
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
        {
            return Err(ErroResultadoPesquisa::LinkDePesquisaComChaveNaoCanonica);
        }
    }
    Ok(())
}

/// Confere a ação e a identidade de um href Web sem realizar a navegação.
fn validar_href_de_processo(href: &str) -> Result<&str, ErroResultadoPesquisa> {
    // Não aceitar múltiplos parâmetros de ação: proxies/servidores podem
    // escolher o primeiro ou o último, tornando o destino indeterminado.
    // tl preserva entidades HTML em atributos de algumas versões do SEI.
    let query = href
        .strip_prefix("controlador.php?")
        .ok_or(ErroResultadoPesquisa::LinkDePesquisaNaoAbreProcesso)?;
    // Somente após um separador & o prefixo amp; pode ser sobra de
    // &amp; no HTML. No primeiro campo, amp;acao é outra chave, não acao.
    let campos: Vec<_> = query
        .split('&')
        .enumerate()
        .map(|(indice, campo)| {
            if indice == 0 {
                campo
            } else {
                campo.strip_prefix("amp;").unwrap_or(campo)
            }
        })
        .collect();
    let acoes: Vec<_> = campos
        .iter()
        .filter_map(|campo| campo.strip_prefix("acao="))
        .collect();
    if acoes != [AcaoDoSei::ProcedimentoTrabalhar.como_na_url()] {
        return Err(ErroResultadoPesquisa::LinkDePesquisaNaoAbreProcesso);
    }
    validar_nomes_dos_parametros(&campos)?;
    // Um link de trabalho sem ID interno não endereça um processo.
    // Dois IDs tornam o destino dependente da interpretação do servidor.
    // O identificador é validado somente para uso local e não é serializado.
    let mut ids = campos
        .iter()
        .filter_map(|campo| campo.strip_prefix("id_procedimento="));
    let primeiro_id = ids
        .next()
        .ok_or(ErroResultadoPesquisa::LinkDePesquisaSemIdentificadorDoProcesso)?;
    if ids.next().is_some() {
        return Err(ErroResultadoPesquisa::LinkDePesquisaComIdentificadoresRepetidos);
    }
    if primeiro_id.is_empty() {
        return Err(ErroResultadoPesquisa::LinkDePesquisaSemIdentificadorDoProcesso);
    }
    if !primeiro_id.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(ErroResultadoPesquisa::LinkDePesquisaComIdentificadorDoProcessoNaoDecimal);
    }
    Ok(primeiro_id)
}

/// A `UnB` 4.0.9 publica, antes do número, um `a.arvore` (ícone da árvore)
/// com o **mesmo** `href` do `a.protocoloNormal`. Só essa forma é aceita:
/// zero ou um por linha, `href` byte a byte igual.
fn contar_arvore_da_linha(
    dom: &tl::VDom<'_>,
    linha: &tl::HTMLTag<'_>,
    href_do_processo: &str,
) -> Result<usize, ErroResultadoPesquisa> {
    // O `a.protocoloNormal` da mesma linha repete `class`: leitura tolerante.
    let arvores =
        nos_por_elemento_e_classe_em_todas_as_ocorrencias(dom, Some(linha), "a", "arvore")
            .map_err(ErroResultadoPesquisa::Atributo)?;
    let arvore = match arvores.as_slice() {
        [] => return Ok(0),
        [arvore] => *arvore,
        _ => {
            return Err(ErroResultadoPesquisa::LinhaDePesquisaComMaisDeUmLinkDaArvore);
        }
    };
    let tag = arvore
        .get(dom.parser())
        .and_then(tl::Node::as_tag)
        .ok_or(ErroResultadoPesquisa::HtmlDePesquisaNaoInterpretado)?;
    if valor_do_atributo(tag, "href")
        .map_err(ErroResultadoPesquisa::Atributo)?
        .as_deref()
        != Some(href_do_processo)
    {
        return Err(ErroResultadoPesquisa::LinkDaArvoreComHrefDiferenteDoLinkDoProcesso);
    }
    Ok(1)
}

/// Resultados observados somente na tabela de pesquisa retornada pelo SEI.
/// Nunca serializa links assinados, IDs técnicos, trechos HTML ou filtros.
pub fn ler_resultados(html: &str) -> Result<Vec<ResultadoPesquisa>, ErroResultadoPesquisa> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroResultadoPesquisa::HtmlDePesquisaNaoInterpretado)?;
    let parser = dom.parser();
    // Classes e `href` vêm da fonte da tag (defeito do tl 0.7.8 no mapa).
    let primeira = match nos_por_elemento_e_classe(&dom, None, "table", "pesquisaResultado")
        .map_err(ErroResultadoPesquisa::Atributo)?
        .as_slice()
    {
        [] => return Err(ErroResultadoPesquisa::TabelaDePesquisaAusente),
        [primeira] => *primeira,
        _ => return Err(ErroResultadoPesquisa::TabelasDePesquisaRepetidas),
    };
    let tabela = primeira
        .get(parser)
        .and_then(tl::Node::as_tag)
        .ok_or(ErroResultadoPesquisa::TabelaDePesquisaAusente)?;
    let linhas = nos_por_elemento_e_classe(&dom, Some(tabela), "tr", "pesquisaTituloRegistro")
        .map_err(ErroResultadoPesquisa::Atributo)?;
    let mut observados = HashSet::new();
    let mut ids_observados = HashSet::new();
    let mut resultados = Vec::new();
    let mut arvores = 0;
    let mut links_de_documento = 0;
    for linha in linhas {
        let tag = linha
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroResultadoPesquisa::LinhaDePesquisaSemLinkDoProcesso)?;
        let links = nos_por_elemento_e_classe_em_todas_as_ocorrencias(
            &dom,
            Some(tag),
            "a",
            "protocoloNormal",
        )
        .map_err(ErroResultadoPesquisa::Atributo)?;
        // Links que só abrem um documento (acerto em documento, Goiás) não
        // disputam o lugar do link do processo.
        let (candidatos, documentos) = documento_da_linha::separar(parser, links)?;
        let link_no = match candidatos.as_slice() {
            [] => return Err(ErroResultadoPesquisa::LinhaDePesquisaSemLinkDoProcesso),
            [link] => *link,
            _ => {
                return Err(ErroResultadoPesquisa::LinhaDePesquisaComMaisDeUmLinkDoProcesso);
            }
        };
        let link = link_no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroResultadoPesquisa::LinhaDePesquisaSemLinkDoProcesso)?;
        let href = valor_do_atributo(link, "href")
            .map_err(ErroResultadoPesquisa::Atributo)?
            .ok_or(ErroResultadoPesquisa::LinkDePesquisaNaoAbreProcesso)?;
        let primeiro_id = validar_href_de_processo(&href)?;
        if !documentos.is_empty() {
            let doc = documento_da_linha::documento_do_processo(&href);
            if documentos.iter().any(|d| Some(d.as_str()) != doc) {
                return Err(ErroResultadoPesquisa::LinkDoDocumentoDaLinhaDiferenteDoProcesso);
            }
        }
        links_de_documento += documentos.len();
        let numero = NumeroProcesso::novo(&link.inner_text(parser))?;
        if !observados.insert(numero.as_str().to_owned()) {
            return Err(ErroResultadoPesquisa::NumeroDePesquisaRepetido);
        }
        // O identificador interno é decimal: 7 e 007 denotam a mesma chave.
        // Normalização por string evita overflow em IDs maiores que u64.
        if !ids_observados.insert(primeiro_id.trim_start_matches('0').to_owned()) {
            return Err(ErroResultadoPesquisa::IdentificadorDoProcessoRepetidoNaPesquisa);
        }
        arvores += contar_arvore_da_linha(&dom, tag, &href)?;
        resultados.push(ResultadoPesquisa { numero });
    }
    // A seleção por classe das linhas pode omitir links reais se a versão
    // do SEI mudar o HTML. Nunca devolver subconjunto silencioso como se
    // representasse todos os links observáveis da tabela de resultados.
    let links_da_tabela = nos_por_elemento_e_classe_em_todas_as_ocorrencias(
        &dom,
        Some(tabela),
        "a",
        "protocoloNormal",
    )
    .map_err(ErroResultadoPesquisa::Atributo)?
    .len();
    if links_da_tabela != resultados.len() + links_de_documento {
        return Err(ErroResultadoPesquisa::LinkDeProcessoForaDaLinhaDePesquisa);
    }
    let links_de_trabalho = links_de_trabalho(parser, tabela)?;
    if links_de_trabalho != resultados.len() + arvores {
        return Err(ErroResultadoPesquisa::LinkDeProcessoSemClasseReconhecida);
    }
    Ok(resultados)
}

/// Todos os links da tabela que abrem `procedimento_trabalhar`, com ou sem
/// a classe esperada.
fn links_de_trabalho(
    parser: &tl::Parser<'_>,
    tabela: &tl::HTMLTag<'_>,
) -> Result<usize, ErroResultadoPesquisa> {
    // A classe CSS do link não é garantia de que ele seja o único link
    // de trabalho na tabela. Uma versão diferente do SEI pode perder
    // essa classe e ocultar processos na projeção: falhar fechado.
    let mut links_de_trabalho = 0;
    for no in tabela.query_selector(parser, "a").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let abre_processo = valor_do_atributo(tag, "href")
            .map_err(ErroResultadoPesquisa::Atributo)?
            .is_some_and(|href| {
                href.strip_prefix("controlador.php?").is_some_and(|query| {
                    query.split('&').enumerate().any(|(indice, campo)| {
                        campo == "acao=procedimento_trabalhar"
                            || (indice > 0 && campo == "amp;acao=procedimento_trabalhar")
                    })
                })
            });
        links_de_trabalho += usize::from(abre_processo);
    }
    Ok(links_de_trabalho)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_arvore;

#[cfg(test)]
pub(crate) mod tests_fixtures;

// Linha com acerto em documento (Goiás): só o processo é publicado.
mod documento_da_linha;

// Replay de conformidade do acervo público pelas peças do comando (Refs #58).
#[cfg(test)]
mod acervo;

// A declaração do formulário fica com a pesquisa, não num catálogo do primitivo
// (docs/arquitetura/0002); `ies processo buscar` a usa pelo `fluxo`.
pub(crate) mod formulario_pesquisa;

// Pesquisa em documentos (`ies documento buscar`): linhas, 302 e resposta.
pub(crate) mod documentos;

// O 302 da pesquisa é interpretado pela própria operação, nunca seguido.
pub(crate) mod redirecionamento;

// Regras do `onsubmit` da pesquisa, lidas do JS servido e escritas à mão.
mod regras_formulario;

// Resposta 200 (lista com barra ou "sem resultado") e paginação `navegar()`.
pub(crate) mod paginacao;
pub(crate) mod resposta;

// Link do menu da pesquisa no Controle e o fluxo do comando.
pub(crate) mod alvo;
pub(crate) mod erro_busca;
pub(crate) mod fluxo;
pub(crate) mod link_da_pesquisa;

// Critérios opcionais derivados estritamente dos controles servidos.
pub(crate) mod filtros;
