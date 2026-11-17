//! Entrada da CLI para `ies processo ciente NUP [--confirmar]`.
//! A prévia não dispara o GET que grava a ciência; o resultado não contém
//! URL, cookie, assinatura ou hash.
use crate::ciencia::{CienciaDoProcesso, PedidoDaCiencia, ciencia_no_processo};
use crate::indice_local::{self, IndiceLocal};
use crate::processo_concluir_cli::com_sessao;
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao};
use serde_json::{Value, json};

pub(crate) fn tentar_ciente(cli: &Cli) -> Option<anyhow::Result<()>> {
    let Domain::Processo(Processo {
        acao: Some(ProcessoAcao::Ciente { numero, confirmar }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    Some(com_sessao(cli, |sessao, indice| {
        let pedido = PedidoDaCiencia {
            nup: numero,
            unidade: cli.unidade.as_deref(),
            confirmar: *confirmar,
        };
        let feito = ciencia_no_processo(sessao, &pedido, indice, &mut EnvioHttps)?;
        publicar_sucesso(cli.json, &feito, indice);
        Ok(())
    }))
}

fn json_de_sucesso(feito: &CienciaDoProcesso) -> Value {
    json!({
        "numero": feito.nup,
        "id_procedimento": feito.id_procedimento,
        "unidade": feito.unidade,
        "origem": "web",
        "confirmado": feito.confirmado,
        "verificado": feito.verificado,
        "id_atividade": feito.id_atividade,
        "usuario": feito.usuario,
        "requisicoes": feito.requisicoes,
    })
}

fn publicar_sucesso(json_mode: bool, feito: &CienciaDoProcesso, indice: &IndiceLocal) {
    if json_mode {
        let mut valor = json_de_sucesso(feito);
        indice_local::anexar_avisos(&mut valor, indice);
        println!("{valor}");
    } else if feito.verificado {
        println!(
            "Ciência no processo {} verificada na lista (atividade {}). Requisições: {}.",
            feito.nup,
            feito.id_atividade.as_deref().unwrap_or("não observada"),
            feito.requisicoes
        );
    } else {
        println!(
            "Prévia: registrar ciência no processo {} na unidade {}. Uma ciência por dia, por usuário. Para executar: --confirmar. Requisições: {}.",
            feito.nup, feito.unidade, feito.requisicoes
        );
    }
}
