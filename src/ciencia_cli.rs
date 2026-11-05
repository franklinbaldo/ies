//! Entrada da CLI para `ies processo ciente NUP [--confirmar]`.
//! A prévia não dispara o GET que grava a ciência; o resultado não contém
//! URL, cookie, assinatura ou hash.
use crate::ciencia::{CienciaDoProcesso, ErroCienciaProcesso, PedidoDaCiencia, ciencia_no_processo};
use crate::processo::NumeroProcesso;
use crate::processo_concluir_cli::{com_sessao, erro_de_uso};
use crate::transporte::StatusHttpRecusado;
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao};
use serde_json::{Value, json};
use std::process::ExitCode;

pub(crate) fn tentar_ciente(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao: Some(ProcessoAcao::Ciente { numero, confirmar }), ..
    }) = &cli.command else {
        return None;
    };
    Some(match NumeroProcesso::novo(numero) {
        Ok(nup) => com_sessao(cli, |sessao, indice| {
            let pedido = PedidoDaCiencia {
                nup: &nup,
                unidade: cli.unidade.as_deref(),
                confirmar: *confirmar,
            };
            match ciencia_no_processo(sessao, &pedido, indice, &mut EnvioHttps) {
                Ok(feito) => publicar_sucesso(cli.json, &feito),
                Err(erro) => publicar_erro(cli.json, &erro),
            }
        }),
        Err(erro) => erro_de_uso(cli.json, erro.subcodigo()),
    })
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
        "requisicoes": feito.requisicoes,
    })
}

fn publicar_sucesso(json_mode: bool, feito: &CienciaDoProcesso) -> ExitCode {
    if json_mode {
        println!("{}", json_de_sucesso(feito));
    } else if feito.verificado {
        println!("Ciência no processo {} verificada na lista (atividade {}). Requisições: {}.",
            feito.nup, feito.id_atividade.as_deref().unwrap_or("não observada"), feito.requisicoes);
    } else {
        println!("Prévia: registrar ciência no processo {} na unidade {}. Uma ciência por dia, por usuário. Para executar: --confirmar. Requisições: {}.",
            feito.nup, feito.unidade, feito.requisicoes);
    }
    ExitCode::SUCCESS
}

fn publicar_erro(json_mode: bool, erro: &ErroCienciaProcesso) -> ExitCode {
    let codigo = if erro.saida() == 3 { "efeito_incerto" } else { "fonte_invalida" };
    let mut objeto = json!({
        "codigo": codigo,
        "subcodigo": erro.subcodigo(),
        "erro": erro.mensagem(),
        "retentavel": false,
    });
    if let ErroCienciaProcesso::EfeitoIncerto { motivo, requisicoes } = erro {
        objeto["verificacao"] = json!(motivo);
        objeto["requisicoes"] = json!(requisicoes);
    }
    let http: Option<StatusHttpRecusado> = match erro {
        ErroCienciaProcesso::Abertura(e) => e.status_http().cloned(),
        ErroCienciaProcesso::Cookie(e) | ErroCienciaProcesso::Tela(e) => e.status_http().cloned(),
        _ => None,
    };
    if let Some(http) = http {
        if http.status != 200 {
            objeto["status_http"] = json!(http.status);
        }
        if let Some(mensagem) = http.mensagem_do_servidor {
            objeto["mensagem_do_servidor"] = json!(mensagem);
        }
    }
    if json_mode {
        println!("{objeto}");
    } else {
        eprintln!("IES: {} [{}]", erro.mensagem(), erro.subcodigo());
        if let Some(requisicoes) = erro.requisicoes() {
            eprintln!("IES: {} requisições, não repetir a escrita; verificar no SEI", requisicoes);
        }
    }
    ExitCode::from(erro.saida())
}
