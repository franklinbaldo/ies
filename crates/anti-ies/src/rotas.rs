//! Rotas HTTP da simulação; um estado SQLite compartilhado entre capacidades.
use crate::{
    Resposta, controlar, documento_arvore, documento_versoes, documentos, eh_controlador_sei,
    parametro_unico, pesquisa_documento, processo_envio, retorno_programado,
};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;

/// Resolve uma única operação a partir do estado da instalação e do cenário.
pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    match parametro_unico(alvo, "acao") {
        Some("procedimento_enviar" | "anotacao_registrar") if eh_controlador_sei(alvo) => {
            processo_envio::responder(metodo, alvo, corpo, banco, instalacao, cenario)
        }
        None if parametro_unico(alvo, "acao_ajax")
            == Some("unidade_auto_completar_envio_processo") =>
        {
            processo_envio::autocompletar(metodo, alvo, corpo, banco, instalacao, cenario)
        }
        Some("protocolo_pesquisa_rapida") if eh_controlador_sei(alvo) => {
            pesquisa_documento::responder(metodo, alvo, corpo, banco, instalacao, cenario)
        }
        Some("retorno_programado_listar") => {
            retorno_programado(metodo, alvo, banco, instalacao, cenario)
        }
        Some("documento_escolher_tipo" | "documento_gerar" | "arvore_visualizar")
            if eh_controlador_sei(alvo) =>
        {
            documentos::atender(metodo, alvo, corpo, banco, instalacao, cenario)
        }
        Some("procedimento_trabalhar")
            if eh_controlador_sei(alvo) && parametro_unico(alvo, "id_documento").is_some() =>
        {
            documento_arvore::responder(metodo, alvo, banco, instalacao, cenario)
        }
        Some("procedimento_trabalhar")
            if eh_controlador_sei(alvo) && parametro_unico(alvo, "id_procedimento").is_some() =>
        {
            processo_envio::navegacao(metodo, alvo, banco, instalacao, cenario).unwrap_or_else(
                || documentos::atender(metodo, alvo, corpo, banco, instalacao, cenario),
            )
        }
        Some("procedimento_visualizar") if eh_controlador_sei(alvo) => processo_envio::navegacao(
            metodo, alvo, banco, instalacao, cenario,
        )
        .unwrap_or_else(|| documento_arvore::responder(metodo, alvo, banco, instalacao, cenario)),
        Some("procedimento_trabalhar")
            if eh_controlador_sei(alvo) && parametro_unico(alvo, "id_protocolo").is_some() =>
        {
            let id = parametro_unico(alvo, "id_protocolo").expect("guard verificou id");
            match iesei::documentos_simulados::ler_processo(banco, instalacao, cenario, id) {
                Ok(Some(_)) => documentos::atender(metodo, alvo, corpo, banco, instalacao, cenario),
                Ok(None) => documento_arvore::responder(metodo, alvo, banco, instalacao, cenario),
                Err(_) => Resposta::nova(crate::Estado::DadosInsuficientes),
            }
        }
        Some("documento_versao_listar" | "documento_visualizar") if eh_controlador_sei(alvo) => {
            documento_versoes::responder(metodo, alvo, banco, instalacao, cenario)
        }
        _ => controlar(metodo, alvo, banco, instalacao, cenario),
    }
}
