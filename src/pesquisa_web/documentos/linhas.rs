//! Linhas de `table.pesquisaResultado` na pesquisa em documentos, na forma
//! observada nas sete instalações (313 linhas do acervo público): título
//! com o link do processo (com `id_documento`), o do documento entre
//! parênteses e o número SEI à direita; snippet; metadados.
use super::{DocumentoEncontrado, ErroDocumentoDaPesquisa};
use crate::acao_do_sei::AcaoDoSei;
use crate::pesquisa_web::documento_da_linha::{documento_do_processo, documento_visualizado};
use crate::pesquisa_web::{ErroResultadoPesquisa, validar_href_de_processo};
use crate::processo::NumeroProcesso;
use crate::transporte::texto_de_recusa::texto_visivel;
use std::collections::HashSet;

type Tag<'a> = crate::tela_html::Elemento<'a>;

/// Texto visível com espaços (inclusive o U+00A0 de `&nbsp;`) colapsados.
fn texto(no: Tag<'_>) -> String {
    texto_visivel(no)
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
    let campos: Vec<&str> = consulta.split('&').collect();
    let valores = |prefixo: &str| -> Vec<&str> {
        campos
            .iter()
            .filter_map(|c| c.strip_prefix(prefixo))
            .collect()
    };
    let decimal = |v: &str| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit());
    valores("acao=")
        .into_iter()
        .map(AcaoDoSei::ler)
        .eq([Some(AcaoDoSei::DocumentoDownloadAnexo)])
        && matches!(valores("id_anexo=").as_slice(), [id] if decimal(id))
}

/// A única célula `td.classe` da linha de título.
fn celula<'a>(linha: Tag<'a>, classe: &str) -> Result<Tag<'a>, ErroDocumentoDaPesquisa> {
    match linha
        .elementos("td")
        .filter(|elemento| elemento.tem_classe(classe))
        .collect::<Vec<_>>()
        .as_slice()
    {
        [celula] => Ok(*celula),
        _ => Err(ErroDocumentoDaPesquisa::CelulaDoTituloDoDocumentoAusenteOuRepetida),
    }
}

fn href(link: Tag<'_>) -> Result<String, ErroDocumentoDaPesquisa> {
    link.atributo("href")
        .map(str::to_owned)
        .ok_or(ErroDocumentoDaPesquisa::LinkDaLinhaDeDocumentoNaoReconhecido)
}

