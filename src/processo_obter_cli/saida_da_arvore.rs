//! Saída do que vem só da árvore: documentos (com a natureza), movidos e
//! `aberto_em` (contracts/cli/processo-obter-situacao.md).
use crate::arvore_processo::{AbertoEm, ArvoreLida, DocumentoMovido, DocumentoNaArvore};
use serde_json::{Map, Value, json};
use std::fmt::Write as _;

/// `aberto_em` (lista ou `null`) e, se não lido, o motivo e a entrada
/// em `nao_lidos`.
pub(super) fn inserir_aberto_em(
    _objeto: &mut Map<String, Value>,
    _aberto_em: &AbertoEm,
    _nao_lidos: &mut Vec<&str>,
) {
}

/// Uma linha: as unidades, a lista vazia ou o motivo.
pub(super) fn humano_do_aberto_em(_aberto_em: &AbertoEm, _texto: &mut String) {}

fn json_do_documento(d: &DocumentoNaArvore) -> Value {
    json!({
        "id_documento": d.id,
        "rotulo": d.rotulo,
        "numero_sei": d.numero_sei,
        "pasta": d.pasta,
    })
}

/// `documentos` e `documentos_movidos`, na ordem dos nós.
pub(super) fn inserir_documentos(objeto: &mut Map<String, Value>, arvore: &ArvoreLida) {
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
            })
        })
        .collect();
    objeto.insert("documentos_movidos".to_owned(), Value::Array(movidos));
}

/// Uma linha por documento, com a pasta.
pub(super) fn humano_dos_documentos(arvore: &ArvoreLida, texto: &mut String) {
    for documento in &arvore.documentos {
        let _ = write!(texto, "\n- {} (id {})", documento.rotulo, documento.id);
        let pasta = documento
            .pasta
            .as_ref()
            .and_then(|id| arvore.pastas.iter().find(|p| &p.id == id));
        if let Some(pasta) = pasta {
            let _ = write!(texto, " [pasta {}]", pasta.rotulo);
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
    }
}
