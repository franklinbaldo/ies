//! `ies processo NUP` sem `--resumo` (contracts/cli/processo-obter-arvore.md
//! e processo-obter-andamentos.md): lê os arquivos privados da sessão e o
//! índice local, abre o processo e publica identidade, documentos e pastas
//! da árvore e andamentos. Nunca URL, hash ou HTML.
use crate::arvore_processo::{PastaNaArvore, VerificacaoDaPasta};
use crate::historico_processo::AndamentosLidos;
use crate::indice_local::IndiceLocal;
use crate::processo::NumeroProcesso;
use crate::processo_obter::{
    Andamentos, PedidoDeAndamentos, PedidoDoProcesso, ProcessoAberto, SessaoDoProcesso,
    obter_processo,
};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, configuracao, erro_fonte, saida_erro, transporte};
use serde_json::{Value, json};
use std::fmt::Write as _;
use std::process::ExitCode;

/// Dados que o comando ainda não lê (`null` seria "não observado").
const NAO_LIDOS: [&str; 2] = ["marcadores", "propriedades"];

fn json_dos_andamentos(lidos: &AndamentosLidos) -> Value {
    let itens: Vec<Value> = lidos
        .itens
        .iter()
        .map(|a| {
            json!({
                "ocorrido_em": a.ocorrido_em,
                "unidade": a.unidade,
                "usuario": a.usuario,
                "descricao": a.descricao,
            })
        })
        .collect();
    json!({
        "tipo": lidos.tipo.como_texto(),
        "total_declarado": lidos.total_declarado,
        "faixa_observada": {"inicio": 1, "fim": lidos.itens.len()},
        "itens": itens,
    })
}

/// A verificação mais fraca entre as pastas conferidas; `None` sem pastas
/// (ou sem nenhuma conferida).
fn verificacao_das_pastas(pastas: &[PastaNaArvore]) -> Option<VerificacaoDaPasta> {
    let regras: Vec<VerificacaoDaPasta> = pastas.iter().filter_map(|p| p.verificacao).collect();
    if regras.contains(&VerificacaoDaPasta::PorContagem) {
        Some(VerificacaoDaPasta::PorContagem)
    } else {
        regras.first().copied()
    }
}

/// O JSON publicado; `documentos` some com `--sem-documentos` e
/// `andamentos` com `--sem-andamentos` (link não publicado: `null`).
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
    });
    let mut nao_lidos: Vec<&str> = NAO_LIDOS.to_vec();
    if let Some(objeto) = valor.as_object_mut() {
        match &aberto.andamentos {
            Andamentos::Lidos(lidos) => {
                objeto.insert("andamentos".to_owned(), json_dos_andamentos(lidos));
            }
            Andamentos::LinkNaoPublicadoNaArvore => {
                objeto.insert("andamentos".to_owned(), Value::Null);
            }
            Andamentos::Omitidos => {}
        }
        if let Some(motivo) = aberto.andamentos.motivo() {
            nao_lidos.insert(0, "andamentos");
            objeto.insert("andamentos_nao_lidos_motivo".to_owned(), json!(motivo));
        }
        objeto.insert("nao_lidos".to_owned(), json!(nao_lidos));
    }
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
                    "pasta": d.pasta,
                })
            })
            .collect();
        objeto.insert("documentos".to_owned(), Value::Array(documentos));
        let pastas: Vec<Value> = aberto
            .arvore
            .pastas
            .iter()
            .map(|p| {
                json!({
                    "id": p.id,
                    "rotulo": p.rotulo,
                    "total": p.total_declarado,
                    "verificacao": p.verificacao.map(VerificacaoDaPasta::como_texto),
                })
            })
            .collect();
        objeto.insert("pastas".to_owned(), Value::Array(pastas));
        let verificacao = verificacao_das_pastas(&aberto.arvore.pastas);
        objeto.insert(
            "verificacao_das_pastas".to_owned(),
            json!(verificacao.map(VerificacaoDaPasta::como_texto)),
        );
        if verificacao == Some(VerificacaoDaPasta::PorContagem) {
            objeto.insert(
                "verificacao_das_pastas_motivo".to_owned(),
                json!("barras_vazias"),
            );
        }
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
        let arvore = &aberto.arvore;
        let _ = write!(
            texto,
            "\n{} documento(s) na árvore, {} pasta(s){}:",
            arvore.documentos.len(),
            arvore.pastas.len(),
            verificacao_das_pastas(&arvore.pastas)
                .map(|v| format!(" conferidas {}", v.como_texto()))
                .unwrap_or_default()
        );
        for documento in &arvore.documentos {
            let _ = write!(texto, "\n- {} (id {})", documento.rotulo, documento.id);
            let pasta = documento
                .pasta
                .as_ref()
                .and_then(|id| arvore.pastas.iter().find(|p| &p.id == id));
            if let Some(pasta) = pasta {
                let _ = write!(texto, " [pasta {}]", pasta.rotulo);
            }
        }
    }
    match &aberto.andamentos {
        Andamentos::Lidos(lidos) => {
            let _ = write!(
                texto,
                "\nAndamentos (visão {}): {} de {} declarados.",
                lidos.tipo.como_texto(),
                lidos.itens.len(),
                lidos.total_declarado
            );
            for a in &lidos.itens {
                let _ = write!(
                    texto,
                    "\n- {} {}: {}",
                    a.ocorrido_em.as_deref().unwrap_or("?"),
                    a.unidade.as_deref().unwrap_or("?"),
                    a.descricao
                );
            }
        }
        outro => {
            let _ = write!(
                texto,
                "\nAndamentos não lidos ({}).",
                outro.motivo().unwrap_or_default()
            );
        }
    }
    texto.push_str("\nNão lidos nesta versão: marcadores e propriedades.");
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
    if comando.andamentos_completos && comando.sem_andamentos {
        return erro_de_uso(
            cli.json,
            "sem_andamentos_incompativel_com_andamentos_completos",
            "--sem-andamentos e --andamentos-completos são incompatíveis",
        );
    }
    let andamentos = if comando.sem_andamentos {
        PedidoDeAndamentos::Omitidos
    } else if comando.andamentos_completos {
        PedidoDeAndamentos::Completos
    } else {
        PedidoDeAndamentos::VisaoServida
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
    let sessao = SessaoDoProcesso {
        url_do_controle: url.trim(),
        origem: origem.trim(),
        cookie_file: &arquivos.cookie,
    };
    let pedido = PedidoDoProcesso {
        nup,
        unidade: cli.unidade.as_deref(),
        andamentos,
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
