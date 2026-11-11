//! CLI do registro de andamento: valida entrada antes da sessão, nunca retry.
use crate::processo_andamento_registrar;
use crate::processo_concluir_cli::com_sessao;
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao};
use serde_json::json;

pub(crate) fn tentar_registrar(cli: &Cli) -> Option<anyhow::Result<()>> {
    let Domain::Processo(Processo {
        acao:
            Some(ProcessoAcao::Andamento {
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
        // Aprovada em treinamento (10/11/2026): grava também em produção.
        if *confirmar
            && let Err(causa) = crate::origem_de_treinamento::exigir(
                crate::origem_de_treinamento::Escrita::ProcessoAndamento,
                sessao.origem,
            )
        {
            return Err(causa.into());
        }
        let r = processo_andamento_registrar::registrar(
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
                "numero": r.numero,
                "texto": r.texto,
                "confirmado": r.confirmado,
                "alterado": r.total_depois.is_some(),
                "origem": "web",
                "total_antes": r.total_antes,
                "total_depois": r.total_depois,
                "requisicoes": r.requisicoes,
                "resposta_inesperada": r.resposta_inesperada,
            });
            crate::indice_local::anexar_avisos(&mut valor, indice);
            println!("{valor}");
        } else if r.confirmado {
            println!(
                "Andamento registrado e conferido no processo {}. Requisições: {}",
                r.numero, r.requisicoes
            );
        } else {
            println!(
                "Prévia: registrar no processo {} o andamento: {:?}. Nenhuma escrita. Requisições: {}. Para gravar, use --confirmar.",
                r.numero, r.texto, r.requisicoes
            );
        }
        Ok(())
    }))
}
