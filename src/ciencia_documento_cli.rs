//! Entrada de `ies documento ciente DOC [--confirmar]`.
//! A prova é a atividade na Lista de Ciências do documento relida.
use crate::ciencia::{
    CienciaDoDocumento, ErroCienciaDocumento, PedidoDaCienciaDocumento, ciencia_no_documento,
};
use crate::documento::NumeroDocumento;
use crate::processo::NumeroProcesso;
use crate::processo_concluir_cli::{com_sessao, erro_de_uso};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Documento, DocumentoAcao, Domain};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn tentar_ciente_documento(cli: &Cli) -> Option<ExitCode> {
    let Domain::Documento(Documento {
        acao: Some(DocumentoAcao::Ciente { numero, confirmar }),
        processo,
        ..
    }) = &cli.command else {
        return None;
    };
    let numero = match NumeroDocumento::novo(numero) {
        Ok(v) => v,
        Err(e) => return Some(erro_de_uso(cli.json, e.subcodigo())),
    };
    let processo = match processo.as_deref().map(NumeroProcesso::novo) {
        None => None,
        Some(Ok(p)) => Some(p),
        Some(Err(e)) => return Some(erro_de_uso(cli.json, e.subcodigo())),
    };
    Some(com_sessao(cli, |sessao, indice| {
        let pedido = PedidoDaCienciaDocumento {
            numero: &numero,
            processo: processo.as_ref(),
            unidade: cli.unidade.as_deref(),
            confirmar: *confirmar,
        };
        match ciencia_no_documento(sessao, &pedido, indice, &mut EnvioHttps) {
            Ok(feito) => sucesso(cli.json, &feito),
            Err(erro) => falha(cli.json, &erro),
        }
    }))
}

fn sucesso(json_mode: bool, feito: &CienciaDoDocumento) -> ExitCode {
    if json_mode {
        println!("{}", json!({
            "numero": feito.numero,
            "id_documento": feito.id_documento,
            "rotulo": feito.rotulo,
            "processo": {"nup": feito.processo_nup, "id": feito.id_procedimento},
            "unidade": feito.unidade,
            "origem": "web",
            "confirmado": feito.confirmado,
            "verificado": feito.verificado,
            "id_atividade": feito.id_atividade,
            "requisicoes": feito.requisicoes,
        }));
    } else if feito.verificado {
        println!(
            "Ciência no documento {} verificada na Lista de Ciências (atividade {}). Requisições: {}.",
            feito.numero, feito.id_atividade.as_deref().unwrap_or("não observada"), feito.requisicoes
        );
    } else {
        println!(
            "Prévia: ciência do usuário desta sessão no documento {} ({}) do processo {}.              O SEI permite uma por documento. Para executar: --confirmar. Requisições: {}.",
            feito.numero, feito.rotulo, feito.processo_nup, feito.requisicoes
        );
    }
    ExitCode::SUCCESS
}

fn falha(json_mode: bool, erro: &ErroCienciaDocumento) -> ExitCode {
    let mut saida = json!({
        "codigo": if erro.saida() == 3 { "efeito_incerto" } else { "fonte_invalida" },
        "subcodigo": erro.subcodigo(),
        "erro": erro.mensagem(),
        "retentavel": false,
    });
    if let ErroCienciaDocumento::EfeitoIncerto { motivo, requisicoes } = erro {
        saida["verificacao"] = json!(motivo);
        saida["requisicoes"] = json!(requisicoes);
    }
    if let Some(http) = erro.status_http() {
        if http.status != 200 {
            saida["status_http"] = json!(http.status);
        }
        if let Some(mensagem) = &http.mensagem_do_servidor {
            saida["mensagem_do_servidor"] = json!(mensagem);
        }
    }
    if json_mode {
        println!("{saida}");
    } else {
        eprintln!("IES: {} [{}]", erro.mensagem(), erro.subcodigo());
        if erro.saida() == 3 {
            eprintln!("IES: efeito incerto; conferir no SEI antes de qualquer nova prévia");
        }
    }
    ExitCode::from(erro.saida())
}
