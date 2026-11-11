//! Tela de relacionamentos do SEI 4.0 (PEN1): leitura estrutural, sem JS
//! executado e sem regex; todos os atributos passam por `html_atributos`.
use crate::acao_do_sei::AcaoDoSei;
use crate::formulario_web::{ErroSubmissaoFormulario, decodificar_atributo, localizar_formulario};
use crate::html_atributos::{ErroAtributoHtml, valor_do_atributo};
use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Relacionamento {
    pub(super) id: String,
    pub(super) nup: String,
    pub(super) pode_remover: bool,
}
pub(super) struct DestinoConsultado {
    pub(super) id: String,
    pub(super) nup: String,
    pub(super) tipo: Option<String>,
}
/// Um atributo da tela ou da resposta de relacionamento que não pôde ser lido.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroAtributoDeRelacionamento {
    AtributoDaTelaAmbiguo(#[source] ErroAtributoHtml),
    EntidadeHtmlNaoInterpretada,
}

/// A lista `frmProcedimentoListarRelacionamentos` fora da forma observada.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroTelaDeRelacionamentos {
    Atributo(#[from] ErroAtributoDeRelacionamento),
    HtmlDeRelacionamentosNaoInterpretado,
    FormularioDeRelacionamentosNaoLocalizado(#[source] ErroSubmissaoFormulario),
    TabelaDeRelacionamentosAusenteOuRepetida,
    NumeroDoRelacionadoAusente,
    RelacionamentosRepetidosNaTela,
    MaisDeUmProcessoNaLinhaDeRelacionamento,
}

/// A resposta XML da consulta `protocolo_RI1023` do destino.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroDestinoDaConsulta {
    Atributo(#[from] ErroAtributoDeRelacionamento),
    RespostaDaConsultaNaoInterpretada,
    ProcessoDeDestinoNaoEncontrado,
    DestinoDaConsultaAmbiguo,
    DestinoARelacionarNaoIdentificado,
}

fn atributo(
    tag: &tl::HTMLTag<'_>,
    nome: &'static str,
) -> Result<Option<String>, ErroAtributoDeRelacionamento> {
    valor_do_atributo(tag, nome)
        .map_err(ErroAtributoDeRelacionamento::AtributoDaTelaAmbiguo)?
        .map(|raw| {
            decodificar_atributo(&raw)
                .ok_or(ErroAtributoDeRelacionamento::EntidadeHtmlNaoInterpretada)
        })
        .transpose()
}
fn id_do_processo(href: &str) -> Option<&str> {
    let query = href.strip_prefix("controlador.php?")?;
    let mut acao = 0;
    let mut ids = Vec::new();
    for parte in query.split('&') {
        let (nome, valor) = parte.split_once('=')?;
        match nome {
            "acao" if AcaoDoSei::ler(valor) == Some(AcaoDoSei::ProcedimentoTrabalhar) => acao += 1,
            "id_procedimento" if !valor.is_empty() && valor.bytes().all(|c| c.is_ascii_digit()) => {
                ids.push(valor);
            }
            "acao" | "id_procedimento" => return None,
            _ => (),
        }
    }
    if let (1, [id]) = (acao, ids.as_slice()) {
        Some(*id)
    } else {
        None
    }
}

/// Só a linha da tabela própria do formulário é evidência da relação.
pub(super) fn ler_linhas(html: &str) -> Result<Vec<Relacionamento>, ErroTelaDeRelacionamentos> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroTelaDeRelacionamentos::HtmlDeRelacionamentosNaoInterpretado)?;
    let parser = dom.parser();
    let form = localizar_formulario(&dom, "frmProcedimentoListarRelacionamentos")
        .map_err(ErroTelaDeRelacionamentos::FormularioDeRelacionamentosNaoLocalizado)?;
    let mut tabelas = Vec::new();
    for tag in form
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        if tag.name().as_utf8_str().eq_ignore_ascii_case("table")
            && atributo(tag, "summary")?.as_deref() == Some("Lista de Processos Relacionados")
        {
            tabelas.push(tag);
        }
    }
    let tabela = match tabelas.as_slice() {
        [unica] => *unica,
        [] => {
            // PEN1 (07/11/2026): a lista vazia não publica tabela,
            // mas o formulário de relacionamento e seus campos existem.
            // Não aceitar uma tela genérica apenas por não conter tabela.
            let mut protocolo = 0;
            let mut id_protocolo = 0;
            for campo in form
                .children()
                .all(parser)
                .iter()
                .filter_map(tl::Node::as_tag)
            {
                if !campo.name().as_utf8_str().eq_ignore_ascii_case("input") {
                    continue;
                }
                match atributo(campo, "id")?.as_deref() {
                    Some("txtProtocolo") => protocolo += 1,
                    Some("hdnIdProtocolo") => id_protocolo += 1,
                    _ => {}
                }
            }
            if protocolo == 1 && id_protocolo == 1 {
                return Ok(Vec::new());
            }
            return Err(ErroTelaDeRelacionamentos::TabelaDeRelacionamentosAusenteOuRepetida);
        }
        _ => return Err(ErroTelaDeRelacionamentos::TabelaDeRelacionamentosAusenteOuRepetida),
    };
    let mut lista = Vec::new();
    let mut ids = HashSet::new();
    for linha in tabela
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
        .filter(|tag| tag.name().as_utf8_str().eq_ignore_ascii_case("tr"))
    {
        let mut processos = Vec::new();
        let mut remocoes = Vec::new();
        for ancora in linha
            .children()
            .all(parser)
            .iter()
            .filter_map(tl::Node::as_tag)
            .filter(|tag| tag.name().as_utf8_str().eq_ignore_ascii_case("a"))
        {
            if let Some(href) = atributo(ancora, "href")?
                && let Some(id) = id_do_processo(&href)
            {
                let nup =
                    html_escape::decode_html_entities(&ancora.inner_text(parser)).into_owned();
                if nup.trim().is_empty() {
                    return Err(ErroTelaDeRelacionamentos::NumeroDoRelacionadoAusente);
                }
                processos.push((id.to_owned(), nup.trim().to_owned()));
            }
            if let Some(acao) = atributo(ancora, "onclick")? {
                remocoes.push(acao);
            }
        }
        match processos.as_slice() {
            [] => {}
            [(id, nup)] => {
                if !ids.insert(id.clone()) {
                    return Err(ErroTelaDeRelacionamentos::RelacionamentosRepetidosNaTela);
                }
                let chamada = format!("acaoRemover('{id}');");
                let pode_remover = remocoes.iter().any(|acao| acao == &chamada);
                lista.push(Relacionamento {
                    id: id.clone(),
                    nup: nup.clone(),
                    pode_remover,
                });
            }
            _ => return Err(ErroTelaDeRelacionamentos::MaisDeUmProcessoNaLinhaDeRelacionamento),
        }
    }
    Ok(lista)
}

