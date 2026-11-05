//! Entrada pública de documento criar (#246).
//! Confirmação ainda não implementada: recusa antes de carregar sessão,
//! e portanto antes de qualquer requisição ou escrita no SEI.
use crate::documento_criar::{ErroPreviaCriacao, PedidoCriacao, PreviaCriacao, obter_previa};
use crate::processo::NumeroProcesso;
use crate::processo_concluir_cli::com_sessao;
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Documento, DocumentoAcao, Domain, saida_erro};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn tentar_criar(cli: &Cli) -> Option<ExitCode> {
    let Domain::Documento(Documento {
        acao: Some(DocumentoAcao::Criar {
            processo,
            tipo,
            descricao,
            nivel,
            confirmar,
        }),
        ..
    }) = &cli.command else {
        return None;
    };
    let nup = match NumeroProcesso::novo(processo) {
        Ok(nup) => nup,
        Err(erro) => return Some(entrada_invalida(cli.json, erro.subcodigo())),
    };
    if tipo.trim().is_empty() {
        return Some(entrada_invalida(cli.json, "tipo_de_documento_vazio"));
    }
    if *confirmar {
        return Some(saida_erro::erro_fonte_http(
            cli.json,
            "criacao_de_documento_ainda_nao_validada",
            "confirmação indisponível: falta integrar e verificar o POST do cadastro",
            None,
        ));
    }
    Some(com_sessao(cli, |sessao, indice| {
        let pedido = PedidoCriacao {
            nup: &nup,
            unidade: cli.unidade.as_deref(),
            tipo,
            descricao: descricao.as_deref(),
            nivel: nivel.as_deref(),
        };
        match obter_previa(sessao, &pedido, indice, &mut EnvioHttps) {
            Ok(previa) => publicar_previa(cli.json, &previa),
            Err(erro) => publicar_erro(cli.json, &erro),
        }
    }))
}

fn entrada_invalida(json_mode: bool, subcodigo: &str) -> ExitCode {
    if json_mode {
        println!("{}", json!({
            "codigo": "entrada_invalida",
            "subcodigo": subcodigo,
            "erro": "parâmetro de criação de documento inválido",
            "retentavel": false
        }));
    } else {
        eprintln!("IES: parâmetro de criação de documento inválido [{subcodigo}]");
    }
    ExitCode::from(2)
}

fn publicar_previa(json_mode: bool, previa: &PreviaCriacao) -> ExitCode {
    if json_mode {
        println!("{}", json!({
            "nup": previa.nup,
            "tipo": previa.tipo,
            "unidade": previa.unidade,
            "origem": "web",
            "confirmado": false,
            "alterado": false,
            "requisicoes": previa.requisicoes
        }));
    } else {
        println!(
            "Prévia: criar documento {} no processo {} (unidade {}).",
            previa.tipo, previa.nup, previa.unidade
        );
        println!("Requisições: {}", previa.requisicoes);
        println!("Confirmação ainda não disponível; nenhum documento foi criado.");
    }
    ExitCode::SUCCESS
}

fn publicar_erro(json_mode: bool, erro: &ErroPreviaCriacao) -> ExitCode {
    saida_erro::erro_fonte_http(
        json_mode,
        erro.subcodigo(),
        "prévia da criação do documento não pôde ser conferida",
        erro.status_http(),
    )
}
