//! Saída do que vem só da árvore: documentos (com a natureza), movidos e
//! `aberto_em` (contracts/cli/processo-obter-situacao.md).
use crate::arvore_processo::{
    AbertoEm, ArvoreLida, DocumentoMovido, DocumentoNaArvore, ErroArvore, MotivoAbertoEm,
};
use serde_json::{Map, Value, json};
use std::fmt::Write as _;

/// `aberto_em` (lista ou `null`) e, se não lido, o motivo e a entrada
/// em `nao_lidos`; `sobrestado_em` (lista só na frase do sobrestamento).
pub(super) fn inserir_aberto_em(
    objeto: &mut Map<String, Value>,
    aberto_em: &AbertoEm,
    nao_lidos: &mut Vec<&str>,
) {
    let valor = match aberto_em {
        AbertoEm::Unidades(unidades) => Value::Array(
            unidades
                .iter()
                .map(|u| {
                    json!({
                        "sigla": u.sigla,
                        "nome": u.nome,
                        "atribuido_a": u.atribuido_a.as_ref().map(
                            |p| json!({"sigla": p.sigla, "nome": p.nome}),
                        ),
                    })
                })
                .collect(),
        ),
        AbertoEm::Sobrestado(_) | AbertoEm::NaoLido(_) => {
            let motivo = match aberto_em {
                AbertoEm::NaoLido(motivo) => *motivo,
                AbertoEm::Unidades(_) | AbertoEm::Sobrestado(_) => {
                    MotivoAbertoEm::RaizPublicaOSobrestamento
                }
            };
            nao_lidos.push("aberto_em");
            objeto.insert(
                "aberto_em_nao_lido_motivo".to_owned(),
                json!(motivo.como_texto()),
            );
            Value::Null
        }
    };
    objeto.insert("aberto_em".to_owned(), valor);
    let sobrestado_em = match aberto_em {
        AbertoEm::Sobrestado(unidades) => Value::Array(
            unidades
                .iter()
                .map(|u| json!({"sigla": u.sigla, "nome": u.nome}))
                .collect(),
        ),
        AbertoEm::Unidades(_) | AbertoEm::NaoLido(_) => Value::Null,
    };
    objeto.insert("sobrestado_em".to_owned(), sobrestado_em);
}

/// Uma linha: as unidades, a lista vazia, o sobrestamento ou o motivo.
pub(super) fn humano_do_aberto_em(aberto_em: &AbertoEm, texto: &mut String) {
    match aberto_em {
        AbertoEm::Unidades(unidades) if unidades.is_empty() => {
            texto.push_str("\nSem andamentos abertos.");
        }
        AbertoEm::Unidades(unidades) => {
            let lista: Vec<String> = unidades
                .iter()
                .map(|u| match &u.atribuido_a {
                    Some(p) => format!("{} (atribuído a {})", u.sigla, p.sigla),
                    None => u.sigla.clone(),
                })
                .collect();
            let _ = write!(texto, "\nAberto em: {}.", lista.join(", "));
        }
        AbertoEm::Sobrestado(unidades) => {
            let siglas: Vec<&str> = unidades.iter().map(|u| u.sigla.as_str()).collect();
            let _ = write!(texto, "\nSobrestado em: {}.", siglas.join(", "));
        }
        AbertoEm::NaoLido(motivo) => {
            let _ = write!(texto, "\nAberto em: não lido ({}).", motivo.como_texto());
        }
    }
}

fn json_do_documento(d: &DocumentoNaArvore) -> Value {
    json!({
        "id_documento": d.id,
        "rotulo": d.rotulo,
        "numero_sei": d.numero_sei,
        "pasta": d.pasta,
        "icone": d.natureza.icone,
        "cancelado": d.natureza.cancelamento.cancelado(),
        "motivo_do_cancelamento": d.natureza.cancelamento.motivo(),
        "dica": d.natureza.dica,
    })
}

