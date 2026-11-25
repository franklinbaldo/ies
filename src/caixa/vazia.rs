//! Lista vazia comprovada pela estrutura da visão R, sem `<table>`.
//!
//! Observado em GO (`unidade-troca-e-volta-go#6`, `bloco-devolver-go#38`,
//! unidade sem processos): o SEI omite `tbl{T}` e deixa vazias
//! `div{T}AreaTabela` e as duas `div{T}AreaPaginacao*`, com `hdn{T}NroItens`
//! e `hdn{T}PaginaAtual` em 0. Nenhuma mensagem de lista vazia. Só o
//! conjunto inteiro, na visão R com a unidade identificada, prova zero
//! processos; qualquer parte ausente, repetida ou diferente mantém
//! `tabela_ausente`.
use crate::controle_contexto::contexto_da_pagina;
use crate::dominio::Categoria;

fn unico<'d>(
    dom: &'d crate::tela_html::Documento,
    id: &str,
) -> Option<crate::tela_html::Elemento<'d>> {
    let nos = dom.com_id(id).collect::<Vec<_>>();
    let [no] = nos.as_slice() else {
        return None;
    };
    Some(*no)
}

/// `div` única só com espaços: nenhuma tag, nenhum texto.
fn div_vazia(dom: &crate::tela_html::Documento, id: &str) -> bool {
    let Some(div) = unico(dom, id) else {
        return false;
    };
    div.nome() == "div"
        && div.nos().all(|filho| {
            filho
                .como_texto()
                .is_some_and(|texto| texto.trim().is_empty())
        })
}

/// `input` único com `value` exatamente `0`.
fn oculto_zero(dom: &crate::tela_html::Documento, id: &str) -> bool {
    let Some(campo) = unico(dom, id) else {
        return false;
    };
    let valor = campo.atributo("value");
    campo.nome() == "input" && valor == Some("0")
}

/// A tela prova que a categoria não tem processos (ver o módulo).
pub(crate) fn lista_vazia_comprovada(
    html: &crate::tela_html::HtmlServido,
    dom: &crate::tela_html::Documento,
    categoria: Categoria,
) -> bool {
    let t = match categoria {
        Categoria::Recebidos => "Recebidos",
        Categoria::Gerados => "Gerados",
    };
    let Ok(contexto) = contexto_da_pagina(html) else {
        return false;
    };
    if !contexto.eh_resumida() || contexto.unidade.is_none() {
        return false;
    }
    [
        "AreaTabela",
        "AreaPaginacaoSuperior",
        "AreaPaginacaoInferior",
    ]
    .iter()
    .all(|div| div_vazia(dom, &format!("div{t}{div}")))
        && oculto_zero(dom, &format!("hdn{t}NroItens"))
        && oculto_zero(dom, &format!("hdn{t}PaginaAtual"))
}
