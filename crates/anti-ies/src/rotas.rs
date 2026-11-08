//! Rotas HTTP da simulação; um estado SQLite compartilhado entre capacidades.
use crate::{
    Resposta, arvore_compartilhada, controlar, documento_arvore, documento_versoes, documentos,
    eh_controlador_sei, parametro_unico, pdf_externo, pesquisa_documento, processo_envio,
    retorno_programado,
};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;

/// Resolve uma única operação a partir do estado da instalação e do cenário.
pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    corpo: &[u8],
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if matches!(
        parametro_unico(alvo, "acao"),
        Some("documento_receber" | "documento_upload_anexo")
    ) && eh_controlador_sei(alvo)
    {
        return pdf_externo::atender(metodo, alvo, corpo, banco, instalacao, cenario)
            .unwrap_or_else(|| Resposta::nova(crate::Estado::NaoImplementado));
    }
    let Ok(corpo) = std::str::from_utf8(corpo) else {
        return Resposta::nova(crate::Estado::RequisicaoInvalida);
    };
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
        Some("documento_escolher_tipo") if eh_controlador_sei(alvo) => {
            pdf_externo::complementar_escolha(
                documentos::atender(metodo, alvo, corpo, banco, instalacao, cenario),
                alvo,
                banco,
                instalacao,
                cenario,
            )
        }
        Some("documento_gerar") if eh_controlador_sei(alvo) => {
            documentos::atender(metodo, alvo, corpo, banco, instalacao, cenario)
        }
        Some("arvore_visualizar") if eh_controlador_sei(alvo) => pdf_externo::complementar_arvore(
            documentos::atender(metodo, alvo, corpo, banco, instalacao, cenario),
            alvo,
            banco,
            instalacao,
            cenario,
        ),
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
        Some("procedimento_visualizar") if eh_controlador_sei(alvo) => {
            arvore_compartilhada::responder(metodo, alvo, banco, instalacao, cenario)
        }
        Some("procedimento_trabalhar")
            if eh_controlador_sei(alvo) && parametro_unico(alvo, "id_protocolo").is_some() =>
        {
            documento_arvore::responder(metodo, alvo, banco, instalacao, cenario)
        }
        Some("documento_versao_listar" | "documento_visualizar") if eh_controlador_sei(alvo) => {
            documento_versoes::responder(metodo, alvo, banco, instalacao, cenario)
        }
        _ => controlar(metodo, alvo, banco, instalacao, cenario),
    }
}
