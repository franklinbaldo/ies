//! Rotas HTTP da simulação; um estado SQLite compartilhado entre capacidades.
use crate::{
    Resposta, arvore_compartilhada, controlar, documento_arvore, documento_versoes, documentos,
    editor_documento, eh_controlador_sei, parametro_unico, pdf_externo, pesquisa_documento, processo_envio,
    retorno_programado,
};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;

// Responde a visualização de documentos a partir de um único estado SQLite.
fn visualizar(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    let pdfs = parametro_unico(alvo, "id_procedimento")
        .map(|id| iesei::pdf_externo::documentos(banco, instalacao, cenario, id))
        .transpose();
    match pdfs {
        Err(_) => Resposta::nova(crate::Estado::DadosInsuficientes),
        Ok(Some(documentos)) if !documentos.is_empty() => {
            arvore_compartilhada::responder(metodo, alvo, banco, instalacao, cenario)
        }
        Ok(_) => processo_envio::navegacao(metodo, alvo, banco, instalacao, cenario)
            .unwrap_or_else(|| {
                if documentos::eh_rota_documentos(alvo)
                    && let Some(id) = parametro_unico(alvo, "id_procedimento")
                {
                    match iesei::documentos_simulados::ler_processo(banco, instalacao, cenario, id)
                    {
                        Ok(Some(_)) => {
                            return documentos::atender(
                                metodo, alvo, corpo, banco, instalacao, cenario,
                            );
                        }
                        Ok(None) => {}
                        Err(_) => {
                            return Resposta::nova(crate::Estado::DadosInsuficientes);
                        }
                    }
                }
                documento_arvore::responder(metodo, alvo, banco, instalacao, cenario)
            }),
    }
}

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
            visualizar(metodo, alvo, corpo, banco, instalacao, cenario)
        }
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
        Some("documento_editar") if eh_controlador_sei(alvo) => {
            editor_documento::responder(metodo, alvo, corpo, banco, instalacao, cenario)
        }
        Some("documento_versao_listar" | "documento_visualizar") if eh_controlador_sei(alvo) => {
            documento_versoes::responder(metodo, alvo, banco, instalacao, cenario)
        }
        _ => controlar(metodo, alvo, banco, instalacao, cenario),
    }
}
