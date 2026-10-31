//! Prévia segura: nunca executa confirmação Web.
use super::{Cli, Documento, DocumentoAcao, Domain, erro_fonte};
use crate::{copia_documento, documento, documento_versoes};
use serde_json::json;
use std::process::ExitCode;

pub(super) fn tentar_salvar_documento(cli: &Cli) -> Option<ExitCode> {
    let Domain::Documento(Documento {
        acao:
            Some(DocumentoAcao::Salvar {
                numero,
                versao,
                confirmar,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    let documento = match documento::NumeroDocumento::novo(numero) {
        Ok(documento) => documento,
        Err(erro) => {
            return Some(erro_fonte(
                cli.json,
                erro.subcodigo(),
                "número SEI inválido",
            ));
        }
    };
    let versao = match versao
        .map(documento_versoes::VersaoDocumento::nova)
        .transpose()
    {
        Ok(versao) => versao,
        Err(erro) => return Some(erro_fonte(cli.json, erro.subcodigo(), "versão inválida")),
    };
    let previa = copia_documento::previa_local(&std::env::current_dir().ok()?, &documento, versao);
    match previa {
        Ok(dados) if !confirmar => {
            if cli.json {
                println!("{}", json!({"previa": dados, "confirmado": false}));
            } else {
                println!(
                    "IES: cópia local {} v{}: {} (sem escrita no SEI)",
                    dados.documento,
                    dados.versao_base,
                    if dados.alterado {
                        "modificada"
                    } else {
                        "inalterada"
                    }
                );
            }
            Some(ExitCode::SUCCESS)
        }
        Ok(_) => Some(erro_fonte(
            cli.json,
            "escrita_remota_nao_validada",
            "confirmação recusada: faltam identidade e concorrência Web verificadas",
        )),
        Err(erro) => Some(erro_fonte(
            cli.json,
            erro.subcodigo(),
            "cópia local não pode ser conferida",
        )),
    }
}
