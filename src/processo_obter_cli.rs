//! `ies processo NUP` sem `--resumo` (contracts/cli/processo-obter-arvore.md):
//! lê os arquivos privados da sessão e o índice local, abre o processo e
//! publica identidade e primeiro nível da árvore. Nunca URL, hash ou HTML.
use crate::indice_local::IndiceLocal;
use crate::processo::NumeroProcesso;
use crate::processo_obter::{PedidoDoProcesso, ProcessoAberto, SessaoDoProcesso, obter_processo};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, configuracao, erro_fonte, saida_erro, transporte};
use serde_json::{Value, json};
use std::fmt::Write as _;
use std::process::ExitCode;

/// Dados que esta fatia não lê (`null` seria "não observado").
const NAO_LIDOS: [&str; 3] = ["andamentos", "marcadores", "propriedades"];

/// O JSON publicado; `documentos` some com `--sem-documentos`.
pub(crate) fn json_do_processo(
    nup: &NumeroProcesso,
    aberto: &ProcessoAberto,
    sem_documentos: bool,
) -> Value {
    let processo = &aberto.arvore.processo;
    let mut valor = json!({
        "numero": nup.as_str(),
        "id_procedimento": processo.id,
        "tipo": processo.tipo,
        "unidade": aberto.unidade,
        "resolvido_por": aberto.resolvido_por.como_texto(),
        "requisicoes": aberto.requisicoes,
        "nao_lidos": NAO_LIDOS,
    });
    if !sem_documentos && let Some(objeto) = valor.as_object_mut() {
        let documentos: Vec<Value> = aberto
            .arvore
            .documentos
            .iter()
            .map(|d| {
                json!({
                    "id_documento": d.id,
                    "rotulo": d.rotulo,
                    "numero_sei": d.numero_sei,
                })
            })
            .collect();
        objeto.insert("documentos".to_owned(), Value::Array(documentos));
    }
    valor
}

fn humano(nup: &NumeroProcesso, aberto: &ProcessoAberto, sem_documentos: bool) -> String {
    let processo = &aberto.arvore.processo;
    let unidade = aberto
        .unidade
        .as_deref()
        .unwrap_or("não publicada pela árvore");
    let mut texto = format!(
        "IES: processo {} (id {}, {}), unidade {unidade}; resolvido por {} em {} requisições.",
        nup.as_str(),
        processo.id,
        processo.tipo,
        aberto.resolvido_por.como_texto(),
        aberto.requisicoes,
    );
    if !sem_documentos {
        let _ = write!(
            texto,
            "\n{} documento(s) no primeiro nível da árvore:",
            aberto.arvore.documentos.len()
        );
        for documento in &aberto.arvore.documentos {
            let _ = write!(texto, "\n- {} (id {})", documento.rotulo, documento.id);
        }
    }
    texto.push_str("\nNão lidos nesta versão: andamentos, marcadores e propriedades.");
    texto
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

fn obter(cli: &Cli, comando: &Processo, nup: &NumeroProcesso) -> ExitCode {
    if comando.andamentos_completos {
        return erro_de_uso(
            cli.json,
            "andamentos_completos_nao_implementados",
            "a leitura do histórico completo ainda não existe",
        );
    }
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
    let sessao = SessaoDoProcesso {
        url_do_controle: url.trim(),
        origem: origem.trim(),
        cookie_file: &arquivos.cookie,
    };
    let pedido = PedidoDoProcesso {
        nup,
        unidade: cli.unidade.as_deref(),
    };
    let indice = IndiceLocal::da_configuracao(&arquivos);
    match obter_processo(&sessao, &pedido, &indice, &mut EnvioHttps) {
        Ok(aberto) => {
            if cli.json {
                let valor = json_do_processo(nup, &aberto, comando.sem_documentos);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&valor).unwrap_or_default()
                );
            } else {
                println!("{}", humano(nup, &aberto, comando.sem_documentos));
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

/// `Some` só para `ies processo NUP` sem `--resumo` nem fonte offline; NUP
/// inválido fica para a validação comum (saída 2).
pub(crate) fn tentar_obter(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(
        comando @ Processo {
            numero: Some(numero),
            resumo: false,
            historico_privado: None,
            resultado_pesquisa_file: None,
            acao: None,
            ..
        },
    ) = &cli.command
    else {
        return None;
    };
    let nup = NumeroProcesso::novo(numero).ok()?;
    Some(obter(cli, comando, &nup))
}

#[cfg(test)]
#[path = "processo_obter_cli/tests.rs"]
mod tests;
