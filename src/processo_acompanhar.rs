//! Núcleo verificado de `ies processo acompanhar` (#254).
//! A leitura/seleção não toca rede; a UI não infere grupo nem sucesso de escrita
//! pela resposta HTTP. Casos do Todos `tests/test_processo_acompanhar_010.py` @323eb01.

use crate::erro_html::ErroHtml;
use crate::html_atributos::{atributo_presente, valor_do_atributo};

mod exclusao;
mod fluxo;
mod submissao;
pub(crate) use fluxo::{Pedido, Previa, previa};

const FORMULARIO: &str = "frmAcompanhamentoCadastro";
const GRUPO: &str = "selGrupoAcompanhamento";
const PROTOCOLO: &str = "hdnIdProtocolo";
const TABELA: &str = "Tabela de Acompanhamentos.";

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroAcompanhar {
    CadastroNaoPublicado,
    CadastroDeOutroProcesso,
    GruposNaoPublicados,
    GrupoNaoOferecido,
    GrupoAmbiguo,
    ListaNaoPublicada,
    GrupoJaAcompanhado,
    NenhumAcompanhamento,
    EscolhaDeAcompanhamentoAmbigua,
    AcompanhamentoNaoConfirmado,
    ExclusaoNaoOferecida,
    Html(#[from] ErroHtml),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GrupoOferecido {
    pub(crate) valor: String,
    pub(crate) nome: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Acompanhamento {
    pub(crate) id: String,
    pub(crate) grupo: String,
    pub(crate) observacao: String,
}

fn texto(texto: &str) -> String {
    html_escape::decode_html_entities(texto)
        .into_owned()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn campo(tag: &tl::HTMLTag<'_>, nome: &'static str) -> Result<Option<String>, ErroAcompanhar> {
    valor_do_atributo(tag, nome)
        .map_err(|e| ErroAcompanhar::Html(ErroHtml::atributo("html_de_acompanhamento_invalido", e)))
}

/// A página de cadastro da `UnB` 4.0.9 tem um form e o select publicado.
/// Confere a identidade do processo no action E no oculto antes de escolher.
pub(crate) fn grupos_do_cadastro(
    html: &str,
    id_procedimento: &str,
) -> Result<Vec<GrupoOferecido>, ErroAcompanhar> {
    if html.len() > crate::MAX_HTML_BYTES {
        return Err(ErroHtml::html("html_de_acompanhamento_invalido").into());
    }
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroHtml::html("html_de_acompanhamento_invalido"))?;
    let parser = dom.parser();
    let mut forms = Vec::new();
    for no in dom.query_selector("form").into_iter().flatten() {
        let tag = no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroHtml::html("html_de_acompanhamento_invalido"))?;
        if campo(tag, "id")?.as_deref() == Some(FORMULARIO) {
            forms.push(tag);
        }
    }
    let form = match forms.as_slice() {
        [] => return Err(ErroAcompanhar::CadastroNaoPublicado),
        [form] => *form,
        _ => return Err(ErroHtml::repetido("formularios_de_acompanhamento_repetidos").into()),
    };
    let action = campo(form, "action")?.ok_or(ErroAcompanhar::CadastroNaoPublicado)?;
    let action = crate::formulario_web::decodificar_atributo(&action)
        .ok_or(ErroHtml::html("html_de_acompanhamento_invalido"))?;
    if !parametro_unico(&action, "acao", "acompanhamento_cadastrar")
        || !parametro_unico(&action, "id_procedimento", id_procedimento)
    {
        return Err(ErroAcompanhar::CadastroDeOutroProcesso);
    }
    let mut protocolos = Vec::new();
    let mut selects = Vec::new();
    for no in form.children().all(parser) {
        let Some(tag) = no.as_tag() else { continue };
        let nome_tag = tag.name().as_utf8_str();
        if nome_tag == "input" && campo(tag, "name")?.as_deref() == Some(PROTOCOLO) {
            protocolos.push(campo(tag, "value")?.unwrap_or_default());
        }
        if nome_tag == "select" && campo(tag, "name")?.as_deref() == Some(GRUPO) {
            selects.push(tag);
        }
    }
    if protocolos != [id_procedimento] {
        return Err(ErroAcompanhar::CadastroDeOutroProcesso);
    }
    let select = match selects.as_slice() {
        [select] => *select,
        _ => return Err(ErroAcompanhar::GruposNaoPublicados),
    };
    let mut grupos = Vec::new();
    for no in select.children().all(parser) {
        let Some(tag) = no.as_tag() else { continue };
        if tag.name() != "option" {
            continue;
        }
        if atributo_presente(tag, "disabled")
            .map_err(|e| ErroHtml::atributo("html_de_acompanhamento_invalido", e))?
        {
            continue;
        }
        let valor = campo(tag, "value")?.ok_or(ErroHtml::valor(
            "opcoes_de_grupo_de_acompanhamento_invalidas",
        ))?;
        let nome = texto(&tag.inner_text(parser));
        if nome.is_empty() && valor.is_empty() {
            // Uma opção em branco pode ter valor vazio; não é ambiguidade de grupo.
            grupos.push(GrupoOferecido { valor, nome });
            continue;
        }
        if valor.is_empty() {
            return Err(ErroHtml::valor("opcoes_de_grupo_de_acompanhamento_invalidas").into());
        }
        grupos.push(GrupoOferecido { valor, nome });
    }
    if grupos.is_empty() {
        return Err(ErroAcompanhar::GruposNaoPublicados);
    }
    Ok(grupos)
}

fn parametro_unico(url: &str, nome: &str, valor: &str) -> bool {
    url.split_once('?').is_some_and(|(_, query)| {
        query
            .split('&')
            .filter_map(|par| par.split_once('='))
            .filter(|(chave, _)| *chave == nome)
            .map(|(_, v)| v)
            .collect::<Vec<_>>()
            == [valor]
    })
}

/// Casa APENAS pelo nome exato. O branco seleciona a opção vazia publicada.
pub(crate) fn escolher_grupo<'a>(
    grupos: &'a [GrupoOferecido],
    pedido: Option<&str>,
) -> Result<&'a GrupoOferecido, ErroAcompanhar> {
    let nome = pedido.unwrap_or("");
    let achados: Vec<_> = grupos.iter().filter(|g| g.nome == nome).collect();
    match achados.as_slice() {
        [] => Err(ErroAcompanhar::GrupoNaoOferecido),
        [grupo] => Ok(grupo),
        _ => Err(ErroAcompanhar::GrupoAmbiguo),
    }
}

