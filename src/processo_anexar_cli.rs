//! CLI de anexação: prévia e confirmação única, com prova independente.
use crate::processo::NumeroProcesso;
use crate::processo_anexar::fluxo::{FalhaAnexar, Pedido, Previa, ResultadoAnexacao, prever};
use crate::processo_concluir_cli::{com_sessao, erro_de_uso};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao, saida_erro};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn tentar_anexar(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao:
            Some(ProcessoAcao::Anexar {
                numero,
                a,
                confirmar,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    let Ok(anexado) = NumeroProcesso::novo(numero) else {
        return Some(erro_de_uso(cli.json));
    };
    let Ok(principal) = NumeroProcesso::novo(a) else {
        return Some(erro_de_uso(cli.json));
    };
    if anexado.as_str() == principal.as_str() {
        return Some(falha(
            cli.json,
            "anexação recusada pela tela ou pelos dados informados",
        ));
    }
    Some(com_sessao(cli, |sessao, indice| {
        if *confirmar
            && let Err(causa) = crate::origem_de_treinamento::exigir(
                crate::origem_de_treinamento::Escrita::ProcessoAnexar,
                sessao.origem,
            )
        {
            return crate::origem_de_treinamento::recusa(cli.json, causa);
        }
        let pedido = Pedido {
            anexado: &anexado,
            principal: &principal,
            unidade: cli.unidade.as_deref(),
            confirmar: *confirmar,
        };
        match prever(sessao, &pedido, indice, &mut EnvioHttps) {
            Ok(preview) => publicar(cli.json, &preview),
            Err(FalhaAnexar::Abertura(erro)) => {
                saida_erro::erro_fonte_http(cli.json, erro.mensagem(), erro.status_http())
            }
            Err(erro) => falha_tipado(cli.json, &erro),
        }
    }))
}
fn falha(json_mode: bool, mensagem: &str) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "operacao_recusada",
                "erro": mensagem,
                "retentavel": false,
            })
        );
    } else {
        eprintln!("IES: {mensagem}");
    }
    ExitCode::from(1)
}
fn falha_tipado(json_mode: bool, erro: &FalhaAnexar) -> ExitCode {
    let FalhaAnexar::NaoConfirmado(_) = erro else {
        return falha(json_mode, &erro.mensagem());
    };
    let mensagem = "O efeito da anexação não pôde ser comprovado. Confira a lista no SEI; não repita --confirmar.";
    if json_mode {
        // A causa vem diretamente da variante Rust serializada por Serde.
        // Não acrescentar um novo subcódigo textual ao catálogo legado.
        println!(
            "{}",
            json!({
                "codigo": "nao_confirmado",
                "erro": mensagem,
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: {mensagem}");
    }
    ExitCode::from(1)
}

fn publicar(json_mode: bool, previa: &Previa) -> ExitCode {
    let confirmado = previa.resultado == ResultadoAnexacao::Confirmada;
    if json_mode {
        println!(
            "{}",
            json!({
                "principal": previa.principal,
                "id_principal": previa.id_principal,
                "anexado": previa.anexado.nup,
                "id_anexado": previa.anexado.id,
                "tipo_anexado": previa.anexado.tipo,
                "documentos_assinados": previa.assinaturas_em_dia,
                "confirmado": confirmado,
                "alterado": confirmado,
                "origem": "web",
                "requisicoes": previa.requisicoes,
            })
        );
    } else {
        println!(
            "{}: anexar {} ao processo {} ({}). Requisições: {}.",
            if confirmado { "Confirmado" } else { "Prévia" },
            previa.anexado.nup,
            previa.principal,
            if previa.assinaturas_em_dia {
                "sem pendência de assinatura"
            } else {
                "há documento sem assinatura"
            },
            previa.requisicoes,
        );
    }
    ExitCode::SUCCESS
}
