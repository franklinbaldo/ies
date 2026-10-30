//! Consulta do Controle de Processos: apresenta ou grava somente a caixa validada.
use crate::{arquivo_caixa, configuracao, dominio, erro_fonte, paginas, saida, transporte};
use serde_json::json;
use std::process::ExitCode;

pub(super) fn listar_processos_da_unidade(
    unidade: Option<&str>,
    json_mode: bool,
    destino: Option<&std::path::Path>,
) -> ExitCode {
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => return erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
    };
    let html = match transporte::read_box_from_file(
        &arquivos.url_controle,
        &arquivos.cookie,
        &arquivos.origem,
    ) {
        Ok(html) => html,
        Err(falha) => return erro_fonte(json_mode, falha.subcodigo(), falha.mensagem()),
    };
    mostrar_caixa(&[html], "web", unidade, json_mode, true, destino)
}

pub(super) fn mostrar_caixa(
    htmls: &[String],
    origem: &str,
    unidade: Option<&str>,
    json_mode: bool,
    exigir_todas_paginas: bool,
    destino: Option<&std::path::Path>,
) -> ExitCode {
    let leitura = match paginas::juntar(htmls, unidade) {
        Ok(valor) => valor,
        Err(erro) => {
            if json_mode {
                println!(
                    "{}",
                    json!({
                        "codigo": "tela_inesperada",
                        "subcodigo": erro.subcodigo(),
                        "erro": erro.to_string(),
                        "categoria": erro.categoria().map(dominio::Categoria::rotulo),
                        "id": erro.id(),
                        "retentavel": false
                    })
                );
            } else {
                eprintln!("IES: {erro}");
            }
            return ExitCode::from(1);
        }
    };
    // Consulta de sessão promete a caixa inteira; replay offline só relata
    // as páginas selecionadas, sem alegar que representam toda a unidade.
    if exigir_todas_paginas
        && (!leitura.recebidos.completa(&leitura.caixa.recebidos)
            || !leitura.gerados.completa(&leitura.caixa.gerados))
    {
        return erro_fonte(
            json_mode,
            "paginas_do_controle_nao_obtidas",
            "o Controle de Processos possui páginas que não foram obtidas",
        );
    }
    let resultado = saida::json(&leitura, origem);
    if destino.is_some() {
        return erro_fonte(
            json_mode,
            "exportacao_da_caixa_nao_implementada",
            "exportação local da caixa ainda não implementada",
        );
    }
    if json_mode {
        println!("{}", serde_json::to_string_pretty(&resultado).unwrap());
    } else {
        println!("{}", saida::humano(&leitura));
    }
    ExitCode::SUCCESS
}