/// Lê a chamada literal que a tela serviu, sem avaliar JavaScript; `None`
/// se a tela não declara exatamente uma consulta relativa.
pub(super) fn url_complementar(html: &str) -> Option<String> {
    const INICIO: &str = "new infraAjaxComplementar('txtProtocolo','";
    let urls: Vec<_> = html
        .lines()
        .filter_map(|linha| {
            let (_, depois) = linha.split_once(INICIO)?;
            let (url, _) = depois.split_once("');")?;
            Some(url.to_owned())
        })
        .collect();
    match urls.as_slice() {
        [url] if url.starts_with("controlador_ajax.php?") => Some(url.clone()),
        _ => None,
    }
}

/// Só devolve identidade observada nos complementos únicos da resposta.
pub(super) fn ler_destino(xml: &str) -> Result<DestinoConsultado, ErroDestinoDaConsulta> {
    let dom = tl::parse(xml, tl::ParserOptions::default())
        .map_err(|_| ErroDestinoDaConsulta::RespostaDaConsultaNaoInterpretada)?;
    let parser = dom.parser();
    if dom
        .query_selector("erros")
        .into_iter()
        .flatten()
        .next()
        .is_some()
    {
        return Err(ErroDestinoDaConsulta::ProcessoDeDestinoNaoEncontrado);
    }
    let mut id = Vec::new();
    let mut numero = Vec::new();
    let mut tipos = Vec::new();
    for no in dom.query_selector("complemento").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let valor = html_escape::decode_html_entities(&tag.inner_text(parser))
            .into_owned()
            .trim()
            .to_owned();
        match atributo(tag, "nome")?.as_deref() {
            Some("IdProcedimento") => id.push(valor),
            Some("ProtocoloProcedimentoFormatado") => numero.push(valor),
            Some("NomeTipoProcedimento") => tipos.push(valor),
            _ => (),
        }
    }
    match (id.as_slice(), numero.as_slice()) {
        ([id], [nup])
            if !id.is_empty() && id.bytes().all(|c| c.is_ascii_digit()) && !nup.is_empty() =>
        {
            let tipo = match tipos.as_slice() {
                [] => None,
                [nome] if nome.is_empty() => None,
                [nome] => Some(nome.clone()),
                _ => return Err(ErroDestinoDaConsulta::DestinoDaConsultaAmbiguo),
            };
            Ok(DestinoConsultado {
                id: id.clone(),
                nup: nup.clone(),
                tipo,
            })
        }
        ([], _) | (_, []) => Err(ErroDestinoDaConsulta::DestinoARelacionarNaoIdentificado),
        _ => Err(ErroDestinoDaConsulta::DestinoDaConsultaAmbiguo),
    }
}

