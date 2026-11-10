//! Rotas HTTP da simulação; um estado SQLite compartilhado entre capacidades.
use crate::{
    Resposta, arvore_compartilhada, controlar, documento_arvore, documento_cancelar,
    documento_ciencia, documento_versoes, documentos, editor_documento, eh_controlador_sei,
    parametro_unico, pdf_externo, pesquisa_documento, prazo_processo, processo_andamento,
    processo_anexacao, processo_atribuicao, processo_ciencia, processo_criacao, processo_envio,
    processo_marcadores, processo_relacionar, processo_sobrestamento, retorno_programado,
};
use iesei::acesso_sessao::{self, AcessoSessao};
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

/// Resolve o destino obtido por pesquisa sem confundir identidade documental
/// com a unidade ativa de um processo em tramitação.
fn trabalhar_protocolo(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    processo_envio::navegacao(metodo, alvo, banco, instalacao, cenario).unwrap_or_else(|| {
        let id = parametro_unico(alvo, "id_protocolo").expect("guard verificou id");
        match iesei::documentos_simulados::ler_processo(banco, instalacao, cenario, id) {
            Ok(Some(_)) => documentos::atender(metodo, alvo, corpo, banco, instalacao, cenario),
            Ok(None) => documento_arvore::responder(metodo, alvo, banco, instalacao, cenario),
            Err(_) => Resposta::nova(crate::Estado::DadosInsuficientes),
        }
    })
}

