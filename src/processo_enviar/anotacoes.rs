//! Leitura da anotação de origem, sem efetuar alterações.
//! Forma observada no SEI 4.0.9 de treinamento (Todos,
//! tests/paginas_reais/processo_anotacao_registrar.html).
use crate::erro_html::ErroHtml;

pub(crate) fn anotacao_presente(html: &str, id: &str) -> Result<bool, ErroHtml> {
    let _ = (html, id);
    Err(ErroHtml::valor("anotacao_ainda_nao_lida"))
}

#[cfg(test)]
mod tests {
    use super::anotacao_presente;

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
    fn bdd_anotacao_vazia_e_estado_observado_sem_anotacao() {
        // Dado formulário da UnB com textarea vazia,
        // quando releio, então há ausência observada de anotação.
        assert_eq!(anotacao_presente(&formulario("50693", ""), "50693"), Ok(false));
        assert_eq!(anotacao_presente(&formulario("50693", " \n  "), "50693"), Ok(false));
    }

    #[test]
    fn bdd_tela_de_outro_processo_nao_comprova_anotacao() {
        // Dado formulário servido para outro id,
        // quando confiro, então não se infere existência nem ausência.
        assert!(anotacao_presente(&formulario("99999", "Anotação"), "50693").is_err());
    }
}
