//! #332 — inspeção do arquivo local antes de abrir sessão/rede.
//! Não apresentar este resultado como prévia do formulário do SEI.
use crate::documento_externo_incluir::{FalhaPdf, contem_controle_de_apresentacao, inspecionar_pdf};
use crate::processo::{NumeroProcesso, NumeroProcessoInvalido};
use crate::{Cli, Documento, DocumentoAcao, Domain};
use serde::Serialize;
use serde_json::json;
use std::path::Path;
use std::process::ExitCode;

#[derive(Debug, Serialize)]
enum FalhaTipoExterno {
    Vazio,
    Controle,
}

#[derive(Debug, Serialize)]
enum FalhaInclusaoExterna {
    Processo(NumeroProcessoInvalido),
    Tipo(FalhaTipoExterno),
    Pdf(FalhaPdf),
    CadastroNaoValidado,
}

fn validar_tipo(tipo: &str) -> Result<(), FalhaTipoExterno> {
    if contem_controle_de_apresentacao(tipo) {
        return Err(FalhaTipoExterno::Controle);
    }
    if tipo.trim().is_empty() {
        return Err(FalhaTipoExterno::Vazio);
    }
    Ok(())
}

fn recusa(json_mode: bool, causa: &FalhaInclusaoExterna, mensagem: &str, status: u8) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "causa": causa,
                "retentavel": false,
                "escrita_remota": false
            })
        );
    } else {
        eprintln!("IES: {mensagem} ({causa:?}). Nenhuma requisição foi enviada.");
    }
    ExitCode::from(status)
}

pub(crate) fn tentar_incluir(cli: &Cli) -> Option<ExitCode> {
    let Domain::Documento(Documento {
        acao:
            Some(DocumentoAcao::IncluirExterno {
                processo,
                arquivo,
                tipo,
                confirmar,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    let processo = match NumeroProcesso::novo(processo) {
        Ok(processo) => processo,
        Err(falha) => {
            return Some(recusa(
                cli.json,
                &FalhaInclusaoExterna::Processo(falha),
                "número do processo não aceito",
                2,
            ));
        }
    };
    if let Err(falha) = validar_tipo(tipo) {
        return Some(recusa(
            cli.json,
            &FalhaInclusaoExterna::Tipo(falha),
            "tipo externo vazio ou com caractere de controle",
            2,
        ));
    }
    let pdf = match inspecionar_pdf(Path::new(arquivo), None) {
        Ok(pdf) => pdf,
        Err(falha) => {
            return Some(recusa(
                cli.json,
                &FalhaInclusaoExterna::Pdf(falha),
                "arquivo PDF local não passou na inspeção",
                2,
            ));
        }
    };
    if *confirmar {
        return Some(recusa(
            cli.json,
            &FalhaInclusaoExterna::CadastroNaoValidado,
            "confirmação fechada antes da sessão e da rede: faltam formulário, upload e releitura observados",
            1,
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
            pdf.nome,
            pdf.tamanho_bytes,
            pdf.sha256,
            processo.as_str(),
            tipo
        );
    }
    Some(ExitCode::SUCCESS)
}
