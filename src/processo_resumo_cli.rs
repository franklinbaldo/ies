//! Caso de uso de resumo: uma única leitura da caixa, sem abrir processo.
use crate::{
    Cli, Domain, Processo, configuracao, erro_fonte, erro_lista, paginas, processo,
    resumo_processo, transporte,
};
use serde_json::json;
use std::process::ExitCode;

fn mostrar_resumo_processo(numero: &str, unidade: Option<&str>, json_mode: bool) -> ExitCode {
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
    if json_mode {
        println!(
            "{}",
            json!({
                "numero": numero,
                "escopo": "resumo_da_caixa",
                "ocorrencias": achados,
                "unidade": leitura.unidade
            })
        );
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
    ))
}
