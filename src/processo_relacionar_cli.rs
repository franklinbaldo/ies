//! CLI de relacionamento: prévia, remoção confirmada e efeito incerto.
use crate::processo_concluir_cli::com_sessao;
use crate::processo_relacionar::{
    FalhaRelacionar, Pedido, Previa, ResultadoRelacionar, mesmo_protocolo, prever,
};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn tentar_relacionar(cli: &Cli) -> Option<anyhow::Result<()>> {
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
    let nup = numero.clone();
    let outro = com.clone();
    // A identidade é conhecida pela própria entrada da CLI: não abrir
    // sessão, índice ou rede para tentar relacionar o processo consigo mesmo.
    if mesmo_protocolo(nup.as_str(), outro.as_str()) {
        return Some(crate::legado(falha(
            cli.json,
            &FalhaRelacionar::RelacionamentoConsigoMesmo,
        )));
    }
    Some(com_sessao(cli, |sessao, indice| {
        // Aprovada em treinamento (10/11/2026): grava também em produção.
        if *confirmar
            && let Err(causa) = crate::origem_de_treinamento::exigir(
                crate::origem_de_treinamento::Escrita::ProcessoRelacionar,
                sessao.origem,
            )
        {
            return Err(causa.into());
        }
        let pedido = Pedido {
            nup: &nup,
            com: &outro,
            unidade: cli.unidade.as_deref(),
            remover: *remover,
            confirmar: *confirmar,
        };
        let preview = prever(sessao, &pedido, indice, &mut EnvioHttps)?;
        crate::legado(publicar(cli.json, &preview))
    }))
}

fn falha(json_mode: bool, erro: &FalhaRelacionar) -> ExitCode {
    let incerto = matches!(erro, FalhaRelacionar::NaoConfirmado(_));
    let mensagem = if incerto {
        "O envio não comprovou a alteração do relacionamento. Confira o SEI antes de qualquer nova confirmação; não repita a escrita.".to_owned()
    } else {
        erro.mensagem()
    };
    if json_mode {
        let mut resposta = json!({
            "codigo": if incerto { "nao_confirmado" } else { "operacao_recusada" },
            "erro": mensagem,
            "retentavel": false,
        });
        if let Some(status) = erro.status_http() {
            resposta["status_http"] = json!(status.status);
            if let Some(texto) = &status.mensagem_do_servidor {
                resposta["mensagem_servidor"] = json!(texto);
            }
        }
        println!("{resposta}");
    } else {
        eprintln!("IES: {mensagem}");
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
                "confirmado": previa.resultado == ResultadoRelacionar::Confirmado,
                "alterado": previa.resultado == ResultadoRelacionar::Confirmado,
                "sem_efeito": previa.sem_efeito(),
                "origem": "web",
                "requisicoes": previa.requisicoes,
            })
        );
    } else if previa.resultado == ResultadoRelacionar::Confirmado {
        println!(
            "Relacionamento {} e conferido: {} com {}. Requisições: {}",
            if previa.remover { "removido" } else { "criado" },
            previa.nup,
            previa.destino,
            previa.requisicoes
        );
    } else if previa.sem_efeito() {
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
            "Prévia: {} processo {} com {}{} (id: {}). Requisições: {}. {}.",
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
            if previa.remover {
                "Remoção ainda não habilitada em treinamento"
            } else {
                "Adição ainda não habilitada em treinamento"
            },
        );
    }
    ExitCode::SUCCESS
}
