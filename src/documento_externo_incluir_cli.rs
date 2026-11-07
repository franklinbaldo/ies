//! #332 — inspeção do arquivo local antes de abrir sessão/rede.
//! Não apresentar este resultado como prévia do formulário do SEI.
use crate::documento_externo_incluir::inspecionar_pdf;
use crate::processo::NumeroProcesso;
use crate::{Cli, Documento, DocumentoAcao, Domain};
use serde_json::json;
use std::path::Path;
use std::process::ExitCode;

fn recusa(json_mode: bool, subcodigo: &str, mensagem: &str) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "entrada_invalida",
                "subcodigo": subcodigo,
                "erro": mensagem,
                "retentavel": false,
                "escrita_remota": false
            })
        );
    } else {
        eprintln!("IES: {mensagem} [{subcodigo}]. Nenhuma requisição foi enviada.");
    }
    ExitCode::from(2)
}

pub(crate) fn tentar_incluir(cli: &Cli) -> Option<ExitCode> {
    let Domain::Documento(Documento {
        acao: Some(DocumentoAcao::IncluirExterno {
            processo,
            arquivo,
            tipo,
            confirmar,
        }),
        ..
    }) = &cli.command else {
        return None;
    };
    let processo = match NumeroProcesso::novo(processo) {
        Ok(processo) => processo,
        Err(falha) => {
            return Some(recusa(
                cli.json,
                falha.subcodigo(),
                "número do processo inválido",
            ));
        }
    };
    if tipo.trim().is_empty() || tipo.chars().any(char::is_control) {
        return Some(recusa(
            cli.json,
            "tipo_externo_invalido",
            "tipo externo vazio ou com caractere de controle",
        ));
    }
    let pdf = match inspecionar_pdf(Path::new(arquivo), None) {
        Ok(pdf) => pdf,
        Err(falha) => {
            return Some(recusa(
                cli.json,
                falha.subcodigo(),
                "arquivo PDF local não passou na inspeção",
            ));
        }
    };
    if *confirmar {
        return Some(crate::erro_fonte(
            cli.json,
            "cadastro_externo_ainda_nao_validado",
            "confirmação fechada antes da sessão e da rede: faltam formulário, upload e releitura observados",
        ));
    }
    if cli.json {
        println!(
            "{}",
            json!({
                "escopo": "inspecao_local",
                "processo": processo.as_str(),
                "tipo_solicitado": tipo,
                "arquivo": {
                    "nome": pdf.nome,
                    "tamanho_bytes": pdf.tamanho_bytes,
                    "sha256": pdf.sha256,
                },
                "cadastro_sei_observado": false,
                "previa_remota": false,
                "confirmado": false,
                "requisicoes": 0,
                "escrita_remota": false
            })
        );
    } else {
        println!(
            "IES: arquivo local {} ({} bytes, SHA-256 {}) inspecionado para {}. Tipo solicitado: {}. Nenhuma tela do SEI foi consultada; não é prévia remota e nenhuma escrita está habilitada.",
            pdf.nome, pdf.tamanho_bytes, pdf.sha256, processo.as_str(), tipo
        );
    }
    Some(ExitCode::SUCCESS)
}