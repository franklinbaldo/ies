//! Validação offline do cadastro de processo servido pelo SEI.
//! Sem envio HTTP, JS, URLs montadas ou deduções a partir do nome do tipo.
use crate::erro_html::ErroHtml;
use crate::html_atributos::{atributo_presente, nos_com_id, valor_do_atributo};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroCadastro {
    TipoDeCadastroDivergente,
    NivelNaoOferecido,
    NivelDesabilitado,
    AssuntoDesabilitado,
    HipoteseLegalDesabilitada,
    HipoteseLegalNaoIdentificada,
    EspecificacaoAcimaDoLimite { limite: usize, tamanho: usize },
    Html(#[from] ErroHtml),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CadastroObservado {
    pub(crate) id_tipo: String,
    pub(crate) assunto: String,
    pub(crate) id_assunto: String,
    pub(crate) nivel: String,
    pub(crate) hipotese_legal: Option<String>,
    pub(crate) limite_especificacao: Option<usize>,
}

impl CadastroObservado {
    /// O maxlength de input é contado em unidades UTF-16, como no navegador.
    /// Sem atributo no formulário servido, não inferir limite universal.
    pub(crate) fn validar_especificacao(&self, especificacao: &str) -> Result<(), ErroCadastro> {
        if let Some(limite) = self.limite_especificacao {
            let tamanho = especificacao.encode_utf16().count();
            if tamanho > limite {
                return Err(ErroCadastro::EspecificacaoAcimaDoLimite { limite, tamanho });
            }
        }
        Ok(())
    }
}

/// Valida que o cadastro efetivamente ofereceu tipo, descrição, um assunto
/// sugerido e o nível pedido. Não autoriza uma submissão: a ação, todos
/// os hidden e os interessados ainda precisam ser conferidos pelo fluxo.
pub(crate) fn conferir_cadastro(
    html: &str,
    id_tipo: &str,
    nivel: &str,
) -> Result<CadastroObservado, ErroCadastro> {
    conferir_cadastro_interno(html, id_tipo, nivel, false)
}

/// A tela pode publicar a lista de hipóteses ainda vazia: ela é carregada
/// por AJAX observado, que o fluxo consulta antes de derivar a escrita.
/// Nada aqui autoriza escolher um valor ausente da resposta real.
pub(crate) fn conferir_cadastro_com_ajax_pendente(
    html: &str,
    id_tipo: &str,
    nivel: &str,
) -> Result<CadastroObservado, ErroCadastro> {
    conferir_cadastro_interno(html, id_tipo, nivel, true)
}

fn conferir_cadastro_interno(
    html: &str,
    id_tipo: &str,
    nivel: &str,
    permite_ajax_pendente: bool,
) -> Result<CadastroObservado, ErroCadastro> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroHtml::html("cadastro_de_processo_html_invalido"))?;
    let formularios = nos_com_id(&dom, "frmProcedimentoCadastro")
        .map_err(|e| ErroHtml::atributo("cadastro_de_processo_html_invalido", e))?;
    let [formulario] = formularios.as_slice() else {
        return Err(ErroHtml::ausente("cadastro_de_processo_ausente_ou_repetido").into());
    };
    let parser = dom.parser();
    let form = formulario
        .get(parser)
        .and_then(tl::Node::as_tag)
        .ok_or(ErroHtml::html("cadastro_de_processo_html_invalido"))?;
    if form.name().as_utf8_str() != "form" {
        return Err(ErroHtml::ausente("cadastro_de_processo_ausente_ou_repetido").into());
    }

    let mut ids = Vec::new();
    let mut descricoes = 0;
    let mut limite_especificacao = None;
    let mut radios = Vec::new();
    for handle in form.query_selector(parser, "input").into_iter().flatten() {
        let tag = handle
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroHtml::html("cadastro_de_processo_html_invalido"))?;
        let nome = valor_do_atributo(tag, "name")
            .map_err(|e| ErroHtml::atributo("cadastro_de_processo_html_invalido", e))?;
        match nome.as_deref() {
            Some("hdnIdTipoProcedimento") => {
                ids.push(
                    valor_do_atributo(tag, "value")
                        .map_err(|e| ErroHtml::atributo("cadastro_de_processo_html_invalido", e))?,
                );
            }
            Some("txtDescricao") => {
                if atributo_presente(tag, "disabled")
                    .map_err(|e| ErroHtml::atributo("cadastro_de_processo_html_invalido", e))?
                {
                    return Err(ErroHtml::ausente("especificacao_nao_publicada_no_cadastro").into());
                }
                // Usar apenas o limite publicado neste cadastro, nunca
                // um valor fixo extrapolado de outras instalações.
                limite_especificacao = valor_do_atributo(tag, "maxlength")
                    .map_err(|e| ErroHtml::atributo("cadastro_de_processo_html_invalido", e))?
                    .map(|literal| {
                        literal.parse::<usize>().map_err(|_| {
                            ErroHtml::valor("limite_da_especificacao_invalido_no_cadastro")
                        })
                    })
                    .transpose()?;
                descricoes += 1;
            }
            Some("rdoNivelAcesso") => {
                let valor = valor_do_atributo(tag, "value")
                    .map_err(|e| ErroHtml::atributo("cadastro_de_processo_html_invalido", e))?;
                let desabilitado = atributo_presente(tag, "disabled")
                    .map_err(|e| ErroHtml::atributo("cadastro_de_processo_html_invalido", e))?;
                radios.push((valor, desabilitado));
            }
            _ => {}
        }
    }
    if ids.as_slice() != [Some(id_tipo.to_owned())] {
        return Err(ErroCadastro::TipoDeCadastroDivergente);
    }
    if descricoes != 1 {
        return Err(ErroHtml::ausente("especificacao_nao_publicada_no_cadastro").into());
    }
    let valor_nivel = match nivel {
        "publico" => "0",
        "restrito" => "1",
        _ => return Err(ErroCadastro::NivelNaoOferecido),
    };
    let correspondentes: Vec<_> = radios
        .iter()
        .filter(|(valor, _)| valor.as_deref() == Some(valor_nivel))
        .collect();
    match correspondentes.as_slice() {
        [(_, false)] => {}
        [(_, true)] => return Err(ErroCadastro::NivelDesabilitado),
        _ => return Err(ErroCadastro::NivelNaoOferecido),
    }

    let (id_assunto, assunto, hipotese_legal) =
        conferir_listas(form, parser, nivel, permite_ajax_pendente)?;
    Ok(CadastroObservado {
        id_tipo: id_tipo.to_owned(),
        assunto,
        id_assunto,
        nivel: nivel.to_owned(),
        hipotese_legal,
        limite_especificacao,
    })
}

