//! `ies processo enviar NUP --para SIGLA [--manter-aberto] [--confirmar]`
//! (contracts/cli/processo-enviar.md): uso inválido sai com 2 antes de
//! qualquer arquivo; prévia sem POST de escrita; com `--confirmar`, um POST
//! conferido por leitura; efeito incerto, 3. A saída nunca traz URL,
//! `infra_hash` ou cookie.
use crate::processo::NumeroProcesso;
use crate::processo_concluir_cli::{
    com_sessao, erro_de_uso, json_base_da_falha, json_da_falha, lista_humana,
    publicar_valor_da_falha,
};
use crate::processo_enviar::{ErroEnviar, PedidoDoEnvio, ProcessoEnviado, enviar_processo};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao, saida_erro};
use serde_json::{Value, json};
use std::process::ExitCode;

/// `Some` só para `ies processo enviar`.
pub(crate) fn tentar_enviar(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao:
            Some(ProcessoAcao::Enviar {
                numero,
                para,
                manter_aberto,
                confirmar,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    let nup = match NumeroProcesso::novo(numero) {
        Ok(nup) => nup,
        Err(erro) => return Some(erro_de_uso(cli.json, erro.subcodigo())),
    };
    let sigla = match sigla_de_destino(para) {
        Ok(sigla) => sigla,
        Err(subcodigo) => return Some(destino_invalido(cli.json, subcodigo)),
    };
    Some(enviar(cli, &nup, sigla, *manter_aberto, *confirmar))
}

/// A única sigla de `--para`, aparada; vazia, com controle ou longa é uso
/// inválido.
pub(crate) fn sigla_de_destino(para: &[String]) -> Result<&str, &'static str> {
    let [sigla] = para else {
        return Err("mais_de_um_destino_nao_suportado");
    };
    let sigla = sigla.trim();
    if sigla.is_empty() || sigla.chars().count() > 100 || sigla.chars().any(char::is_control) {
        return Err("sigla_de_destino_invalida");
    }
    Ok(sigla)
}

fn destino_invalido(json_mode: bool, subcodigo: &str) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "entrada_invalida",
                "subcodigo": subcodigo,
                "erro": "destino inválido: informe uma sigla de unidade em --para",
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: destino inválido: informe uma sigla de unidade em --para [{subcodigo}]");
    }
    ExitCode::from(2)
}

fn enviar(
    cli: &Cli,
    nup: &NumeroProcesso,
    para: &str,
    manter_aberto: bool,
    confirmar: bool,
) -> ExitCode {
    com_sessao(cli, |sessao, indice| {
        let pedido = PedidoDoEnvio {
            nup,
            unidade: cli.unidade.as_deref(),
            para,
            manter_aberto,
            confirmar,
        };
        match enviar_processo(sessao, &pedido, indice, &mut EnvioHttps) {
            Ok(feito) => {
                if cli.json {
                    if let Err(erro) = crate::saida::imprimir_json(&resultado_envio(&feito)) {
                        eprintln!("IES: falha ao emitir JSON: {erro}");
                        return std::process::ExitCode::from(1);
                    }
                } else {
                    println!("{}", humano(&feito));
                }
                ExitCode::SUCCESS
            }
            Err(ErroEnviar::Abertura(erro)) => saida_erro::erro_fonte_http(
                cli.json,
                erro.subcodigo(),
                erro.mensagem(),
                erro.status_http(),
            ),
            Err(erro) => {
                let valor = json_de_falha_com_sugestoes(&erro, sessao, indice, para);
                let resultado = publicar_valor_da_falha(cli.json, &valor, erro.saida());
                if !cli.json
                    && let Some(mensagem) = texto_das_sugestoes(&valor)
                {
                    eprintln!("{mensagem}");
                }
                resultado
            }
        }
    })
}

/// Compõe a falha observada na Web com sugestões exclusivamente locais.
/// A lista guardada não autoriza envio e nunca faz outra requisição ao SEI.
fn json_de_falha_com_sugestoes(
    erro: &ErroEnviar,
    sessao: &crate::processo_obter::SessaoDoProcesso<'_>,
    indice: &crate::indice_local::IndiceLocal,
    para: &str,
) -> Value {
    let mut valor = json_de_falha(erro);
    // Apenas quando o SEI não ofereceu a sigla pedida:
    // sugestões locais não alteram a decisão do autocomplete.
    if matches!(erro, ErroEnviar::DestinoNaoOferecidoPeloSei { .. }) {
        let instalacao = crate::instalacao::Instalacao::interpretar(sessao.origem);
        let ativa = crate::indice_local::unidade_dos_links(sessao.url_do_controle);
        if let (Some(instalacao), Some(ativa)) = (instalacao, ativa) {
            let sugestoes =
                indice.sugestoes_de_unidades_guardadas(&instalacao.url_base(), &ativa, para);
            if !sugestoes.is_empty() {
                valor["sugestoes_guardadas"] = json!(sugestoes);
            }
        }
    }
    valor
}

/// Complemento apenas da apresentação humana de processo enviar. A lista
/// vem do índice local; jamais equivale a destinos aceitos pelo SEI.
fn texto_das_sugestoes(valor: &Value) -> Option<String> {
    let sugeridas = valor.get("sugestoes_guardadas")?.as_array()?;
    let siglas: Vec<_> = sugeridas.iter().filter_map(Value::as_str).collect();
    if siglas.is_empty() {
        return None;
    }
    Some(format!(
        "IES: sugestões guardadas (não confirmadas pelo SEI): {}",
        siglas.join(", ")
    ))
}

