//! #332 — inspeção do arquivo local antes de abrir sessão/rede.
//! Não apresentar este resultado como prévia do formulário do SEI.
use crate::documento_externo_incluir::{
    FalhaPdf, contem_controle_de_apresentacao, fluxo::executar_previa, inspecionar_pdf,
};
#[path = "documento_externo_incluir_cli/confirmar.rs"]
mod confirmar_remoto;

use crate::processo::{NumeroProcesso, NumeroProcessoInvalido};
use crate::processo_concluir::escrita::Contador;
use crate::processo_concluir_cli::com_sessao;
use crate::visao_do_controle::EnvioHttps;
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
                "requisicoes": 0,
                "escrita_remota": false
            })
        );
    } else {
        eprintln!("IES: {mensagem} ({causa:?}). Nenhuma requisição foi enviada.");
    }
    ExitCode::from(status)
}

#[allow(clippy::too_many_lines)] // Entrada sequencial: inspeção, prévia e recusa de escrita preservadas.
pub(crate) fn tentar_incluir(cli: &Cli) -> Option<ExitCode> {
    let Domain::Documento(Documento {
        acao:
            Some(DocumentoAcao::IncluirExterno {
                processo,
                arquivo,
                tipo,
                confirmar,
                consultar_sei,
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
        return Some(confirmar_remoto::executar(
            cli,
            &processo,
            tipo,
            &pdf,
            Path::new(arquivo),
        ));
    }
    if *consultar_sei {
        return Some(com_sessao(cli, |sessao, indice| {
            let mut envio = EnvioHttps;
            let mut contagem = Contador::novo(&mut envio);
            match executar_previa(
                sessao,
                &processo,
                cli.unidade.as_deref(),
                tipo,
                &pdf,
                indice,
                &mut contagem,
            ) {
                Ok(previa) => {
                    if cli.json {
                        println!(
                            "{}",
                            json!({
                                "escopo": "previa_web",
                                "processo": processo.as_str(),
                                "tipo_solicitado": tipo,
                                "id_tipo": previa.id_tipo,
                                "unidade": previa.unidade,
                                "arquivo": {
                                    "nome": pdf.nome,
                                    "tamanho_bytes": pdf.tamanho_bytes,
                                    "sha256": pdf.sha256,
                                },
                                "limite_pdf_bytes": previa.limite_pdf_bytes,
                                "campos_oferecidos": previa.campos_oferecidos,
                                "cadastro_sei_observado": true,
                                "previa_remota": true,
                                "confirmado": false,
                                "requisicoes": contagem.feitas(),
                                "escrita_remota": false
                            })
                        );
                    } else {
                        println!(
                            "Prévia Web: incluir PDF {} ({} bytes, SHA-256 {}) como {} no processo {} (unidade {}). Limite do SEI: {} bytes. Nenhum arquivo foi enviado e nenhum documento foi cadastrado.",
                            pdf.nome,
                            pdf.tamanho_bytes,
                            pdf.sha256,
                            tipo,
                            processo.as_str(),
                            previa.unidade,
                            previa.limite_pdf_bytes,
                        );
                        println!(
                            "Campos oferecidos no cadastro: {}",
                            if previa.campos_oferecidos.is_empty() {
                                "(nenhum)".to_owned()
                            } else {
                                previa.campos_oferecidos.join(", ")
                            }
                        );
                        println!("Requisições: {}", contagem.feitas());
                    }
                    ExitCode::SUCCESS
                }
                Err(erro) => {
                    if cli.json {
                        let status = erro.status_http();
                        println!(
                            "{}",
                            json!({
                                "causa": erro.causa(),
                                "status_http": status.map(|e| e.status),
                                "mensagem_do_servidor": status.and_then(|e| e.mensagem_do_servidor.as_deref()),
                                "requisicoes": contagem.feitas(),
                                "retentavel": false,
                                "escrita_remota": false
                            })
                        );
                    } else {
                        eprintln!(
                            "IES: prévia Web não comprovada ({:?}). Requisições: {}. Nenhum upload ou cadastro enviado.",
                            erro.causa(),
                            contagem.feitas()
                        );
                    }
                    ExitCode::from(1)
                }
            }
        }));
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
