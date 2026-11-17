//! #332 — conecta a confirmação única do PDF à saída pública.
//! Validada ao vivo na `UnB` de treinamento (09/11/2026, saída 0).
use crate::Cli;
use crate::documento_externo_incluir::{PdfLocal, fluxo_confirmar::executar_confirmacao_https};
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::Contador;
use crate::processo_concluir_cli::com_sessao;
use crate::visao_do_controle::EnvioHttps;
use serde_json::json;
use std::path::Path;

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