/// Os `a.protocoloNormal` sob `raiz`; com `class` repetido vale o
/// primeiro, como no navegador (PEN3 5.0.4, processo visitado).
fn links(raiz: Tag<'_>) -> Vec<Tag<'_>> {
    raiz.elementos("a")
        .filter(|elemento| elemento.tem_classe("protocoloNormal"))
        .collect()
}

/// Zero ou um `a.arvore` na linha, com o `href` idêntico ao do processo
/// (a mesma regra de `processo buscar`).
fn arvore_da_linha(
    linha: Tag<'_>,
    href_do_processo: &str,
) -> Result<usize, ErroDocumentoDaPesquisa> {
    let arvores = linha
        .elementos("a")
        .filter(|elemento| elemento.tem_classe("arvore"))
        .collect::<Vec<_>>();
    let arvore = match arvores.as_slice() {
        [] => return Ok(0),
        [arvore] => *arvore,
        _ => {
            return Err(ErroDocumentoDaPesquisa::Linha(
                ErroResultadoPesquisa::LinhaDePesquisaComMaisDeUmLinkDaArvore,
            ));
        }
    };
    if arvore.atributo("href") != Some(href_do_processo) {
        return Err(ErroDocumentoDaPesquisa::Linha(
            ErroResultadoPesquisa::LinkDaArvoreComHrefDiferenteDoLinkDoProcesso,
        ));
    }
    Ok(1)
}

/// Os metadados publicados ("Unidade:", "Inclusão:") das linhas do registro.
/// Sem um deles, o registro falha: a ausência é recusada, não publicada.
fn metadados(linhas: &[Tag<'_>]) -> Result<(String, String), ErroDocumentoDaPesquisa> {
    let (mut unidade, mut inclusao, mut usuario) = (None, None, false);
    for linha in linhas {
        let celulas = linha
            .elementos("td")
            .filter(|elemento| elemento.tem_classe("pesquisaMetatag"))
            .collect::<Vec<_>>();
        for no in celulas {
            let lido = texto(no);
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
    let unidade = unidade
        .filter(|valor| !valor.is_empty())
        .ok_or(ErroDocumentoDaPesquisa::UnidadeDoDocumentoAusente)?;
    let inclusao = inclusao.ok_or(ErroDocumentoDaPesquisa::InclusaoDoDocumentoAusente)?;
    Ok((unidade, inclusao))
}

/// `dd/mm/aaaa`, como mostrado.
fn eh_data(valor: &str) -> bool {
    crate::pesquisa_web::filtros::ano_mes_dia(valor).is_some()
}

/// Um registro: a linha de título e as seguintes até o próximo título.
fn ler_registro(
    titulo: Tag<'_>,
    seguintes: &[Tag<'_>],
) -> Result<(DocumentoEncontrado, String, usize), ErroDocumentoDaPesquisa> {
    let esquerda = celula(titulo, "pesquisaTituloEsquerda")?;
    let direita = celula(titulo, "pesquisaTituloDireita")?;
    let (mut processos, mut documentos) = (Vec::new(), Vec::new());
    for no in links(esquerda) {
        let href = href(no)?;
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
    let numero = match links(direita).as_slice() {
        [no] => *no,
        _ => return Err(ErroDocumentoDaPesquisa::NumeroSeiComLinkDiferenteDoDocumento),
    };
    if href(numero)? != href_documento {
        return Err(ErroDocumentoDaPesquisa::NumeroSeiComLinkDiferenteDoDocumento);
    }
    let numero_sei = texto(numero);
    if numero_sei.is_empty() {
        return Err(ErroDocumentoDaPesquisa::NumeroSeiAusenteNaLinha);
    }
    let rotulo = texto(documento);
    if rotulo.is_empty() {
        return Err(ErroDocumentoDaPesquisa::RotuloDoDocumentoAusente);
    }
    let processo = NumeroProcesso::novo(&texto(processo))?;
    let arvores = arvore_da_linha(titulo, &href_processo)?;
    let (unidade, inclusao) = metadados(seguintes)?;
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
    html: &crate::tela_html::HtmlServido,
) -> Result<Vec<DocumentoEncontrado>, ErroDocumentoDaPesquisa> {
    let dom = crate::tela_html::arvore(html);
    let tabela = match dom
        .elementos("table")
        .filter(|elemento| elemento.tem_classe("pesquisaResultado"))
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] => {
            return Err(ErroDocumentoDaPesquisa::Linha(
                ErroResultadoPesquisa::TabelaDePesquisaAusente,
            ));
        }
        [tabela] => *tabela,
        _ => {
            return Err(ErroDocumentoDaPesquisa::Linha(
                ErroResultadoPesquisa::TabelasDePesquisaRepetidas,
            ));
        }
    };
    // Linhas na ordem do documento; cada título abre um registro.
    let mut registros: Vec<(Tag<'_>, Vec<Tag<'_>>)> = Vec::new();
    for linha in tabela.elementos("tr") {
        let titulo = linha.tem_classe("pesquisaTituloRegistro");
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
        let (documento, id, arvore) = ler_registro(*titulo, seguintes)?;
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
    if links(tabela).len() != 3 * documentos.len() {
        return Err(ErroDocumentoDaPesquisa::Linha(
            ErroResultadoPesquisa::LinkDeProcessoForaDaLinhaDePesquisa,
        ));
    }
    let mut de_trabalho = 0;
    for link in tabela.elementos("a") {
        let abre = link
            .atributo("href")
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
