//! Oferta de cancelamento e POST único do SEI simulado (#273).
//! A tela é derivada de identidade e estado SQLite; não é cópia de cassette.
use crate::{
    Estado, Resposta, despachar, documentos::campo_formulario_unico,
    eh_controlador_sei, parametro_unico,
};
use iesei::cancelamento_documento::{self, Aplicacao, Situacao};
use iesei::documento_versoes::ler;
use iesei::instalacao::Instalacao;
use rusqlite::Connection;

/// Não anunciar cancelamento de documento de outro processo.
pub(crate) fn estado(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id_documento: &str,
    id_procedimento: &str,
) -> Result<Option<Situacao>, Estado> {
    cancelamento_documento::consultar(
        banco, instalacao, cenario, id_documento, id_procedimento,
    )
    .map_err(|_| Estado::DadosInsuficientes)
}

fn formulario(id_doc: &str, id_proc: &str) -> Resposta {
    let hash = "a".repeat(64);
    Resposta::html(format!(
        "<html><head><meta charset='utf-8'></head><body>         <form id='frmDocumentoCancelar' method='post' action='controlador.php?acao=documento_cancelar&amp;acao_origem=documento_cancelar&amp;arvore=1&amp;id_procedimento={id_proc}&amp;id_documento={id_doc}&amp;infra_hash={hash}'>         <input type='hidden' id='hdnInfraTipoPagina' name='hdnInfraTipoPagina' value='2'>         <button type='submit' id='sbmSalvar' name='sbmSalvar' value='Salvar'>Salvar</button>         <textarea id='txaMotivo' name='txaMotivo'></textarea>         </form></body></html>"
    ))
}

/// Uma rota só aceita a identidade completa da árvore e nunca inventa acesso.
#[must_use]
pub fn responder(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Resposta {
    if !eh_controlador_sei(alvo) || parametro_unico(alvo, "acao") != Some("documento_cancelar") {
        return despachar(metodo, alvo);
    }
    let (Some(id_doc), Some(id_proc)) = (
        parametro_unico(alvo, "id_documento")
            .filter(|id| id.bytes().all(|b| b.is_ascii_digit())),
        parametro_unico(alvo, "id_procedimento")
            .filter(|id| id.bytes().all(|b| b.is_ascii_digit())),
    ) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    if !matches!(metodo, "GET" | "POST") {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let documento = match ler(banco, origem, cenario, id_doc) {
        Ok(Some(documento)) if documento.id_procedimento == id_proc => documento,
        Ok(_) => return Resposta::nova(Estado::NaoEncontrado),
        Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
    };
    let situacao = match estado(banco, origem, cenario, id_doc, id_proc) {
        Ok(Some(situacao)) => situacao,
        Ok(None) => return Resposta::nova(Estado::DadosInsuficientes),
        Err(estado) => return Resposta::nova(estado),
    };
    if matches!(situacao, Situacao::Cancelado { .. }) {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    if metodo == "GET" {
        return formulario(&documento.id_documento, &documento.id_procedimento);
    }
    let (Some("2"), Some("Salvar"), Some(motivo)) = (
        campo_formulario_unico(corpo, "hdnInfraTipoPagina").as_deref(),
        campo_formulario_unico(corpo, "sbmSalvar").as_deref(),
        campo_formulario_unico(corpo, "txaMotivo"),
    ) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    match cancelamento_documento::aplicar(
        banco, origem, cenario, id_doc, id_proc, &motivo,
    ) {
        Ok(Aplicacao::Aplicada) => Resposta::redirecionar(format!(
            "controlador.php?acao=arvore_visualizar&acao_origem=documento_cancelar&arvore=1&id_procedimento={id_proc}&id_documento={id_doc}&atualizar_arvore=1&infra_hash={}",
            "c".repeat(64)
        )),
        Ok(Aplicacao::JaCancelado) => Resposta::nova(Estado::MetodoNaoPermitido),
        Ok(Aplicacao::DocumentoNaoEncontrado) => Resposta::nova(Estado::NaoEncontrado),
        Err(_) => Resposta::nova(Estado::DadosInsuficientes),
    }
}