/// Resolve uma única operação a partir do estado da instalação e do cenário.
#[allow(clippy::too_many_lines)] // Despachante de ações HTTP: cada capacidade permanece em módulo próprio.
pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    corpo: &[u8],
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    // A recusa de sessão sintética vale inclusive antes de um upload.
    if eh_controlador_sei(alvo) {
        match acesso_sessao::consultar(banco, instalacao, cenario) {
            Ok(AcessoSessao::Recusado) => return Resposta::nova(crate::Estado::AcessoNegado),
            Ok(AcessoSessao::Permitido) => {}
            Err(_) => return Resposta::nova(crate::Estado::DadosInsuficientes),
        }
    }
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
        Some("procedimento_controlar") if eh_controlador_sei(alvo) => {
            processo_criacao::controle_com_menu(metodo, alvo, banco, instalacao, cenario)
        }
        Some("procedimento_escolher_tipo" | "procedimento_gerar") if eh_controlador_sei(alvo) => {
            processo_criacao::responder(metodo, alvo, banco, instalacao, cenario)
        }
        Some("procedimento_consultar_historico" | "procedimento_atualizar_andamento")
            if eh_controlador_sei(alvo) =>
        {
            processo_andamento::responder(metodo, alvo, corpo, banco, instalacao, cenario)
        }
        Some("acompanhamento_gerenciar" | "acompanhamento_excluir") if eh_controlador_sei(alvo) => {
            crate::processo_acompanhar::responder(metodo, alvo, corpo, banco, instalacao, cenario)
        }
        Some("comentario_listar" | "comentario_cadastrar") if eh_controlador_sei(alvo) => {
            crate::processo_comentarios::responder(metodo, alvo, corpo, banco, instalacao, cenario)
        }
        Some("procedimento_anexar") if eh_controlador_sei(alvo) => {
            processo_anexacao::responder(metodo, alvo, corpo, banco, instalacao, cenario)
        }
        None if parametro_unico(alvo, "acao_ajax") == Some("procedimento_anexacao_verificar") => {
            processo_anexacao::consulta(metodo, alvo, corpo, banco, instalacao, cenario)
        }
        Some("procedimento_sobrestar" | "procedimento_remover_sobrestamento")
            if eh_controlador_sei(alvo) =>
        {
            processo_sobrestamento::responder(metodo, alvo, corpo, banco, instalacao, cenario)
        }
        Some(
            "andamento_marcador_gerenciar"
            | "andamento_marcador_remover"
            | "andamento_marcador_cadastrar",
        ) if eh_controlador_sei(alvo) => {
            processo_marcadores::responder(metodo, alvo, corpo, banco, instalacao, cenario)
        }
        Some("procedimento_atribuicao_cadastrar") if eh_controlador_sei(alvo) => {
            processo_atribuicao::responder(metodo, alvo, corpo, banco, instalacao, cenario)
        }
        Some("controle_prazo_definir") if eh_controlador_sei(alvo) => {
            prazo_processo::responder(metodo, alvo, corpo, banco, instalacao, cenario)
        }
        Some("controle_prazo_listar" | "controle_prazo_excluir") if eh_controlador_sei(alvo) => {
            prazo_processo::responder_lista(metodo, alvo, corpo, banco, instalacao, cenario)
        }
        Some("procedimento_relacionar" | "procedimento_excluir_relacionamento")
            if eh_controlador_sei(alvo) =>
        {
            processo_relacionar::responder(metodo, alvo, corpo, banco, instalacao, cenario)
        }
        None if parametro_unico(alvo, "acao_ajax") == Some("protocolo_RI1023") => {
            processo_anexacao::identificar(metodo, alvo, corpo, banco, instalacao, cenario)
                .unwrap_or_else(|| {
                    processo_relacionar::consulta(metodo, alvo, corpo, banco, instalacao, cenario)
                })
        }
        Some("protocolo_pesquisar") if eh_controlador_sei(alvo) => {
            pesquisa_documento::responder_pesquisa(metodo, alvo, corpo, banco, instalacao, cenario)
        }
        Some("protocolo_pesquisa_rapida") if eh_controlador_sei(alvo) => {
            pesquisa_documento::responder(metodo, alvo, corpo, banco, instalacao, cenario)
        }
        Some("bloco_assinatura_listar" | "bloco_interno_listar" | "bloco_reuniao_listar")
            if eh_controlador_sei(alvo) =>
        {
            crate::blocos::responder(metodo, alvo, corpo, banco, instalacao, cenario)
        }
        Some("rel_bloco_protocolo_listar") if eh_controlador_sei(alvo) => {
            crate::bloco_detalhe::responder(metodo, alvo, banco, instalacao, cenario)
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
        Some("arvore_visualizar") if eh_controlador_sei(alvo) => {
            processo_sobrestamento::tela(metodo, alvo, banco, instalacao, cenario)
                .or_else(|| processo_ciencia::tela(metodo, alvo, banco, instalacao, cenario))
                .or_else(|| documento_ciencia::tela(metodo, alvo, banco, instalacao, cenario))
                .unwrap_or_else(|| {
                    pdf_externo::complementar_arvore(
                        documentos::atender(metodo, alvo, corpo, banco, instalacao, cenario),
                        alvo,
                        banco,
                        instalacao,
                        cenario,
                    )
                })
        }
        Some("procedimento_trabalhar")
            if eh_controlador_sei(alvo)
                // Duplicacao ou vazio de protocolo/processo nao equivalem a ausencia.
                // Validar antes de qualquer ramo que possa escolher outra rota.
                && ["id_protocolo", "id_procedimento"]
                    .iter()
                    .any(|nome| documento_versoes::parametro(alvo, nome).is_err()) =>
        {
            Resposta::nova(crate::Estado::RequisicaoInvalida)
        }
        Some("procedimento_trabalhar")
            if eh_controlador_sei(alvo)
                // Identidade documental inválida também deve chegar ao
                // validador, nunca cair silenciosamente na rota de processo.
                && !matches!(
                    documento_arvore::identidade_documental(alvo, true),
                    Ok(None)
                ) =>
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
            trabalhar_protocolo(metodo, alvo, corpo, banco, instalacao, cenario)
        }
        Some("procedimento_ciencia") if eh_controlador_sei(alvo) => {
            processo_ciencia::responder(metodo, alvo, banco, instalacao, cenario)
        }
        Some("protocolo_ciencia_listar")
            if eh_controlador_sei(alvo) && parametro_unico(alvo, "id_documento").is_none() =>
        {
            processo_ciencia::responder(metodo, alvo, banco, instalacao, cenario)
        }
        Some("documento_ciencia" | "protocolo_ciencia_listar") if eh_controlador_sei(alvo) => {
            documento_ciencia::responder(metodo, alvo, banco, instalacao, cenario)
        }
        Some("documento_cancelar") if eh_controlador_sei(alvo) => {
            documento_cancelar::responder(metodo, alvo, corpo, banco, instalacao, cenario)
        }
        Some("documento_download_anexo") if eh_controlador_sei(alvo) => {
            pdf_externo::baixar(metodo, alvo, banco, instalacao, cenario)
        }
        Some("documento_editar" | "editor_montar") if eh_controlador_sei(alvo) => {
            editor_documento::responder(metodo, alvo, corpo, banco, instalacao, cenario)
        }
        Some("editor_salvar") => {
            editor_documento::responder(metodo, alvo, corpo, banco, instalacao, cenario)
        }
        Some("documento_versao_listar" | "documento_visualizar") if eh_controlador_sei(alvo) => {
            documento_versoes::responder(metodo, alvo, banco, instalacao, cenario)
        }
        _ => controlar(metodo, alvo, banco, instalacao, cenario),
    }
}
