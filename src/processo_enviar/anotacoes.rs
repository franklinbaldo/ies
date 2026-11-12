//! Leitura da anotação de origem, sem efetuar alterações.
//! Forma observada no SEI 4.0.9 de treinamento (Todos,
//! `tests/paginas_reais/processo_anotacao_registrar.html`).
use crate::acao_do_sei::AcaoDoSei;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, ResultadoDoEnvio,
};

/// Por que a anotação da origem não foi lida antes de enviar. Etapa
/// própria: o erro nasce aqui e chega ao envio por `From`.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroDasAnotacoes {
    LinkDasAnotacoes(#[source] crate::arvore_processo::ErroLinkDasAnotacoes),
    TelaDasAnotacoes(#[from] crate::transporte::FalhaTransporte),
    TelaDasAnotacoesRedirecionada,
    LeituraDasAnotacoes(#[from] ErroLeituraDaAnotacao),
}

/// Por que o formulário de anotação não comprova o texto do processo.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroLeituraDaAnotacao {
    FormularioDeAnotacoesAusente(#[source] crate::formulario_web::ErroSubmissaoFormulario),
    AnotacaoDeOutroProcesso,
    AnotacaoNaoPublicada,
    CamposDeAnotacaoRepetidos,
}

/// POST do botão Salvar no formulário de anotação observado no SEI 4.0.9.
pub(crate) const REGISTRAR_ANOTACAO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::AnotacaoRegistrar,
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
pub(crate) fn salvar_oferecido(
    html: &crate::tela_html::HtmlServido,
) -> Result<bool, ErroLeituraDaAnotacao> {
    let dom = crate::tela_html::arvore(html);
    let form = crate::formulario_web::localizar_formulario(&dom, "frmAnotacaoCadastro")
        .map_err(ErroLeituraDaAnotacao::FormularioDeAnotacoesAusente)?;
    for tag in form.descendentes() {
        if !tag.nome().eq_ignore_ascii_case("button") {
            continue;
        }
        let nome = tag.atributo("name");
        if nome == Some("sbmRegistrarAnotacao") && !tag.presente("disabled") {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(crate) fn texto_anotacao(
    html: &crate::tela_html::HtmlServido,
    id: &str,
) -> Result<String, ErroLeituraDaAnotacao> {
    use crate::formulario_web::localizar_formulario;

    let dom = crate::tela_html::arvore(html);
    let form = localizar_formulario(&dom, "frmAnotacaoCadastro")
        .map_err(ErroLeituraDaAnotacao::FormularioDeAnotacoesAusente)?;
    let valor = |tag: crate::tela_html::Elemento<'_>,
                 nome: &'static str|
     -> Result<Option<String>, ErroLeituraDaAnotacao> {
        Ok(tag.atributo(nome).map(str::to_owned))
    };
    let mut ids = Vec::new();
    let mut descricoes = Vec::new();
    for tag in form.descendentes() {
        match tag.nome() {
            "input" if valor(tag, "name")?.as_deref() == Some("hdnIdProtocolo") => {
                ids.push(valor(tag, "value")?);
            }
            "textarea" if valor(tag, "name")?.as_deref() == Some("txaDescricao") => {
                descricoes.push(tag.texto());
            }
            _ => {}
        }
    }
    match (ids.as_slice(), descricoes.as_slice()) {
        ([Some(publicado)], [texto]) if publicado == id => Ok(texto.clone()),
        ([Some(_)], [_]) => Err(ErroLeituraDaAnotacao::AnotacaoDeOutroProcesso),
        ([], _) | (_, []) => Err(ErroLeituraDaAnotacao::AnotacaoNaoPublicada),
        _ => Err(ErroLeituraDaAnotacao::CamposDeAnotacaoRepetidos),
    }
}

/// Ausência só é comprovada se a tela do mesmo processo publicar o texto vazio.
pub(crate) fn anotacao_presente(
    html: &crate::tela_html::HtmlServido,
    id: &str,
) -> Result<bool, ErroLeituraDaAnotacao> {
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
        let html = crate::tela_html::HtmlServido::de_teste(&formulario(
            "50693",
            "Observa&ccedil;&atilde;o &amp; prioridade",
        ));
        assert_eq!(anotacao_presente(&html, "50693"), Ok(true));
    }

    #[test]
    fn texto_do_formulario_e_lido_sem_perder_entidades() {
        let html = crate::tela_html::HtmlServido::de_teste(&formulario(
            "50693",
            "Observa&ccedil;&atilde;o &amp; prioridade",
        ));
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
            anotacao_presente(
                &crate::tela_html::HtmlServido::de_teste(&formulario("50693", "")),
                "50693"
            ),
            Ok(false)
        );
        assert_eq!(
            anotacao_presente(
                &crate::tela_html::HtmlServido::de_teste(&formulario("50693", " \n  ")),
                "50693"
            ),
            Ok(false)
        );
    }

    #[test]
    fn bdd_tela_de_outro_processo_nao_comprova_anotacao() {
        // Dado formulário servido para outro id,
        // quando confiro, então não se infere existência nem ausência.
        assert_eq!(
            anotacao_presente(
                &crate::tela_html::HtmlServido::de_teste(&formulario("99999", "Anotação")),
                "50693"
            ),
            Err(super::ErroLeituraDaAnotacao::AnotacaoDeOutroProcesso)
        );
    }
}
