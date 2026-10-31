//! Caso de uso de resumo: uma única leitura da caixa, sem abrir processo.
use crate::{
    Cli, Domain, Processo, configuracao, erro_fonte, erro_lista, paginas, processo,
    resumo_exportacao, resumo_pagina, transporte,
};
use serde_json::json;
use std::path::Path;
use std::process::ExitCode;

fn mostrar_resumo_processo(
    numero: &str,
    unidade: Option<&str>,
    json_mode: bool,
    destino: Option<&Path>,
    aceitar_pagina_parcial: bool,
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
        Err(erro) => return erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
    };
    let leitura = match paginas::juntar(&[html], unidade) {
        Ok(leitura) => leitura,
        Err(erro) => {
            return erro_fonte(
                json_mode,
                erro.subcodigo(),
                "Controle de Processos não reconhecido",
            );
        }
    };
    let observacao = match resumo_pagina::observar(&leitura, numero, aceitar_pagina_parcial) {
        Ok(observacao) => observacao,
        Err(erro) => return erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
    };
    let mut modelo = resumo_exportacao::modelo(
        numero,
        leitura.unidade.as_deref(),
        &observacao.ocorrencias,
    );
    if observacao.pagina_parcial {
        modelo["escopo"] = json!("resumo_da_pagina");
        modelo["demais_paginas"] = json!("nao_consultadas");
    }
    if let Some(destino) = destino {
        return match resumo_exportacao::gravar(destino, &modelo) {
            Ok(bytes) => {
                if json_mode {
                    println!(
                        "{}",
                        json!({
                            "arquivo": "gravado",
                            "bytes": bytes,
                            "escopo": "resumo_da_caixa"
                        })
                    );
                } else {
                    println!("IES: resumo da caixa gravado em arquivo privado ({bytes} bytes).");
                }
                ExitCode::SUCCESS
            }
            Err(erro) => erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
        };
    }
    if json_mode {
        println!("{modelo}");
    } else {
        println!(
            "IES: processo {numero} observado na caixa da unidade ({} ocorrência(s)); detalhes não consultados.",
            observacao.ocorrencias.len()
        );
    }
    ExitCode::SUCCESS
}

pub(super) fn tentar_resumo(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        numero: Some(numero),
        resumo: true,
        andamentos_completos,
        saida,
        observado_na_pagina,
        acao: None,
        ..
    }) = &cli.command
    else {
        return None;
    };
    if *andamentos_completos {
        return Some(erro_lista(
            cli.json,
            "opcoes_incompativeis",
            "o resumo da caixa não inclui andamentos",
            2,
        ));
    }
    processo::NumeroProcesso::novo(numero).ok()?;
    Some(mostrar_resumo_processo(
        numero,
        cli.unidade.as_deref(),
        cli.json,
        saida.as_deref(),
        *observado_na_pagina,
    ))
}

pub(super) fn validar_numero(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        numero: Some(numero),
        acao: None,
        ..
    }) = &cli.command
    else {
        return None;
    };
    let Err(erro) = processo::NumeroProcesso::novo(numero) else {
        return None;
    };
    if cli.json {
        println!(
            "{}",
            json!({
                "codigo": "entrada_invalida",
                "subcodigo": erro.subcodigo(),
                "erro": "número de processo inválido",
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: número de processo inválido [{}]", erro.subcodigo());
    }
    Some(ExitCode::from(2))
}
