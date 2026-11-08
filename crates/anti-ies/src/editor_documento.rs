//! Editor de documentos sintéticos sobre o mesmo estado do histórico (#89).
//! Hipótese de HTML/formulário para integração IES ↔ anti-IES; não é
//! evidência de formato real do editor SEI 4/5.
use crate::{
    Estado, Resposta, despachar, documentos::campo_formulario_unico, eh_controlador_sei, escapar,
    parametro_unico,
};
use iesei::documento_versoes::ler;
use iesei::editor_documento::{Edicao, Falha, consultar_oferta, publicar};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::num::NonZeroU32;

fn resposta_de_falha(falha: &Falha) -> Resposta {
    let estado = match falha {
        Falha::NaoObservado | Falha::Sqlite(_) => Estado::DadosInsuficientes,
        Falha::NaoOferecido => Estado::MetodoNaoPermitido,
        Falha::VersaoDivergente
        | Falha::ConteudoDivergente
        | Falha::AssinaturasNaoConfirmadas
        | Falha::DadoInvalido => Estado::RequisicaoInvalida,
    };
    Resposta::nova(estado)
}

fn formulario(banco: &Connection, origem: &Instalacao, cenario: &str, id: &str) -> Resposta {
    let oferta = match consultar_oferta(banco, origem, cenario, id) {
        Ok(Some(oferta)) => oferta,
        Ok(None) => return Resposta::nova(Estado::DadosInsuficientes),
        Err(falha) => return resposta_de_falha(&falha),
    };
    if !oferta.editavel {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Ok(Some(documento)) = ler(banco, origem, cenario, id) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let Some(ultima) = documento.revisoes.iter().max_by_key(|r| r.versao) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let conteudo = escapar(&ultima.html_corpo);
    let assinaturas = oferta.assinaturas;
    let aviso = if oferta.assinaturas > 0 {
        format!(
            "<p id='avisoAssinaturas'>A edição pode remover {} assinatura(s). Confirmação explícita necessária.</p>",
            oferta.assinaturas
        )
    } else {
        String::new()
    };
    Resposta::html(format!(
        "<html><body><h1>Editar documento {}</h1><output id='assinaturasAtuais'>{assinaturas}</output>{aviso}\
         <form id='frmDocumentoEditar' method='post' action='controlador.php?acao=documento_editar&amp;id_documento={id}&amp;infra_hash={}'>\
         <input type='hidden' name='versao_base' value='{}'>\
         <textarea name='html_base' hidden>{conteudo}</textarea>\
         <textarea name='html_editado'>{conteudo}</textarea>\
         <label><input type='checkbox' name='confirmar_assinaturas' value='1'>\
         Confirmo possível perda de assinaturas</label>\
         <button type='submit'>Salvar edição</button></form></body></html>",
        escapar(&documento.numero),
        "e".repeat(64),
        ultima.versao
    ))
}

fn salvar(
    corpo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
) -> Resposta {
    let versao_base = campo_formulario_unico(corpo, "versao_base")
        .and_then(|texto| texto.parse::<u32>().ok())
        .and_then(NonZeroU32::new);
    let (Some(versao_base), Some(html_base), Some(html_editado)) = (
        versao_base,
        campo_formulario_unico(corpo, "html_base"),
        campo_formulario_unico(corpo, "html_editado"),
    ) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let confirmar_assinaturas = match campo_formulario_unico(corpo, "confirmar_assinaturas") {
        None => false,
        Some(texto) if texto == "1" => true,
        Some(texto) if texto == "0" => false,
        _ => return Resposta::nova(Estado::RequisicaoInvalida),
    };
    match publicar(
        banco,
        origem,
        cenario,
        id,
        Edicao {
            versao_base,
            html_base: &html_base,
            html_editado: &html_editado,
            confirmar_assinaturas,
        },
    ) {
        Ok(publicada) => Resposta::redirecionar(format!(
            "controlador.php?acao=documento_visualizar&id_documento={id}&versao={}",
            publicada.versao_nova
        )),
        Err(falha) => resposta_de_falha(&falha),
    }
}

/// Projeção SEI 4 guiada pela observação do editor `UnB` 4.0.9
/// (docs/como-o-sei-funciona/editor-do-documento.md).
/// Os campos são uma hipótese funcional: o membro JSON original ainda
/// precisa ser confrontado; não é reprodução de cassette.
fn formulario_editor_4(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
) -> Resposta {
    let oferta = match consultar_oferta(banco, origem, cenario, id) {
        Ok(Some(oferta)) if oferta.editavel => oferta,
        Ok(Some(_)) => return Resposta::nova(Estado::MetodoNaoPermitido),
        Ok(None) => return Resposta::nova(Estado::DadosInsuficientes),
        Err(falha) => return resposta_de_falha(&falha),
    };
    let Ok(Some(documento)) = ler(banco, origem, cenario, id) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let Some(ultima) = documento.revisoes.iter().max_by_key(|r| r.versao) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    Resposta::html(format!(
        "<html><body><form id='frmEditor' hidden method='post' target='ifrEditorSalvar' \
         action='editor/editor_processar.php?acao=editor_salvar&amp;id_documento={id}&amp;infra_hash={}'>\
         <textarea name='txaEditor_1'>{}</textarea>\
         <input type='hidden' id='hdnVersao' name='hdnVersao' value='{}'>\
         <input type='hidden' id='hdnIgnorarNovaVersao' name='hdnIgnorarNovaVersao' value='N'>\
         </form><iframe id='ifrEditorSalvar' name='ifrEditorSalvar'></iframe>\
         <output id='assinaturasAtuais'>{}</output></body></html>",
        "e".repeat(64),
        escapar(&ultima.html_corpo),
        ultima.versao,
        oferta.assinaturas
    ))
}

/// Simula `editor_salvar` com versão e seção do formulário observado,
/// sem fingir que o POST gravado no acervo contém todos os campos.
fn salvar_editor_4(
    corpo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
) -> Resposta {
    let (Some(versao_base), Some(html_editado), Some("N")) = (
        campo_formulario_unico(corpo, "hdnVersao")
            .and_then(|v| v.parse::<u32>().ok())
            .and_then(NonZeroU32::new),
        campo_formulario_unico(corpo, "txaEditor_1"),
        campo_formulario_unico(corpo, "hdnIgnorarNovaVersao").as_deref(),
    ) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let Ok(Some(documento)) = ler(banco, origem, cenario, id) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let Some(ultima) = documento.revisoes.iter().max_by_key(|r| r.versao) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    match publicar(
        banco,
        origem,
        cenario,
        id,
        Edicao {
            versao_base,
            html_base: &ultima.html_corpo,
            html_editado: &html_editado,
            confirmar_assinaturas: false,
        },
    ) {
        Ok(revisao) => Resposta::html(format!("OK {}", revisao.versao_nova)),
        Err(falha) => resposta_de_falha(&falha),
    }
}

/// GET exibe a oferta observada; POST único publica somente no SQLite local.
/// Nenhum endpoint de instalação externa é chamado.
#[must_use]
pub fn responder(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Resposta {
    let acao = parametro_unico(alvo, "acao");
    let base = origem.url_base();
    let pasta = base
        .strip_prefix(&format!("https://{}", origem.host()))
        .unwrap_or("");
    let destino_editor = format!("{pasta}/editor/editor_processar.php");
    let caminho = alvo.split_once('?').map_or(alvo, |(caminho, _)| caminho);
    let controlador = eh_controlador_sei(alvo);
    if !((controlador && matches!(acao, Some("documento_editar" | "editor_montar")))
        || (caminho == destino_editor && acao == Some("editor_salvar")))
    {
        return despachar(metodo, alvo);
    }
    let Some(id) = parametro_unico(alvo, "id_documento")
        .filter(|valor| valor.bytes().all(|b| b.is_ascii_digit()))
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    match (acao, metodo) {
        (Some("documento_editar"), "GET") => formulario(banco, origem, cenario, id),
        (Some("documento_editar"), "POST") => salvar(corpo, banco, origem, cenario, id),
        (Some("editor_montar"), "GET") => formulario_editor_4(banco, origem, cenario, id),
        (Some("editor_salvar"), "POST") => salvar_editor_4(corpo, banco, origem, cenario, id),
        _ => Resposta::nova(Estado::MetodoNaoPermitido),
    }
}
