//! #332 — conecta a confirmação única do PDF à saída pública.
//! Validada ao vivo na `UnB` de treinamento (09/11/2026, saída 0).
use crate::Cli;
use crate::documento_externo_incluir::{PdfLocal, fluxo_confirmar::executar_confirmacao_https};
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::Contador;
use crate::processo_concluir_cli::com_sessao;
use crate::visao_do_controle::EnvioHttps;
use serde_json::json;
use std::path::{Path, PathBuf};
use super::arquivo_json;
use crate::documento_externo_incluir::fluxo_confirmar::prevalidar_confirmacao;

/// Os contadores incluem as tentativas de rede que falharam. O multipart
/// usa o mesmo cliente HTTPS, mas não passa pelo Contador dos GET/POST comuns.
pub(super) fn executar(
    cli: &Cli,
    processo: &NumeroProcesso,
    tipo: &str,
    pdf: &PdfLocal,
    caminho_pdf: &Path,
) -> anyhow::Result<()> {
    com_sessao(cli, |sessao, indice| {
        // Aprovada em treinamento (09/11/2026): grava também em produção.
        if let Err(causa) = crate::origem_de_treinamento::exigir(
            crate::origem_de_treinamento::Escrita::DocumentoIncluirExterno,
            sessao.origem,
        ) {
            return Err(causa.into());
        }
        let mut envio = EnvioHttps;
        let mut contagem = Contador::novo(&mut envio);
        let mut upload_tentado = false;
        let resultado = executar_confirmacao_https(
            sessao,
            processo,
            cli.unidade.as_deref(),
            tipo,
            pdf,
            caminho_pdf,
            indice,
            &mut contagem,
            &mut upload_tentado,
        );
        let requisicoes = contagem.feitas().saturating_add(u8::from(upload_tentado));
        let feito = resultado.map_err(|erro| {
            anyhow::Error::from(erro).context(TentativaNaoComprovada {
                upload_tentado,
                requisicoes,
            })
        })?;
        {
            if cli.json {
                let mut valor = json!({
                    "escopo": "confirmacao_web",
                    "processo": processo.as_str(),
                    "tipo_solicitado": tipo,
                    "numero_sei": feito.numero_sei,
                    "unidade": feito.unidade,
                    "confirmado": true,
                    "escrita_remota": true,
                    "upload_tentado": true,
                    "cadastro_tentado": true,
                    "reconciliado_por_leitura": true,
                    "requisicoes": requisicoes
                });
                crate::indice_local::anexar_avisos(&mut valor, indice);
                println!("{valor}");
            } else {
                println!(
                    "Documento externo {} incluído no processo {} (unidade {}), comprovado por releitura independente. Upload temporário: 1 tentativa; cadastro definitivo: 1 tentativa. Requisições: {}.",
                    feito.numero_sei,
                    processo.as_str(),
                    feito.unidade,
                    requisicoes
                );
            }
        }
        Ok(())
    })
}

/// Fatos da tentativa que acompanham a falha: o upload temporário pode ter
/// deixado resíduo. Não repita a escrita automaticamente.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
struct TentativaNaoComprovada {
    upload_tentado: bool,
    requisicoes: u8,
}

/// Lote não atômico: todos os cadastros são previamente conferidos, depois
/// cada documento usa a mesma confirmação individual, com formulário e foto novos.
pub(super) fn executar_lote(
    cli: &Cli,
    processo: &NumeroProcesso,
    tipo: &str,
    pdfs: &[PdfLocal],
    caminhos: &[PathBuf],
) -> anyhow::Result<()> {
    com_sessao(cli, |sessao, indice| {
        crate::origem_de_treinamento::exigir(
            crate::origem_de_treinamento::Escrita::DocumentoIncluirExternoLote,
            sessao.origem,
        )?;
        let mut envio = EnvioHttps;
        for pdf in pdfs {
            prevalidar_confirmacao(
                sessao,
                processo,
                cli.unidade.as_deref(),
                tipo,
                pdf,
                indice,
                &mut envio,
            )?;
        }
        // Reconfere todos os arquivos antes da primeira escrita; a sequência
        // individual ainda reabre e confere o descritor efetivamente enviado.
        for (pdf, caminho) in pdfs.iter().zip(caminhos) {
            crate::documento_externo_incluir::abrir_pdf_para_envio(caminho, pdf)?;
        }
        let mut itens = Vec::new();
        for (posicao, (pdf, caminho)) in pdfs.iter().zip(caminhos).enumerate() {
            let mut contagem = Contador::novo(&mut envio);
            let mut upload_tentado = false;
            let resultado = executar_confirmacao_https(
                sessao,
                processo,
                cli.unidade.as_deref(),
                tipo,
                pdf,
                caminho,
                indice,
                &mut contagem,
                &mut upload_tentado,
            );
            let requisicoes = contagem.feitas().saturating_add(u8::from(upload_tentado));
            match resultado {
                Ok(feito) => itens.push(json!({
                    "arquivo": arquivo_json(pdf),
                    "estado": "confirmado",
                    "numero_sei": feito.numero_sei,
                    "unidade": feito.unidade,
                    "requisicoes": requisicoes
                })),
                Err(erro) => {
                    itens.push(json!({
                        "arquivo": arquivo_json(pdf),
                        "estado": if upload_tentado { "nao_comprovado" } else { "sem_upload" },
                        "upload_tentado": upload_tentado,
                        "requisicoes": requisicoes
                    }));
                    itens.extend(pdfs[posicao + 1..].iter().map(|restante| json!({
                        "arquivo": arquivo_json(restante),
                        "estado": "nao_tentado"
                    })));
                    apresentar_lote(cli, processo, tipo, &itens, false);
                    return Err(anyhow::Error::from(erro).context(TentativaNaoComprovada {
                        upload_tentado,
                        requisicoes,
                    }));
                }
            }
        }
        apresentar_lote(cli, processo, tipo, &itens, true);
        Ok(())
    })
}

fn apresentar_lote(
    cli: &Cli,
    processo: &NumeroProcesso,
    tipo: &str,
    itens: &[serde_json::Value],
    confirmado: bool,
) {
    if cli.json {
        println!("{}", json!({
            "escopo": "confirmacao_web_lote",
            "processo": processo.as_str(),
            "tipo_solicitado": tipo,
            "itens": itens,
            "confirmado": confirmado
        }));
    } else {
        for item in itens {
            println!("{}: {}", item["arquivo"]["nome"], item["estado"]);
            if let Some(numero) = item["numero_sei"].as_str() {
                println!("Documento SEI: {numero}");
            }
        }
        if !confirmado {
            println!("Lote interrompido. Não repita sem conferir o processo.");
        }
    }
}
