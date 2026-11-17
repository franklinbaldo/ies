//! Saída do que vem só da árvore: documentos (com a natureza), movidos e
//! `aberto_em` (contracts/cli/processo-obter-situacao.md).
use crate::arvore_processo::{
    AbertoEm, ArvoreLida, DocumentoMovido, DocumentoNaArvore, ErroArvore, MotivoAbertoEm,
};
use serde_json::{Map, Value, json};
use std::fmt::Write as _;

/// Leitura de `aberto_em` que falhou: erro tipado da CLI, em vez de publicar
/// `aberto_em` sem valor.
#[derive(Debug, thiserror::Error)]
#[error("aberto_em não lido ({})", .0.como_texto())]
pub(crate) struct AbertoEmNaoLido(pub(crate) MotivoAbertoEm);

/// Falha se `aberto_em` não foi lido; sem leitura, não há o que publicar.
pub(super) fn exigir_aberto_em_lido(aberto_em: &AbertoEm) -> Result<(), AbertoEmNaoLido> {
    match aberto_em {
        AbertoEm::NaoLido(motivo) => Err(AbertoEmNaoLido(*motivo)),
        AbertoEm::Unidades(_) | AbertoEm::Sobrestado(_) => Ok(()),
    }
}

/// `aberto_em` (lista; a chave sai quando sobrestado ou não lido), e, se não
/// lido, o motivo e a entrada em `nao_lidos`; `sobrestado_em` (lista só na
/// frase do sobrestamento, a chave sai nos outros casos).
pub(super) fn inserir_aberto_em(
    objeto: &mut Map<String, Value>,
    aberto_em: &AbertoEm,
    nao_lidos: &mut Vec<&str>,
) {
    match aberto_em {
        AbertoEm::Unidades(unidades) => {
            let lista: Vec<Value> = unidades
                .iter()
                .map(|u| {
                    let mut unidade = Map::new();
                    unidade.insert("sigla".to_owned(), json!(u.sigla));
                    unidade.insert("nome".to_owned(), json!(u.nome));
                    if let Some(p) = &u.atribuido_a {
                        unidade.insert(
                            "atribuido_a".to_owned(),
                            json!({"sigla": p.sigla, "nome": p.nome}),
                        );
                    }
                    Value::Object(unidade)
                })
                .collect();
            objeto.insert("aberto_em".to_owned(), Value::Array(lista));
        }
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
        }
    }
    if let AbertoEm::Sobrestado(unidades) = aberto_em {
        let lista: Vec<Value> = unidades
            .iter()
            .map(|u| json!({"sigla": u.sigla, "nome": u.nome}))
            .collect();
        objeto.insert("sobrestado_em".to_owned(), Value::Array(lista));
    }
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

/// Campos sem valor por natureza (sem SEI, sem pasta, sem motivo, sem dica)
/// saem da saída: a chave é omitida, nunca `null`.
fn json_do_documento(d: &DocumentoNaArvore) -> Value {
    let mut objeto = Map::new();
    objeto.insert("id_documento".to_owned(), json!(d.id));
    objeto.insert("rotulo".to_owned(), json!(d.rotulo));
    if let Some(numero_sei) = &d.numero_sei {
        objeto.insert("numero_sei".to_owned(), json!(numero_sei));
    }
    if let Some(pasta) = &d.pasta {
        objeto.insert("pasta".to_owned(), json!(pasta));
    }
    objeto.insert("icone".to_owned(), json!(d.natureza.icone));
    objeto.insert(
        "cancelado".to_owned(),
        json!(d.natureza.cancelamento.cancelado()),
    );
    if let Some(motivo) = d.natureza.cancelamento.motivo() {
        objeto.insert("motivo_do_cancelamento".to_owned(), json!(motivo));
    }
    if let Some(dica) = &d.natureza.dica {
        objeto.insert("dica".to_owned(), json!(dica));
    }
    Value::Object(objeto)
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
            let mut movido = Map::new();
            movido.insert("id".to_owned(), json!(m.id));
            movido.insert("rotulo".to_owned(), json!(m.rotulo));
            if let Some(numero_sei) = &m.numero_sei {
                movido.insert("numero_sei".to_owned(), json!(numero_sei));
            }
            if let Some(d) = &m.destino {
                movido.insert(
                    "destino".to_owned(),
                    json!({"numero": d.numero, "id_procedimento": d.id_procedimento}),
                );
            }
            movido.insert(
                "presente_entre_os_documentos".to_owned(),
                json!(m.presente_entre_os_documentos),
            );
            Value::Object(movido)
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
        if let Some(motivo) = natureza.cancelamento.motivo() {
            let _ = write!(texto, " [cancelado: {motivo}]");
        } else if let Some(dica) = &natureza.dica {
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
