//! Leitura da anotação de origem, sem efetuar alterações.
//! Forma observada no SEI 4.0.9 de treinamento (Todos,
//! `tests/paginas_reais/processo_anotacao_registrar.html`).
use crate::erro_html::ErroHtml;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, ResultadoDoEnvio,
};

/// Por que a anotação da origem não foi lida antes de enviar. Etapa
/// própria: o erro nasce aqui e chega ao envio por `From`.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroDasAnotacoes {
    LinkDasAnotacoes(#[source] ErroHtml),
    TelaDasAnotacoes(#[from] crate::transporte::FalhaTransporte),
    TelaDasAnotacoesRedirecionada,
    LeituraDasAnotacoes(#[source] ErroHtml),
}

/// POST do botão Salvar no formulário de anotação observado no SEI 4.0.9.
pub(crate) const REGISTRAR_ANOTACAO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "anotacao_registrar",
    formulario: "frmAnotacaoCadastro",
    envio: EnvioDoFormulario::BotaoDeEnvio("sbmRegistrarAnotacao"),
    campos_ocultos_exigidos: &["hdnIdProtocolo"],
    campos_ocultos_reescritos: &[],
    campos_usuario: &["txaDescricao"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::AnotacaoNaUnidade,
    },
};

/// Sem o botão de salvar a tela não oferece edição, apenas leitura.
pub(crate) fn salvar_oferecido(html: &str) -> bool {
    let Ok(dom) = tl::parse(html, tl::ParserOptions::default()) else {
        return false;
    };
    let Ok(form) = crate::formulario_web::localizar_formulario(&dom, "frmAnotacaoCadastro") else {
        return false;
    };
    form.children()
        .all(dom.parser())
        .iter()
        .filter_map(tl::Node::as_tag)
        .any(|tag| {
            tag.name().as_utf8_str().eq_ignore_ascii_case("button")
                && crate::html_atributos::valor_do_atributo(tag, "name")
                    .ok()
                    .flatten()
                    .as_deref()
                    == Some("sbmRegistrarAnotacao")
                && !crate::html_atributos::atributo_presente(tag, "disabled").unwrap_or(true)
        })
}

pub(crate) fn texto_anotacao(html: &str, id: &str) -> Result<String, ErroHtml> {
    use crate::formulario_web::{decodificar_atributo, localizar_formulario};
    use crate::html_atributos::valor_do_atributo;

    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroHtml::html("tela_de_anotacoes_nao_interpretada"))?;
    let form = localizar_formulario(&dom, "frmAnotacaoCadastro")
        .map_err(|_| ErroHtml::ausente("formulario_de_anotacoes_ausente"))?;
    let parser = dom.parser();
    let valor = |tag: &tl::HTMLTag<'_>, nome: &'static str| -> Result<Option<String>, ErroHtml> {
        valor_do_atributo(tag, nome)
            .map_err(|e| ErroHtml::atributo("tela_de_anotacoes_nao_interpretada", e))?
            .map(|bruto| {
                decodificar_atributo(&bruto)
                    .ok_or(ErroHtml::html("tela_de_anotacoes_nao_interpretada"))
            })
            .transpose()
    };
    let mut ids = Vec::new();
    let mut descricoes = Vec::new();
    for tag in form
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        match tag.name().as_utf8_str().as_ref() {
            "input" if valor(tag, "name")?.as_deref() == Some("hdnIdProtocolo") => {
                ids.push(valor(tag, "value")?);
            }
            "textarea" if valor(tag, "name")?.as_deref() == Some("txaDescricao") => {
                descricoes
                    .push(html_escape::decode_html_entities(&tag.inner_text(parser)).into_owned());
            }
            _ => {}
        }
    }
    match (ids.as_slice(), descricoes.as_slice()) {
        ([Some(publicado)], [texto]) if publicado == id => Ok(texto.clone()),
        ([Some(_)], [_]) => Err(ErroHtml::valor("anotacao_de_outro_processo")),
        ([], _) | (_, []) => Err(ErroHtml::ausente("anotacao_nao_publicada")),
        _ => Err(ErroHtml::repetido("campos_de_anotacao_repetidos")),
    }
}

/// Ausência só é comprovada se a tela do mesmo processo publicar o texto vazio.
pub(crate) fn anotacao_presente(html: &str, id: &str) -> Result<bool, ErroHtml> {
    texto_anotacao(html, id).map(|texto| !texto.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::{anotacao_presente, texto_anotacao};

    fn formulario(id: &str, texto: &str) -> String {
        format!(
            r#"<html><form id="frmAnotacaoCadastro" method="post" action="controlador.php?acao=anotacao_registrar">
            <input type="hidden" id="hdnIdProtocolo" name="hdnIdProtocolo" value="{id}">
            <input type="text" id="txtSiglaUsuario" name="txtSiglaUsuario" readonly value="aluno4">
            <textarea id="txaDescricao" name="txaDescricao">{texto}</textarea>
            <input type="checkbox" id="chkSinPrioridade" name="chkSinPrioridade">
            </form></html>"#
        )
    }

    #[test]
    fn bdd_anotacao_presente_na_tela_do_processo() {
        // Dado formulário de anotação da UnB com texto publicado,
        // quando leio o mesmo processo, então há anotação verificável.
        let html = formulario("50693", "Observa&ccedil;&atilde;o &amp; prioridade");
        assert_eq!(anotacao_presente(&html, "50693"), Ok(true));
    }

    #[test]
    fn texto_do_formulario_e_lido_sem_perder_entidades() {
        let html = formulario("50693", "Observa&ccedil;&atilde;o &amp; prioridade");
        assert_eq!(
            texto_anotacao(&html, "50693"),
            Ok("Observação & prioridade".into())
        );
    }

    #[test]
    fn bdd_anotacao_vazia_e_estado_observado_sem_anotacao() {
        // Dado formulário da UnB com textarea vazia,
        // quando releio, então há ausência observada de anotação.
        assert_eq!(
            anotacao_presente(&formulario("50693", ""), "50693"),
            Ok(false)
        );
        assert_eq!(
            anotacao_presente(&formulario("50693", " \n  "), "50693"),
            Ok(false)
        );
    }

    #[test]
    fn bdd_tela_de_outro_processo_nao_comprova_anotacao() {
        // Dado formulário servido para outro id,
        // quando confiro, então não se infere existência nem ausência.
        assert!(anotacao_presente(&formulario("99999", "Anotação"), "50693").is_err());
    }
}
