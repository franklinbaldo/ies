//! Conferência estrutural do formulário de sobrestamento observado na `UnB` (#269).
//! Fonte: `Todos tests/test_ciclo_sobrestar_010.py` @323eb01.
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, ResultadoDoEnvio, localizar_formulario,
};
use crate::html_atributos::{atributo_presente, valor_do_atributo};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroTela {
    NaoInterpretada,
    FormularioAusente,
    OutroProcesso,
    OpcaoNaoOferecida,
    DestinoPreenchido,
    CampoInesperado,
}
pub(crate) const OPERACAO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "procedimento_sobrestar",
    formulario: "frmDesentranharDocumento",
    envio: EnvioDoFormulario::BotaoDeEnvio("sbmSalvar"),
    campos_ocultos_exigidos: &[
        "hdnInfraTipoPagina",
        "hdnIdProtocolos",
        "hdnIdProcedimentoDestino",
    ],
    campos_ocultos_reescritos: &[],
    campos_usuario: &[
        "rdoSobrestar",
        "txtProcedimentoDestino",
        "txtIdentificacaoProcedimentoDestino",
        "txaMotivo",
    ],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::Sobrestamento,
    },
};

/// Confere os controles antes de derivar um único POST.
pub(crate) fn conferir_formulario(html: &str, id: &str) -> Result<(), ErroTela> {
    let dom =
        tl::parse(html, tl::ParserOptions::default()).map_err(|_| ErroTela::NaoInterpretada)?;
    let form =
        localizar_formulario(&dom, OPERACAO.formulario).map_err(|_| ErroTela::FormularioAusente)?;
    let atributo = |tag: &tl::HTMLTag<'_>, nome: &'static str| {
        valor_do_atributo(tag, nome).map_err(|_| ErroTela::NaoInterpretada)
    };
    let presente = |tag: &tl::HTMLTag<'_>, nome: &'static str| {
        atributo_presente(tag, nome).map_err(|_| ErroTela::NaoInterpretada)
    };
    let action = atributo(form, "action")?.ok_or(ErroTela::OutroProcesso)?;
    let action =
        crate::formulario_web::decodificar_atributo(&action).ok_or(ErroTela::NaoInterpretada)?;
    let ids: Vec<_> = action
        .split_once('?')
        .map(|(_, consulta)| consulta)
        .unwrap_or_default()
        .split('&')
        .filter_map(|par| par.strip_prefix("id_procedimento="))
        .collect();
    if ids.as_slice() != [id] {
        return Err(ErroTela::OutroProcesso);
    }
    if !atributo(form, "method")?.is_some_and(|v| v.eq_ignore_ascii_case("post")) {
        return Err(ErroTela::CampoInesperado);
    }
    // An external form-associated field changes the browser POST.
    for elemento in ["input", "button", "select", "textarea"] {
        for no in dom.query_selector(elemento).into_iter().flatten() {
            let Some(tag) = no.get(dom.parser()).and_then(tl::Node::as_tag) else {
                continue;
            };
            if atributo(tag, "form")?.is_some() {
                return Err(ErroTela::CampoInesperado);
            }
        }
    }
    let mut vistos: HashMap<String, usize> = HashMap::new();
    for tag in form
        .children()
        .all(dom.parser())
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        let elemento = tag.name().as_utf8_str().to_ascii_lowercase();
        if elemento == "fieldset" && presente(tag, "disabled")? {
            return Err(ErroTela::CampoInesperado);
        }
        if elemento == "option" {
            if presente(tag, "selected")? {
                return Err(ErroTela::DestinoPreenchido);
            }
            continue;
        }
        if !matches!(
            elemento.as_str(),
            "input" | "button" | "select" | "textarea"
        ) {
            continue;
        }
        if presente(tag, "disabled")? || atributo(tag, "form")?.is_some() {
            return Err(ErroTela::CampoInesperado);
        }
        conferir_campo(&dom, tag, id, &mut vistos)?;
    }
    let esperados = [
        ("hdnInfraTipoPagina", 1),
        ("hdnIdProtocolos", 1),
        ("hdnIdProcedimentoDestino", 1),
        ("sbmSalvar", 1),
        ("selProcedimentos", 1),
        ("txtProcedimentoDestino", 1),
        ("txtIdentificacaoProcedimentoDestino", 1),
        ("txaMotivo", 1),
        ("rdoSobrestar", 2),
        ("rdoSobrestar:S", 1),
        ("rdoSobrestar:V", 1),
    ];
    if esperados
        .iter()
        .all(|(nome, n)| vistos.get(*nome) == Some(n))
    {
        Ok(())
    } else if vistos.get("rdoSobrestar:S") != Some(&1) || vistos.get("rdoSobrestar:V") != Some(&1) {
        Err(ErroTela::OpcaoNaoOferecida)
    } else {
        Err(ErroTela::CampoInesperado)
    }
}

