//! BDD-ANOT-01/02/03/04: consulta, prévia, registro e retirada conferida.
use crate::processo_anotar::{self};
use crate::processo_concluir_cli::com_sessao;
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn tentar_anotar(cli: &Cli) -> Option<anyhow::Result<()>> {
    let Domain::Processo(Processo {
        acao:
            Some(ProcessoAcao::Anotar {
                numero,
                texto,
                remover,
                confirmar,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    let nup = numero.clone();
    let texto = texto.as_deref();
    let entrada_invalida = if texto.is_some_and(|pedido| pedido.trim().is_empty()) {
        Some("o texto da anotação não pode ser vazio")
    } else if *confirmar && texto.is_none() && !*remover {
        Some("a confirmação exige --texto ou --remover")
    } else {
        None
    };
    if let Some(mensagem) = entrada_invalida {
        if cli.json {
            println!(
                "{}",
                json!({
                    "codigo": "entrada_invalida",
                    "erro": mensagem,
                    "retentavel": false
                })
            );
        } else {
            eprintln!("IES: {mensagem}");
        }
        return Some(crate::legado(ExitCode::from(2)));
    }
    Some(com_sessao(cli, |sessao, indice| {
        if *confirmar
            && let Err(causa) = crate::origem_de_treinamento::exigir(
                crate::origem_de_treinamento::Escrita::ProcessoAnotar,
                sessao.origem,
            )
        {
            return Err(causa.into());
        }
        let intencao = if *remover {
            processo_anotar::Intencao::Retirada {
                confirmar: *confirmar,
            }
        } else if let Some(texto) = texto {
            processo_anotar::Intencao::Registro {
                texto,
                confirmar: *confirmar,
            }
        } else {
            processo_anotar::Intencao::Consulta
        };
        let preview = processo_anotar::executar(
            sessao,
            &nup,
            &intencao,
            cli.unidade.as_deref(),
            indice,
            &mut EnvioHttps,
        )?;
        if *remover {
            exibir_retirada(&preview, *confirmar, cli.json);
        } else if texto.is_none() {
            exibir_consulta(&preview, cli.json);
        } else {
            exibir_registro(&preview, *confirmar, cli.json);
        }
        Ok(())
    }))
}

/// A saída de registro é uma operação de apresentação, separada do despacho.
fn exibir_registro(preview: &processo_anotar::Previa, confirmar: bool, modo_json: bool) {
    if modo_json {
        println!(
            "{}",
            json!({
                "nup": preview.nup,
                "id_procedimento": preview.id_procedimento,
                "unidade": preview.unidade,
                "texto": preview.texto,
                "anotacao_antes": preview.anotacao_antes,
                "confirmado": confirmar,
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
}

/// A retirada distingue uma prévia da ausência confirmada por releitura.
fn exibir_retirada(preview: &processo_anotar::Previa, confirmar: bool, modo_json: bool) {
    if modo_json {
        println!(
            "{}",
            json!({
                "nup": preview.nup,
                "id_procedimento": preview.id_procedimento,
                "unidade": preview.unidade,
                "texto": "",
                "anotacao_antes": preview.anotacao_antes,
                "remover": true,
                "confirmado": confirmar,
                "alterado": preview.alterado,
                "origem": "web",
                "requisicoes": preview.requisicoes,
            })
        );
    } else if preview.alterado {
        println!(
            "Anotação retirada e ausência conferida no processo {} ({}). Requisições: {}.",
            preview.nup, preview.unidade, preview.requisicoes
        );
    } else if preview.anotacao_antes.is_empty() {
        println!(
            "Nenhuma anotação a retirar no processo {} ({}). Nenhuma escrita realizada. Requisições: {}.",
            preview.nup, preview.unidade, preview.requisicoes
        );
    } else {
        println!(
            "Prévia: retirar anotação {:?} do processo {} ({}). Nenhuma escrita realizada. Requisições: {}.",
            preview.anotacao_antes, preview.nup, preview.unidade, preview.requisicoes
        );
    }
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
