//! Linhas de `table.pesquisaResultado` na pesquisa em documentos, na forma
//! observada nas sete instalações (313 linhas do acervo público): título
//! com o link do processo (com `id_documento`), o do documento entre
//! parênteses e o número SEI à direita; snippet; metadados.
use super::{DocumentoEncontrado, ErroDocumentoDaPesquisa};
use crate::html_atributos::{
    atributos, nos_por_elemento_e_classe, nos_por_elemento_e_classe_em_todas_as_ocorrencias,
    valor_do_atributo,
};
use crate::pesquisa_web::documento_da_linha::{documento_do_processo, documento_visualizado};
use crate::pesquisa_web::{ErroResultadoPesquisa, validar_href_de_processo};
use crate::processo::NumeroProcesso;
use crate::transporte::texto_de_recusa::texto_visivel;
use std::collections::HashSet;

type Tag<'a> = tl::HTMLTag<'a>;

fn tag<'a>(
    no: tl::NodeHandle,
    parser: &'a tl::Parser<'a>,
) -> Result<&'a Tag<'a>, ErroDocumentoDaPesquisa> {
    no.get(parser)
        .and_then(tl::Node::as_tag)
        .ok_or(ErroDocumentoDaPesquisa::Linha(
            ErroResultadoPesquisa::HtmlDePesquisaNaoInterpretado,
        ))
}

/// Texto visível, entidades decodificadas e espaços colapsados.
fn texto(no: tl::NodeHandle, parser: &tl::Parser<'_>) -> String {
    let bruto = html_escape::decode_html_entities(&texto_visivel(no, parser)).into_owned();
    bruto
        .split(|c: char| c.is_whitespace() || c == '\u{a0}')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// O `id_anexo` único e decimal de um `documento_download_anexo`.
fn anexo_baixado(href: &str) -> bool {
    let Some(consulta) = href.strip_prefix("controlador.php?") else {
        return false;
    };
    let campos: Vec<&str> = consulta
        .split('&')
        .enumerate()
        .map(|(i, c)| {
            if i == 0 {
                c
            } else {
                c.strip_prefix("amp;").unwrap_or(c)
            }
        })
        .collect();
    let valores = |prefixo: &str| -> Vec<&str> {
        campos
            .iter()
            .filter_map(|c| c.strip_prefix(prefixo))
            .collect()
    };
    let decimal = |v: &str| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit());
    valores("acao=") == ["documento_download_anexo"]
        && matches!(valores("id_anexo=").as_slice(), [id] if decimal(id))
}

/// A única célula `td.classe` da linha de título.
fn celula<'a>(
    dom: &'a tl::VDom<'a>,
    linha: &Tag<'_>,
    classe: &str,
) -> Result<tl::NodeHandle, ErroDocumentoDaPesquisa> {
    match nos_por_elemento_e_classe(dom, Some(linha), "td", classe)?.as_slice() {
        [celula] => Ok(*celula),
        _ => Err(ErroDocumentoDaPesquisa::CelulaDoTituloDoDocumentoAusenteOuRepetida),
    }
}

fn href(link: &Tag<'_>) -> Result<String, ErroDocumentoDaPesquisa> {
    valor_do_atributo(link, "href")
        .map_err(ErroDocumentoDaPesquisa::AtributoHtml)?
        .ok_or(ErroDocumentoDaPesquisa::LinkDaLinhaDeDocumentoNaoReconhecido)
}

/// Os `a.protocoloNormal` sob `raiz`; `class` repetido é aceito quando
/// todas as ocorrências trazem a classe (PEN3 5.0.4, processo visitado).
fn links(
    dom: &tl::VDom<'_>,
    raiz: &Tag<'_>,
) -> Result<Vec<tl::NodeHandle>, ErroDocumentoDaPesquisa> {
    nos_por_elemento_e_classe_em_todas_as_ocorrencias(dom, Some(raiz), "a", "protocoloNormal")
        .map_err(ErroDocumentoDaPesquisa::AtributoHtml)
}

/// Zero ou um `a.arvore` na linha, com o `href` idêntico ao do processo
/// (a mesma regra de `processo buscar`, com a tolerância de `links`).
fn arvore_da_linha(
    dom: &tl::VDom<'_>,
    linha: &Tag<'_>,
    href_do_processo: &str,
) -> Result<usize, ErroDocumentoDaPesquisa> {
    let arvores =
        nos_por_elemento_e_classe_em_todas_as_ocorrencias(dom, Some(linha), "a", "arvore")?;
    let arvore = match arvores.as_slice() {
        [] => return Ok(0),
        [arvore] => tag(*arvore, dom.parser())?,
        _ => {
            return Err(ErroDocumentoDaPesquisa::Linha(
                ErroResultadoPesquisa::LinhaDePesquisaComMaisDeUmLinkDaArvore,
            ));
        }
    };
    if valor_do_atributo(arvore, "href")
        .map_err(ErroDocumentoDaPesquisa::AtributoHtml)?
        .as_deref()
        != Some(href_do_processo)
    {
        return Err(ErroDocumentoDaPesquisa::Linha(
            ErroResultadoPesquisa::LinkDaArvoreComHrefDiferenteDoLinkDoProcesso,
        ));
    }
    Ok(1)
}

