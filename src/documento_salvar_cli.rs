//! Prévia offline e confirmação exclusivamente contra o anti-IES de ensaio.
//! O par local é
//! `DOC.vN.html` (a foto) e `DOC.v(N+1).html` (a edição), #207. Offline:
//! não lê índice, configuração nem sessão, e não verifica a base.
use super::{Cli, Documento, DocumentoAcao, Domain, erro_fonte};
use crate::{copia_documento, copia_listagem, documento, documento_versoes};
use serde_json::json;

#[path = "documento_salvar_cli/remoto.rs"]
mod remoto;
use std::process::ExitCode;

fn mostrar_listagem_copias(documento: &documento::NumeroDocumento, json_mode: bool) -> ExitCode {
    let Ok(pasta) = std::env::current_dir() else {
        return erro_fonte(json_mode, "diretório de trabalho não pode ser lido");
    };
    match copia_listagem::listar(&pasta, documento) {
        Ok(copias) => {
            if json_mode {
                println!(
                    "{}",
                    json!({
                        "copias": copias,
                        "escopo": "arquivos_locais",
                        "versao_remota_confirmada": false,
                        "escrita_remota": false
                    })
                );
            } else {
                println!(
                    "IES: {} arquivo(s) local(is); nenhuma versão conferida com o SEI.",
                    copias.len()
                );
                for copia in &copias {
                    println!(
                        "  {} v{}: {} bytes",
                        copia.documento, copia.versao, copia.bytes
                    );
                }
            }
            ExitCode::SUCCESS
        }
        Err(_) => erro_fonte(json_mode, "listagem local de cópias não foi verificada"),
    }
}

pub(super) fn tentar_salvar_documento(cli: &Cli) -> Option<anyhow::Result<()>> {
    let Domain::Documento(Documento {
        acao:
            Some(DocumentoAcao::Salvar {
                numero,
                versao,
                confirmar,
                listar_copias,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    let documento = numero.clone();
    if *listar_copias {
        return Some(crate::legado(mostrar_listagem_copias(&documento, cli.json)));
    }
    let Ok(versao) = versao
        .map(documento_versoes::VersaoDocumento::nova)
        .transpose()
    else {
        return Some(crate::legado(erro_fonte(cli.json, "versão inválida")));
    };
    let previa = copia_documento::previa_local(&std::env::current_dir().ok()?, &documento, versao);
    match previa {
        Ok(dados) if !confirmar => {
            if cli.json {
                println!(
                    "{}",
                    json!({
                        "previa": dados,
                        "confirmado": false,
                        "assinaturas_remotas": null,
                        "aviso_assinaturas": "editar pode remover assinaturas; a oferta do editor ainda não foi verificada"
                    })
                );
            } else {
                println!(
                    "IES: {} v{} editada sobre a v{} local ({} → {} bytes; base não verificada; sem escrita no SEI)",
                    dados.documento,
                    dados.versao_editada,
                    dados.versao_base,
                    dados.bytes_base,
                    dados.bytes_editada,
                );
                println!(
                    "Diferenças locais (não comprovam alteração remota):\n{}",
                    dados.diff_humano
                );
                println!(
                    "Aviso: editar pode remover assinaturas; a oferta de edição do SEI ainda não foi verificada."
                );
            }
            Some(Ok(()))
        }
        Ok(dados) => Some(remoto::confirmar(cli, &documento, &dados)),
        Err(_) => Some(crate::legado(erro_fonte(
            cli.json,
            "cópia local não pode ser conferida",
        ))),
    }
}

pub(super) fn validar_documentos(numeros: &[String], json_mode: bool) -> Option<ExitCode> {
    for numero in numeros {
        if let Err(erro) = documento::NumeroDocumento::novo(numero) {
            if json_mode {
                println!(
                    "{}",
                    json!({
                        "codigo": "entrada_invalida",
                        "erro": "número SEI de documento inválido",
                        "retentavel": false
                    })
                );
            } else {
                eprintln!("IES: número SEI de documento inválido ({erro:?})");
            }
            return Some(ExitCode::from(2));
        }
    }
    None
}

pub(super) fn tentar_validar_versao(cli: &Cli) -> Option<ExitCode> {
    if let Domain::Documento(Documento {
        acao: None,
        versao: Some(versao),
        ..
    }) = &cli.command
        && let Err(_) = documento_versoes::VersaoDocumento::nova(*versao)
    {
        return Some(erro_fonte(cli.json, "versão do documento inválida"));
    }
    None
}

#[cfg(test)]
mod testes_listagem {
    use super::*;
    use clap::Parser;

    #[test]
    fn opcoes_de_listagem_local_nao_permitem_confirmacao_nem_versao_especifica() {
        assert!(
            Cli::try_parse_from(["ies", "documento", "salvar", "0007", "--listar-copias",]).is_ok()
        );
        assert!(
            Cli::try_parse_from([
                "ies",
                "documento",
                "salvar",
                "0007",
                "--listar-copias",
                "--versao",
                "2",
            ])
            .is_err()
        );
        assert!(
            Cli::try_parse_from([
                "ies",
                "documento",
                "salvar",
                "0007",
                "--listar-copias",
                "--confirmar",
            ])
            .is_err()
        );
    }
}
