//! Editor de documentos sintéticos sobre o mesmo estado do histórico (#89).
//! Hipótese de HTML/formulário para integração IES ↔ anti-IES; não é
//! evidência de formato real do editor SEI 4/5.
use crate::{
    Estado, Resposta, despachar, documentos::campo_formulario_unico, eh_controlador_sei, escapar,
    parametro_unico,
};
use iesei::documento_versoes::ler;
use iesei::editor_documento::{
    Edicao, ErroEditorDocumento, consultar_oferta, ler_secoes, publicar, publicar_secoes,
};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::fmt::Write as _;
use std::num::NonZeroU32;

fn resposta_de_falha(falha: &ErroEditorDocumento) -> Resposta {
    let estado = match falha {
        ErroEditorDocumento::NaoObservado | ErroEditorDocumento::Sqlite(_) => {
            Estado::DadosInsuficientes
        }
        ErroEditorDocumento::NaoOferecido => Estado::MetodoNaoPermitido,
        ErroEditorDocumento::VersaoDivergente
        | ErroEditorDocumento::ConteudoDivergente
        | ErroEditorDocumento::AssinaturasNaoConfirmadas
        | ErroEditorDocumento::DadoInvalido => Estado::RequisicaoInvalida,
    };
    Resposta::nova(estado)
}