/// Um controle enviável, conferido contra a forma observada antes do POST.
fn conferir_campo(
    dom: &tl::VDom<'_>,
    tag: &tl::HTMLTag<'_>,
    id: &str,
    vistos: &mut HashMap<String, usize>,
) -> Result<(), ErroTela> {
    let atributo =
        |nome: &'static str| valor_do_atributo(tag, nome).map_err(|_| ErroTela::NaoInterpretada);
    let presente =
        |nome: &'static str| atributo_presente(tag, nome).map_err(|_| ErroTela::NaoInterpretada);
    let elemento = tag.name().as_utf8_str().to_ascii_lowercase();
    let nome = atributo("name")?.ok_or(ErroTela::CampoInesperado)?;
    let tipo = atributo("type")?.unwrap_or_else(|| {
        if elemento == "button" {
            "submit"
        } else {
            "text"
        }
        .to_owned()
    });
    let valor = atributo("value")?.unwrap_or_default();
    *vistos.entry(nome.clone()).or_default() += 1;
    let valido = match (elemento.as_str(), tipo.as_str(), nome.as_str()) {
        ("input", "hidden", "hdnIdProtocolos") => {
            if valor != id {
                return Err(ErroTela::OutroProcesso);
            }
            true
        }
        ("input", "hidden", "hdnInfraTipoPagina") => valor == "2",
        ("input", "hidden", "hdnIdProcedimentoDestino")
        | ("input", "text", "txtProcedimentoDestino" | "txtIdentificacaoProcedimentoDestino") => {
            valor.is_empty()
        }
        ("input", "radio", "rdoSobrestar") => {
            // "S" pode vir marcado (UFRJ e GO): é a opção enviada.
            let valido = valor == "S" || (valor == "V" && !presente("checked")?);
            if valido {
                *vistos.entry(format!("rdoSobrestar:{valor}")).or_default() += 1;
            }
            valido
        }
        ("textarea", _, "txaMotivo") => {
            !presente("readonly")? && tag.inner_text(dom.parser()).is_empty()
        }
        ("input" | "button", "submit", "sbmSalvar") => {
            valor == "Salvar"
                && !presente("formaction")?
                && !presente("formmethod")?
                && !presente("formenctype")?
        }
        ("input" | "button", "button", "btnCancelar" | "btnPesquisar") => true,
        ("select", _, "selProcedimentos") => presente("multiple")?,
        _ => false,
    };
    if valido {
        Ok(())
    } else {
        match nome.as_str() {
            "hdnIdProcedimentoDestino"
            | "txtProcedimentoDestino"
            | "txtIdentificacaoProcedimentoDestino"
                if !valor.is_empty() =>
            {
                Err(ErroTela::DestinoPreenchido)
            }
            "rdoSobrestar" => Err(ErroTela::OpcaoNaoOferecida),
            _ => Err(ErroTela::CampoInesperado),
        }
    }
}

/// Aceite da resposta HTTP 302 da sobrestada: mesmo processo, ação e origem.
/// A resposta não dispensa a releitura; a URL nunca é seguida.
pub(crate) fn redirecionamento_esperado(
    localizacao: Option<&str>,
    url_do_envio: &str,
    id: &str,
) -> bool {
    let Some(consulta) =
        localizacao.and_then(|l| crate::formulario_web::consulta_do_controlador(l, url_do_envio))
    else {
        return false;
    };
    let valores = |chave: &str| -> Vec<&str> {
        consulta
            .split('&')
            .filter_map(|par| par.split_once('='))
            .filter(|(k, _)| *k == chave)
            .map(|(_, v)| v)
            .collect()
    };
    valores("acao") == ["arvore_visualizar"]
        && valores("acao_origem") == ["procedimento_sobrestar"]
        && valores("id_procedimento") == [id]
}
