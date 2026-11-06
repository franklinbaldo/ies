//! `ies documento [--processo NUP] assinar DOC [--cargo TEXTO] [--confirmar]`
//! (contracts/cli/documento-assinar.md): número inválido sai com 2 antes de
//! qualquer arquivo (`--confirmar` sem `--cargo`, também 2, pelo clap);
//! prévia sem POST; com `--confirmar`, a senha do arquivo privado
//! `IES_CREDENCIAIS_FILE` é lida antes da primeira requisição e só vai ao
//! corpo do POST. A saída nunca traz URL, `infra_hash`, cookie ou senha.
use crate::arvore_processo::Assinante;
use crate::documento::NumeroDocumento;
use crate::documento_assinar::{
    Confirmacao, DocumentoAssinado, ErroAssinar, FalhaDoEnvio, PedidoDaAssinatura,
    assinar_documento, edicao_como_texto,
};
use crate::indice_local::IndiceLocal;
use crate::processo::NumeroProcesso;
use crate::processo_concluir_cli::{com_sessao, json_base_da_falha};
use crate::processo_obter::SessaoDoProcesso;
use crate::sessao_login::{Credenciais, ErroLogin};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Documento, DocumentoAcao, Domain, saida_erro};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::process::ExitCode;

/// `Some` só para `ies documento assinar`.
pub(crate) fn tentar_assinar(cli: &Cli) -> Option<ExitCode> {
    let Domain::Documento(Documento {
        acao:
            Some(DocumentoAcao::Assinar {
                numero,
                cargo,
                confirmar,
            }),
        processo,
        ..
    }) = &cli.command
    else {
        return None;
    };
    let numero = match NumeroDocumento::novo(numero) {
        Ok(numero) => numero,
        Err(erro) => {
            return Some(invalida(
                cli.json,
                erro.subcodigo(),
                "número de documento inválido",
            ));
        }
    };
    let processo = match processo.as_deref().map(NumeroProcesso::novo) {
        None => None,
        Some(Ok(nup)) => Some(nup),
        Some(Err(erro)) => {
            return Some(invalida(
                cli.json,
                erro.subcodigo(),
                "número de processo inválido",
            ));
        }
    };
    let pedido = Entrada {
        numero: &numero,
        processo: processo.as_ref(),
        cargo: cargo.as_deref(),
        confirmar: *confirmar,
    };
    Some(com_sessao(cli, |sessao, indice| {
        assinar_na_sessao(cli, sessao, indice, &pedido)
    }))
}

struct Entrada<'a> {
    numero: &'a NumeroDocumento,
    processo: Option<&'a NumeroProcesso>,
    cargo: Option<&'a str>,
    confirmar: bool,
}

fn invalida(json_mode: bool, subcodigo: &str, mensagem: &str) -> ExitCode {
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

/// As credenciais do arquivo privado, só com `--confirmar`.
fn credenciais() -> Result<Credenciais, ErroLogin> {
    let caminho = std::env::var_os("IES_CREDENCIAIS_FILE")
        .filter(|valor| !valor.is_empty())
        .map(PathBuf::from)
        .ok_or(ErroLogin::CaminhoDasCredenciaisNaoInformado)?;
    Credenciais::ler(&caminho)
}

fn assinar_na_sessao(
    cli: &Cli,
    sessao: &SessaoDoProcesso<'_>,
    indice: &IndiceLocal,
    entrada: &Entrada<'_>,
) -> ExitCode {
    let lidas = if entrada.confirmar {
        match credenciais() {
            Ok(lidas) => Some(lidas),
            Err(erro) => return publicar_falha(cli.json, &ErroAssinar::Credenciais(erro)),
        }
    } else {
        None
    };
    let confirmacao = match (&lidas, entrada.cargo) {
        (Some(credenciais), Some(cargo)) => Some(Confirmacao { cargo, credenciais }),
        _ => None,
    };
    let pedido = PedidoDaAssinatura {
        numero: entrada.numero,
        unidade: cli.unidade.as_deref(),
        processo: entrada.processo,
        cargo: entrada.cargo,
        confirmacao,
    };
    match assinar_documento(sessao, &pedido, indice, &mut EnvioHttps) {
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
        Err(ErroAssinar::Abertura(erro)) => saida_erro::erro_fonte_http(
            cli.json,
            erro.subcodigo(),
            erro.mensagem(),
            erro.status_http(),
        ),
        Err(erro) => publicar_falha(cli.json, &erro),
    }
}

fn json_das_assinaturas(lista: &[Assinante]) -> Value {
    Value::Array(
        lista
            .iter()
            .map(|a| json!({"nome": a.nome, "cargo": a.cargo, "unidade": a.unidade}))
            .collect(),
    )
}

pub(crate) fn json_da_falha(falha: &FalhaDoEnvio) -> Value {
    let mut valor = json!({"subcodigo": falha.subcodigo()});
    if let (Some(http), Some(objeto)) = (falha.status_http(), valor.as_object_mut()) {
        if http.status != 200 {
            objeto.insert("status_http".to_owned(), json!(http.status));
        }
        if let Some(texto) = http.mensagem_do_servidor {
            objeto.insert("mensagem_do_servidor".to_owned(), json!(texto));
        }
    }
    valor
}

/// JSON do sucesso (prévia, já assinado, ou assinado e conferido).
pub(crate) fn json_de_sucesso(feito: &DocumentoAssinado) -> Value {
    let mut valor = json!({
        "numero": feito.numero,
        "id_documento": feito.id_documento,
        "rotulo": feito.rotulo,
        "processo": {"nup": feito.processo.nup, "id": feito.processo.id},
        "unidade": feito.unidade,
        "origem": "web",
        "confirmado": feito.confirmado,
        "alterado": feito.alterado,
        "assinante": feito.assinante,
        "cargo": feito.cargo,
        "cargo_da_tela": feito.cargo_da_tela,
        "cargos": feito.cargos,
        "ja_assinado_pela_sessao": feito.ja_assinado_pela_sessao,
        "assinaturas": json_das_assinaturas(&feito.assinaturas),
        "assinaturas_depois": feito.assinaturas_depois.as_deref().map(json_das_assinaturas),
        "edicao": edicao_como_texto(feito.edicao),
        "edicao_depois": feito.edicao_depois.map(edicao_como_texto),
        "reconciliado_por_leitura": feito.reconciliado_por.is_some(),
        "requisicoes": feito.requisicoes,
    });
    if let (Some(falha), Some(objeto)) = (&feito.reconciliado_por, valor.as_object_mut()) {
        objeto.insert("falha_do_envio".to_owned(), json_da_falha(falha));
    }
    valor
}

/// JSON da falha depois da abertura.
pub(crate) fn json_de_falha(erro: &ErroAssinar) -> Value {
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
        ErroAssinar::CargoNaoOferecidoPelaTela { cargos } => {
            objeto.insert("cargos".to_owned(), json!(cargos));
        }
        ErroAssinar::DocumentoAssinadoPorOutroUsuario { antes } => {
            objeto.insert("assinaturas".to_owned(), json_das_assinaturas(antes));
        }
        ErroAssinar::AssinaturaNaoAplicada {
            antes,
            falha_do_envio,
        } => {
            objeto.insert("assinaturas".to_owned(), json_das_assinaturas(antes));
            if let Some(falha) = falha_do_envio {
                objeto.insert("falha_do_envio".to_owned(), json_da_falha(falha));
            }
        }
        ErroAssinar::EfeitoIncertoNaAssinatura {
            antes,
            falha_do_envio,
            verificacao,
        } => {
            objeto.insert("assinaturas".to_owned(), json_das_assinaturas(antes));
            if let crate::documento_assinar::VerificacaoDaAssinatura::Lida(depois) =
                verificacao.as_ref()
            {
                objeto.insert(
                    "assinaturas_depois".to_owned(),
                    json_das_assinaturas(depois),
                );
            }
            objeto.insert("verificacao".to_owned(), json!(verificacao.motivo()));
            if let Some(falha) = falha_do_envio {
                objeto.insert("falha_do_envio".to_owned(), json_da_falha(falha));
            }
        }
        _ => {}
    }
    valor
}

