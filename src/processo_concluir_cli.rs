//! `ies processo concluir NUP [--confirmar]` (contracts/cli/processo-concluir.md):
//! NUP inválido sai com 2 antes de qualquer arquivo; prévia sem escrita;
//! com `--confirmar`, uma escrita conferida por leitura; efeito incerto, 3.
//! A saída nunca traz URL, `infra_hash` ou cookie.
use crate::indice_local::IndiceLocal;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::{
    ErroConcluir, FalhaDoEnvio, PedidoDaConclusao, ProcessoConcluido, concluir_processo,
};
use crate::processo_obter::SessaoDoProcesso;
use crate::transporte::StatusHttpRecusado;
use crate::visao_do_controle::EnvioHttps;
use crate::{
    Cli, Domain, Processo, ProcessoAcao, configuracao, erro_fonte, saida_erro, transporte,
};
use serde_json::{Map, Value, json};
use std::process::ExitCode;

/// `Some` só para `ies processo concluir`.
pub(crate) fn tentar_concluir(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao: Some(ProcessoAcao::Concluir { numero, confirmar }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    Some(match NumeroProcesso::novo(numero) {
        Ok(nup) => concluir(cli, &nup, *confirmar),
        Err(erro) => erro_de_uso(cli.json, erro.subcodigo()),
    })
}

pub(crate) fn erro_de_uso(json_mode: bool, subcodigo: &str) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "entrada_invalida",
                "subcodigo": subcodigo,
                "erro": "número de processo inválido",
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: número de processo inválido [{subcodigo}]");
    }
    ExitCode::from(2)
}

/// Lê os arquivos da sessão e chama `executar` com ela e o índice; a falha
/// de configuração sai com 1 `fonte_invalida`, sem rede (também a de
/// `processo reabrir`).
pub(crate) fn com_sessao(
    cli: &Cli,
    executar: impl FnOnce(&SessaoDoProcesso<'_>, &IndiceLocal) -> ExitCode,
) -> ExitCode {
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => return erro_fonte(cli.json, erro.subcodigo(), erro.mensagem()),
    };
    let lidos = transporte::sessao::ReferenciasDaSessao::ler(
        &arquivos.url_controle,
        &arquivos.origem,
        &arquivos.cookie,
    );
    let referencias = match lidos {
        Ok(referencias) => referencias,
        Err(falha) => return erro_fonte(cli.json, falha.subcodigo(), falha.mensagem()),
    };
    let sessao = SessaoDoProcesso {
        url_do_controle: referencias.url_do_controle(),
        origem: referencias.origem(),
        cookie: referencias.cookie(),
    };
    executar(&sessao, &IndiceLocal::da_configuracao(&arquivos))
}

fn concluir(cli: &Cli, nup: &NumeroProcesso, confirmar: bool) -> ExitCode {
    com_sessao(cli, |sessao, indice| {
        concluir_na_sessao(cli, sessao, indice, nup, confirmar)
    })
}

fn concluir_na_sessao(
    cli: &Cli,
    sessao: &SessaoDoProcesso<'_>,
    indice: &IndiceLocal,
    nup: &NumeroProcesso,
    confirmar: bool,
) -> ExitCode {
    let pedido = PedidoDaConclusao {
        nup,
        unidade: cli.unidade.as_deref(),
        confirmar,
    };
    match concluir_processo(sessao, &pedido, indice, &mut EnvioHttps) {
        Ok(feito) => {
            if cli.json {
                if let Err(erro) = crate::saida::imprimir_json(&json_de_sucesso(&feito)) {
                    eprintln!("IES: falha ao emitir JSON: {erro}");
                    return std::process::ExitCode::from(1);
                }
            } else {
                println!("{}", humano(&feito));
            }
            ExitCode::SUCCESS
        }
        Err(ErroConcluir::Abertura(erro)) => saida_erro::erro_fonte_http(
            cli.json,
            erro.subcodigo(),
            erro.mensagem(),
            erro.status_http(),
        ),
        Err(erro) => publicar_falha(cli.json, &erro),
    }
}

pub(crate) fn json_da_falha(falha: &FalhaDoEnvio) -> Value {
    let mut valor = Map::new();
    valor.insert("subcodigo".to_owned(), json!(falha.subcodigo()));
    if let Some(http) = falha.status_http() {
        if http.status != 200 {
            valor.insert("status_http".to_owned(), json!(http.status));
        }
        if let Some(texto) = http.mensagem_do_servidor {
            valor.insert("mensagem_do_servidor".to_owned(), json!(texto));
        }
    }
    Value::Object(valor)
}

/// JSON do sucesso (prévia, nada a fazer, ou concluído e conferido).
pub(crate) fn json_de_sucesso(feito: &ProcessoConcluido) -> Value {
    let mut valor = json!({
        "nup": feito.nup,
        "id_procedimento": feito.id_procedimento,
        "unidade": feito.unidade,
        "origem": "web",
        "confirmado": feito.confirmado,
        "alterado": feito.alterado,
        "forma": feito.forma.map(crate::processo_concluir::FormaDaConclusao::como_texto),
        "aberto_em": feito.aberto_em,
        "depois_esperado": feito.depois_esperado(),
        "depois": feito.depois,
        "reconciliado_por_leitura": feito.reconciliado_por.is_some(),
        "requisicoes": feito.requisicoes,
    });
    if let (Some(falha), Some(objeto)) = (&feito.reconciliado_por, valor.as_object_mut()) {
        objeto.insert("falha_do_envio".to_owned(), json_da_falha(falha));
    }
    valor
}