/// Deriva a action do handler literal acaoRemover, nunca de texto solto.
/// Não interpreta JavaScript. A submissão ainda precisa ser validada
/// pela fronteira de formulários antes de iniciar um único POST. `None`
/// se a tela não declara exatamente um `acaoRemover` na forma observada.
pub(super) fn url_exclusao(html: &str) -> Option<String> {
    const FUNCAO: &str = "function acaoRemover(id){";
    const ID: &str = "document.getElementById('hdnInfraItemId').value=id;";
    const ACTION: &str = "document.getElementById('frmProcedimentoListarRelacionamentos').action='";
    const ENVIAR: &str =
        "document.getElementById('frmProcedimentoListarRelacionamentos').submit();";

    let dom = tl::parse(html, tl::ParserOptions::default()).ok()?;
    let parser = dom.parser();
    // Um script dentro de template não oferece evento acionável.
    let inertes: Vec<_> = dom
        .query_selector("template")
        .into_iter()
        .flatten()
        .filter_map(|n| n.get(parser).and_then(tl::Node::as_tag))
        .flat_map(|t| t.children().all(parser))
        .filter_map(tl::Node::as_tag)
        .filter(|t| t.name().as_utf8_str().eq_ignore_ascii_case("script"))
        .map(|t| t.raw().as_utf8_str().into_owned())
        .collect();

    let mut urls = Vec::new();
    for no in dom.query_selector("script").into_iter().flatten() {
        let Some(script) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        // Fonte bruta, não inner_text: o SEI 4 da `UnB` envolve o script em
        // `<!--//--><![CDATA[//><!--`, que o tokenizador lê como comentário
        // e inner_text omite (live de 10/11/2026, lista de relacionamentos).
        let texto = script.raw().as_utf8_str().into_owned();
        if inertes.contains(&texto) {
            continue;
        }
        let linhas: Vec<_> = texto.lines().map(str::trim).collect();
        let candidatos: Vec<_> = linhas
            .iter()
            .enumerate()
            .filter_map(|(i, linha)| linha.starts_with(FUNCAO).then_some(i))
            .collect();
        let [inicio] = candidatos.as_slice() else {
            if !candidatos.is_empty() {
                return None;
            }
            continue;
        };
        let (antes, a_partir) = linhas.split_at(*inicio);
        let [funcao, depois @ ..] = a_partir else {
            return None;
        };
        if *funcao != FUNCAO {
            return None;
        }
        // Uma declaração dentro de comentário de bloco não é executável.
        let prefixo = antes.join("\n");
        let ultimo_abre = prefixo.rfind("/*");
        let ultimo_fecha = prefixo.rfind("*/");
        if ultimo_abre.is_some_and(|p| ultimo_fecha.is_none_or(|f| f < p)) {
            continue;
        }
        // O primeiro fechamento pertence à guarda confirm da função observada.
        let Some(fim) = depois.iter().position(|linha| *linha == "}") else {
            continue;
        };
        let (corpo, _) = depois.split_at(fim);
        let pos_id = corpo.iter().position(|l| *l == ID);
        let pos_action = corpo.iter().position(|l| l.starts_with(ACTION));
        let pos_enviar = corpo.iter().position(|l| *l == ENVIAR);
        let (Some(i), Some(a), Some(s)) = (pos_id, pos_action, pos_enviar) else {
            continue;
        };
        if !(i < a && a < s) {
            continue;
        }
        let Some(url) = corpo
            .get(a)
            .and_then(|linha| linha.strip_prefix(ACTION))
            .and_then(|l| l.strip_suffix("';"))
        else {
            continue;
        };
        if url.starts_with("controlador.php?acao=procedimento_excluir_relacionamento&") {
            urls.push(url.to_owned());
        }
    }
    match urls.as_slice() {
        [unica] => Some(unica.clone()),
        _ => None,
    }
}
