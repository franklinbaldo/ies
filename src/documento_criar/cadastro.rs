//! Validação estrutural do cadastro de documento interno (#246).
//! Formulário/controles provêm exclusivamente da tela servida pelo SEI.
use super::ErroCriar;
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::{nos_com_id, valor_do_atributo};

/// Um id presente no documento não equivale a um campo ativo do cadastro:
/// templates, campos fora do form e ancestrais inativos não são oferta.
fn controle_oferecido(
    dom: &tl::VDom<'_>,
    formulario: &tl::HTMLTag<'_>,
    controle: &tl::HTMLTag<'_>,
) -> Result<bool, ErroCriar> {
    let inicio = controle.boundaries(dom.parser()).0;
    let (inicio_formulario, fim_formulario) = formulario.boundaries(dom.parser());
    let inertes =
        super::guarda_script::faixas_inertes(dom).ok_or(ErroCriar::CadastroForaDaForma)?;
    let inativos = super::guarda_script::faixas_de_controles_inativos(dom)
        .ok_or(ErroCriar::CadastroForaDaForma)?;
    Ok(inicio > inicio_formulario
        && inicio <= fim_formulario
        && !super::guarda_script::dentro_de_template(inicio, &inertes)
        && !inativos
            .iter()
            .any(|&(primeiro, ultimo)| (primeiro..=ultimo).contains(&inicio)))
}

/// Verifica a identidade do cadastro antes de qualquer POST de criação.
pub(crate) fn conferir_cadastro(
    html: &str,
    tipo: &str,
    descricao: Option<&str>,
    nivel: Option<&str>,
) -> Result<(), ErroCriar> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroCriar::CadastroForaDaForma)?;
    let formularios =
        nos_com_id(&dom, "frmDocumentoCadastro").map_err(|_| ErroCriar::CadastroForaDaForma)?;
    let [formulario] = formularios.as_slice() else {
        return Err(ErroCriar::CadastroForaDaForma);
    };
    let formulario = formulario
        .get(dom.parser())
        .and_then(tl::Node::as_tag)
        .ok_or(ErroCriar::CadastroForaDaForma)?;
    if !formulario.name().as_utf8_str().eq_ignore_ascii_case("form")
        || valor_do_atributo(formulario, "method")
            .map_err(|_| ErroCriar::CadastroForaDaForma)?
            .as_deref()
            != Some("post")
    {
        return Err(ErroCriar::CadastroForaDaForma);
    }
    let rotulos = nos_com_id(&dom, "lblSerieTitulo").map_err(|_| ErroCriar::CadastroForaDaForma)?;
    let [rotulo] = rotulos.as_slice() else {
        return Err(ErroCriar::CadastroForaDaForma);
    };
    let rotulo = rotulo
        .get(dom.parser())
        .and_then(tl::Node::as_tag)
        .ok_or(ErroCriar::CadastroForaDaForma)?;
    if !rotulo.name().as_utf8_str().eq_ignore_ascii_case("label") {
        return Err(ErroCriar::CadastroForaDaForma);
    }
    let lido = rotulo
        .inner_text(dom.parser())
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let lido = decodificar_atributo(&lido).ok_or(ErroCriar::CadastroForaDaForma)?;
    if lido != tipo {
        return Err(ErroCriar::TipoDoCadastroDiferente);
    }
    if descricao.is_some_and(|valor| !valor.is_empty()) {
        let controles =
            nos_com_id(&dom, "txtDescricao").map_err(|_| ErroCriar::CadastroForaDaForma)?;
        let [controle] = controles.as_slice() else {
            return Err(ErroCriar::DescricaoNaoOferecida);
        };
        let controle = controle
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .ok_or(ErroCriar::DescricaoNaoOferecida)?;
        if !controle.name().as_utf8_str().eq_ignore_ascii_case("input")
            || !controle_oferecido(&dom, formulario, controle)?
            || valor_do_atributo(controle, "name")
                .map_err(|_| ErroCriar::CadastroForaDaForma)?
                .as_deref()
                != Some("txtDescricao")
            || crate::html_atributos::atributo_presente(controle, "disabled")
                .map_err(|_| ErroCriar::CadastroForaDaForma)?
        {
            return Err(ErroCriar::DescricaoNaoOferecida);
        }
    }
    if let Some(nivel) = nivel {
        let (id, valor) = match nivel {
            "publico" => ("optPublico", "0"),
            "restrito" => ("optRestrito", "1"),
            "sigiloso" => ("optSigiloso", "2"),
            _ => return Err(ErroCriar::NivelNaoOferecido),
        };
        let controles = nos_com_id(&dom, id).map_err(|_| ErroCriar::CadastroForaDaForma)?;
        let [controle] = controles.as_slice() else {
            return Err(ErroCriar::NivelNaoOferecido);
        };
        let controle = controle
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .ok_or(ErroCriar::NivelNaoOferecido)?;
        if !controle.name().as_utf8_str().eq_ignore_ascii_case("input")
            || !controle_oferecido(&dom, formulario, controle)?
            || valor_do_atributo(controle, "name")
                .map_err(|_| ErroCriar::CadastroForaDaForma)?
                .as_deref()
                != Some("rdoNivelAcesso")
            || valor_do_atributo(controle, "type")
                .map_err(|_| ErroCriar::CadastroForaDaForma)?
                .as_deref()
                != Some("radio")
            || valor_do_atributo(controle, "value")
                .map_err(|_| ErroCriar::CadastroForaDaForma)?
                .as_deref()
                != Some(valor)
            || crate::html_atributos::atributo_presente(controle, "disabled")
                .map_err(|_| ErroCriar::CadastroForaDaForma)?
        {
            return Err(ErroCriar::NivelNaoOferecido);
        }
    }
    if nivel.is_some() {
        conferir_controles_obrigatorios(&dom, formulario, descricao, nivel)?;
    }
    Ok(())
}

