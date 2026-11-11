//! `ies documento [--processo NUP] assinar DOC [--cargo TEXTO] [--confirmar]`
//! (contracts/cli/documento-assinar.md): número inválido sai com 2 antes de
//! qualquer arquivo (`--confirmar` sem `--cargo`, também 2, pelo clap);
//! prévia sem POST; com `--confirmar`, a senha do arquivo privado
//! `IES_CREDENCIAIS_FILE` é lida antes da primeira requisição e só vai ao
//! corpo do POST. A saída nunca traz URL, `infra_hash`, cookie ou senha.
use crate::arvore_processo::Assinante;
use crate::documento::NumeroDocumento;
use crate::documento_assinar::{
    AcaoDaAssinatura, Confirmacao, DocumentoAssinado, ErroAssinar, PedidoDaAssinatura,
    ResultadoDaAssinatura, assinar_documento, edicao_como_texto,
};
use crate::indice_local::IndiceLocal;
use crate::processo::NumeroProcesso;
use crate::processo_concluir_cli::com_sessao;
use crate::processo_obter::SessaoDoProcesso;
use crate::sessao_login::{Credenciais, ErroLogin};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Documento, DocumentoAcao, Domain};
#[path = "documento_assinar_cli/falha.rs"]
mod falha;
use falha::assinaturas_publicas;
use serde_json::{Value, json};
use std::path::PathBuf;

/// `Some` só para `ies documento assinar`.
pub(crate) fn tentar_assinar(cli: &Cli) -> Option<anyhow::Result<()>> {
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
    let pedido = Entrada {
        numero,
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
) -> anyhow::Result<()> {
    let lidas = if entrada.confirmar {
        Some(credenciais().map_err(ErroAssinar::Credenciais)?)
    } else {
        None
    };
    let acao = match (&lidas, entrada.cargo) {
        (Some(credenciais), Some(cargo)) => {
            AcaoDaAssinatura::Confirmar(Confirmacao { cargo, credenciais })
        }
        (_, cargo) => AcaoDaAssinatura::Previa { cargo },
    };
    let pedido = PedidoDaAssinatura {
        numero: entrada.numero,
        unidade: cli.unidade.as_deref(),
        processo: entrada.processo,
        acao,
    };
    let feito = assinar_documento(sessao, &pedido, indice, &mut EnvioHttps)?;
    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json_de_sucesso(&feito)?)?
        );
    } else {
        println!("{}", humano(&feito));
    }
    Ok(())
}

fn json_das_assinaturas(lista: &[Assinante]) -> Result<Value, serde_json::Error> {
    serde_json::to_value(assinaturas_publicas(lista))
}

/// JSON do sucesso (prévia, já assinado, ou assinado e conferido).
pub(crate) fn json_de_sucesso(feito: &DocumentoAssinado) -> Result<Value, serde_json::Error> {
    let (confirmado, alterado, ja_assinado, assinaturas_depois, edicao_depois, reconciliado_por) =
        match &feito.resultado {
            ResultadoDaAssinatura::Previa => (false, false, false, None, None, None),
            ResultadoDaAssinatura::JaAssinadoPelaSessao { confirmacao_pedida } => {
                (*confirmacao_pedida, false, true, None, None, None)
            }
            ResultadoDaAssinatura::Comprovada {
                assinatura_depois,
                edicao_depois,
                reconciliado_por,
            } => (
                true,
                true,
                false,
                Some(std::slice::from_ref(assinatura_depois)),
                Some(*edicao_depois),
                reconciliado_por.as_ref(),
            ),
        };
    let mut valor = json!({
        "numero": feito.numero,
        "id_documento": feito.id_documento,
        "rotulo": feito.rotulo,
        "processo": {"nup": feito.processo.nup, "id": feito.processo.id},
        "unidade": feito.unidade,
        "origem": "web",
        "confirmado": confirmado,
        "alterado": alterado,
        "assinante": feito.assinante,
        "cargo": feito.cargo,
        "cargo_da_tela": feito.cargo_da_tela,
        "cargos": feito.cargos,
        "ja_assinado_pela_sessao": ja_assinado,
        "assinaturas": json_das_assinaturas(&feito.assinaturas)?,
        "assinaturas_depois": assinaturas_depois.map(json_das_assinaturas).transpose()?,
        "edicao": edicao_como_texto(feito.edicao),
        "edicao_depois": edicao_depois.map(edicao_como_texto),
        "reconciliado_por_leitura": reconciliado_por.is_some(),
        "requisicoes": feito.requisicoes,
    });
    if let (Some(falha), Some(objeto)) = (reconciliado_por, valor.as_object_mut()) {
        objeto.insert("falha_do_envio".to_owned(), json!(format!("{falha:?}")));
    }
    Ok(valor)
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
    match &feito.resultado {
        ResultadoDaAssinatura::JaAssinadoPelaSessao { .. } => {
            linhas.push("Nada a fazer: o documento já está assinado por este usuário.".to_owned());
        }
        ResultadoDaAssinatura::Comprovada {
            assinatura_depois,
            reconciliado_por,
            ..
        } => {
            linhas.push(format!(
                "Assinado. Assinaturas, relidas: {}",
                linha_das_assinaturas(std::slice::from_ref(assinatura_depois))
            ));
            if let Some(falha) = reconciliado_por {
                linhas.push(format!(
                    "A resposta não foi a esperada ({falha:?}); a leitura confirmou."
                ));
            }
        }
        ResultadoDaAssinatura::Previa => {
            linhas.push(format!("Cargos da tela: {}", feito.cargos.join("; ")));
            linhas.push(format!(
                "Cargo sugerido pela tela: {}",
                feito.cargo_da_tela.as_deref().unwrap_or("(nenhum)")
            ));
            if let Some(cargo) = &feito.cargo {
                linhas.push(format!("Cargo pedido: {cargo} (conferido)"));
            }
            linhas
                .push("Para assinar (não se desfaz), repita com --cargo e --confirmar".to_owned());
        }
    }
    linhas.push(format!("Requisições: {}", feito.requisicoes));
    linhas.join("\n")
}
