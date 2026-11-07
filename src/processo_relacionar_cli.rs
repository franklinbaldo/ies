//! CLI de prévia de relacionamento. Escrita intencionalmente bloqueada.
use crate::processo::NumeroProcesso;
use crate::processo_concluir_cli::{com_sessao, erro_de_uso};
use crate::processo_relacionar::{FalhaRelacionar, Pedido, Previa, prever};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao, saida_erro};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn tentar_relacionar(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao:
            Some(ProcessoAcao::Relacionar {
                numero,
                com,
                remover,
                confirmar,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    let nup = match NumeroProcesso::novo(numero) {
        Ok(n) => n,
        Err(erro) => return Some(erro_de_uso(cli.json, erro.subcodigo())),
    };
    let outro = match NumeroProcesso::novo(com) {
        Ok(n) => n,
        Err(erro) => return Some(erro_de_uso(cli.json, erro.subcodigo())),
    };
    if *confirmar {
        // Zero rede, zero escrita até integrar a reconciliação.
        return Some(falha(
            cli.json,
            &FalhaRelacionar::ConfirmacaoAindaNaoValidada,
        ));
    }
    Some(com_sessao(cli, |sessao, indice| {
        let pedido = Pedido {
            nup: &nup,
            com: &outro,
            unidade: cli.unidade.as_deref(),
            remover: *remover,
            confirmar: *confirmar,
        };
        match prever(sessao, &pedido, indice, &mut EnvioHttps) {
            Ok(preview) => publicar(cli.json, &preview),
            Err(FalhaRelacionar::Abertura(erro)) => saida_erro::erro_fonte_http(
                cli.json,
                erro.subcodigo(),
                erro.mensagem(),
                erro.status_http(),
            ),
            Err(erro) => falha(cli.json, &erro),
        }
    }))
}

fn falha(json_mode: bool, erro: &FalhaRelacionar) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "operacao_recusada",
                "subcodigo": erro.subcodigo(),
                "erro": "prévia indisponível ou confirmação ainda não validada",
                "retentavel": false,
            })
        );
    } else {
        eprintln!("IES: operação recusada [{}]", erro.subcodigo());
    }
    ExitCode::from(1)
}

fn publicar(json_mode: bool, previa: &Previa) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "nup": previa.nup,
                "id_procedimento": previa.id_procedimento,
                "destino": previa.destino,
                "id_destino": previa.id_destino,
                "tipo_destino": previa.tipo_destino,
                "remover": previa.remover,
                "confirmado": false,
                "alterado": false,
                "sem_efeito": previa.sem_efeito,
                "origem": "web",
                "requisicoes": previa.requisicoes,
            })
        );
    } else if previa.sem_efeito {
        println!(
            "Nada a fazer: relação no estado pedido. Requisições: {}",
            previa.requisicoes
        );
    } else {
        let tipo = previa
            .tipo_destino
            .as_deref()
            .map(|nome| format!(" (tipo: {nome})"))
            .unwrap_or_default();
        println!(
            "Prévia: {} processo {} com {}{} (id: {}). Requisições: {}. Escrita ainda bloqueada.",
            if previa.remover {
                "desvincular"
            } else {
                "relacionar"
            },
            previa.nup,
            previa.destino,
            tipo,
            previa.id_destino.as_deref().unwrap_or("não observado"),
            previa.requisicoes,
        );
    }
    ExitCode::SUCCESS
}
