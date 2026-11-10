//! CLI de `ies bloco incluir BLOCO --documento DOC | --processo NUP [--confirmar]`
//! (#381; contracts/cli/bloco-incluir.md). BDD-381-01/02: prévia, confirmação
//! documental causal e releitura; `--processo` ainda recusa antes da rede.
use crate::bloco_incluir::{PedidoDaInclusao, PreviaDocumental, prever_documento};
use crate::documento::NumeroDocumento;
use crate::indice_local::IndiceLocal;
use crate::processo_obter::SessaoDoProcesso;
use crate::saida_erro::erro_fonte;
use crate::transporte::sessao::ReferenciasDaSessao;
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, configuracao};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn executar(
    cli: &Cli,
    bloco: &str,
    documento: Option<&str>,
    processo: Option<&str>,
    confirmar: bool,
) -> ExitCode {
    if bloco.is_empty() || !bloco.bytes().all(|b| b.is_ascii_digit()) {
        return uso(cli.json, "número de bloco deve ser decimal");
    }
    if processo.is_some() {
        return erro_fonte(
            cli.json,
            "inclusão de processo em bloco ainda não implementada; nenhuma requisição enviada",
        );
    }
    let Some(Ok(documento)) = documento.map(NumeroDocumento::novo) else {
        return uso(cli.json, "número de documento inválido");
    };
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => return erro_fonte(cli.json, erro.mensagem()),
    };
    let referencias = match ReferenciasDaSessao::ler(
        &arquivos.url_controle,
        &arquivos.origem,
        &arquivos.cookie,
    ) {
        Ok(referencias) => referencias,
        Err(falha) => return erro_fonte(cli.json, falha.mensagem()),
    };
    if confirmar
        && let Err(causa) = crate::origem_de_treinamento::exigir(
            crate::origem_de_treinamento::Escrita::BlocoIncluirDocumento,
            referencias.origem(),
        )
    {
        return crate::origem_de_treinamento::recusa(cli.json, causa);
    }
    let sessao = SessaoDoProcesso {
        url_do_controle: referencias.url_do_controle(),
        origem: referencias.origem(),
        cookie: referencias.cookie(),
    };
    match prever_documento(
        &sessao,
        &referencias,
        cli.unidade.as_deref(),
        PedidoDaInclusao {
            bloco,
            documento: &documento,
            confirmar,
        },
        &IndiceLocal::da_configuracao(&arquivos),
        &mut EnvioHttps,
    ) {
        Ok(previa) => publicar(cli.json, &previa),
        Err(crate::bloco_incluir::ErroIncluir::NaoConfirmado) => {
            let mensagem = "inclusão não confirmada; consulte o bloco antes de repetir";
            if cli.json {
                println!(
                    "{}",
                    json!({
                        "codigo": "nao_confirmado", "erro": mensagem, "retentavel": false
                    })
                );
                ExitCode::from(1)
            } else {
                erro_fonte(false, mensagem)
            }
        }
        Err(erro) => erro_fonte(cli.json, &erro.mensagem()),
    }
}

fn publicar(json_mode: bool, previa: &PreviaDocumental) -> ExitCode {
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
    ExitCode::SUCCESS
}

fn uso(json_mode: bool, mensagem: &str) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({"codigo": "entrada_invalida", "erro": mensagem, "retentavel": false})
        );
    } else {
        eprintln!("IES: {mensagem}");
    }
    ExitCode::from(2)
}