/// O corpo comum das falhas depois da abertura (também as de `processo
/// reabrir`): códigos, mensagem e o status HTTP, quando houver.
pub(crate) fn json_base_da_falha(
    codigo: &str,
    subcodigo: &str,
    mensagem: &str,
    status: Option<StatusHttpRecusado>,
) -> Value {
    let mut valor = json!({
        "codigo": codigo,
        "subcodigo": subcodigo,
        "erro": mensagem,
        "retentavel": false,
    });
    if let (Some(http), Some(objeto)) = (status, valor.as_object_mut()) {
        if http.status != 200 {
            objeto.insert("status_http".to_owned(), json!(http.status));
        }
        if let Some(texto) = http.mensagem_do_servidor {
            objeto.insert("mensagem_do_servidor".to_owned(), json!(texto));
        }
    }
    valor
}

/// JSON da falha depois da abertura.
pub(crate) fn json_de_falha(erro: &ErroConcluir) -> Value {
    let mut valor = json_base_da_falha(
        erro.codigo(),
        erro.subcodigo(),
        erro.mensagem(),
        erro.status_http(),
    );
    let Some(objeto) = valor.as_object_mut() else {
        return valor;
    };
    match erro {
        ErroConcluir::SituacaoDoProcessoNaoLidaAntesDeConcluir { motivo } => {
            objeto.insert("motivo".to_owned(), json!(motivo.como_texto()));
        }
        ErroConcluir::UnidadeDaSessaoAusenteDaSituacao { aberto_em }
        | ErroConcluir::ConclusaoNaoPublicadaNaBarra { aberto_em } => {
            objeto.insert("aberto_em".to_owned(), json!(aberto_em));
        }
        ErroConcluir::ConclusaoNaoAplicada {
            aberto_em,
            depois,
            falha_do_envio,
        } => {
            objeto.insert("aberto_em".to_owned(), json!(aberto_em));
            objeto.insert("depois".to_owned(), json!(depois));
            if let Some(falha) = falha_do_envio {
                objeto.insert("falha_do_envio".to_owned(), json_da_falha(falha));
            }
        }
        ErroConcluir::EfeitoIncertoNaConclusao {
            aberto_em,
            falha_do_envio,
            verificacao,
        } => {
            objeto.insert("aberto_em".to_owned(), json!(aberto_em));
            objeto.insert("verificacao".to_owned(), json!(verificacao.motivo()));
            if let Some(falha) = falha_do_envio {
                objeto.insert("falha_do_envio".to_owned(), json_da_falha(falha));
            }
        }
        _ => {}
    }
    valor
}

fn publicar_falha(json_mode: bool, erro: &ErroConcluir) -> ExitCode {
    publicar_valor_da_falha(json_mode, &json_de_falha(erro), erro.saida())
}

/// Publica a falha já em JSON (também a de `processo reabrir`): o JSON, ou
/// a mensagem, as listas, a resposta da escrita e a verificação no stderr.
pub(crate) fn publicar_valor_da_falha(json_mode: bool, valor: &Value, saida: u8) -> ExitCode {
    if json_mode {
        println!("{valor}");
    } else {
        eprintln!(
            "IES: {} [{}]",
            valor["erro"].as_str().unwrap_or_default(),
            valor["subcodigo"].as_str().unwrap_or_default()
        );
        for campo in ["aberto_em", "depois"] {
            if let Some(lista) = valor[campo].as_array() {
                let siglas: Vec<_> = lista.iter().filter_map(Value::as_str).collect();
                eprintln!("IES: {campo}: {}", lista_humana(&siglas));
            }
        }
        if let Some(falha) = valor["falha_do_envio"]["subcodigo"].as_str() {
            eprintln!("IES: resposta da escrita: {falha}");
        }
        if let Some(motivo) = valor["verificacao"].as_str() {
            eprintln!("IES: verificação: {motivo}");
        }
    }
    ExitCode::from(saida)
}

pub(crate) fn lista_humana(siglas: &[&str]) -> String {
    if siglas.is_empty() {
        "(nenhuma unidade)".to_owned()
    } else {
        siglas.join(", ")
    }
}

/// Saída humana do sucesso.
pub(crate) fn humano(feito: &ProcessoConcluido) -> String {
    let refs = |v: &[String]| lista_humana(&v.iter().map(String::as_str).collect::<Vec<_>>());
    let mut linhas = vec![
        format!("Processo {} na unidade {}", feito.nup, feito.unidade),
        format!("Aberto em: {}", refs(&feito.aberto_em)),
    ];
    match (feito.forma, feito.confirmado, &feito.depois) {
        (None, _, _) => linhas.push(
            "Nada a fazer: o processo não está aberto nesta unidade e a barra não oferece concluir."
                .to_owned(),
        ),
        (Some(forma), false, _) => {
            linhas.push(format!("Ação na barra: {}", forma.como_texto()));
            linhas.push(format!("Depois de concluir: {}", refs(&feito.depois_esperado())));
            linhas.push("Para concluir, repita com --confirmar".to_owned());
        }
        (Some(_), true, depois) => {
            let depois = depois.as_deref().unwrap_or_default();
            linhas.push(format!("Concluído. Aberto em, relido: {}", refs(depois)));
            if let Some(falha) = &feito.reconciliado_por {
                linhas.push(format!(
                    "A resposta não foi a esperada ({}); a leitura confirmou.",
                    falha.subcodigo()
                ));
            }
        }
    }
    linhas.push(format!("Requisições: {}", feito.requisicoes));
    linhas.join("\n")
}

#[cfg(test)]
#[path = "processo_concluir_cli/tests.rs"]
mod tests;