/// Uma lista vazia SÓ é comprovada por tabela com cabeçalho, nunca por HTML
/// cortado, tela de login ou ausência de tabela.
pub(crate) fn acompanhamentos_da_lista(html: &str) -> Result<Vec<Acompanhamento>, ErroAcompanhar> {
    if html.len() > crate::MAX_HTML_BYTES {
        return Err(ErroHtml::html("html_de_acompanhamento_invalido").into());
    }
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroHtml::html("html_de_acompanhamento_invalido"))?;
    let parser = dom.parser();
    let mut tabelas = Vec::new();
    for no in dom.query_selector("table").into_iter().flatten() {
        let tag = no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroHtml::html("html_de_acompanhamento_invalido"))?;
        if campo(tag, "summary")?.as_deref() == Some(TABELA) {
            tabelas.push(tag);
        }
    }
    let tabela = match tabelas.as_slice() {
        [] => return Err(ErroAcompanhar::ListaNaoPublicada),
        [tabela] => *tabela,
        _ => return Err(ErroHtml::repetido("listas_de_acompanhamentos_repetidas").into()),
    };
    let mut cabecalho = false;
    let mut linhas = Vec::new();
    for no in tabela.children().all(parser) {
        let Some(tr) = no.as_tag() else { continue };
        if tr.name() != "tr" {
            continue;
        }
        let mut th = Vec::new();
        let mut td = Vec::new();
        for celula in tr.children().all(parser) {
            let Some(tag) = celula.as_tag() else { continue };
            if tag.name() == "th" {
                th.push(texto(&tag.inner_text(parser)));
            }
            if tag.name() == "td" {
                td.push(tag);
            }
        }
        if !th.is_empty() {
            if cabecalho
                || !th.iter().any(|h| h == "Grupo")
                || !th.iter().any(|h| h == "Observação")
            {
                return Err(ErroHtml::valor("colunas_de_acompanhamentos_invalidas").into());
            }
            cabecalho = true;
            continue;
        }
        if td.is_empty() {
            continue;
        }
        if td.len() != 6 {
            return Err(ErroHtml::valor("linha_de_acompanhamento_invalida").into());
        }
        let mut ids = Vec::new();
        for item in td[0].children().all(parser) {
            let Some(tag) = item.as_tag() else { continue };
            if tag.name() != "input" {
                continue;
            }
            if campo(tag, "type")?.as_deref() == Some("checkbox") {
                ids.push(campo(tag, "value")?.unwrap_or_default());
            }
        }
        let id = match ids.as_slice() {
            [id] if !id.is_empty() => id.clone(),
            _ => return Err(ErroHtml::valor("linha_de_acompanhamento_invalida").into()),
        };
        if linhas.iter().any(|l: &Acompanhamento| l.id == id) {
            return Err(ErroHtml::repetido("id_de_acompanhamento_repetido").into());
        }
        linhas.push(Acompanhamento {
            id,
            grupo: texto(&td[1].inner_text(parser)),
            observacao: texto(&td[2].inner_text(parser)),
        });
    }
    if !cabecalho {
        return Err(ErroHtml::valor("colunas_de_acompanhamentos_invalidas").into());
    }
    Ok(linhas)
}

