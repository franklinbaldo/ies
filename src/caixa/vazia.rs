//! Lista vazia comprovada pela estrutura da visão R, sem `<table>`.
//!
//! Observado em GO (`unidade-troca-e-volta-go#6`, `bloco-devolver-go#38`,
//! unidade sem processos): o SEI omite `tbl{T}` e deixa vazias
//! `div{T}AreaTabela` e as duas `div{T}AreaPaginacao*`, com `hdn{T}NroItens`
//! e `hdn{T}PaginaAtual` em 0. Nenhuma mensagem de lista vazia. Só o
//! conjunto inteiro, na visão R com a unidade identificada, prova zero
//! processos; qualquer parte ausente, repetida ou diferente mantém
//! `tabela_ausente`.
use crate::controle_contexto::contexto_do_dom;
use crate::dominio::Categoria;
use crate::erro_web::ErroCaixa;
use crate::html_atributos::{nos_com_id, valor_do_atributo};

fn unico<'d>(dom: &'d tl::VDom<'_>, id: &str) -> Result<Option<&'d tl::HTMLTag<'d>>, ErroCaixa> {
    let nos = nos_com_id(dom, id).map_err(ErroCaixa::AtributoDaCaixaIlegivel)?;
    let [no] = nos.as_slice() else {
        return Ok(None);
    };
    Ok(no.get(dom.parser()).and_then(tl::Node::as_tag))
}

/// `div` única só com espaços: nenhuma tag, nenhum texto.
fn div_vazia(dom: &tl::VDom<'_>, id: &str) -> Result<bool, ErroCaixa> {
    let Some(div) = unico(dom, id)? else {
        return Ok(false);
    };
    let parser = dom.parser();
    Ok(div.name() == "div"
        && div.children().top().iter().all(|filho| {
            filho
                .get(parser)
                .and_then(tl::Node::as_raw)
                .is_some_and(|texto| texto.as_utf8_str().trim().is_empty())
        }))
}

/// `input` único com `value` exatamente `0`.
fn oculto_zero(dom: &tl::VDom<'_>, id: &str) -> Result<bool, ErroCaixa> {
    let Some(campo) = unico(dom, id)? else {
        return Ok(false);
    };
    let valor = valor_do_atributo(campo, "value").map_err(ErroCaixa::AtributoDaCaixaIlegivel)?;
    Ok(campo.name() == "input" && valor.as_deref() == Some("0"))
}

/// A tela prova que a categoria não tem processos (ver o módulo).
pub(crate) fn lista_vazia_comprovada(
    dom: &tl::VDom<'_>,
    categoria: Categoria,
) -> Result<bool, ErroCaixa> {
    let t = match categoria {
        Categoria::Recebidos => "Recebidos",
        Categoria::Gerados => "Gerados",
    };
    let Ok(contexto) = contexto_do_dom(dom) else {
        return Ok(false);
    };
    if !contexto.eh_resumida() || contexto.unidade.is_none() {
        return Ok(false);
    }
    for div in [
        "AreaTabela",
        "AreaPaginacaoSuperior",
        "AreaPaginacaoInferior",
    ] {
        if !div_vazia(dom, &format!("div{t}{div}"))? {
            return Ok(false);
        }
    }
    Ok(oculto_zero(dom, &format!("hdn{t}NroItens"))?
        && oculto_zero(dom, &format!("hdn{t}PaginaAtual"))?)
}
