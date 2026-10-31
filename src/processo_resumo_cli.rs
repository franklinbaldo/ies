//! Caso de uso de resumo: uma única leitura da caixa, sem abrir processo.
use crate::{
    Cli, Domain, Processo, configuracao, erro_fonte, erro_lista, paginas, processo,
    resumo_exportacao, resumo_processo, transporte,
};
use serde_json::json;
use std::path::Path;
use std::process::ExitCode;

fn mostrar_resumo_processo(
    numero: &str,
    unidade: Option<&str>,
    json_mode: bool,
    destino: Option<&Path>,
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
    if !leitura.recebidos.completa(&leitura.caixa.recebidos)
        || !leitura.gerados.completa(&leitura.caixa.gerados)
    {
        return erro_fonte(
            json_mode,
            "paginas_do_controle_nao_obtidas",
            "a caixa da unidade exige páginas que não foram obtidas",
        );
    }
    let achados = resumo_processo::ocorrencias(&leitura.caixa, numero);
    if achados.is_empty() {
        return erro_fonte(
            json_mode,
            "numero_nao_observado_na_caixa",
            "o número solicitado não aparece na Caixa de Entrada desta unidade; isso não prova que o processo não existe",
        );
    }
    let modelo = resumo_exportacao::modelo(numero, leitura.unidade.as_deref(), &achados);
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
            achados.len()
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
