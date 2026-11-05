//! Validação offline do cadastro de processo servido pelo SEI.
//! Sem envio HTTP, JS, URLs montadas ou deduções a partir do nome do tipo.
use crate::html_atributos::{atributo_presente, nos_com_id, valor_do_atributo};

#[derive(Debug, Clone, Copy, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroCadastro {
    #[causa(nome = "tipo_do_cadastro_divergente")]
    TipoDeCadastroDivergente,
    #[causa(nome = "nivel_de_acesso_nao_oferecido")]
    NivelNaoOferecido,
    #[causa(nome = "nivel_de_acesso_desabilitado")]
    NivelDesabilitado,
    HipoteseLegalNaoIdentificada,
    #[causa(interna)]
    Html(crate::erro_html::ErroHtml),
}

impl ErroCadastro {
    pub(crate) fn subcodigo(self) -> &'static str {
        crate::erro_publico::folha(&self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CadastroObservado {
    pub(crate) id_tipo: String,
    pub(crate) assunto: String,
    pub(crate) id_assunto: String,
    pub(crate) nivel: String,
    pub(crate) hipotese_legal: Option<String>,
}

/// Valida que o cadastro efetivamente ofereceu tipo, descrição, um assunto
/// sugerido e o nível pedido. Não autoriza uma submissão: a ação, todos
/// os hidden e os interessados ainda precisam ser conferidos pelo fluxo.
#[allow(clippy::too_many_lines)] // Cada ErroHtml por extenso (ADR 0005); dividir a seguir.
pub(crate) fn conferir_cadastro(
    html: &str,
    id_tipo: &str,
    nivel: &str,
) -> Result<CadastroObservado, ErroCadastro> {
    use ErroCadastro as E;
    let especificacao_nao_publicada_no_cadastro = E::Html(crate::erro_html::ErroHtml::ausente(
        "especificacao_nao_publicada_no_cadastro",
        "a tela não tem o elemento observado",
    ));
    let cadastro_de_processo_html_invalido = E::Html(crate::erro_html::ErroHtml::html(
        "cadastro_de_processo_html_invalido",
        "o HTML não pôde ser interpretado",
    ));
    let cadastro_de_processo_ausente_ou_repetido = E::Html(crate::erro_html::ErroHtml::ausente(
        "cadastro_de_processo_ausente_ou_repetido",
        "a tela não tem o elemento observado",
    ));
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| cadastro_de_processo_html_invalido)?;
    let formularios = nos_com_id(&dom, "frmProcedimentoCadastro").map_err(|e| {
        E::Html(crate::erro_html::ErroHtml::atributo(
            "cadastro_de_processo_html_invalido",
            "o HTML não pôde ser interpretado",
            e,
        ))
    })?;
    let [formulario] = formularios.as_slice() else {
        return Err(cadastro_de_processo_ausente_ou_repetido);
    };
    let parser = dom.parser();
    let form = formulario
        .get(parser)
        .and_then(tl::Node::as_tag)
        .ok_or(cadastro_de_processo_html_invalido)?;
    if form.name().as_utf8_str() != "form" {
        return Err(cadastro_de_processo_ausente_ou_repetido);
    }

    let mut ids = Vec::new();
    let mut descricoes = 0;
    let mut radios = Vec::new();
    for handle in form.query_selector(parser, "input").into_iter().flatten() {
        let tag = handle
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(cadastro_de_processo_html_invalido)?;
        let nome = valor_do_atributo(tag, "name").map_err(|e| {
            E::Html(crate::erro_html::ErroHtml::atributo(
                "cadastro_de_processo_html_invalido",
                "o HTML não pôde ser interpretado",
                e,
            ))
        })?;
        match nome.as_deref() {
            Some("hdnIdTipoProcedimento") => {
                ids.push(valor_do_atributo(tag, "value").map_err(|e| {
                    E::Html(crate::erro_html::ErroHtml::atributo(
                        "cadastro_de_processo_html_invalido",
                        "o HTML não pôde ser interpretado",
                        e,
                    ))
                })?);
            }
            Some("txtDescricao") => {
                if atributo_presente(tag, "disabled").map_err(|e| {
                    E::Html(crate::erro_html::ErroHtml::atributo(
                        "cadastro_de_processo_html_invalido",
                        "o HTML não pôde ser interpretado",
                        e,
                    ))
                })? {
                    return Err(especificacao_nao_publicada_no_cadastro);
                }
                descricoes += 1;
            }
            Some("rdoNivelAcesso") => {
                let valor = valor_do_atributo(tag, "value").map_err(|e| {
                    E::Html(crate::erro_html::ErroHtml::atributo(
                        "cadastro_de_processo_html_invalido",
                        "o HTML não pôde ser interpretado",
                        e,
                    ))
                })?;
                let desabilitado = atributo_presente(tag, "disabled").map_err(|e| {
                    E::Html(crate::erro_html::ErroHtml::atributo(
                        "cadastro_de_processo_html_invalido",
                        "o HTML não pôde ser interpretado",
                        e,
                    ))
                })?;
                radios.push((valor, desabilitado));
            }
            _ => {}
        }
    }
    if ids.as_slice() != [Some(id_tipo.to_owned())] {
        return Err(E::TipoDeCadastroDivergente);
    }
    if descricoes != 1 {
        return Err(especificacao_nao_publicada_no_cadastro);
    }
    let valor_nivel = match nivel {
        "publico" => "0",
        "restrito" => "1",
        _ => return Err(E::NivelNaoOferecido),
    };
    let correspondentes: Vec<_> = radios
        .iter()
        .filter(|(valor, _)| valor.as_deref() == Some(valor_nivel))
        .collect();
    match correspondentes.as_slice() {
        [(_, false)] => {}
        [(_, true)] => return Err(E::NivelDesabilitado),
        _ => return Err(E::NivelNaoOferecido),
    }

    let (id_assunto, assunto, hipotese_legal) = conferir_listas(form, parser, nivel)?;
    Ok(CadastroObservado {
        id_tipo: id_tipo.to_owned(),
        assunto,
        id_assunto,
        nivel: nivel.to_owned(),
        hipotese_legal,
    })
}

/// Lê apenas listas publicadas dentro do mesmo formulário, sem inferir
/// itens ausentes. Dois assuntos não equivalem a um sugerido único.
fn conferir_listas(
    form: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    nivel: &str,
) -> Result<(String, String, Option<String>), ErroCadastro> {
    use ErroCadastro as E;
    let mut assuntos = Vec::new();
    let mut hipotese = Vec::new();
    let mut selects_assunto = 0;
    let mut selects_hipotese = 0;
    for handle in form.query_selector(parser, "select").into_iter().flatten() {
        let select = handle
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(E::Html(crate::erro_html::ErroHtml::html(
                "cadastro_de_processo_html_invalido",
                "o HTML não pôde ser interpretado",
            )))?;
        let nome = valor_do_atributo(select, "name").map_err(|e| {
            E::Html(crate::erro_html::ErroHtml::atributo(
                "cadastro_de_processo_html_invalido",
                "o HTML não pôde ser interpretado",
                e,
            ))
        })?;
        let destino = match nome.as_deref() {
            Some("selAssuntos") => {
                selects_assunto += 1;
                &mut assuntos
            }
            Some("selHipoteseLegal") => {
                selects_hipotese += 1;
                &mut hipotese
            }
            _ => continue,
        };
        for opcao in select
            .query_selector(parser, "option")
            .into_iter()
            .flatten()
        {
            let tag = opcao.get(parser).and_then(tl::Node::as_tag).ok_or(E::Html(
                crate::erro_html::ErroHtml::html(
                    "cadastro_de_processo_html_invalido",
                    "o HTML não pôde ser interpretado",
                ),
            ))?;
            if atributo_presente(tag, "disabled").map_err(|e| {
                E::Html(crate::erro_html::ErroHtml::atributo(
                    "cadastro_de_processo_html_invalido",
                    "o HTML não pôde ser interpretado",
                    e,
                ))
            })? {
                continue;
            }
            let valor = valor_do_atributo(tag, "value").map_err(|e| {
                E::Html(crate::erro_html::ErroHtml::atributo(
                    "cadastro_de_processo_html_invalido",
                    "o HTML não pôde ser interpretado",
                    e,
                ))
            })?;
            let titulo =
                crate::transporte::texto_de_recusa::decodificar_entidades(&tag.inner_text(parser));
            if let Some(valor) = valor.filter(|v| !v.is_empty()) {
                destino.push((valor, titulo.trim().to_owned()));
            }
        }
    }
    let [(id_assunto, assunto)] = assuntos.as_slice() else {
        return Err(E::Html(crate::erro_html::ErroHtml::valor(
            "assunto_sugerido_nao_unico",
            "o valor não está na forma observada",
        )));
    };
    if selects_assunto != 1 || assunto.is_empty() {
        return Err(E::Html(crate::erro_html::ErroHtml::valor(
            "assunto_sugerido_nao_unico",
            "o valor não está na forma observada",
        )));
    }
    let hipotese_legal = if nivel == "restrito" {
        let [(id, titulo)] = hipotese.as_slice() else {
            return Err(E::HipoteseLegalNaoIdentificada);
        };
        if selects_hipotese != 1 || titulo.is_empty() {
            return Err(E::HipoteseLegalNaoIdentificada);
        }
        Some(id.clone())
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
            Err(ErroCadastro::Html(crate::erro_html::ErroHtml::ausente(
                "cadastro_de_processo_ausente_ou_repetido",
                "a tela não tem o elemento observado"
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
                Err(ErroCadastro::Html(crate::erro_html::ErroHtml::valor(
                    "assunto_sugerido_nao_unico",
                    "o valor não está na forma observada"
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
