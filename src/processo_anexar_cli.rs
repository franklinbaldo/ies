//! CLI de anexação: prévia e confirmação única, com prova independente.
use crate::processo_anexar::fluxo::{FalhaAnexar, Pedido, Previa, ResultadoAnexacao, prever};
use crate::processo_concluir_cli::com_sessao;
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao};
use serde_json::json;

pub(crate) fn tentar_anexar(cli: &Cli) -> Option<anyhow::Result<()>> {
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
    let anexado = numero.clone();
    let principal = a.clone();
    if anexado.as_str() == principal.as_str() {
        return Some(Err(FalhaAnexar::AnexacaoAoProprioProcesso.into()));
    }
    Some(com_sessao(cli, |sessao, indice| {
        if *confirmar
            && let Err(causa) = crate::origem_de_treinamento::exigir(
                crate::origem_de_treinamento::Escrita::ProcessoAnexar,
                sessao.origem,
            )
        {
            return Err(causa.into());
        }
        let pedido = Pedido {
            anexado: &anexado,
            principal: &principal,
            unidade: cli.unidade.as_deref(),
            confirmar: *confirmar,
        };
        let preview = prever(sessao, &pedido, indice, &mut EnvioHttps)?;
        publicar(cli.json, &preview);
        Ok(())
    }))
}

fn publicar(json_mode: bool, previa: &Previa) {
    let confirmado = previa.resultado == ResultadoAnexacao::Confirmada;
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
                "confirmado": confirmado,
                "alterado": confirmado,
                "origem": "web",
                "requisicoes": previa.requisicoes,
            })
        );
    } else {
        println!(
            "{}: anexar {} ao processo {} ({}). Requisições: {}.",
            if confirmado { "Confirmado" } else { "Prévia" },
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
}