/// Os metadados publicados ("Unidade:", "Inclusão:") das linhas do registro.
fn metadados(
    dom: &tl::VDom<'_>,
    linhas: &[&Tag<'_>],
) -> Result<(Option<String>, Option<String>), ErroDocumentoDaPesquisa> {
    let parser = dom.parser();
    let (mut unidade, mut inclusao, mut usuario) = (None, None, false);
    for linha in linhas {
        let celulas = nos_por_elemento_e_classe(dom, Some(linha), "td", "pesquisaMetatag")?;
        for no in celulas {
            let lido = texto(no, parser);
            let repetido = || ErroDocumentoDaPesquisa::MetadadoDoDocumentoRepetido;
            if let Some(valor) = lido.strip_prefix("Unidade:") {
                if unidade.replace(valor.trim().to_owned()).is_some() {
                    return Err(repetido());
                }
            } else if let Some(valor) = lido.strip_prefix("Inclusão:") {
                let valor = valor.trim();
                if !eh_data(valor) {
                    return Err(ErroDocumentoDaPesquisa::DataDeInclusaoForaDaFormaObservada);
                }
                if inclusao.replace(valor.to_owned()).is_some() {
                    return Err(repetido());
                }
            } else if lido.starts_with("Usuário:") {
                // Não publicado (pessoa); só a repetição é conferida.
                if std::mem::replace(&mut usuario, true) {
                    return Err(repetido());
                }
            }
        }
    }
    Ok((unidade.filter(|u| !u.is_empty()), inclusao))
}

/// `dd/mm/aaaa`, como mostrado.
fn eh_data(valor: &str) -> bool {
    crate::pesquisa_web::filtros::ano_mes_dia(valor).is_some()
}

/// Um registro: a linha de título e as seguintes até o próximo título.
fn ler_registro(
    dom: &tl::VDom<'_>,
    titulo: &Tag<'_>,
    seguintes: &[&Tag<'_>],
) -> Result<(DocumentoEncontrado, String, usize), ErroDocumentoDaPesquisa> {
    let parser = dom.parser();
    let esquerda = tag(celula(dom, titulo, "pesquisaTituloEsquerda")?, parser)?;
    let direita = tag(celula(dom, titulo, "pesquisaTituloDireita")?, parser)?;
    let (mut processos, mut documentos) = (Vec::new(), Vec::new());
    for no in links(dom, esquerda)? {
        let link = tag(no, parser)?;
        let href = href(link)?;
        if documento_visualizado(&href).is_some() || anexo_baixado(&href) {
            documentos.push((no, href));
        } else if href.contains("acao=procedimento_trabalhar") {
            processos.push((no, href));
        } else {
            return Err(ErroDocumentoDaPesquisa::LinkDaLinhaDeDocumentoNaoReconhecido);
        }
    }
    let (processo, href_processo) = match processos.as_slice() {
        [] => {
            return Err(ErroDocumentoDaPesquisa::Linha(
                ErroResultadoPesquisa::LinhaDePesquisaSemLinkDoProcesso,
            ));
        }
        [um] => um.clone(),
        _ => {
            return Err(ErroDocumentoDaPesquisa::Linha(
                ErroResultadoPesquisa::LinhaDePesquisaComMaisDeUmLinkDoProcesso,
            ));
        }
    };
    validar_href_de_processo(&href_processo)?;
    let id_documento = documento_do_processo(&href_processo)
        .ok_or(ErroDocumentoDaPesquisa::LinhaDeDocumentoSemIdDocumentoNoLinkDoProcesso)?;
    let (documento, href_documento) = match documentos.as_slice() {
        [] => return Err(ErroDocumentoDaPesquisa::LinkDoDocumentoAusenteNaLinha),
        [um] => um.clone(),
        _ => return Err(ErroDocumentoDaPesquisa::LinksDoDocumentoRepetidosNaLinha),
    };
    let anexo = anexo_baixado(&href_documento);
    if !anexo && documento_visualizado(&href_documento) != Some(id_documento) {
        return Err(ErroDocumentoDaPesquisa::LinkDoDocumentoDiferenteDoProcesso);
    }
    let numero = match links(dom, direita)?.as_slice() {
        [no] => *no,
        _ => return Err(ErroDocumentoDaPesquisa::NumeroSeiComLinkDiferenteDoDocumento),
    };
    if href(tag(numero, parser)?)? != href_documento {
        return Err(ErroDocumentoDaPesquisa::NumeroSeiComLinkDiferenteDoDocumento);
    }
    let numero_sei = texto(numero, parser);
    if numero_sei.is_empty() {
        return Err(ErroDocumentoDaPesquisa::NumeroSeiAusenteNaLinha);
    }
    let rotulo = texto(documento, parser);
    if rotulo.is_empty() {
        return Err(ErroDocumentoDaPesquisa::RotuloDoDocumentoAusente);
    }
    let processo = NumeroProcesso::novo(&texto(processo, parser))?;
    let arvores = arvore_da_linha(dom, titulo, &href_processo)?;
    let (unidade, inclusao) = metadados(dom, seguintes)?;
    let encontrado = DocumentoEncontrado {
        numero_sei,
        rotulo,
        processo,
        unidade,
        inclusao,
        anexo,
    };
    Ok((encontrado, id_documento.to_owned(), arvores))
}

/// Documentos observados na tabela de resultado, na ordem servida. Nunca
/// publica links assinados, ids técnicos, trechos ou o usuário.
pub(crate) fn ler_documentos(
    html: &str,
) -> Result<Vec<DocumentoEncontrado>, ErroDocumentoDaPesquisa> {
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| {
        ErroDocumentoDaPesquisa::Linha(ErroResultadoPesquisa::HtmlDePesquisaNaoInterpretado)
    })?;
    let parser = dom.parser();
    let tabela =
        match nos_por_elemento_e_classe(&dom, None, "table", "pesquisaResultado")?.as_slice() {
            [] => {
                return Err(ErroDocumentoDaPesquisa::Linha(
                    ErroResultadoPesquisa::TabelaDePesquisaAusente,
                ));
            }
            [tabela] => tag(*tabela, parser)?,
            _ => {
                return Err(ErroDocumentoDaPesquisa::Linha(
                    ErroResultadoPesquisa::TabelasDePesquisaRepetidas,
                ));
            }
        };
    // Linhas na ordem do documento; cada título abre um registro.
    let mut nos: Vec<tl::NodeHandle> = tabela
        .query_selector(parser, "tr")
        .into_iter()
        .flatten()
        .collect();
    nos.sort_by_key(tl::NodeHandle::get_inner);
    let mut registros: Vec<(&Tag<'_>, Vec<&Tag<'_>>)> = Vec::new();
    for no in nos {
        let linha = tag(no, parser)?;
        let titulo = atributos(linha).and_then(|a| a.tem_classe("pesquisaTituloRegistro"))?;
        if titulo {
            registros.push((linha, Vec::new()));
        } else {
            let (_, seguintes) = registros
                .last_mut()
                .ok_or(ErroDocumentoDaPesquisa::LinhaDaPesquisaForaDeUmRegistro)?;
            seguintes.push(linha);
        }
    }
    let (mut numeros, mut ids) = (HashSet::new(), HashSet::new());
    let (mut documentos, mut arvores) = (Vec::new(), 0);
    for (titulo, seguintes) in &registros {
        let (documento, id, arvore) = ler_registro(&dom, titulo, seguintes)?;
        if !numeros.insert(documento.numero_sei.clone())
            || !ids.insert(id.trim_start_matches('0').to_owned())
        {
            return Err(ErroDocumentoDaPesquisa::NumeroSeiRepetidoNaPesquisa);
        }
        arvores += arvore;
        documentos.push(documento);
    }
    // Nenhum link fora dos registros lidos: três `a.protocoloNormal` por
    // linha e um `procedimento_trabalhar` por linha (mais as árvores).
    if links(&dom, tabela)?.len() != 3 * documentos.len() {
        return Err(ErroDocumentoDaPesquisa::Linha(
            ErroResultadoPesquisa::LinkDeProcessoForaDaLinhaDePesquisa,
        ));
    }
    let mut de_trabalho = 0;
    for no in tabela.query_selector(parser, "a").into_iter().flatten() {
        let link = tag(no, parser)?;
        let abre = valor_do_atributo(link, "href")
            .map_err(ErroDocumentoDaPesquisa::AtributoHtml)?
            .is_some_and(|h| h.contains("acao=procedimento_trabalhar"));
        de_trabalho += usize::from(abre);
    }
    if de_trabalho != documentos.len() + arvores {
        return Err(ErroDocumentoDaPesquisa::Linha(
            ErroResultadoPesquisa::LinkDeProcessoSemClasseReconhecida,
        ));
    }
    Ok(documentos)
}
