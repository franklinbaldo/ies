//! Conferência estrutural do formulário de sobrestamento observado na `UnB` (#269).
//! Fonte: `Todos tests/test_ciclo_sobrestar_010.py` @323eb01.
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::{
    ConsultaDoControlador, ErroChaveDaConsulta, ErroConsultaDoControlador,
};
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, ErroSubmissaoFormulario,
    EstadoDoProcesso, OperacaoDeFormulario, ResultadoDoEnvio, localizar_formulario,
};
use crate::html_atributos::{atributo_presente, valor_do_atributo};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroTela {
    /// O tokenizador de atributos recusou a tag.
    Atributo(#[source] crate::html_atributos::ErroAtributoHtml),
    /// O tokenizador HTML recusou a tela.
    FormularioAusente(#[source] ErroSubmissaoFormulario),
    OutroProcesso,
    /// A consulta do `action` não se lê em pares `chave=valor`.
    ConsultaDaActionIlegivel(#[source] ErroConsultaDoControlador),
    /// O `action` não tem exatamente um `id_procedimento`.
    ChaveDaActionIlegivel(#[source] ErroChaveDaConsulta),
    OpcaoNaoOferecida,
    DestinoPreenchido,
    CampoInesperado,
}
pub(crate) const OPERACAO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::ProcedimentoSobrestar,
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
pub(crate) fn conferir_formulario(
    html: &crate::tela_html::HtmlServido,
    id: &str,
) -> Result<(), ErroTela> {
    let dom = crate::tela_html::interpretar(html);
    let form =
        localizar_formulario(&dom, OPERACAO.formulario).map_err(ErroTela::FormularioAusente)?;
    let atributo = |tag: &tl::HTMLTag<'_>, nome: &'static str| {
        valor_do_atributo(tag, nome).map_err(ErroTela::Atributo)
    };
    let presente = |tag: &tl::HTMLTag<'_>, nome: &'static str| {
        atributo_presente(tag, nome).map_err(ErroTela::Atributo)
    };
    let action = atributo(form, "action")?.ok_or(ErroTela::OutroProcesso)?;
    let action = crate::formulario_web::decodificar_atributo(&action);
    conferir_processo_da_action(&action, id)?;
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
    let atributo = |nome: &'static str| valor_do_atributo(tag, nome).map_err(ErroTela::Atributo);
    let presente = |nome: &'static str| atributo_presente(tag, nome).map_err(ErroTela::Atributo);
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

/// O `action` aponta para o processo `id`, com `id_procedimento` uma vez só.
fn conferir_processo_da_action(action: &str, id: &str) -> Result<(), ErroTela> {
    let Some((_, consulta)) = action.split_once('?') else {
        return Err(ErroTela::OutroProcesso);
    };
    let consulta =
        ConsultaDoControlador::ler(consulta).map_err(ErroTela::ConsultaDaActionIlegivel)?;
    let processo = consulta
        .unico("id_procedimento")
        .map_err(ErroTela::ChaveDaActionIlegivel)?;
    if processo != id {
        return Err(ErroTela::OutroProcesso);
    }
    Ok(())
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
    let Ok(consulta) = ConsultaDoControlador::ler(consulta) else {
        return false;
    };
    consulta.acao() == Ok(AcaoDoSei::ArvoreVisualizar)
        && consulta.acao_de_origem() == Ok(AcaoDoSei::ProcedimentoSobrestar)
        && consulta.unico("id_procedimento") == Ok(id)
}