/// Consulta AJAX do editor nativo: a oferta persistida distingue zero
/// assinaturas de estado desconhecido. A resposta é sintética, sem rede.
pub(crate) fn consulta_assinatura(
    metodo: &str,
    alvo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Resposta {
    let caminho = alvo.split_once('?').map_or(alvo, |(caminho, _)| caminho);
    if !caminho.ends_with("/controlador_ajax.php") {
        return Resposta::nova(Estado::NaoEncontrado);
    }
    if metodo != "GET" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Some(id) =
        parametro_unico(alvo, "id_documento").filter(|id| id.bytes().all(|b| b.is_ascii_digit()))
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let oferta = match consultar_oferta(banco, origem, cenario, id) {
        Ok(Some(oferta)) => oferta,
        Ok(None) => return Resposta::nova(Estado::DadosInsuficientes),
        Err(falha) => return resposta_de_falha(&falha),
    };
    let sinal = if oferta.assinaturas == 0 { "N" } else { "S" };
    Resposta {
        estado: Estado::Ok,
        tipo_conteudo: "text/xml; charset=utf-8",
        corpo: format!(
            "<resposta><complemento nome='SinAssinado'>{sinal}</complemento></resposta>"
        ),
        localizacao: None,
        corpo_binario: None,
    }
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
        crate::protocolo_sintetico::INFRA_HASH,
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
    let Ok(secoes) = ler_secoes(banco, origem, cenario, id, ultima.versao) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let mut campos = String::new();
    let mut editores = String::new();
    if secoes.is_empty() {
        campos = format!(
            "<textarea name='txaEditor_1'>{}</textarea>",
            escapar(&ultima.html_corpo)
        );
        editores = "CKEDITOR.replace('txaEditor_1', {\"readOnly\":false});".into();
    } else {
        for secao in secoes {
            write!(
                campos,
                "<textarea name='{}'>{}</textarea>",
                secao.nome,
                escapar(&secao.html)
            )
            .expect("formatação String");
            write!(
                editores,
                "CKEDITOR.replace('{}', {{\"readOnly\":{}}});",
                secao.nome, !secao.principal
            )
            .expect("formatação String");
        }
    }
    Resposta::html(format!(
        "<html><body><form id='frmEditor' hidden method='post' target='ifrEditorSalvar' \
         action='editor/editor_processar.php?acao=editor_salvar&amp;id_procedimento={}&amp;id_documento={id}&amp;infra_hash={}'>\
         {campos}\
         <input type='hidden' id='hdnVersao' name='hdnVersao' value='{}'>\
         <input type='hidden' id='hdnIgnorarNovaVersao' name='hdnIgnorarNovaVersao' value='N'>\
         <label><input type='checkbox' name='confirmar_assinaturas' value='1'>Confirmar perda de assinaturas (anti-IES)</label>\
         </form><iframe id='ifrEditorSalvar' name='ifrEditorSalvar'></iframe>\
         <output id='assinaturasAtuais'>{}</output>\
         <script>{editores}</script></body></html>",
        escapar(&documento.id_procedimento),
        crate::protocolo_sintetico::INFRA_HASH,
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
    procedimento: Option<&str>,
) -> Resposta {
    let (Some(versao_base), Some("N")) = (
        campo_formulario_unico(corpo, "hdnVersao")
            .and_then(|v| v.parse::<u32>().ok())
            .and_then(NonZeroU32::new),
        campo_formulario_unico(corpo, "hdnIgnorarNovaVersao").as_deref(),
    ) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let confirmar_assinaturas = match campo_formulario_unico(corpo, "confirmar_assinaturas") {
        None => false,
        Some(texto) if texto == "1" => true,
        Some(texto) if texto == "0" => false,
        _ => return Resposta::nova(Estado::RequisicaoInvalida),
    };
    let Ok(Some(documento)) = ler(banco, origem, cenario, id) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    if procedimento != Some(documento.id_procedimento.as_str()) {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    let Some(ultima) = documento.revisoes.iter().max_by_key(|r| r.versao) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let Ok(mut secoes) = ler_secoes(banco, origem, cenario, id, ultima.versao) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let enviados = corpo
        .split('&')
        .filter_map(|parte| parte.split_once('='))
        .filter(|(nome, _)| nome.starts_with("txaEditor_"))
        .count();
    if enviados != secoes.len().max(1) {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    for secao in &mut secoes {
        let Some(html) = campo_formulario_unico(corpo, &secao.nome) else {
            return Resposta::nova(Estado::RequisicaoInvalida);
        };
        secao.html = html;
    }
    let html_editado = if secoes.is_empty() {
        campo_formulario_unico(corpo, "txaEditor_1")
    } else {
        secoes.iter().find(|s| s.principal).map(|s| s.html.clone())
    };
    let Some(html_editado) = html_editado else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let edicao = Edicao {
        versao_base,
        html_base: &ultima.html_corpo,
        html_editado: &html_editado,
        confirmar_assinaturas,
    };
    let resultado = if secoes.is_empty() {
        publicar(banco, origem, cenario, id, edicao)
    } else {
        publicar_secoes(banco, origem, cenario, id, edicao, &secoes)
    };
    match resultado {
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
        (Some("editor_salvar"), "POST") => salvar_editor_4(
            corpo,
            banco,
            origem,
            cenario,
            id,
            parametro_unico(alvo, "id_procedimento"),
        ),
        _ => Resposta::nova(Estado::MetodoNaoPermitido),
    }
}

#[cfg(test)]
mod testes_assinatura {
    use super::*;
    use iesei::documento_versoes::{DocumentoSimulado, Revisao, guardar};
    use iesei::editor_documento::{Oferta, registrar_oferta};

    #[test]
    fn consulta_ajax_do_editor_le_assinaturas_sem_alterar_revisoes() {
        let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
        let mut banco = Connection::open_in_memory().unwrap();
        iesei::banco_cassettes::iniciar(&banco).unwrap();
        let documento = DocumentoSimulado {
            instalacao: origem.clone(),
            id_documento: "21".into(),
            id_procedimento: "10".into(),
            numero_processo: "00001.000001/2026-01".into(),
            numero: "0000101".into(),
            tipo: "Despacho".into(),
            revisoes: vec![Revisao {
                versao: NonZeroU32::new(1).unwrap(),
                usuario: "Teste".into(),
                unidade: "TESTE".into(),
                modificacao: "26/11/2026".into(),
                html_corpo: "<p>Antes</p>".into(),
            }],
        };
        guardar(&mut banco, "ensaio", &documento).unwrap();
        let alvo =
            "/sei/controlador_ajax.php?acao_ajax=documento_verificar_assinatura&id_documento=21";
        let pedir = |metodo, alvo, cenario| {
            crate::rotas::responder(metodo, alvo, b"", &banco, &origem, cenario)
        };
        assert_eq!(
            pedir("GET", alvo, "ensaio").estado,
            Estado::DadosInsuficientes
        );
        for (assinaturas, sinal) in [(0, "N"), (2, "S")] {
            registrar_oferta(
                &banco,
                &origem,
                "ensaio",
                "21",
                Oferta {
                    editavel: true,
                    assinaturas,
                },
            )
            .unwrap();
            let resposta = pedir("GET", alvo, "ensaio");
            assert_eq!(resposta.estado, Estado::Ok);
            assert_eq!(resposta.tipo_conteudo, "text/xml; charset=utf-8");
            assert_eq!(
                resposta.corpo,
                format!(
                    "<resposta><complemento nome='SinAssinado'>{sinal}</complemento></resposta>"
                )
            );
            assert_eq!(
                ler(&banco, &origem, "ensaio", "21").unwrap(),
                Some(documento.clone())
            );
        }
        assert_eq!(
            pedir("GET", alvo, "outro").estado,
            Estado::DadosInsuficientes
        );
        let alvo_ambiguo = format!("{alvo}&id_documento=22");
        assert_eq!(
            pedir("GET", &alvo_ambiguo, "ensaio").estado,
            Estado::RequisicaoInvalida
        );
        assert_eq!(
            pedir("POST", alvo, "ensaio").estado,
            Estado::MetodoNaoPermitido
        );
    }
}
