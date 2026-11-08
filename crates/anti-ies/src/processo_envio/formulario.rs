//! Formulario de envio projetado do estado SQLite (hipotese sintetica).
use super::html;
use iesei::processos::ProcessoSimulado;
use std::fmt::Write;

pub(super) fn pagina_de_envio(processo: &ProcessoSimulado) -> String {
    let mut pagina = format!(
        "<html><head><script type='text/javascript'>\n         objAutoCompletarUnidade = new infraAjaxAutoCompletar('hdnIdUnidade','txtUnidade','controlador_ajax.php?acao_ajax=unidade_auto_completar_envio_processo&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={}');\n         return 'palavras_pesquisa='+document.getElementById('txtUnidade').value+'&id_orgao='+document.getElementById('selOrgao').value;\n         </script></head><body>",
        "2".repeat(64)
    );
    // Ausencia de observacao da anotacao nao significa ausencia de texto:
    // nesse cenario a UI simulada nao oferece remocao.
    let remocao_oferecida = processo
        .anotacao_origem
        .as_ref()
        .map(|_| "<input type='checkbox' name='chkSinRemoverAnotacoes'>")
        .unwrap_or("");
    write!(
        pagina,
        "<form id='frmAtividadeListar' method='post' action='controlador.php?acao=procedimento_enviar&acao_origem=procedimento_enviar&arvore=1&id_procedimento={}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={}'>         <input type='hidden' name='hdnInfraTipoPagina' value='2'>         <button type='submit' name='sbmEnviar' value='Enviar'>Enviar</button>         <select id='selProcedimentos' name='selProcedimentos' multiple>         <option value='{}'>{} - Processo simulado</option></select>         <select id='selOrgao' name='selOrgao'>         <option value='null' selected>&nbsp;</option>         <option value='' selected>Todos</option></select>         <input type='text' id='txtUnidade' name='txtUnidade'>         <input type='hidden' name='hdnIdUnidade' value=''>         <select id='selUnidades' name='selUnidades' multiple></select>         <input type='checkbox' name='chkSinManterAberto'>         {remocao_oferecida}         <input type='checkbox' name='chkSinEnviarEmailNotificacao'>         <input type='radio' name='rdoPrazo' value='1'>         <input type='text' name='txtPrazo' value=''>         <input type='radio' name='rdoPrazo' value='2'>         <input type='text' name='txtDias' value=''>         <input type='checkbox' name='chkSinDiasUteis'>         <input type='hidden' name='hdnIdProtocolos' value='{}'>         <input type='hidden' name='hdnIdAtividades' value='1'>         <input type='hidden' name='hdnUnidades' value=''>         </form></body></html>",
        html(&processo.id),
        "3".repeat(64),
        html(&processo.id),
        html(&processo.numero),
        html(&processo.id),
    )
    .expect("formatação de String");
    pagina
}
