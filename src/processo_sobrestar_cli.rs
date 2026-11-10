//! Despacho da prévia e confirmação de sobrestar/retomar (#269).
//! Nenhuma credencial nem URL assinada é publicada; efeito incerto sai 3.
use crate::processo::NumeroProcesso;
use crate::processo_concluir_cli::com_sessao;
use crate::processo_sobrestar::{Acao, PedidoDaMudanca, executar};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao};
use serde_json::json;

pub(crate) fn tentar_operar(cli: &Cli) -> Option<anyhow::Result<()>> {
    match &cli.command {
        Domain::Processo(Processo {
            acao:
                Some(ProcessoAcao::Sobrestar {
                    numero,
                    motivo,
                    confirmar,
                }),
            ..
        }) => Some(operar(
            cli,
            numero,
            Some(motivo),
            *confirmar,
            Acao::Sobrestar,
        )),
        Domain::Processo(Processo {
            acao: Some(ProcessoAcao::Retomar { numero, confirmar }),
            ..
        }) => Some(operar(cli, numero, None, *confirmar, Acao::Retomar)),
        _ => None,
    }
}

fn operar(
    cli: &Cli,
    nup: &NumeroProcesso,
    motivo: Option<&str>,
    confirmar: bool,
    acao: Acao,
) -> anyhow::Result<()> {
    com_sessao(cli, |sessao, indice| {
        let pedido = PedidoDaMudanca {
            nup,
            unidade: cli.unidade.as_deref(),
            acao,
            motivo,
            confirmar,
        };
        let previa = executar(sessao, &pedido, indice, &mut EnvioHttps)?;
        if cli.json {
            println!(
                "{}",
                json!({
                    "nup": previa.nup,
                    "id_procedimento": previa.id_procedimento,
                    "unidade": previa.unidade,
                    "acao": previa.acao.como_texto(),
                    "motivo": motivo,
                    "confirmado": previa.estado.aplicada(),
                    "alterado": previa.estado.aplicada(),
                    "reconciliado_por_leitura": previa.estado.aplicada(),
                    "origem": "web",
                    "requisicoes": previa.requisicoes
                })
            );
        } else if previa.estado.aplicada() {
            println!(
                "Processo {}: {} aplicado e conferido por leitura. Requisições: {}.",
                previa.nup,
                previa.acao.como_texto(),
                previa.requisicoes
            );
        } else {
            println!(
                "Prévia: {} o processo {} na unidade {}. Requisições: {}. Nenhuma escrita enviada.",
                previa.acao.como_texto(),
                previa.nup,
                previa.unidade,
                previa.requisicoes
            );
        }
        Ok(())
    })
}
