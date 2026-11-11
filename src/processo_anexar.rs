//! Tela de Anexação de Processos do SEI, sem rede nem escrita.
//! Oráculo: Todos src/todos/anexacao.py @323eb01 (`UnB` 4.0.9).
//! Nenhum JavaScript executado, atributos só via `html_atributos`.
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::formulario_web::{decodificar_atributo, localizar_formulario};
use crate::html_atributos::{ErroAtributoHtml, valor_do_atributo};
use std::collections::HashSet;

pub(crate) mod fluxo;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TelaDeAnexacao {
    pub(crate) anexados: Vec<String>,
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Destino {
    pub(crate) id: String,
    pub(crate) nup: String,
    pub(crate) tipo: String,
}
/// Um atributo da tela ou da resposta da anexação que não pôde ser lido.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroAtributoDaAnexacao {
    AtributosAmbiguosNaAnexacao(#[source] ErroAtributoHtml),
    EntidadeHtmlNaoInterpretadaNaAnexacao,
}

/// A tela `frmProcedimentoAnexar` fora da forma observada.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroTelaDeAnexacao {
    Atributo(#[from] ErroAtributoDaAnexacao),
    HtmlDeAnexacaoNaoInterpretado(#[source] tl::ParseError),
    FormularioDeAnexacaoNaoLocalizado(#[source] ErroSubmissaoFormulario),
    ControlesDaAnexacaoAusentesOuRepetidos,
    ListasDeAnexadosRepetidas,
    IdDoAnexadoAusenteOuInvalido,
    IdsDeAnexadosRepetidos,
}

/// O XML das consultas `protocolo_RI1023` e `procedimento_anexacao_verificar`.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroComplementosDaAnexacao {
    Atributo(#[from] ErroAtributoDaAnexacao),
    ComplementosDaAnexacaoNaoInterpretados(#[source] tl::ParseError),
    ProcessoAAnexarNaoIdentificado,
    IdentidadeDeAnexadoAmbigua,
    AnexacaoNaoVerificada,
}

/// A anexação pedida que o SEI aceitaria, mas que o IES recusa antes de gravar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroAnexacaoRecusada {
    AnexacaoAoMesmoProcesso,
    ProcessoJaAnexado,
}

fn atributo(
    tag: &tl::HTMLTag<'_>,
    nome: &'static str,
) -> Result<Option<String>, ErroAtributoDaAnexacao> {
    valor_do_atributo(tag, nome)
        .map_err(ErroAtributoDaAnexacao::AtributosAmbiguosNaAnexacao)?
        .map(|v| {
            decodificar_atributo(&v)
                .ok_or(ErroAtributoDaAnexacao::EntidadeHtmlNaoInterpretadaNaAnexacao)
        })
        .transpose()
}
/// Exige controles únicos dentro do formulário real.
pub(crate) fn ler_tela(html: &str) -> Result<TelaDeAnexacao, ErroTelaDeAnexacao> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(ErroTelaDeAnexacao::HtmlDeAnexacaoNaoInterpretado)?;
    let parser = dom.parser();
    let form = localizar_formulario(&dom, "frmProcedimentoAnexar")
        .map_err(ErroTelaDeAnexacao::FormularioDeAnexacaoNaoLocalizado)?;
    let esperados = [
        "txtProtocolo",
        "txtIdentificacaoProtocolo",
        "hdnIdProtocolo",
        "hdnAnexacao",
        "sbmAnexar",
    ];
    let mut vistos = [0usize; 5];
    let mut tabelas = Vec::new();
    for tag in form
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        let elemento = tag.name().as_utf8_str();
        // `UnB` 4.0.9 publica o envio como <button type="submit">.
        if (elemento.eq_ignore_ascii_case("input") || elemento.eq_ignore_ascii_case("button"))
            && let Some(nome) = atributo(tag, "name")?
            && let Some(pos) = esperados.iter().position(|c| *c == nome)
            && let Some(visto) = vistos.get_mut(pos)
        {
            *visto += 1;
            let tipo = atributo(tag, "type")?;
            let valido = match pos {
                0 | 1 => tipo
                    .as_deref()
                    .is_none_or(|v| v.eq_ignore_ascii_case("text")),
                2 | 3 => tipo
                    .as_deref()
                    .is_some_and(|v| v.eq_ignore_ascii_case("hidden")),
                _ => tipo
                    .as_deref()
                    .is_some_and(|v| v.eq_ignore_ascii_case("submit")),
            };
            if !valido {
                return Err(ErroTelaDeAnexacao::ControlesDaAnexacaoAusentesOuRepetidos);
            }
        }
    }
    if vistos != [1; 5] {
        return Err(ErroTelaDeAnexacao::ControlesDaAnexacaoAusentesOuRepetidos);
    }
    for table in dom.query_selector("table").into_iter().flatten() {
        let Some(tag) = table.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        if atributo(tag, "summary")?.as_deref() == Some("Lista de Processos Anexados") {
            tabelas.push(tag);
        }
    }
    let tabela = match tabelas.as_slice() {
        [] => {
            return Ok(TelaDeAnexacao {
                anexados: Vec::new(),
            });
        }
        [unica] => unica,
        _ => return Err(ErroTelaDeAnexacao::ListasDeAnexadosRepetidas),
    };
    let mut anexados = Vec::new();
    let mut ids = HashSet::new();
    for tag in tabela
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        if !tag.name().as_utf8_str().eq_ignore_ascii_case("input")
            || !atributo(tag, "type")?.is_some_and(|v| v.eq_ignore_ascii_case("checkbox"))
        {
            continue;
        }
        let id = atributo(tag, "value")?
            .filter(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
            .ok_or(ErroTelaDeAnexacao::IdDoAnexadoAusenteOuInvalido)?;
        if !ids.insert(id.clone()) {
            return Err(ErroTelaDeAnexacao::IdsDeAnexadosRepetidos);
        }
        anexados.push(id);
    }
    Ok(TelaDeAnexacao { anexados })
}
/// Consulta `protocolo_RI1023`: ID, NUP e tipo únicos.
pub(crate) fn ler_destino(xml: &str) -> Result<Destino, ErroComplementosDaAnexacao> {
    let valores = ler_complementos(xml)?;
    let ids: Vec<_> = valores
        .iter()
        .filter(|(n, _)| n == "IdProcedimento")
        .collect();
    let nups: Vec<_> = valores
        .iter()
        .filter(|(n, _)| n == "ProtocoloProcedimentoFormatado")
        .collect();
    let tipos: Vec<_> = valores
        .iter()
        .filter(|(n, _)| n == "NomeTipoProcedimento")
        .collect();
    if ids.is_empty() || nups.is_empty() {
        return Err(ErroComplementosDaAnexacao::ProcessoAAnexarNaoIdentificado);
    }
    let ([(_, id)], [(_, nup)], false) = (ids.as_slice(), nups.as_slice(), tipos.len() > 1) else {
        return Err(ErroComplementosDaAnexacao::IdentidadeDeAnexadoAmbigua);
    };
    let (id, nup) = (id.as_str(), nup.as_str());
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) || nup.is_empty() {
        return Err(ErroComplementosDaAnexacao::ProcessoAAnexarNaoIdentificado);
    }
    Ok(Destino {
        id: id.to_owned(),
        nup: nup.to_owned(),
        tipo: tipos.first().map_or("", |v| v.1.as_str()).to_owned(),
    })
}
/// Consulta `procedimento_anexacao_verificar`: S=tudo assinado, N=há pendência.
pub(crate) fn ler_confirmacao(xml: &str) -> Result<bool, ErroComplementosDaAnexacao> {
    let valores = ler_complementos(xml)?;
    let encontrados: Vec<_> = valores.iter().filter(|(n, _)| n == "Anexacao").collect();
    match encontrados.as_slice() {
        [(_, valor)] if valor == "S" => Ok(true),
        [(_, valor)] if valor == "N" => Ok(false),
        _ => Err(ErroComplementosDaAnexacao::AnexacaoNaoVerificada),
    }
}
fn ler_complementos(xml: &str) -> Result<Vec<(String, String)>, ErroComplementosDaAnexacao> {
    let dom = tl::parse(xml, tl::ParserOptions::default())
        .map_err(ErroComplementosDaAnexacao::ComplementosDaAnexacaoNaoInterpretados)?;
    let parser = dom.parser();
    if dom
        .query_selector("erros")
        .into_iter()
        .flatten()
        .next()
        .is_some()
    {
        return Err(ErroComplementosDaAnexacao::ProcessoAAnexarNaoIdentificado);
    }
    let mut valores = Vec::new();
    for no in dom.query_selector("complemento").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        if let Some(nome) = atributo(tag, "nome")? {
            valores.push((
                nome,
                html_escape::decode_html_entities(&tag.inner_text(parser)).into_owned(),
            ));
        }
    }
    Ok(valores)
}
/// Recusa identidade igual ou já anexada antes da operação irreversível.
pub(crate) fn decidir(
    principal: &str,
    id_principal: &str,
    anexar: &Destino,
    existentes: &[String],
) -> Result<bool, ErroAnexacaoRecusada> {
    if principal == anexar.nup || id_principal == anexar.id {
        return Err(ErroAnexacaoRecusada::AnexacaoAoMesmoProcesso);
    }
    if existentes.contains(&anexar.id) {
        return Err(ErroAnexacaoRecusada::ProcessoJaAnexado);
    }
    Ok(true)
}
#[cfg(test)]
#[path = "processo_anexar/acervo.rs"]
mod acervo;
#[cfg(test)]
#[path = "processo_anexar/tests.rs"]
mod tests;