/// Recusa controles obrigatórios que não possuem valor da operação.
/// `required` e `disabled` são booleanos: sua presença é verificada
/// por `html_atributos`, inclusive sem sinal de igualdade.
fn conferir_controles_obrigatorios(
    dom: &tl::VDom<'_>,
    formulario: &tl::HTMLTag<'_>,
    descricao: Option<&str>,
    nivel: Option<&str>,
) -> Result<(), ErroCriar> {
    for tag in formulario
        .children()
        .all(dom.parser())
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        let nome_tag = tag.name().as_utf8_str();
        if !nome_tag.eq_ignore_ascii_case("input")
            && !nome_tag.eq_ignore_ascii_case("select")
            && !nome_tag.eq_ignore_ascii_case("textarea")
        {
            continue;
        }
        if !crate::html_atributos::atributo_presente(tag, "required")
            .map_err(|_| ErroCriar::CadastroForaDaForma)?
            || crate::html_atributos::atributo_presente(tag, "disabled")
                .map_err(|_| ErroCriar::CadastroForaDaForma)?
            || !controle_oferecido(dom, formulario, tag)?
        {
            continue;
        }
        let tipo_controle =
            valor_do_atributo(tag, "type").map_err(|_| ErroCriar::CadastroForaDaForma)?;
        if matches!(
            tipo_controle.as_deref(),
            Some("hidden" | "submit" | "button" | "reset" | "image")
        ) {
            continue;
        }
        let nome = valor_do_atributo(tag, "name").map_err(|_| ErroCriar::CadastroForaDaForma)?;
        let atendido = match nome.as_deref() {
            Some("txtDescricao") => descricao.is_some_and(|s| !s.trim().is_empty()),
            Some("rdoNivelAcesso") => nivel.is_some(),
            _ => false,
        };
        if !atendido {
            return Err(ErroCriar::CampoObrigatorioNaoAtendido);
        }
    }
    Ok(())
}
