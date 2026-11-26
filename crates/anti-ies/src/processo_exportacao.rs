//! Hipótese de exportação sobre PDFs externos cadastrados no SQLite (#889).
//! PDF de anexo único preserva bytes; ZIP admite seleção de anexos.
//! Não simula composição PDF, documentos internos nem geração assíncrona.
use crate::{Estado, Resposta, parametro_unico};
use iesei::instalacao::Instalacao;
use iesei::pdf_externo;
use rusqlite::Connection;
use std::fmt::Write as _;
use std::io::{Cursor, Write as _};

fn url(acao: &str, id: &str) -> String {
    format!(
        "controlador.php?acao={acao}&id_procedimento={id}&infra_hash={}",
        crate::protocolo_sintetico::INFRA_HASH
    )
}

pub(crate) fn links_da_barra(id: &str, anexo_unico: bool) -> String {
    let zip = crate::escapar(&url("procedimento_gerar_zip", id));
    let mut links = format!(r#"<a href="{zip}">Gerar ZIP do processo</a>"#);
    if anexo_unico {
        let pdf = crate::escapar(&url("procedimento_gerar_pdf", id));
        let _ = write!(links, r#"<a href="{pdf}">Gerar PDF do processo</a>"#);
    }
    links
}

fn documentos(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
) -> Result<Vec<pdf_externo::Documento>, Estado> {
    if pdf_externo::processo(banco, origem, cenario, id)
        .map_err(|_| Estado::DadosInsuficientes)?
        .is_none()
    {
        return Err(Estado::NaoEncontrado);
    }
    // Não publicar arquivo parcial como integral: nós internos, anexados e
    // pastas pendentes não pertencem à hipótese desta fatia.
    let (_, proximo) = crate::processo_envio::nos_e_proximo_indice(banco, origem, cenario, id)?;
    if proximo != 1 {
        return Err(Estado::NaoImplementado);
    }
    let docs = pdf_externo::documentos(banco, origem, cenario, id)
        .map_err(|_| Estado::DadosInsuficientes)?;
    if docs.is_empty() {
        return Err(Estado::DadosInsuficientes);
    }
    Ok(docs)
}

fn zip_do_processo(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    docs: &[pdf_externo::Documento],
) -> Result<Vec<u8>, Estado> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let opcoes =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for doc in docs {
        let bytes = pdf_externo::conteudo_do_anexo(banco, origem, cenario, &doc.id.to_string())
            .map_err(|_| Estado::DadosInsuficientes)?
            .ok_or(Estado::DadosInsuficientes)?;
        zip.start_file(format!("{}.pdf", doc.numero_sei), opcoes)
            .map_err(|_| Estado::DadosInsuficientes)?;
        zip.write_all(&bytes)
            .map_err(|_| Estado::DadosInsuficientes)?;
    }
    Ok(zip
        .finish()
        .map_err(|_| Estado::DadosInsuficientes)?
        .into_inner())
}

fn gerar(corpo: &str, formato: &str, id: &str, docs: &[pdf_externo::Documento]) -> Resposta {
    let Some(pares): Option<Vec<_>> = corpo.split('&').map(|p| p.split_once('=')).collect() else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let campo = |nome| crate::processo_envio::campo_unico(&pares, nome);
    if campo("hdnFlagGerar") != Some("1") {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    let mut download = format!("{}&formato={formato}", url("exibir_arquivo", id));
    match campo("rdoTipo") {
        Some("T")
            if pares.len() == 2
                || (pares.len() == 3 && campo("hdnDocumentosApenas") == Some("")) => {}
        Some("A") if formato == "zip" && pares.len() == 3 => {
            let Some(valor) = campo("hdnDocumentosApenas")
                .and_then(crate::processo_envio::decodificar_formulario)
            else {
                return Resposta::nova(Estado::RequisicaoInvalida);
            };
            let mut ids = Vec::new();
            for item in valor.split('¥') {
                let Some((id, titulo)) = item.split_once('±') else {
                    return Resposta::nova(Estado::RequisicaoInvalida);
                };
                if ids.contains(&id)
                    || !docs.iter().any(|doc| {
                        doc.id_publico() == id
                            && titulo == format!("{} - {} ", doc.numero_sei, doc.tipo)
                    })
                {
                    return Resposta::nova(Estado::RequisicaoInvalida);
                }
                ids.push(id);
            }
            let _ = write!(download, "&documentos={}", ids.join(","));
        }
        _ => return Resposta::nova(Estado::RequisicaoInvalida),
    }
    Resposta::html(format!(
        r#"<html><body><iframe id="ifrDownload"></iframe><script>
document.getElementById('ifrDownload').src = '{download}';
</script></body></html>"#
    ))
}

pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Resposta {
    let Some(id) = parametro_unico(alvo, "id_procedimento")
        .filter(|id| id.bytes().all(|b| b.is_ascii_digit()))
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let docs = match documentos(banco, origem, cenario, id) {
        Ok(docs) => docs,
        Err(estado) => return Resposta::nova(estado),
    };
    let acao = parametro_unico(alvo, "acao");
    let formato = match acao {
        Some("procedimento_gerar_pdf") => "pdf",
        Some("procedimento_gerar_zip") => "zip",
        Some("exibir_arquivo") => match parametro_unico(alvo, "formato") {
            Some("pdf") => "pdf",
            Some("zip") => "zip",
            _ => return Resposta::nova(Estado::RequisicaoInvalida),
        },
        _ => return Resposta::nova(Estado::MetodoNaoPermitido),
    };
    if formato == "pdf" && docs.len() != 1 {
        return Resposta::nova(Estado::NaoImplementado);
    }
    match (acao, metodo) {
        (Some(gerar @ ("procedimento_gerar_zip" | "procedimento_gerar_pdf")), "GET") => {
            let action = crate::escapar(&url(gerar, id));
            let form = if formato == "pdf" {
                "frmProcedimentoPdf"
            } else {
                "frmProcedimentoZip"
            };
            let selecao = if formato == "zip" {
                r#"<input type="radio" name="rdoTipo" value="A">
<input type="hidden" name="hdnDocumentosApenas" value="">
<select name="selDocumentosApenas" multiple></select>"#
            } else {
                ""
            };
            Resposta::html(format!(
                r#"<html><body><form id="{form}" method="post" action="{action}">
<input type="radio" name="rdoTipo" value="T" checked>
<input type="hidden" name="hdnFlagGerar" value="0">
{selecao}
</form></body></html>"#
            ))
        }
        (Some("procedimento_gerar_zip" | "procedimento_gerar_pdf"), "POST") => {
            gerar(corpo, formato, id, &docs)
        }
        (Some("exibir_arquivo"), "GET") if formato == "pdf" => {
            let [doc] = docs.as_slice() else {
                return Resposta::nova(Estado::NaoImplementado);
            };
            match pdf_externo::conteudo_do_anexo(banco, origem, cenario, &doc.id.to_string()) {
                Ok(Some(bytes)) => Resposta {
                    estado: Estado::Ok,
                    tipo_conteudo: "application/pdf",
                    corpo: String::new(),
                    localizacao: None,
                    corpo_binario: Some(bytes),
                },
                _ => Resposta::nova(Estado::DadosInsuficientes),
            }
        }
        (Some("exibir_arquivo"), "GET") => {
            let docs = if let Some(ids) = parametro_unico(alvo, "documentos") {
                let mut selecionados = Vec::new();
                for id in ids.split(',') {
                    let Some(doc) = docs.iter().find(|doc| doc.id_publico() == id) else {
                        return Resposta::nova(Estado::RequisicaoInvalida);
                    };
                    if selecionados
                        .iter()
                        .any(|d: &pdf_externo::Documento| d.id == doc.id)
                    {
                        return Resposta::nova(Estado::RequisicaoInvalida);
                    }
                    selecionados.push(doc.clone());
                }
                selecionados
            } else {
                docs
            };
            match zip_do_processo(banco, origem, cenario, &docs) {
                Ok(bytes) => Resposta {
                    estado: Estado::Ok,
                    tipo_conteudo: "application/zip",
                    corpo: String::new(),
                    localizacao: None,
                    corpo_binario: Some(bytes),
                },
                Err(estado) => Resposta::nova(estado),
            }
        }
        _ => Resposta::nova(Estado::MetodoNaoPermitido),
    }
}
