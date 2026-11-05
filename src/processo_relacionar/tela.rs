//! Tela de relacionamentos do SEI 4.0 (PEN1): leitura estrutural, sem JS
//! executado e sem regex; todos os atributos passam por html_atributos.
use crate::formulario_web::{decodificar_atributo, localizar_formulario};
use crate::html_atributos::valor_do_atributo;
use crate::transporte::texto_de_recusa::decodificar_entidades;
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
}
fn atributo(tag: &tl::HTMLTag<'_>, nome: &'static str) -> Result<Option<String>, &'static str> {
    valor_do_atributo(tag, nome)
        .map_err(|_| "atributo_da_tela_ambiguo")?
        .map(|raw| decodificar_atributo(&raw).ok_or("entidade_html_nao_interpretada"))
        .transpose()
}
fn id_do_processo(href: &str) -> Option<&str> {
    let query = href.strip_prefix("controlador.php?")?;
    let mut acao = 0;
    let mut ids = Vec::new();
    for parte in query.split('&') {
        let (nome, valor) = parte.split_once('=')?;
        match nome {
            "acao" if valor == "procedimento_trabalhar" => acao += 1,
            "id_procedimento" if !valor.is_empty() && valor.bytes().all(|c| c.is_ascii_digit()) => ids.push(valor),
            "acao" | "id_procedimento" => return None,
            _ => (),
        }
    }
    if acao == 1 && ids.len() == 1 { Some(ids[0]) } else { None }
}

/// Só a linha da tabela própria do formulário é evidência da relação.
pub(super) fn ler_linhas(html: &str) -> Result<Vec<Relacionamento>, &'static str> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| "html_de_relacionamentos_nao_interpretado")?;
    let parser = dom.parser();
    let form = localizar_formulario(&dom, "frmProcedimentoListarRelacionamentos")
        .map_err(|_| "formulario_de_relacionamentos_ausente_ou_repetido")?;
    let mut tabelas = Vec::new();
    for tag in form.children().all(parser).iter().filter_map(tl::Node::as_tag) {
        if tag.name().as_utf8_str().eq_ignore_ascii_case("table")
            && atributo(tag, "summary")?.as_deref() == Some("Lista de Processos Relacionados")
        {
            tabelas.push(tag);
        }
    }
    let [tabela] = tabelas.as_slice() else {
        return Err("tabela_de_relacionamentos_ausente_ou_repetida");
    };
    let mut lista = Vec::new();
    let mut ids = HashSet::new();
    for linha in tabela.children().all(parser).iter()
        .filter_map(tl::Node::as_tag)
        .filter(|tag| tag.name().as_utf8_str().eq_ignore_ascii_case("tr"))
    {
        let mut processos = Vec::new();
        let mut remocoes = Vec::new();
        for ancora in linha.children().all(parser).iter()
            .filter_map(tl::Node::as_tag)
            .filter(|tag| tag.name().as_utf8_str().eq_ignore_ascii_case("a"))
        {
            if let Some(href) = atributo(ancora, "href")?
                && let Some(id) = id_do_processo(&href)
            {
                let nup = decodificar_entidades(&ancora.inner_text(parser));
                if nup.trim().is_empty() {
                    return Err("numero_do_relacionado_ausente");
                }
                processos.push((id.to_owned(), nup.trim().to_owned()));
            }
            if let Some(acao) = atributo(ancora, "onclick")? {
                remocoes.push(acao);
            }
        }
        match processos.as_slice() {
            [] => continue,
            [(id, nup)] => {
                if !ids.insert(id.clone()) {
                    return Err("relacionamentos_repetidos_na_tela");
                }
                let chamada = format!("acaoRemover('{id}');");
                let pode_remover = remocoes.iter().any(|acao| acao == &chamada);
                lista.push(Relacionamento { id: id.clone(), nup: nup.clone(), pode_remover });
            }
            _ => return Err("mais_de_um_processo_na_linha_de_relacionamento"),
        }
    }
    Ok(lista)
}

/// Lê a chamada literal que a tela serviu, sem avaliar JavaScript.
pub(super) fn url_complementar(html: &str) -> Result<String, &'static str> {
    const INICIO: &str = "new infraAjaxComplementar('txtProtocolo','";
    let urls: Vec<_> = html.lines().filter_map(|linha| {
        let (_, depois) = linha.split_once(INICIO)?;
        let (url, _) = depois.split_once("');")?;
        Some(url.to_owned())
    }).collect();
    match urls.as_slice() {
        [url] if url.starts_with("controlador_ajax.php?") => Ok(url.clone()),
        _ => Err("consulta_complementar_de_relacionamento_nao_declarada"),
    }
}

/// Só devolve identidade observada nos complementos únicos da resposta.
pub(super) fn ler_destino(xml: &str) -> Result<DestinoConsultado, &'static str> {
    let dom = tl::parse(xml, tl::ParserOptions::default())
        .map_err(|_| "resposta_da_consulta_nao_interpretada")?;
    let parser = dom.parser();
    if dom.query_selector("erros").into_iter().flatten().next().is_some() {
        return Err("processo_de_destino_nao_encontrado");
    }
    let mut id = Vec::new();
    let mut numero = Vec::new();
    for no in dom.query_selector("complemento").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else { continue };
        let valor = decodificar_entidades(&tag.inner_text(parser)).trim().to_owned();
        match atributo(tag, "nome")?.as_deref() {
            Some("IdProcedimento") => id.push(valor),
            Some("ProtocoloProcedimentoFormatado") => numero.push(valor),
            _ => (),
        }
    }
    match (id.as_slice(), numero.as_slice()) {
        ([id], [nup]) if !id.is_empty() && id.bytes().all(|c| c.is_ascii_digit()) && !nup.is_empty() => {
            Ok(DestinoConsultado { id: id.clone(), nup: nup.clone() })
        }
        ([], _) | (_, []) => Err("destino_a_relacionar_nao_identificado"),
        _ => Err("destino_da_consulta_ambiguo"),
    }
}
