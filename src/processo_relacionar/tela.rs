//! Tela de relacionamentos do SEI 4.0 (PEN1): leitura estrutural, sem JS
//! executado e sem regex; atributos lidos da árvore `tela_html`.
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::formulario_web::{ErroSubmissaoFormulario, localizar_formulario};
use crate::resposta_ajax::{ErroXml, RespostaAjax};
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
pub(crate) enum ErroAtributoDeRelacionamento {}

/// A lista `frmProcedimentoListarRelacionamentos` fora da forma observada.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroTelaDeRelacionamentos {
    Atributo(#[from] ErroAtributoDeRelacionamento),
    FormularioDeRelacionamentosNaoLocalizado(#[source] ErroSubmissaoFormulario),
    TabelaDeRelacionamentosAusenteOuRepetida,
    NumeroDoRelacionadoAusente,
    RelacionamentosRepetidosNaTela,
    MaisDeUmProcessoNaLinhaDeRelacionamento,
}

/// A resposta XML da consulta `protocolo_RI1023` do destino.
#[derive(Clone, Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroDestinoDaConsulta {
    Xml(#[from] ErroXml),
    ProcessoDeDestinoNaoEncontrado,
    DestinoDaConsultaAmbiguo,
    DestinoARelacionarNaoIdentificado,
}

fn id_do_processo(href: &str) -> Option<&str> {
    let query = href.strip_prefix("controlador.php?")?;
    let Ok(query) = ConsultaDoControlador::ler(query) else {
        return None;
    };
    let Ok(id) = query.unico("id_procedimento") else {
        return None;
    };
    (query.acao() == Ok(AcaoDoSei::ProcedimentoTrabalhar.como_na_url())
        && !id.is_empty()
        && id.bytes().all(|c| c.is_ascii_digit()))
    .then_some(id)
}

/// Só a linha da tabela própria do formulário é evidência da relação.
pub(super) fn ler_linhas(
    html: &crate::tela_html::HtmlServido,
) -> Result<Vec<Relacionamento>, ErroTelaDeRelacionamentos> {
    let dom = crate::tela_html::arvore(html);
    let form = localizar_formulario(&dom, "frmProcedimentoListarRelacionamentos")
        .map_err(ErroTelaDeRelacionamentos::FormularioDeRelacionamentosNaoLocalizado)?;
    let mut tabelas = Vec::new();
    for tag in form.descendentes() {
        if tag.nome().eq_ignore_ascii_case("table")
            && tag.atributo("summary") == Some("Lista de Processos Relacionados")
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
            for campo in form.descendentes() {
                if !campo.nome().eq_ignore_ascii_case("input") {
                    continue;
                }
                match campo.atributo("id") {
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
        .descendentes()
        .filter(|tag| tag.nome().eq_ignore_ascii_case("tr"))
    {
        let mut processos = Vec::new();
        let mut remocoes = Vec::new();
        for ancora in linha
            .descendentes()
            .filter(|tag| tag.nome().eq_ignore_ascii_case("a"))
        {
            if let Some(href) = ancora.atributo("href").map(str::to_owned)
                && let Some(id) = id_do_processo(&href)
            {
                let nup = ancora.texto();
                if nup.trim().is_empty() {
                    return Err(ErroTelaDeRelacionamentos::NumeroDoRelacionadoAusente);
                }
                processos.push((id.to_owned(), nup.trim().to_owned()));
            }
            remocoes.extend(ancora.atributo("onclick").map(str::to_owned));
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
    let resposta = RespostaAjax::ler(xml)?;
    if !resposta.elementos("erros").is_empty() {
        return Err(ErroDestinoDaConsulta::ProcessoDeDestinoNaoEncontrado);
    }
    let mut id = Vec::new();
    let mut numero = Vec::new();
    let mut tipos = Vec::new();
    for complemento in resposta.elementos("complemento") {
        let valor = complemento.texto().trim().to_owned();
        match complemento.atributo("nome") {
            Some("IdProcedimento") => id.push(valor),
            Some("ProtocoloProcedimentoFormatado") => numero.push(valor),
            Some("NomeTipoProcedimento") => tipos.push(valor),
            Some(_) | None => (),
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
pub(super) fn url_exclusao(html: &crate::tela_html::HtmlServido) -> Option<String> {
    const FUNCAO: &str = "function acaoRemover(id){";
    const ID: &str = "document.getElementById('hdnInfraItemId').value=id;";
    const ACTION: &str = "document.getElementById('frmProcedimentoListarRelacionamentos').action='";
    const ENVIAR: &str =
        "document.getElementById('frmProcedimentoListarRelacionamentos').submit();";

    let dom = crate::tela_html::arvore(html);
    // Um script dentro de template não oferece evento acionável; como no
    // navegador, ele nem está na árvore.
    let mut urls = Vec::new();
    for no in dom.elementos("script") {
        // O conteúdo inteiro do script, inclusive o invólucro
        // `<!--//--><![CDATA[//><!--` do SEI 4 da `UnB` (live de 10/11/2026).
        let texto = no.texto();
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