/// Lê apenas listas publicadas dentro do mesmo formulário, sem inferir
/// itens ausentes. Dois assuntos não equivalem a um sugerido único.
fn conferir_listas(
    form: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    nivel: &str,
    permite_ajax_pendente: bool,
) -> Result<(String, String, Option<String>), ErroCadastro> {
    let mut assuntos = Vec::new();
    let mut hipotese = Vec::new();
    let mut selects_assunto = 0;
    let mut selects_hipotese = 0;
    for handle in form.query_selector(parser, "select").into_iter().flatten() {
        let select = handle
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroHtml::html("cadastro_de_processo_html_invalido"))?;
        let nome = valor_do_atributo(select, "name")
            .map_err(|e| ErroHtml::atributo("cadastro_de_processo_html_invalido", e))?;
        let destino = match nome.as_deref() {
            Some("selAssuntos") => {
                selects_assunto += 1;
                if atributo_presente(select, "disabled")
                    .map_err(|e| ErroHtml::atributo("cadastro_de_processo_html_invalido", e))?
                {
                    return Err(ErroCadastro::AssuntoDesabilitado);
                }
                &mut assuntos
            }
            Some("selHipoteseLegal") => {
                selects_hipotese += 1;
                // No nível público, um seletor restrito desabilitado
                // não impede a prévia nem exige hipótese legal.
                if nivel == "restrito"
                    && atributo_presente(select, "disabled")
                        .map_err(|e| ErroHtml::atributo("cadastro_de_processo_html_invalido", e))?
                {
                    return Err(ErroCadastro::HipoteseLegalDesabilitada);
                }
                &mut hipotese
            }
            _ => continue,
        };
        for opcao in select
            .query_selector(parser, "option")
            .into_iter()
            .flatten()
        {
            let tag = opcao
                .get(parser)
                .and_then(tl::Node::as_tag)
                .ok_or(ErroHtml::html("cadastro_de_processo_html_invalido"))?;
            if atributo_presente(tag, "disabled")
                .map_err(|e| ErroHtml::atributo("cadastro_de_processo_html_invalido", e))?
            {
                continue;
            }
            let valor = valor_do_atributo(tag, "value")
                .map_err(|e| ErroHtml::atributo("cadastro_de_processo_html_invalido", e))?;
            let titulo = html_escape::decode_html_entities(&tag.inner_text(parser)).into_owned();
            if let Some(valor) = valor.filter(|v| !v.is_empty() && v != "null") {
                destino.push((valor, titulo));
            }
        }
    }
    let [(id_assunto, assunto)] = assuntos.as_slice() else {
        return Err(ErroHtml::valor("assunto_sugerido_nao_unico").into());
    };
    if selects_assunto != 1 || assunto.is_empty() {
        return Err(ErroHtml::valor("assunto_sugerido_nao_unico").into());
    }
    let hipotese_legal = if nivel == "restrito" {
        if selects_hipotese != 1 {
            return Err(ErroCadastro::HipoteseLegalNaoIdentificada);
        }
        match hipotese.as_slice() {
            [(id, titulo)] if !titulo.is_empty() => Some(id.clone()),
            [] if permite_ajax_pendente => None,
            _ => return Err(ErroCadastro::HipoteseLegalNaoIdentificada),
        }
    } else {
        None
    };
    Ok((id_assunto.clone(), assunto.clone(), hipotese_legal))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tela(assuntos: &str, publico: &str, hipotese: &str) -> String {
        format!(
            "<form id='frmProcedimentoCadastro'><input type='hidden' name='hdnIdTipoProcedimento' value='12'><input name='txtDescricao'><input type='radio' name='rdoNivelAcesso' value='0' {publico}><input type='radio' name='rdoNivelAcesso' value='1'><select name='selAssuntos'>{assuntos}</select><select name='selHipoteseLegal'>{hipotese}</select></form>"
        )
    }
    const ASSUNTO: &str = "<option value='21'>Assunto de teste</option>";
    const HIPOTESE: &str = "<option value='31'>Base legal de teste</option>";

    #[test]
    fn valida_cadastro_publico_e_restrito_sem_escrever() {
        let html = tela(ASSUNTO, "", HIPOTESE);
        let publico = conferir_cadastro(&html, "12", "publico").unwrap();
        assert_eq!(publico.id_assunto, "21");
        assert_eq!(publico.assunto, "Assunto de teste");
        assert_eq!(publico.hipotese_legal, None);
        let restrito = conferir_cadastro(&html, "12", "restrito").unwrap();
        assert_eq!(restrito.hipotese_legal.as_deref(), Some("31"));
    }

    #[test]
    fn recusa_formulario_ausente_e_tipo_de_outro_cadastro() {
        assert_eq!(
            conferir_cadastro("<div></div>", "12", "publico"),
            Err(ErroCadastro::Html(ErroHtml::ausente(
                "cadastro_de_processo_ausente_ou_repetido"
            )))
        );
        assert_eq!(
            conferir_cadastro(&tela(ASSUNTO, "", HIPOTESE), "13", "publico"),
            Err(ErroCadastro::TipoDeCadastroDivergente)
        );
    }

    #[test]
    fn nao_inventa_assunto_unico() {
        for opcoes in [
            "",
            "<option value='1'>Um</option><option value='2'>Outro</option>",
        ] {
            assert_eq!(
                conferir_cadastro(&tela(opcoes, "", HIPOTESE), "12", "publico"),
                Err(ErroCadastro::Html(ErroHtml::valor(
                    "assunto_sugerido_nao_unico"
                )))
            );
        }
    }

    #[test]
    fn recusa_nivel_desabilitado() {
        assert_eq!(
            conferir_cadastro(&tela(ASSUNTO, "disabled", HIPOTESE), "12", "publico"),
            Err(ErroCadastro::NivelDesabilitado)
        );
    }

    #[test]
    fn placeholder_null_precisa_de_ajax_antes_de_uma_escrita() {
        let html = tela(ASSUNTO, "", "<option value='null'> </option>");
        assert_eq!(
            conferir_cadastro(&html, "12", "restrito"),
            Err(ErroCadastro::HipoteseLegalNaoIdentificada)
        );
        let observado = conferir_cadastro_com_ajax_pendente(&html, "12", "restrito").unwrap();
        assert_eq!(observado.hipotese_legal, None);
    }

    /// Atributo HTML `disabled` torna o select não selecionável. Não
    /// converter opção visível dentro de select desabilitado em prova de
    /// assunto/hipótese legal para a escrita #245.
    #[test]
    fn select_desabilitado_nao_comprova_assunto_ou_hipotese() {
        // Teste sintético da semântica HTML; oráculo de campos:
        // Todos tests/paginas_reais/procedimento_gerar.html @323eb01.
        let assunto_desabilitado = tela(ASSUNTO, "", HIPOTESE)
            .replace("name='selAssuntos'", "name='selAssuntos' disabled");
        assert_eq!(
            conferir_cadastro(&assunto_desabilitado, "12", "publico"),
            Err(ErroCadastro::AssuntoDesabilitado)
        );
        let hipotese_desabilitada = tela(ASSUNTO, "", HIPOTESE).replace(
            "name='selHipoteseLegal'",
            "name='selHipoteseLegal' disabled",
        );
        assert_eq!(
            conferir_cadastro(&hipotese_desabilitada, "12", "restrito"),
            Err(ErroCadastro::HipoteseLegalDesabilitada)
        );
        // Hipótese desabilitada é irrelevante quando o nível é público.
        assert!(conferir_cadastro(&hipotese_desabilitada, "12", "publico").is_ok());
    }

    #[test]
    fn restrito_sem_hipotese_determinada_eh_recusado() {
        for opcoes in [
            "",
            "<option value='1'>Uma</option><option value='2'>Outra</option>",
        ] {
            assert_eq!(
                conferir_cadastro(&tela(ASSUNTO, "", opcoes), "12", "restrito"),
                Err(ErroCadastro::HipoteseLegalNaoIdentificada)
            );
        }
    }
}
