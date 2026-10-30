//! Projeção offline da resposta documento_imprimir_web já verificada.
use crate::{cassette, documento_web, erro_fonte};
use serde_json::json;
use std::process::ExitCode;

pub(super) fn mostrar_documento_replay(
    card: &std::path::Path,
    zip: &std::path::Path,
    passos: &[usize],
    json_mode: bool,
    texto: bool,
) -> ExitCode {
    match cassette::read_cassette_document(card, zip, passos) {
        Ok((html, instalacao)) => match documento_web::identidade_do_titulo(&html) {
            Ok(documento) => {
                if json_mode {
                    println!(
                        "{}",
                        json!({
                            "documento": documento,
                            "instalacao": instalacao,
                            "escopo": "titulo_documento_offline"
                        })
                    );
                } else {
                    println!(
                        "Documento SEI {}: {} (título de HTML offline)",
                        documento.numero, documento.tipo
                    );
                }
                ExitCode::SUCCESS
            }
            Err(erro) => erro_fonte(
                json_mode,
                erro.subcodigo(),
                "título do documento não identifica número e tipo",
            ),
        },
        Err(_) => erro_fonte(
            json_mode,
            "replay_documento_nao_selecionado",
            "nenhuma resposta única de documento foi selecionada",
        ),
    }
}

