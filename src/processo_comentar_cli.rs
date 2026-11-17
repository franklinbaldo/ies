//! Comentários no processo: prévia e confirmação condicionada à origem.
//! Um POST e releitura causal, sem retry ou escrita em produção.
use crate::processo_comentar_web;
use crate::processo_concluir_cli::com_sessao;
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao};
use serde_json::json;

pub(crate) fn tentar_comentar(cli: &Cli) -> Option<anyhow::Result<()>> {
    let Domain::Processo(Processo {
        acao:
            Some(ProcessoAcao::Comentar {
                numero,
                texto,
                confirmar,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    let numero = numero.clone();
    Some(com_sessao(cli, |sessao, indice| {
        if *confirmar
            && let Err(causa) = crate::origem_de_treinamento::exigir(
                crate::origem_de_treinamento::Escrita::ProcessoComentar,
                sessao.origem,
            )
        {
            return Err(causa.into());
        }
        let preview = processo_comentar_web::prever(
            sessao,
            &numero,
            texto,
            *confirmar,
            cli.unidade.as_deref(),
            indice,
            &mut EnvioHttps,
        )?;
        if cli.json {
            let mut valor = json!({
                "nup": preview.numero,
                "id_procedimento": preview.id_procedimento,
                "texto": preview.texto,
                "confirmado": preview.confirmado,
                "alterado": preview.confirmado,
                "origem": "web",
                "requisicoes": preview.requisicoes,
            });
            crate::indice_local::anexar_avisos(&mut valor, indice);
            println!("{valor}");
        } else if preview.confirmado {
            println!(
                "Comentário confirmado por releitura no processo {}. Requisições: {}.",
                preview.numero, preview.requisicoes
            );
        } else {
            println!(
                "Prévia: comentar no processo {}: {:?}. Nenhuma escrita. Requisições: {}.",
                preview.numero, preview.texto, preview.requisicoes
            );
        }
        Ok(())
    }))
}
