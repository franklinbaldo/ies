//! CLI de `ies bloco incluir BLOCO --documento DOC | --processo NUP [--confirmar]`
//! (#381; contracts/cli/bloco-incluir.md). BDD-381-01/02: prévia e confirmação
//! causal documental; `--processo` permanece não implementado.
//! Erros sobem até o `main` (#695).
use crate::bloco_incluir::{PedidoDaInclusao, PreviaDocumental, prever_documento};
use crate::documento::NumeroDocumento;
use crate::indice_local::IndiceLocal;
use crate::processo_obter::SessaoDoProcesso;
use crate::transporte::sessao::ReferenciasDaSessao;
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, configuracao};
use serde_json::json;

/// O que esta fatia ainda não faz: recusado antes de qualquer requisição.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
enum AindaNaoImplementado {
    InclusaoDeProcesso,
}

pub(crate) fn executar(
    cli: &Cli,
    bloco: &str,
    documento: Option<&NumeroDocumento>,
    processo: Option<&str>,
    confirmar: bool,
) -> anyhow::Result<()> {
    if processo.is_some() {
        return Err(AindaNaoImplementado::InclusaoDeProcesso.into());
    }
    let documento = documento.ok_or(AindaNaoImplementado::InclusaoDeProcesso)?;
    let arquivos = configuracao::arquivos_de_acesso()?;
    let referencias =
        ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem, &arquivos.cookie)?;
    if confirmar {
        crate::origem_de_treinamento::exigir(
            crate::origem_de_treinamento::Escrita::BlocoIncluirDocumento,
            referencias.origem(),
        )?;
    }
    let sessao = SessaoDoProcesso {
        url_do_controle: referencias.url_do_controle(),
        origem: referencias.origem(),
        cookie: referencias.cookie(),
    };
    let previa = prever_documento(
        &sessao,
        &referencias,
        cli.unidade.as_deref(),
        &PedidoDaInclusao {
            bloco,
            documento,
            confirmar,
        },
        &IndiceLocal::da_configuracao(&arquivos),
        &mut EnvioHttps,
    )?;
    publicar(cli.json, &previa);
    Ok(())
}

fn publicar(json_mode: bool, previa: &PreviaDocumental) {
    if json_mode {
        println!(
            "{}",
            json!({
                "bloco": previa.bloco,
                "tipo_bloco": "assinatura",
                "alvo": {"documento": previa.documento},
                "confirmado": previa.confirmado,
                "alterado": previa.alterado,
                "presente_antes": previa.presente_antes,
                "presente_depois": previa.presente_depois,
                "origem": "web",
                "requisicoes": previa.requisicoes,
            })
        );
    } else if previa.alterado {
        println!(
            "Documento {} incluído e conferido no bloco de assinatura {}. Requisições: {}",
            previa.documento, previa.bloco, previa.requisicoes
        );
    } else if previa.presente_antes == Some(true) {
        println!(
            "Documento {} já está no bloco de assinatura {}; nenhuma alteração necessária. Requisições: {}",
            previa.documento, previa.bloco, previa.requisicoes
        );
    } else {
        let antes = match previa.presente_antes {
            Some(true) => "já está no bloco; nada a incluir",
            Some(false) => "não está no bloco",
            None => "não observado no detalhe do bloco",
        };
        println!(
            "Prévia: documento {} no bloco de assinatura {}: {antes}. Para incluir, repita com --confirmar. Requisições: {}",
            previa.documento, previa.bloco, previa.requisicoes
        );
    }
}
