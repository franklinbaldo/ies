//! Formulário de prazo baseado em estado SQLite tipado e explicitamente observado.
//! Apenas GET de leitura/ prévia; POST permanece bloqueado no binário IES.
use crate::{Estado, Resposta, escapar, parametro_unico};
use iesei::instalacao::Instalacao;
use iesei::prazos::{self, EstadoDoPrazo};
use rusqlite::Connection;

/// Serve o formulário sintético sem inferir "sem prazo" de ausência de captura.
pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if metodo != "GET" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    if parametro_unico(alvo, "acao") != Some("controle_prazo_definir") {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    let Some(id) = parametro_unico(alvo, "id_procedimento")
        .filter(|id| id.bytes().all(|byte| byte.is_ascii_digit()))
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let Ok(Some(estado)) = prazos::ler(banco, instalacao, cenario, id) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let (radio_data, radio_concluido, data) = match estado {
        EstadoDoPrazo::SemPrazo => ("", "", String::new()),
        EstadoDoPrazo::Data(data) => (" checked", "", escapar(&data)),
        EstadoDoPrazo::Concluido => ("", " checked", String::new()),
    };
    let id = escapar(id);
    let assinatura = crate::protocolo_sintetico::INFRA_HASH;
    Resposta::html(format!(
        "<html><body><form id='frmControlePrazoCadastro' method='post' \
         action='controlador.php?acao=controle_prazo_definir&amp;id_procedimento={id}&amp;infra_hash={assinatura}'>\
         <input type='hidden' name='hdnInfraTipoPagina' value='2'>\
         <input type='hidden' name='hdnIdControlePrazo' value=''>\
         <input type='hidden' name='hdnIdProcedimentos' value='{id}'>\
         <input type='radio' name='rdoPrazo' value='1'{radio_data}>\
         <input type='text' name='txtPrazo' value='{data}'>\
         <input type='radio' name='rdoPrazo' value='2'>\
         <input type='text' name='txtDias' value=''>\
         <input type='radio' name='rdoPrazo' value='3'{radio_concluido}>\
         <input type='checkbox' name='chkSinDiasUteis' value='on'>\
         <button type='submit' name='sbmDefinirControlePrazo'>Salvar</button>\
         </form></body></html>"
    ))
}
