//! Validação offline do cadastro de processo servido pelo SEI.
//! Sem envio HTTP, JS, URLs montadas ou deduções a partir do nome do tipo.

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroCadastro {
    TipoDeCadastroDivergente,
    NivelNaoOferecido,
    NivelDesabilitado,
    AssuntoDesabilitado,
    HipoteseLegalDesabilitada,
    HipoteseLegalNaoIdentificada,
    EspecificacaoAcimaDoLimite {
        limite: usize,
        tamanho: usize,
    },
    AssuntoSugeridoNaoUnico,
    CadastroDeProcessoAusenteOuRepetido,
    EspecificacaoNaoPublicadaNoCadastro,
    LimiteDaEspecificacaoInvalidoNoCadastro(#[source] std::num::ParseIntError),
    /// O campo da especificação existe, mas o formulário não publica `maxlength`.
    LimiteDaEspecificacaoNaoPublicado,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CadastroObservado {
    pub(crate) id_tipo: String,
    pub(crate) assunto: String,
    pub(crate) id_assunto: String,
    pub(crate) nivel: String,
    pub(crate) hipotese_legal: Option<String>,
    pub(crate) limite_especificacao: usize,
}

impl CadastroObservado {
    /// O maxlength de input é contado em unidades UTF-16, como no navegador.
    pub(crate) fn validar_especificacao(&self, especificacao: &str) -> Result<(), ErroCadastro> {
        let limite = self.limite_especificacao;
        let tamanho = especificacao.encode_utf16().count();
        if tamanho > limite {
            return Err(ErroCadastro::EspecificacaoAcimaDoLimite { limite, tamanho });
        }
        Ok(())
    }
}

/// Valida que o cadastro efetivamente ofereceu tipo, descrição, um assunto
/// sugerido e o nível pedido. Não autoriza uma submissão: a ação, todos
/// os hidden e os interessados ainda precisam ser conferidos pelo fluxo.
pub(crate) fn conferir_cadastro(
    html: &crate::tela_html::HtmlServido,
    id_tipo: &str,
    nivel: &str,
) -> Result<CadastroObservado, ErroCadastro> {
    conferir_cadastro_interno(html, id_tipo, nivel, false)
}

/// A tela pode publicar a lista de hipóteses ainda vazia: ela é carregada
/// por AJAX observado, que o fluxo consulta antes de derivar a escrita.
/// Nada aqui autoriza escolher um valor ausente da resposta real.
pub(crate) fn conferir_cadastro_com_ajax_pendente(
    html: &crate::tela_html::HtmlServido,
    id_tipo: &str,
    nivel: &str,
) -> Result<CadastroObservado, ErroCadastro> {
    conferir_cadastro_interno(html, id_tipo, nivel, true)
}

fn conferir_cadastro_interno(
    html: &crate::tela_html::HtmlServido,
    id_tipo: &str,
    nivel: &str,
    permite_ajax_pendente: bool,
) -> Result<CadastroObservado, ErroCadastro> {
    let dom = crate::tela_html::arvore(html);
    let formularios = dom.com_id("frmProcedimentoCadastro").collect::<Vec<_>>();
    let [formulario] = formularios.as_slice() else {
        return Err(ErroCadastro::CadastroDeProcessoAusenteOuRepetido);
    };
    let form = *formulario;
    if form.nome() != "form" {
        return Err(ErroCadastro::CadastroDeProcessoAusenteOuRepetido);
    }

    let mut ids = Vec::new();
    let mut descricoes = 0;
    let mut limite_especificacao = None;
    let mut radios = Vec::new();
    for handle in form.elementos("input") {
        let nome = handle.atributo("name");
        match nome {
            Some("hdnIdTipoProcedimento") => {
                ids.push(handle.atributo("value").map(str::to_owned));
            }
            Some("txtDescricao") => {
                if handle.presente("disabled") {
                    return Err(ErroCadastro::EspecificacaoNaoPublicadaNoCadastro);
                }
                // Usar apenas o limite publicado neste cadastro, nunca
                // um valor fixo extrapolado de outras instalações.
                limite_especificacao = handle
                    .atributo("maxlength")
                    .map(str::to_owned)
                    .map(|literal| {
                        literal
                            .parse::<usize>()
                            .map_err(ErroCadastro::LimiteDaEspecificacaoInvalidoNoCadastro)
                    })
                    .transpose()?;
                descricoes += 1;
            }
            Some("rdoNivelAcesso") => {
                let valor = handle.atributo("value").map(str::to_owned);
                let desabilitado = handle.presente("disabled");
                radios.push((valor, desabilitado));
            }
            _ => {}
        }
    }
    if ids.as_slice() != [Some(id_tipo.to_owned())] {
        return Err(ErroCadastro::TipoDeCadastroDivergente);
    }
    if descricoes != 1 {
        return Err(ErroCadastro::EspecificacaoNaoPublicadaNoCadastro);
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
        conferir_listas(form, nivel, permite_ajax_pendente)?;
    let limite_especificacao =
        limite_especificacao.ok_or(ErroCadastro::LimiteDaEspecificacaoNaoPublicado)?;
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
    form: crate::tela_html::Elemento<'_>,
    nivel: &str,
    permite_ajax_pendente: bool,
) -> Result<(String, String, Option<String>), ErroCadastro> {
    let mut assuntos = Vec::new();
    let mut hipotese = Vec::new();
    let mut selects_assunto = 0;
    let mut selects_hipotese = 0;
    for handle in form.elementos("select") {
        let nome = handle.atributo("name");
        let destino = match nome {
            Some("selAssuntos") => {
                selects_assunto += 1;
                if handle.presente("disabled") {
                    return Err(ErroCadastro::AssuntoDesabilitado);
                }
                &mut assuntos
            }
            Some("selHipoteseLegal") => {
                selects_hipotese += 1;
                // No nível público, um seletor restrito desabilitado
                // não impede a prévia nem exige hipótese legal.
                if nivel == "restrito" && handle.presente("disabled") {
                    return Err(ErroCadastro::HipoteseLegalDesabilitada);
                }
                &mut hipotese
            }
            _ => continue,
        };
        for opcao in handle.elementos("option") {
            if opcao.presente("disabled") {
                continue;
            }
            let valor = opcao.atributo("value").map(str::to_owned);
            let titulo = opcao.texto();
            if let Some(valor) = valor.filter(|v| !v.is_empty() && v != "null") {
                destino.push((valor, titulo));
            }
        }
    }
    let [(id_assunto, assunto)] = assuntos.as_slice() else {
        return Err(ErroCadastro::AssuntoSugeridoNaoUnico);
    };
    if selects_assunto != 1 || assunto.is_empty() {
        return Err(ErroCadastro::AssuntoSugeridoNaoUnico);
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
            "<form id='frmProcedimentoCadastro'><input type='hidden' name='hdnIdTipoProcedimento' value='12'><input name='txtDescricao' maxlength='250'><input type='radio' name='rdoNivelAcesso' value='0' {publico}><input type='radio' name='rdoNivelAcesso' value='1'><select name='selAssuntos'>{assuntos}</select><select name='selHipoteseLegal'>{hipotese}</select></form>"
        )
    }
    const ASSUNTO: &str = "<option value='21'>Assunto de teste</option>";
    const HIPOTESE: &str = "<option value='31'>Base legal de teste</option>";

    #[test]
    fn valida_cadastro_publico_e_restrito_sem_escrever() {
        let html = crate::tela_html::HtmlServido::de_teste(&tela(ASSUNTO, "", HIPOTESE));
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
            conferir_cadastro(
                &crate::tela_html::HtmlServido::de_teste("<div></div>"),
                "12",
                "publico"
            ),
            Err(ErroCadastro::CadastroDeProcessoAusenteOuRepetido)
        );
        assert_eq!(
            conferir_cadastro(
                &crate::tela_html::HtmlServido::de_teste(&tela(ASSUNTO, "", HIPOTESE)),
                "13",
                "publico"
            ),
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
                conferir_cadastro(
                    &crate::tela_html::HtmlServido::de_teste(&tela(opcoes, "", HIPOTESE)),
                    "12",
                    "publico"
                ),
                Err(ErroCadastro::AssuntoSugeridoNaoUnico)
            );
        }
    }

    #[test]
    fn recusa_nivel_desabilitado() {
        assert_eq!(
            conferir_cadastro(
                &crate::tela_html::HtmlServido::de_teste(&tela(ASSUNTO, "disabled", HIPOTESE)),
                "12",
                "publico"
            ),
            Err(ErroCadastro::NivelDesabilitado)
        );
    }

    #[test]
    fn placeholder_null_precisa_de_ajax_antes_de_uma_escrita() {
        let html = crate::tela_html::HtmlServido::de_teste(&tela(
            ASSUNTO,
            "",
            "<option value='null'> </option>",
        ));
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
        let assunto_desabilitado = crate::tela_html::HtmlServido::de_teste(
            &tela(ASSUNTO, "", HIPOTESE)
                .replace("name='selAssuntos'", "name='selAssuntos' disabled"),
        );
        assert_eq!(
            conferir_cadastro(&assunto_desabilitado, "12", "publico"),
            Err(ErroCadastro::AssuntoDesabilitado)
        );
        let hipotese_desabilitada =
            crate::tela_html::HtmlServido::de_teste(&tela(ASSUNTO, "", HIPOTESE).replace(
                "name='selHipoteseLegal'",
                "name='selHipoteseLegal' disabled",
            ));
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
                conferir_cadastro(
                    &crate::tela_html::HtmlServido::de_teste(&tela(ASSUNTO, "", opcoes)),
                    "12",
                    "restrito"
                ),
                Err(ErroCadastro::HipoteseLegalNaoIdentificada)
            );
        }
    }
}
