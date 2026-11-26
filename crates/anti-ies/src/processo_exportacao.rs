//! Hipótese de exportação ZIP sobre PDFs externos cadastrados no SQLite (#889).
//! Não simula conversão PDF, seleção, documentos internos nem geração assíncrona.
use crate::{Estado, Resposta, parametro_unico};
use iesei::instalacao::Instalacao;
use iesei::pdf_externo;
use rusqlite::Connection;
use std::io::{Cursor, Write as _};

fn url(acao: &str, id: &str) -> String {
    format!(
        "controlador.php?acao={acao}&id_procedimento={id}&infra_hash={}",
        crate::protocolo_sintetico::INFRA_HASH
    )
}

pub(crate) fn link_da_barra(id: &str) -> String {
    let endereco = crate::escapar(&url("procedimento_gerar_zip", id));
    format!(r#"<a href="{endereco}">Gerar ZIP do processo</a>"#)
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
    // Não publicar ZIP parcial como integral: nós internos, anexados e
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
    match (parametro_unico(alvo, "acao"), metodo) {
        (Some("procedimento_gerar_zip"), "GET") => {
            let action = crate::escapar(&url("procedimento_gerar_zip", id));
            Resposta::html(format!(
                r#"<html><body><form id="frmProcedimentoZip" method="post" action="{action}">
<input type="radio" name="rdoTipo" value="T" checked>
<input type="hidden" name="hdnFlagGerar" value="0">
</form></body></html>"#
            ))
        }
        (Some("procedimento_gerar_zip"), "POST") => {
            if !matches!(
                corpo,
                "rdoTipo=T&hdnFlagGerar=1" | "hdnFlagGerar=1&rdoTipo=T"
            ) {
                return Resposta::nova(Estado::RequisicaoInvalida);
            }
            let download = url("exibir_arquivo", id);
            Resposta::html(format!(
                r#"<html><body><iframe id="ifrDownload"></iframe><script>
document.getElementById('ifrDownload').src = '{download}';
</script></body></html>"#
            ))
        }
        (Some("exibir_arquivo"), "GET") => match zip_do_processo(banco, origem, cenario, &docs) {
            Ok(bytes) => Resposta {
                estado: Estado::Ok,
                tipo_conteudo: "application/zip",
                corpo: String::new(),
                localizacao: None,
                corpo_binario: Some(bytes),
            },
            Err(estado) => Resposta::nova(estado),
        },
        _ => Resposta::nova(Estado::MetodoNaoPermitido),
    }
}