/// JSON do sucesso (prévia, ou enviado e conferido).
/// O contrato externo de destino continua isolado do tipo de domínio.
#[derive(serde::Serialize)]
struct DestinoPublico<'a> {
    descricao: &'a str,
    id_unidade: &'a str,
    sigla: &'a str,
}

#[expect(clippy::struct_excessive_bools, reason = "quatro campos independentes exigidos pelo contrato JSON")]
#[derive(serde::Serialize)]
struct ResultadoEnvio<'a> {
    aberto_em: &'a [String],
    alterado: bool,
    confirmado: bool,
    depois: Option<&'a [String]>,
    depois_esperado: Vec<String>,
    destino: DestinoPublico<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    falha_do_envio: Option<crate::processo_concluir_cli::FalhaEnvioPublica>,
    id_procedimento: &'a str,
    manter_aberto: bool,
    nup: &'a str,
    origem: &'static str,
    reconciliado_por_leitura: bool,
    requisicoes: u8,
    unidade: &'a str,
}

fn resultado_envio(feito: &ProcessoEnviado) -> ResultadoEnvio<'_> {
    ResultadoEnvio {
        nup: &feito.nup,
        id_procedimento: &feito.id_procedimento,
        unidade: &feito.unidade,
        origem: "web",
        confirmado: feito.confirmado,
        alterado: feito.alterado,
        manter_aberto: feito.manter_aberto,
        destino: DestinoPublico {
            sigla: &feito.destino.sigla,
            descricao: &feito.destino.descricao,
            id_unidade: &feito.destino.id_unidade,
        },
        aberto_em: &feito.aberto_em,
        depois_esperado: feito.depois_esperado(),
        depois: feito.depois.as_deref(),
        reconciliado_por_leitura: feito.reconciliado_por.is_some(),
        requisicoes: feito.requisicoes,
        falha_do_envio: feito
            .reconciliado_por
            .as_ref()
            .map(crate::processo_concluir_cli::FalhaEnvioPublica::de),
    }
}

/// Compatibilidade com testes do contrato, sem Value no fluxo de produção.
#[cfg(test)]
pub(crate) fn json_de_sucesso(feito: &ProcessoEnviado) -> Value {
    serde_json::to_value(resultado_envio(feito))
        .expect("a projeção tipada do envio só possui valores JSON")
}

/// JSON da falha depois da abertura.
pub(crate) fn json_de_falha(erro: &ErroEnviar) -> Value {
    let mut valor = json_base_da_falha(
        erro.codigo(),
        erro.subcodigo(),
        erro.mensagem(),
        erro.status_http(),
    );
    let Some(objeto) = valor.as_object_mut() else {
        return valor;
    };
    let mut inserir = |campo: &str, dado: Value| {
        objeto.insert(campo.to_owned(), dado);
    };
    match erro {
        ErroEnviar::SituacaoDoProcessoNaoLidaAntesDeEnviar { motivo } => {
            inserir("motivo", json!(motivo.como_texto()));
        }
        ErroEnviar::ProcessoNaoAbertoNaUnidadeDaSessao { aberto_em }
        | ErroEnviar::UnidadeDaSessaoAusenteDaSituacao { aberto_em }
        | ErroEnviar::EnvioNaoPublicadoNaBarra { aberto_em }
        | ErroEnviar::ProcessoJaAbertoNoDestino { aberto_em } => {
            inserir("aberto_em", json!(aberto_em));
        }
        ErroEnviar::DestinoNaoOferecidoPeloSei { oferecidos } => {
            inserir("oferecidos", json!(oferecidos));
        }
        ErroEnviar::VariosDestinosComASigla { candidatos } => {
            inserir("candidatos", json!(candidatos));
        }
        ErroEnviar::EnvioNaoAplicado {
            aberto_em,
            depois,
            falha_do_envio,
        }
        | ErroEnviar::SituacaoDaOrigemDiferenteDaPedidaAposEnvio {
            aberto_em,
            depois,
            falha_do_envio,
        } => {
            inserir("aberto_em", json!(aberto_em));
            inserir("depois", json!(depois));
            if let Some(falha) = falha_do_envio {
                inserir("falha_do_envio", json_da_falha(falha));
            }
        }
        ErroEnviar::EfeitoIncertoNoEnvio {
            aberto_em,
            falha_do_envio,
            verificacao,
        } => {
            inserir("aberto_em", json!(aberto_em));
            inserir("verificacao", json!(verificacao.motivo()));
            if let Some(falha) = falha_do_envio {
                inserir("falha_do_envio", json_da_falha(falha));
            }
        }
        _ => {}
    }
    valor
}

/// Saída humana do sucesso.
pub(crate) fn humano(feito: &ProcessoEnviado) -> String {
    let refs = |v: &[String]| lista_humana(&v.iter().map(String::as_str).collect::<Vec<_>>());
    let mut linhas = vec![
        format!("Processo {} na unidade {}", feito.nup, feito.unidade),
        format!("Aberto em: {}", refs(&feito.aberto_em)),
        format!("Destino: {}", feito.destino.descricao),
        format!(
            "Manter aberto na unidade atual: {}",
            if feito.manter_aberto { "sim" } else { "não" }
        ),
    ];
    match (feito.confirmado, &feito.depois) {
        (false, _) => {
            linhas.push(format!(
                "Depois de enviar: {}",
                refs(&feito.depois_esperado())
            ));
            linhas.push("Para enviar, repita com --confirmar".to_owned());
        }
        (true, depois) => {
            let depois = depois.as_deref().unwrap_or_default();
            linhas.push(format!("Enviado. Aberto em, relido: {}", refs(depois)));
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
#[path = "processo_enviar_cli/tests.rs"]
mod tests;
