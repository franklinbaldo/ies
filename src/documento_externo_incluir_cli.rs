//! #332 — inspeção do arquivo local antes de abrir sessão/rede.
//! Não apresentar este resultado como prévia do formulário do SEI.
use crate::documento_externo_incluir::{
    contem_controle_de_apresentacao, fluxo::executar_previa, inspecionar_pdf,
};
#[path = "documento_externo_incluir_cli/confirmar.rs"]
mod confirmar_remoto;

use crate::processo_concluir::escrita::Contador;
use crate::processo_concluir_cli::com_sessao;
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Documento, DocumentoAcao, Domain};
use serde_json::json;
use std::path::Path;

/// Tipo externo vazio ou com caractere de controle: erro de uso (saída 2),
/// antes de qualquer arquivo ou requisição.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaTipoExterno {
    Vazio,
    Controle,
}

pub(crate) fn tipo_externo(tipo: &str) -> Result<String, FalhaTipoExterno> {
    if contem_controle_de_apresentacao(tipo) {
        return Err(FalhaTipoExterno::Controle);
    }
    if tipo.trim().is_empty() {
        return Err(FalhaTipoExterno::Vazio);
    }
    Ok(tipo.to_owned())
}

#[allow(clippy::too_many_lines)] // Entrada sequencial: inspeção, prévia e recusa de escrita preservadas.
pub(crate) fn tentar_incluir(cli: &Cli) -> Option<anyhow::Result<()>> {
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
    let processo = processo.clone();
    // Inspeção local antes de qualquer sessão: nenhuma requisição na recusa.
    let pdf = match inspecionar_pdf(Path::new(arquivo), None) {
        Ok(pdf) => pdf,
        Err(falha) => return Some(Err(falha.into())),
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
            let previa = executar_previa(
                sessao,
                &processo,
                cli.unidade.as_deref(),
                tipo,
                &pdf,
                indice,
                &mut contagem,
            )?;
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
            Ok(())
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
    Some(Ok(()))
}
