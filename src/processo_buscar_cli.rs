//! `ies processo buscar TERMO [--pagina N]` (contracts/cli/processo-buscar.md):
//! valida o termo (saída 2 antes de ler a configuração), lê os arquivos
//! privados da sessão e publica a resposta observada da pesquisa Web.
use crate::pesquisa_web::fluxo::{BuscaRealizada, PedidoDeBusca, SessaoDaBusca, buscar_processo};
use crate::pesquisa_web::resposta::RespostaDaPesquisa;
use crate::processo::NumeroProcesso;
use crate::visao_do_controle::EnvioHttps;
use crate::{
    Cli, Domain, Processo, ProcessoAcao, configuracao, erro_fonte, saida_erro, transporte,
};
use serde_json::{Value, json};
use std::process::ExitCode;

/// Opções de leitura de `ies processo NUP` que não combinam com `buscar`.
fn opcoes_de_leitura_presentes(comando: &Processo) -> bool {
    comando.numero.is_some()
        || comando.resumo
        || comando.historico_privado.is_some()
        || comando.resultado_pesquisa_file.is_some()
        || comando.saida.is_some()
        || comando.caixa_local.is_some()
        || comando.observado_na_pagina
        || !comando.paginas_capturadas.is_empty()
        || comando.sem_documentos
        || comando.sem_marcadores
        || comando.sem_andamentos
        || comando.sem_propriedades
        || comando.andamentos_completos
}

fn erro_de_uso(json_mode: bool, subcodigo: &str, mensagem: &str) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "entrada_invalida",
                "subcodigo": subcodigo,
                "erro": mensagem,
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: {mensagem} [{subcodigo}]");
    }
    ExitCode::from(2)
}

/// O JSON da resposta observada; nunca HTML, links, cookies ou `infra_hash`.
pub(crate) fn json_da_busca(termo: &NumeroProcesso, busca: &BuscaRealizada) -> Value {
    let mut valor = json!({
        "termo": termo.as_str(),
        "unidade": busca.unidade,
        "origem": "web",
        "requisicoes": busca.requisicoes,
    });
    let extra = match &busca.resposta {
        // O 302 só traz o identificador interno: nenhum número é inventado.
        RespostaDaPesquisa::ProcessoUnico { id } => json!({
            "resultado": "processo_unico",
            "processo": {"id": id.as_str()},
        }),
        RespostaDaPesquisa::Lista(lista) => json!({
            "resultado": "lista",
            "total_declarado": lista.total_declarado,
            "faixa_observada": {"inicio": lista.faixa.inicio, "fim": lista.faixa.fim},
            "processos": lista
                .processos
                .iter()
                .map(|p| json!({"numero": p.numero.as_str()}))
                .collect::<Vec<_>>(),
        }),
        // Resposta observada para esta conta/unidade, não inexistência.
        RespostaDaPesquisa::SemResultado => json!({
            "resultado": "sem_resultado",
            "total_declarado": 0,
            "faixa_observada": null,
            "processos": [],
        }),
    };
    if let (Some(objeto), Value::Object(campos)) = (valor.as_object_mut(), extra) {
        objeto.extend(campos);
    }
    valor
}

fn humano(termo: &NumeroProcesso, busca: &BuscaRealizada) -> String {
    let detalhe = match &busca.resposta {
        RespostaDaPesquisa::ProcessoUnico { id } => {
            format!("um processo (id interno {})", id.as_str())
        }
        RespostaDaPesquisa::Lista(lista) => {
            let numeros: Vec<&str> = lista.processos.iter().map(|p| p.numero.as_str()).collect();
            format!(
                "{} de {} declarado(s), faixa {} a {}: {}",
                numeros.len(),
                lista.total_declarado,
                lista.faixa.inicio,
                lista.faixa.fim,
                numeros.join(", ")
            )
        }
        RespostaDaPesquisa::SemResultado => {
            "nenhum resultado para esta conta e unidade (não prova inexistência)".to_owned()
        }
    };
    format!(
        "IES: pesquisa de {:?} na unidade {}: {detalhe} ({} requisições).",
        termo.as_str(),
        busca.unidade,
        busca.requisicoes
    )
}

fn buscar(cli: &Cli, comando: &Processo, termo: &str, pagina: Option<u32>) -> ExitCode {
    if opcoes_de_leitura_presentes(comando) {
        return erro_de_uso(
            cli.json,
            "opcoes_incompativeis_com_busca",
            "opções de leitura do processo não combinam com buscar",
        );
    }
    let termo = match NumeroProcesso::novo(termo) {
        Ok(termo) => termo,
        Err(erro) => return erro_de_uso(cli.json, erro.subcodigo(), "termo de pesquisa inválido"),
    };
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => return erro_fonte(cli.json, erro.subcodigo(), erro.mensagem()),
    };
    let lidos = transporte::arquivo_privado(&arquivos.url_controle, 8_192).and_then(|url| {
        transporte::arquivo_privado(&arquivos.origem, 512).map(|origem| (url, origem))
    });
    let (url, origem) = match lidos {
        Ok(lidos) => lidos,
        Err(falha) => return erro_fonte(cli.json, falha.subcodigo(), falha.mensagem()),
    };
    let sessao = SessaoDaBusca {
        url_do_controle: url.trim(),
        origem: origem.trim(),
        cookie_file: &arquivos.cookie,
    };
    let pedido = PedidoDeBusca {
        termo: &termo,
        pagina,
        unidade: cli.unidade.as_deref(),
    };
    match buscar_processo(&sessao, &pedido, &mut EnvioHttps) {
        Ok(busca) => {
            if cli.json {
                let valor = json_da_busca(&termo, &busca);
                println!("{}", serde_json::to_string_pretty(&valor).unwrap());
            } else {
                println!("{}", humano(&termo, &busca));
            }
            ExitCode::SUCCESS
        }
        Err(erro) => saida_erro::erro_fonte_http(
            cli.json,
            erro.subcodigo(),
            erro.mensagem(),
            erro.status_http(),
        ),
    }
}

/// `Some` só para `ies processo buscar`.
pub(crate) fn tentar_buscar(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(comando) = &cli.command else {
        return None;
    };
    let Some(ProcessoAcao::Buscar { termo, pagina }) = &comando.acao else {
        return None;
    };
    Some(buscar(cli, comando, termo, *pagina))
}

#[cfg(test)]
#[path = "processo_buscar_cli/tests.rs"]
mod tests;
