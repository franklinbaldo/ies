//! Despacho da prévia e confirmação de sobrestar/retomar (#269).
//! Nenhuma credencial nem URL assinada é publicada; efeito incerto sai 3.
use crate::processo::NumeroProcesso;
use crate::processo_concluir_cli::com_sessao;
use crate::processo_sobrestar::{Acao, PedidoDaMudanca, executar};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao};
use serde_json::json;

pub(crate) fn tentar_operar(cli: &Cli) -> Option<anyhow::Result<()>> {
    // Despacho: só as duas ações deste módulo; qualquer outro comando é de outro.
    let Domain::Processo(Processo {
        acao: Some(acao), ..
    }) = &cli.command
    else {
        return None;
    };
    if let ProcessoAcao::Sobrestar {
        numero,
        motivo,
        confirmar,
    } = acao
    {
        return Some(operar(
            cli,
            numero,
            Some(motivo),
            *confirmar,
            Acao::Sobrestar,
        ));
    }
    if let ProcessoAcao::Retomar { numero, confirmar } = acao {
        return Some(operar(cli, numero, None, *confirmar, Acao::Retomar));
    }
    None
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
            let mut valor = json!({
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
            });
            crate::indice_local::anexar_avisos(&mut valor, indice);
            println!("{valor}");
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
