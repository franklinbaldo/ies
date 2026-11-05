//! Entrada pública de documento criar (#246).
//! A confirmação deriva um único POST do cadastro e só reconhece o efeito
//! depois de releitura da árvore. Não há retry.
use crate::documento_criar::{ErroPreviaCriacao, PedidoCriacao, PreviaCriacao, executar_criacao};
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
    Some(com_sessao(cli, |sessao, indice| {
        let pedido = PedidoCriacao {
            nup: &nup,
            unidade: cli.unidade.as_deref(),
            tipo,
            descricao: descricao.as_deref(),
            nivel: nivel.as_deref(),
        };
        match executar_criacao(sessao, &pedido, indice, &mut EnvioHttps, *confirmar) {
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
            "confirmado": previa.confirmado,
            "alterado": previa.confirmado,
            "numero_sei": previa.numero_sei,
            "requisicoes": previa.requisicoes
        }));
    } else {
        if previa.confirmado {
            println!(
                "Documento {} criado no processo {} (unidade {}), número SEI {}.",
                previa.tipo,
                previa.nup,
                previa.unidade,
                previa.numero_sei.as_deref().unwrap_or("não observado")
            );
        } else {
            println!(
                "Prévia: criar documento {} no processo {} (unidade {}).",
                previa.tipo, previa.nup, previa.unidade
            );
            println!("Nenhum documento foi criado.");
        }
        println!("Requisições: {}", previa.requisicoes);
    }
    ExitCode::SUCCESS
}

fn publicar_erro(json_mode: bool, erro: &ErroPreviaCriacao) -> ExitCode {
    if matches!(erro, ErroPreviaCriacao::EfeitoIncerto) {
        if json_mode {
            println!("{}", json!({
                "codigo": "efeito_incerto",
                "subcodigo": erro.subcodigo(),
                "erro": "escrita enviada; não é possível comprovar o efeito por leitura",
                "retentavel": false
            }));
        } else {
            eprintln!("IES: efeito incerto na criação do documento; não repita --confirmar");
        }
        return ExitCode::from(3);
    }
    saida_erro::erro_fonte_http(
        json_mode,
        erro.subcodigo(),
        "criação do documento não pôde ser conferida",
        erro.status_http(),
    )
}
