//! Prévia Web de atribuição por login exato; confirmação apenas nas
//! instalação de treinamento `UnB` com efeito verificado por releitura.
use crate::processo_atribuir::{
    ErroProcessoAtribuir, EstadoDaAtribuicao, PedidoDaAtribuicao, ProcessoAtribuido,
    executar_previa,
};
use crate::processo_concluir_cli::{com_sessao, json_base_da_falha};
use crate::saida_erro;
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn tentar_atribuir(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao:
            Some(ProcessoAcao::Atribuir {
                numero,
                usuario,
                remover,
                confirmar,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    let nup = numero.clone();
    Some(com_sessao(cli, |sessao, indice| {
        // Limite inviolável: escrita só em instalação de treinamento.
        if *confirmar
            && let Err(causa) = crate::origem_de_treinamento::exigir(
                crate::origem_de_treinamento::Escrita::ProcessoAtribuir,
                sessao.origem,
            )
        {
            return crate::origem_de_treinamento::recusa(cli.json, causa);
        }
        let pedido = PedidoDaAtribuicao {
            nup: &nup,
            unidade: cli.unidade.as_deref(),
            usuario: if *remover { None } else { usuario.as_deref() },
            confirmar: *confirmar,
        };
        match executar_previa(sessao, &pedido, indice, &mut EnvioHttps) {
            Ok(feito) => publicar_previa(cli.json, &feito),
            Err(erro) => publicar_falha(cli.json, &erro),
        }
    }))
}

/// Toda recusa HTTP expõe status e mensagem saneada; a causa é preservada.
/// Fatos de diagnóstico da prévia são publicados sem extrapolar uma unidade
/// da árvore para outra: lista vazia = observada sem abertos; null = não lida.
pub(crate) fn json_de_falha(erro: &ErroProcessoAtribuir) -> serde_json::Value {
    let mut valor = json_base_da_falha(
        "atribuicao_recusada",
        erro.mensagem(),
        erro.status_http().cloned(),
    );
    if let Some(objeto) = valor.as_object_mut() {
        match erro {
            ErroProcessoAtribuir::SituacaoDaAtribuicaoNaoObservada {
                unidade,
                aberto_em,
                motivo,
                requisicoes,
            } => {
                objeto.insert("unidade".to_owned(), json!(unidade));
                objeto.insert("aberto_em".to_owned(), json!(aberto_em));
                objeto.insert("motivo".to_owned(), json!(motivo));
                objeto.insert("requisicoes".to_owned(), json!(requisicoes));
            }
            ErroProcessoAtribuir::NaoConfirmadoNaAtribuicao {
                requisicoes,
                atribuicao_relida,
                ..
            } => {
                objeto.insert("requisicoes".to_owned(), json!(requisicoes));
                // Estado observado após o POST, não evidência de autoria.
                objeto.insert("atribuicao_relida".to_owned(), json!(atribuicao_relida));
            }
            _ => {}
        }
    }
    valor
}

fn publicar_falha(json_mode: bool, erro: &ErroProcessoAtribuir) -> ExitCode {
    if json_mode {
        println!("{}", json_de_falha(erro));
    } else {
        eprintln!(
            "{}",
            saida_erro::linha_humana(erro.mensagem(), erro.status_http())
        );
        // O estado observado depois do POST pode ser um resíduo real,
        // mas não identifica qual sessão provocou a mudança.
        if let ErroProcessoAtribuir::NaoConfirmadoNaAtribuicao {
            atribuicao_relida: Some(login),
            ..
        } = erro
        {
            let estado = if login.is_empty() { "nenhuma" } else { login };
            eprintln!("Estado relido: {estado} (não comprova o efeito deste POST).");
        }
    }
    ExitCode::from(1)
}

/// Traduz o estado único do domínio para os dois campos do contrato JSON legado.
pub(crate) fn json_da_previa(feito: &ProcessoAtribuido) -> serde_json::Value {
    let (confirmado, alterado) = match feito.estado {
        EstadoDaAtribuicao::Previa => (false, false),
        EstadoDaAtribuicao::AlteracaoConfirmada => (true, true),
    };
    json!({
        "nup": feito.nup,
        "id_procedimento": feito.id_procedimento,
        "unidade": feito.unidade,
        "usuario_atual": feito.usuario_atual,
        "usuario_pedido": feito.usuario_pedido,
        "remover": feito.usuario_pedido.is_none(),
        "valor_da_opcao": feito.valor_da_opcao,
        "confirmado": confirmado,
        "alterado": alterado,
        "origem": "web",
        "requisicoes": feito.requisicoes
    })
}

/// O texto depende do mesmo estado tipado que os dois campos JSON.
/// Uma alteração já comprovada não pode ser apresentada como mera prévia.
pub(crate) fn texto_humano(feito: &ProcessoAtribuido) -> String {
    let titulo = match (feito.estado, feito.usuario_pedido.as_ref()) {
        (EstadoDaAtribuicao::Previa, _) => "Prévia",
        (EstadoDaAtribuicao::AlteracaoConfirmada, Some(_)) => "Atribuição confirmada",
        (EstadoDaAtribuicao::AlteracaoConfirmada, None) => "Remoção confirmada",
    };
    let mut linhas = vec![format!(
        "{titulo}: processo {}, unidade {}.",
        feito.nup, feito.unidade
    )];
    let atual = if feito.usuario_atual.is_empty() {
        "nenhuma"
    } else {
        &feito.usuario_atual
    };
    if feito.estado == EstadoDaAtribuicao::Previa {
        linhas.push(format!("Atribuição atual: {atual}."));
        match &feito.usuario_pedido {
            Some(login) => linhas.push(format!("Atribuir a {login}; opção verificada na tela.")),
            None => linhas.push("Remover a atribuição observada.".to_owned()),
        }
    } else {
        linhas.push(format!("Atribuição anterior: {atual}."));
        let nova = feito.usuario_pedido.as_deref().unwrap_or("nenhuma");
        linhas.push(format!("Atribuição agora: {nova}."));
    }
    linhas.push(format!("Requisições: {}.", feito.requisicoes));
    linhas.join("\n")
}

fn publicar_previa(json_mode: bool, feito: &ProcessoAtribuido) -> ExitCode {
    if json_mode {
        println!("{}", json_da_previa(feito));
    } else {
        println!("{}", texto_humano(feito));
    }
    ExitCode::SUCCESS
}