fn publicar_falha(json_mode: bool, erro: &ErroAssinar) -> ExitCode {
    let valor = json_de_falha(erro);
    if json_mode {
        println!("{valor}");
    } else {
        eprintln!(
            "IES: {} [{}]",
            valor["erro"].as_str().unwrap_or_default(),
            valor["subcodigo"].as_str().unwrap_or_default()
        );
        if let Some(cargos) = valor["cargos"].as_array() {
            let lista: Vec<_> = cargos.iter().filter_map(Value::as_str).collect();
            eprintln!("IES: cargos da tela: {}", lista.join("; "));
        }
        if let Some(falha) = valor["falha_do_envio"]["subcodigo"].as_str() {
            eprintln!("IES: resposta da escrita: {falha}");
        }
        if let Some(motivo) = valor["verificacao"].as_str() {
            eprintln!("IES: verificação: {motivo}");
        }
    }
    ExitCode::from(erro.saida())
}

fn linha_das_assinaturas(lista: &[Assinante]) -> String {
    if lista.is_empty() {
        return "nenhuma".to_owned();
    }
    lista
        .iter()
        .map(|a| format!("{} ({}, {})", a.nome, a.cargo, a.unidade))
        .collect::<Vec<_>>()
        .join("; ")
}

/// Saída humana do sucesso.
pub(crate) fn humano(feito: &DocumentoAssinado) -> String {
    let mut linhas = vec![
        format!(
            "Documento {} ({}) do processo {}",
            feito.numero, feito.rotulo, feito.processo.nup
        ),
        format!("Assinante: {}", feito.assinante),
        format!("Assinaturas: {}", linha_das_assinaturas(&feito.assinaturas)),
    ];
    if feito.ja_assinado_pela_sessao {
        linhas.push("Nada a fazer: o documento já está assinado por este usuário.".to_owned());
    } else if feito.alterado {
        let depois = feito.assinaturas_depois.as_deref().unwrap_or_default();
        linhas.push(format!(
            "Assinado. Assinaturas, relidas: {}",
            linha_das_assinaturas(depois)
        ));
        if let Some(falha) = &feito.reconciliado_por {
            linhas.push(format!(
                "A resposta não foi a esperada ({}); a leitura confirmou.",
                falha.subcodigo()
            ));
        }
    } else {
        linhas.push(format!("Cargos da tela: {}", feito.cargos.join("; ")));
        linhas.push(format!(
            "Cargo sugerido pela tela: {}",
            feito.cargo_da_tela.as_deref().unwrap_or("(nenhum)")
        ));
        if let Some(cargo) = &feito.cargo {
            linhas.push(format!("Cargo pedido: {cargo} (conferido)"));
        }
        linhas.push("Para assinar (não se desfaz), repita com --cargo e --confirmar".to_owned());
    }
    linhas.push(format!("Requisições: {}", feito.requisicoes));
    linhas.join("\n")
}
