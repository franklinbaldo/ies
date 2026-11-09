//! Projeção GET do formulário de atribuição observado no SQLite efêmero.
//! Escrita fechada: nenhuma resposta sintética confirma um POST de atribuição.
use crate::{Estado, Resposta, parametro_unico};
use iesei::atribuicoes;
use iesei::banco_cassettes;
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::fmt::Write as _;

#[must_use]
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
    let Some(id) = parametro_unico(alvo, "id_procedimento")
        .filter(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let unidade = match banco_cassettes::ler(banco, instalacao, cenario) {
        Ok(Some(controle)) => controle.unidade.as_str().to_owned(),
        Ok(None) | Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
    };
    let observacao = match atribuicoes::ler(banco, instalacao, cenario, id, &unidade) {
        Ok(Some(observacao)) => observacao,
        Ok(None) | Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
    };
    let hash = crate::protocolo_sintetico::INFRA_HASH;
    let mut html = format!(
        "<html><body><form id='frmAtividadeAtribuir' method='post' \
         action='controlador.php?acao=procedimento_atribuicao_cadastrar&amp;id_procedimento={id}&amp;infra_hash={hash}'>\
         <input type='hidden' name='hdnInfraTipoPagina' value='2'>\
         <input type='hidden' name='hdnIdProtocolo' value='{id}'>\
         <select id='selAtribuicao' name='selAtribuicao'>\
         <option value='null' selected='selected'>Selecione</option>"
    );
    for usuario in observacao.usuarios {
        write!(
            html,
            "<option value='{}'>{} - {}</option>",
            crate::escapar(&usuario.id),
            crate::escapar(&usuario.login),
            crate::escapar(&usuario.nome)
        )
        .expect("formatação em String");
    }
    html.push_str(
        "</select><button type='submit' name='sbmSalvar' value='Salvar'>Salvar</button></form></body></html>",
    );
    Resposta::html(html)
}
