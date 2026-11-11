//! Prévia Web de atribuição por login exato; confirmação apenas nas
//! instalação de treinamento `UnB` com efeito verificado por releitura.
use crate::processo_atribuir::{
    EstadoDaAtribuicao, PedidoDaAtribuicao, ProcessoAtribuido, executar_previa,
};
use crate::processo_concluir_cli::com_sessao;
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao};
use serde_json::json;

pub(crate) fn tentar_atribuir(cli: &Cli) -> Option<anyhow::Result<()>> {
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
            return Err(causa.into());
        }
        let pedido = PedidoDaAtribuicao {
            nup: &nup,
            unidade: cli.unidade.as_deref(),
            usuario: if *remover { None } else { usuario.as_deref() },
            confirmar: *confirmar,
        };
        let feito = executar_previa(sessao, &pedido, indice, &mut EnvioHttps)?;
        publicar_previa(cli.json, &feito);
        Ok(())
    }))
}

/// Traduz o estado único do domínio para os dois campos do contrato JSON legado.
pub(crate) fn json_da_previa(feito: &ProcessoAtribuido) -> serde_json::Value {
    let (confirmado, alterado) = match feito.estado {
        EstadoDaAtribuicao::Previa => (false, false),
        EstadoDaAtribuicao::AlteracaoConfirmada => (true, true),
    };
    let mut valor = json!({
        "nup": feito.nup,
        "id_procedimento": feito.id_procedimento,
        "unidade": feito.unidade,
        "remover": feito.usuario_pedido.is_none(),
        "valor_da_opcao": feito.valor_da_opcao,
        "confirmado": confirmado,
        "alterado": alterado,
        "origem": "web",
        "requisicoes": feito.requisicoes
    });
    // Sem atribuição, a chave é omitida: nunca texto vazio como sentinela.
    if let (Some(campos), Some(login)) = (valor.as_object_mut(), feito.usuario_atual.as_deref()) {
        campos.insert("usuario_atual".to_owned(), json!(login));
    }
    // Na remoção não há usuário pedido: a chave é omitida.
    if let (Some(campos), Some(pedido)) = (valor.as_object_mut(), feito.usuario_pedido.as_deref()) {
        campos.insert("usuario_pedido".to_owned(), json!(pedido));
    }
    valor
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
    let atual = feito.usuario_atual.as_deref().unwrap_or("nenhuma");
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

fn publicar_previa(json_mode: bool, feito: &ProcessoAtribuido) {
    if json_mode {
        println!("{}", json_da_previa(feito));
    } else {
        println!("{}", texto_humano(feito));
    }
}