/// Converte o motivo tipado em aviso legível somente na fronteira da CLI.
fn texto_do_aviso(causa: &ErroArvore) -> String {
    format!("leitura parcial do processo: lista de documentos incompleta ({causa:?})")
}

/// `documentos` e `documentos_movidos`, na ordem dos nós.
pub(super) fn inserir_documentos(objeto: &mut Map<String, Value>, arvore: &ArvoreLida) {
    if !arvore.documentos_incluidos {
        objeto.insert("documentos_incluidos".to_owned(), json!(false));
        objeto.insert("documentos".to_owned(), Value::Array(Vec::new()));
        objeto.insert("documentos_movidos".to_owned(), Value::Array(Vec::new()));
        if let Some(causa) = arvore.aviso_leitura_parcial.clone() {
            objeto.insert(
                "aviso_leitura_parcial".to_owned(),
                json!(texto_do_aviso(&causa)),
            );
        }
        return;
    }
    let documentos: Vec<Value> = arvore.documentos.iter().map(json_do_documento).collect();
    objeto.insert("documentos".to_owned(), Value::Array(documentos));
    let movidos: Vec<Value> = arvore
        .documentos_movidos
        .iter()
        .map(|m| {
            json!({
                "id": m.id,
                "rotulo": m.rotulo,
                "numero_sei": m.numero_sei,
                "destino": m.destino.as_ref().map(
                    |d| json!({"numero": d.numero, "id_procedimento": d.id_procedimento}),
                ),
                "presente_entre_os_documentos": m.presente_entre_os_documentos,
            })
        })
        .collect();
    objeto.insert("documentos_movidos".to_owned(), Value::Array(movidos));
}

/// Uma linha por documento, com a pasta, ou aviso de leitura parcial.
pub(super) fn humano_dos_documentos(arvore: &ArvoreLida, texto: &mut String) {
    if !arvore.documentos_incluidos {
        let aviso = arvore.aviso_leitura_parcial.as_ref().map_or_else(
            || "leitura parcial do processo: lista de documentos incompleta".to_owned(),
            texto_do_aviso,
        );
        let _ = write!(texto, "\nAviso: {aviso}.");
        return;
    }
    for documento in &arvore.documentos {
        let _ = write!(texto, "\n- {} (id {})", documento.rotulo, documento.id);
        let pasta = documento
            .pasta
            .as_ref()
            .and_then(|id| arvore.pastas.iter().find(|p| &p.id == id));
        if let Some(pasta) = pasta {
            let _ = write!(texto, " [pasta {}]", pasta.rotulo);
        }
        let natureza = &documento.natureza;
        match natureza.cancelamento.cancelado() {
            Some(true) => {
                let motivo = natureza.cancelamento.motivo().unwrap_or("");
                let _ = write!(texto, " [cancelado: {motivo}]");
            }
            None if natureza.icone.is_some() => {
                texto.push_str(" [cancelamento não confirmado]");
            }
            _ => {}
        }
        if natureza.cancelamento.cancelado() != Some(true)
            && let Some(dica) = &natureza.dica
        {
            for linha in dica.lines().filter(|l| !l.is_empty()) {
                let _ = write!(texto, "\n    {}", linha.trim());
            }
        }
    }
}

pub(super) fn humano_dos_movidos(movidos: &[DocumentoMovido], texto: &mut String) {
    if movidos.is_empty() {
        return;
    }
    let _ = write!(
        texto,
        "\n{} documento(s) movido(s) para outro processo:",
        movidos.len()
    );
    for m in movidos {
        let _ = write!(texto, "\n- {} (id {}): ", m.rotulo, m.id);
        match &m.destino {
            Some(d) => {
                let _ = write!(texto, "movido para {}, ies processo {}", d.numero, d.numero);
            }
            None => texto.push_str("destino não publicado"),
        }
        if m.presente_entre_os_documentos {
            texto.push_str(" (o documento está de novo neste processo)");
        }
    }
}
