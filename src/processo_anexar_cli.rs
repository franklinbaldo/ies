//! CLI da prévia segura de anexação; confirmação ainda fechada antes da rede.
use crate::processo::NumeroProcesso;
use crate::processo_anexar::fluxo::{FalhaAnexar, Pedido, Previa, prever};
use crate::processo_concluir_cli::{com_sessao, erro_de_uso};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao, saida_erro};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn tentar_anexar(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao:
            Some(ProcessoAcao::Anexar {
                numero,
                a,
                confirmar,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    let anexado = match NumeroProcesso::novo(numero) {
        Ok(n) => n,
        Err(erro) => return Some(erro_de_uso(cli.json, erro.subcodigo())),
    };
    let principal = match NumeroProcesso::novo(a) {
        Ok(n) => n,
        Err(erro) => return Some(erro_de_uso(cli.json, erro.subcodigo())),
    };
    if anexado.as_str() == principal.as_str() {
        return Some(falha(cli.json, "anexacao_ao_mesmo_processo"));
    }
    if *confirmar {
        return Some(falha(cli.json, "confirmacao_de_anexacao_nao_validada"));
    }
    Some(com_sessao(cli, |sessao, indice| {
        let pedido = Pedido {
            anexado: &anexado,
            principal: &principal,
            unidade: cli.unidade.as_deref(),
            confirmar: false,
        };
        match prever(sessao, &pedido, indice, &mut EnvioHttps) {
            Ok(preview) => publicar(cli.json, &preview),
            Err(FalhaAnexar::Abertura(erro)) => saida_erro::erro_fonte_http(
                cli.json,
                erro.subcodigo(),
                erro.mensagem(),
                erro.status_http(),
            ),
            Err(erro) => falha(cli.json, erro.subcodigo()),
        }
    }))
}
fn falha(json_mode: bool, subcodigo: &str) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "operacao_recusada",
                "subcodigo": subcodigo,
                "erro": "anexação não autorizada pela tela ou confirmação pendente",
                "retentavel": false,
            })
        );
    } else {
        eprintln!("IES: anexação recusada [{subcodigo}]");
    }
    ExitCode::from(1)
}
fn publicar(json_mode: bool, previa: &Previa) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "principal": previa.principal,
                "id_principal": previa.id_principal,
                "anexado": previa.anexado.nup,
                "id_anexado": previa.anexado.id,
                "tipo_anexado": previa.anexado.tipo,
                "documentos_assinados": previa.assinaturas_em_dia,
                "confirmado": false,
                "alterado": false,
                "origem": "web",
                "requisicoes": previa.requisicoes,
            })
        );
    } else {
        println!(
            "Prévia: anexar {} ao processo {} ({}). Requisições: {}. Escrita bloqueada até validação.",
            previa.anexado.nup,
            previa.principal,
            if previa.assinaturas_em_dia {
                "sem pendência de assinatura"
            } else {
                "há documento sem assinatura"
            },
            previa.requisicoes,
        );
    }
    ExitCode::SUCCESS
}
