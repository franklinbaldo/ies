//! Formulário de prazo baseado em estado SQLite tipado e explicitamente observado.
//! POST sintético modifica somente o SQLite efêmero; nenhuma escrita externa.
use crate::{Estado, Resposta, escapar, parametro_unico};
use iesei::instalacao::Instalacao;
use iesei::prazos::{self, EstadoDoPrazo};
use rusqlite::Connection;

/// Serve o formulário sintético sem inferir "sem prazo" de ausência de captura.
pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if !matches!(metodo, "GET" | "POST") {
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
    if metodo == "POST" {
        return gravar_prazo(corpo, banco, instalacao, cenario, id);
    }
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

/// A escrita é uma hipótese sintética, não uma reprodução do POST do SEI.
/// O formulário deve pertencer ao processo e conter um pedido único.
fn gravar_prazo(
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
) -> Resposta {
    let pedido = (
        campo_unico(corpo, "hdnIdProcedimentos"),
        campo_unico(corpo, "rdoPrazo"),
        campo_unico(corpo, "txtPrazo"),
        campo_unico(corpo, "txtDias"),
    );
    let (Some(processo), Some(opcao), Some(data), Some(dias)) = pedido else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    if processo != id || opcao != "1" || data.is_empty() || !dias.is_empty() {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    match prazos::guardar(banco, instalacao, cenario, id, &EstadoDoPrazo::Data(data)) {
        Ok(()) => Resposta::redirecionar(format!(
            "controlador.php?acao=controle_prazo_definir&resultado=1&id_procedimento={id}"
        )),
        Err(_) => Resposta::nova(Estado::DadosInsuficientes),
    }
}

fn campo_unico(corpo: &str, campo: &str) -> Option<String> {
    let mut valores = corpo
        .split('&')
        .filter_map(|parte| parte.split_once('='))
        .filter(|(nome, _)| *nome == campo)
        .map(|(_, valor)| valor);
    let valor = valores.next()?;
    if valores.next().is_some() {
        return None;
    }
    let mut bytes = Vec::new();
    let mut entrada = valor.as_bytes().iter().copied();
    while let Some(byte) = entrada.next() {
        match byte {
            b'+' => bytes.push(b' '),
            b'%' => {
                let alto = char::from(entrada.next()?).to_digit(16)?;
                let baixo = char::from(entrada.next()?).to_digit(16)?;
                bytes.push(u8::try_from(alto * 16 + baixo).ok()?);
            }
            outro => bytes.push(outro),
        }
    }
    String::from_utf8(bytes).ok()
}
