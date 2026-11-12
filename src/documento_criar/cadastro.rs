//! Validação estrutural do cadastro de documento interno (#246).
//! Formulário/controles provêm exclusivamente da tela servida pelo SEI.
use super::ErroCriar;

/// Um id presente no documento não equivale a um campo do cadastro: o
/// controle precisa estar dentro do form.
fn controle_no_formulario(
    formulario: crate::tela_html::Elemento<'_>,
    controle: crate::tela_html::Elemento<'_>,
) -> bool {
    formulario
        .descendentes()
        .any(|elemento| elemento == controle)
}

/// Verifica a identidade do cadastro antes de qualquer POST de criação.
pub(crate) fn conferir_cadastro(
    html: &crate::tela_html::HtmlServido,
    tipo: &str,
    descricao: Option<&str>,
    nivel: Option<&str>,
) -> Result<(), ErroCriar> {
    let dom = crate::tela_html::arvore(html);
    let formularios = dom.com_id("frmDocumentoCadastro").collect::<Vec<_>>();
    let [formulario] = formularios.as_slice() else {
        return Err(ErroCriar::CadastroForaDaForma);
    };
    let formulario = *formulario;
    if !formulario.nome().eq_ignore_ascii_case("form")
        || formulario.atributo("method") != Some("post")
    {
        return Err(ErroCriar::CadastroForaDaForma);
    }
    let rotulos = dom.com_id("lblSerieTitulo").collect::<Vec<_>>();
    let [rotulo] = rotulos.as_slice() else {
        return Err(ErroCriar::CadastroForaDaForma);
    };
    let rotulo = *rotulo;
    if !rotulo.nome().eq_ignore_ascii_case("label") {
        return Err(ErroCriar::CadastroForaDaForma);
    }
    let lido = super::nome_visivel(&rotulo.texto());
    if lido != tipo {
        return Err(ErroCriar::TipoDoCadastroDiferente);
    }
    if descricao.is_some_and(|valor| !valor.is_empty()) {
        let controles = dom.com_id("txtDescricao").collect::<Vec<_>>();
        let [controle] = controles.as_slice() else {
            return Err(ErroCriar::DescricaoNaoOferecida);
        };
        let controle = *controle;
        if !controle.nome().eq_ignore_ascii_case("input")
            || !controle_no_formulario(formulario, controle)
            || controle.atributo("name") != Some("txtDescricao")
            || controle.presente("disabled")
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
        let controles = dom.com_id(id).collect::<Vec<_>>();
        let [controle] = controles.as_slice() else {
            return Err(ErroCriar::NivelNaoOferecido);
        };
        let controle = *controle;
        if !controle.nome().eq_ignore_ascii_case("input")
            || !controle_no_formulario(formulario, controle)
            || controle.atributo("name") != Some("rdoNivelAcesso")
            || controle.atributo("type") != Some("radio")
            || controle.atributo("value") != Some(valor)
            || controle.presente("disabled")
        {
            return Err(ErroCriar::NivelNaoOferecido);
        }
    }
    if nivel.is_some() {
        conferir_controles_obrigatorios(formulario, descricao, nivel)?;
    }
    Ok(())
}

/// Recusa controles obrigatórios que não possuem valor da operação.
/// `required` e `disabled` são booleanos: vale a presença, inclusive sem
/// sinal de igualdade.
fn conferir_controles_obrigatorios(
    formulario: crate::tela_html::Elemento<'_>,
    descricao: Option<&str>,
    nivel: Option<&str>,
) -> Result<(), ErroCriar> {
    for tag in formulario.descendentes() {
        let nome_tag = tag.nome();
        if !nome_tag.eq_ignore_ascii_case("input")
            && !nome_tag.eq_ignore_ascii_case("select")
            && !nome_tag.eq_ignore_ascii_case("textarea")
        {
            continue;
        }
        if !tag.presente("required") || tag.presente("disabled") {
            continue;
        }
        let tipo_controle = tag.atributo("type").map(str::to_owned);
        if matches!(
            tipo_controle.as_deref(),
            Some("hidden" | "submit" | "button" | "reset" | "image")
        ) {
            continue;
        }
        let nome = tag.atributo("name");
        let atendido = match nome {
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
