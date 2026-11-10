//! BDD-ANOT-01/02/03: prévia, confirmação única e consulta sem escrita.
use crate::processo::NumeroProcesso;
use crate::processo_anotar::{self, Erro};
use crate::processo_concluir_cli::{com_sessao, erro_de_uso};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao, saida_erro};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn tentar_anotar(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao:
            Some(ProcessoAcao::Anotar {
                numero,
                texto,
                confirmar,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    let Ok(nup) = NumeroProcesso::novo(numero) else {
        return Some(erro_de_uso(cli.json));
    };
    let texto = texto.as_deref();
    if texto.is_some_and(|pedido| pedido.trim().is_empty()) {
        if cli.json {
            println!(
                "{}",
                json!({
                    "codigo": "entrada_invalida",
                    "erro": "o texto da anotação não pode ser vazio",
                    "retentavel": false
                })
            );
        } else {
            eprintln!("IES: o texto da anotação não pode ser vazio");
        }
        return Some(ExitCode::from(2));
    }
    Some(com_sessao(cli, |sessao, indice| {
        if *confirmar
            && let Err(causa) = crate::origem_de_treinamento::exigir(
                crate::origem_de_treinamento::Escrita::ProcessoAnotar,
                sessao.origem,
            )
        {
            return crate::origem_de_treinamento::recusa(cli.json, causa);
        }
        match processo_anotar::executar(
            sessao,
            &nup,
            texto.unwrap_or(""),
            *confirmar,
            cli.unidade.as_deref(),
            indice,
            &mut EnvioHttps,
        ) {
            Ok(preview) => {
                if texto.is_none() {
                    exibir_consulta(&preview, cli.json);
                } else if cli.json {
                    println!(
                        "{}",
                        json!({
                            "nup": preview.nup,
                            "id_procedimento": preview.id_procedimento,
                            "unidade": preview.unidade,
                            "texto": preview.texto,
                            "anotacao_antes": preview.anotacao_antes,
                            "confirmado": *confirmar,
                            "alterado": preview.alterado,
                            "origem": "web",
                            "requisicoes": preview.requisicoes,
                        })
                    );
                } else if preview.alterado {
                    println!(
                        "Anotação registrada e conferida no processo {} ({}): {:?}. Requisições: {}.",
                        preview.nup, preview.unidade, preview.texto, preview.requisicoes
                    );
                } else {
                    println!(
                        "Prévia: anotar no processo {} ({}): {:?}. Anotação atual: {:?}. Nenhuma escrita realizada. Requisições: {}.",
                        preview.nup,
                        preview.unidade,
                        preview.texto,
                        preview.anotacao_antes,
                        preview.requisicoes
                    );
                }
                ExitCode::SUCCESS
            }
            Err(Erro::Abertura(erro)) => {
                saida_erro::erro_fonte_http(cli.json, erro.mensagem(), erro.status_http())
            }
            Err(erro) => falha_da_previa(cli, &erro),
        }
    }))
}

/// A ausência só é publicada após o formulário do próprio processo confirmar
/// explicitamente uma textarea vazia. Formulário ausente falha antes daqui.
fn exibir_consulta(preview: &processo_anotar::Previa, modo_json: bool) {
    let anotacao = (!preview.anotacao_antes.is_empty()).then_some(preview.anotacao_antes.as_str());
    if modo_json {
        println!(
            "{}",
            json!({
                "nup": preview.nup,
                "id_procedimento": preview.id_procedimento,
                "unidade": preview.unidade,
                "anotacao": anotacao,
                "alterado": false,
                "origem": "web",
                "requisicoes": preview.requisicoes,
            })
        );
    } else if let Some(texto) = anotacao {
        println!(
            "Anotação atual do processo {} ({}): {texto:?}. Nenhuma alteração realizada. Requisições: {}.",
            preview.nup, preview.unidade, preview.requisicoes
        );
    } else {
        println!(
            "Nenhuma anotação no processo {} ({}). Nenhuma alteração realizada. Requisições: {}.",
            preview.nup, preview.unidade, preview.requisicoes
        );
    }
}

/// Emite a causa tipada sem catálogo de subcódigos por operação.
fn falha_da_previa(cli: &Cli, erro: &Erro) -> ExitCode {
    use crate::erro_publico::Causa;
    let motivo_situacao = match &erro {
        Erro::SituacaoNaoLida(motivo) => Some(motivo.como_texto()),
        _ => None,
    };
    let incerto = matches!(erro, Erro::EfeitoIncerto);
    let mensagem = if incerto {
        "efeito incerto após tentativa de escrita; não repita --confirmar; consulte a anotação"
    } else {
        "o SEI não publicou as condições para anotar este processo nesta unidade"
    };
    if cli.json {
        let mut saida = json!({
            "codigo": if incerto { "efeito_incerto" } else { "fonte_invalida" },
            "erro": mensagem,
            "retentavel": false
        });
        if let Some(motivo) = motivo_situacao {
            saida["motivo_situacao"] = json!(motivo);
        }
        if let Some(http) = erro.status_http() {
            if http.status != 200 {
                saida["status_http"] = json!(http.status);
            }
            if let Some(mensagem_do_servidor) = &http.mensagem_do_servidor {
                saida["mensagem_do_servidor"] = json!(mensagem_do_servidor);
            }
        }
        println!("{saida}");
    } else if let Some(motivo) = motivo_situacao {
        eprintln!("IES: {mensagem} ({motivo})");
    } else {
        eprintln!("IES: {mensagem}");
    }
    ExitCode::from(if incerto { 3 } else { 1 })
}
