//! Molde sintético da `UnB`, sem URLs assinadas de contas reais.
//! Caso: `todos tests/test_ciclo_sobrestar_010.py` @323eb01.
use super::tela::{ErroTela, conferir_formulario};
use crate::consulta_do_controlador::{ErroChaveDaConsulta, ErroConsultaDoControlador};
const ID: &str = "10";
pub(super) fn formulario() -> String {
    format!(
        r#"<html><body>
<form id="frmDesentranharDocumento" method="post" action="controlador.php?acao=procedimento_sobrestar&acao_origem=procedimento_sobrestar&id_procedimento={ID}&infra_hash={hash}">
<input type="hidden" name="hdnInfraTipoPagina" value="2" />
<button type="submit" name="sbmSalvar" id="sbmSalvar" value="Salvar">Salvar</button>
<button type="button" name="btnCancelar" value="Cancelar">Cancelar</button>
<select id="selProcedimentos" name="selProcedimentos" multiple="multiple"><option value="{ID}">Sintético</option></select>
<input type="radio" name="rdoSobrestar" id="optSomenteSobrestar" value="S" />
<input type="radio" name="rdoSobrestar" id="optSobrestarVincular" value="V" />
<input type="text" name="txtProcedimentoDestino" value="" />
<input type="text" name="txtIdentificacaoProcedimentoDestino" readonly="readonly" value="" />
<textarea id="txaMotivo" name="txaMotivo"></textarea>
<input type="hidden" name="hdnIdProtocolos" value="{ID}" />
<input type="hidden" name="hdnIdProcedimentoDestino" value="" />
</form></body></html>"#,
        hash = "f".repeat(64)
    )
}
#[test]
fn formulario_observado_com_radio_desmarcado_e_motivo_aceito() {
    assert_eq!(conferir_formulario(&formulario(), ID), Ok(()));
}
#[test]
fn formulario_de_outro_processo_ou_id_repetido_recusado() {
    let origem = formulario();
    assert_eq!(
        conferir_formulario(
            &origem.replace(
                "name=\"hdnIdProtocolos\" value=\"10\"",
                "name=\"hdnIdProtocolos\" value=\"999\""
            ),
            ID
        ),
        Err(ErroTela::OutroProcesso)
    );
    let outro = origem.replace("<input type=\"hidden\" name=\"hdnIdProtocolos\" value=\"10\" />", "<input type=\"hidden\" name=\"hdnIdProtocolos\" value=\"10\" /><input type=\"hidden\" name=\"hdnIdProtocolos\" value=\"10\" />");
    assert_eq!(
        conferir_formulario(&outro, ID),
        Err(ErroTela::CampoInesperado)
    );
}
#[test]
fn action_com_par_sem_igual_ou_id_repetido_recusa_com_a_causa() {
    let origem = formulario();
    assert_eq!(
        conferir_formulario(&origem.replace("&infra_hash=", "&lixo&infra_hash="), ID),
        Err(ErroTela::ConsultaDaActionIlegivel(
            ErroConsultaDoControlador::ParSemIgual
        ))
    );
    assert_eq!(
        conferir_formulario(
            &origem.replace("&infra_hash=", "&id_procedimento=10&infra_hash="),
            ID
        ),
        Err(ErroTela::ChaveDaActionIlegivel(
            ErroChaveDaConsulta::Repetida {
                chave: "id_procedimento"
            }
        ))
    );
}

#[test]
fn opcao_de_sobrestar_desconhecida_recusada() {
    assert_eq!(
        conferir_formulario(
            &formulario().replace("value=\"S\" />", "value=\"X\" />"),
            ID
        ),
        Err(ErroTela::OpcaoNaoOferecida)
    );
}
#[test]
fn vinculo_preenchido_ou_campo_extra_falha_fechado() {
    assert_eq!(
        conferir_formulario(
            &formulario().replace(
                "name=\"hdnIdProcedimentoDestino\" value=\"\"",
                "name=\"hdnIdProcedimentoDestino\" value=\"99\""
            ),
            ID
        ),
        Err(ErroTela::DestinoPreenchido)
    );
    assert_eq!(
        conferir_formulario(
            &formulario().replace("</form>", "<input name=\"campoNovo\" value=\"1\" /></form>"),
            ID
        ),
        Err(ErroTela::CampoInesperado)
    );
}

#[test]
fn formulario_com_submissao_diferente_do_navegador_recusa() {
    // Casos de Todos `tests/test_ciclo_sobrestar_010.py` @323eb01.
    let base = formulario();
    let casos = [
        base.replace("value=\"Salvar\"", "value=\"Salvar\" formmethod=\"get\""),
        base.replace(
            "value=\"Salvar\"",
            "value=\"Salvar\" formaction=\"controlador.php?acao=outro\"",
        ),
        base.replace(
            "value=\"Salvar\"",
            "value=\"Salvar\" formenctype=\"multipart/form-data\"",
        ),
        base.replace(
            "</form>",
            "</form><input name=\"x\" value=\"1\" form=\"frmDesentranharDocumento\" />",
        ),
        base.replace(
            "<textarea id=\"txaMotivo\"",
            "<textarea readonly=\"readonly\" id=\"txaMotivo\"",
        ),
    ];
    for formulario_alterado in casos {
        assert_eq!(
            conferir_formulario(&formulario_alterado, ID),
            Err(ErroTela::CampoInesperado),
            "envio que o navegador faria de outro modo precisa ser recusado",
        );
    }
}

#[test]
fn botao_pesquisar_do_destino_nao_e_campo_enviado() {
    // UnB 4.0.9 ao vivo (10/11/2026): o form traz o botão btnPesquisar
    // (type="button") ao lado do processo de destino; ele não vai no POST.
    let tela = formulario().replace(
        "</form>",
        "<button type=\"button\" name=\"btnPesquisar\" id=\"btnPesquisar\" value=\"Pesquisar\">Pesquisar</button></form>",
    );
    assert_eq!(conferir_formulario(&tela, ID), Ok(()));
}

#[test]
fn somente_sobrestar_ja_marcado_e_a_opcao_enviada() {
    // UFRJ e GO ao vivo (10/11/2026): optSomenteSobrestar vem checked.
    let marcado = formulario().replace("value=\"S\" />", "value=\"S\" checked=\"checked\" />");
    assert_eq!(conferir_formulario(&marcado, ID), Ok(()));
    let vincular = formulario().replace("value=\"V\" />", "value=\"V\" checked=\"checked\" />");
    assert!(conferir_formulario(&vincular, ID).is_err());
}