/// Se já existe este grupo na mesma unidade, não escrever nem quando a
/// observação é diferente: o SEI recusa duplicatas de grupo.
pub(crate) fn conferir_novo_grupo(
    antes: &[Acompanhamento],
    grupo: &str,
) -> Result<(), ErroAcompanhar> {
    if antes.iter().any(|a| a.grupo == grupo) {
        Err(ErroAcompanhar::GrupoJaAcompanhado)
    } else {
        Ok(())
    }
}

/// Escolha de alvo da remoção; sem grupo só quando há exatamente uma linha.
pub(crate) fn alvo_para_deixar<'a>(
    antes: &'a [Acompanhamento],
    grupo: Option<&str>,
) -> Result<&'a Acompanhamento, ErroAcompanhar> {
    let candidatos: Vec<_> = antes
        .iter()
        .filter(|a| grupo.is_none_or(|g| a.grupo == g))
        .collect();
    match candidatos.as_slice() {
        [] => Err(ErroAcompanhar::NenhumAcompanhamento),
        [a] => Ok(a),
        _ => Err(ErroAcompanhar::EscolhaDeAcompanhamentoAmbigua),
    }
}

/// A operação só está confirmada se apareceu exatamente UMA linha nova
/// com grupo/observação pedidos e TODAS as linhas anteriores intactas.
pub(crate) fn conferir_inclusao(
    antes: &[Acompanhamento],
    depois: &[Acompanhamento],
    grupo: &str,
    observacao: &str,
) -> Result<(), ErroAcompanhar> {
    if depois.len() != antes.len() + 1
        || !antes.iter().all(|a| depois.contains(a))
        || depois
            .iter()
            .filter(|a| !antes.iter().any(|b| b.id == a.id))
            .filter(|a| a.grupo == grupo && a.observacao == observacao)
            .count()
            != 1
    {
        return Err(ErroAcompanhar::AcompanhamentoNaoConfirmado);
    }
    Ok(())
}

/// Só remova quando a identidade exata desaparecer e todo o resto for idêntico.
pub(crate) fn conferir_remocao(
    antes: &[Acompanhamento],
    depois: &[Acompanhamento],
    id: &str,
) -> Result<(), ErroAcompanhar> {
    if antes.len() != depois.len() + 1
        || !antes.iter().any(|a| a.id == id)
        || depois.iter().any(|a| a.id == id)
        || !antes
            .iter()
            .filter(|a| a.id != id)
            .all(|a| depois.contains(a))
    {
        return Err(ErroAcompanhar::AcompanhamentoNaoConfirmado);
    }
    Ok(())
}

#[cfg(test)]
#[path = "processo_acompanhar_tests.rs"]
mod tests;
