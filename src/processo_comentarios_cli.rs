//! Lista comentários publicados na barra de um processo, sem POST mutador.
use crate::processo::NumeroProcesso;
use crate::processo_comentarios::VisaoDosComentarios;
use crate::processo_comentarios_web::{self, Erro};
use crate::processo_concluir_cli::{com_sessao, erro_de_uso};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn tentar_ler(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao: Some(ProcessoAcao::Comentarios { numero }), ..
    }) = &cli.command else { return None; };
    let nup = match NumeroProcesso::novo(numero) {
        Ok(nup) => nup,
        Err(e) => return Some(erro_de_uso(cli.json, e.subcodigo())),
    };
    Some(com_sessao(cli, |sessao, indice| {
        match processo_comentarios_web::ler(
            sessao, &nup, cli.unidade.as_deref(), indice, &mut EnvioHttps,
        ) {
            Ok(resultado) => {
                if cli.json {
                    let itens: Vec<_> = resultado.itens.iter().map(|c| json!({
                        "id": c.id, "data_hora": c.data_hora, "unidade": c.unidade,
                        "usuario": c.usuario, "texto": c.texto,
                        "protocolo": c.protocolo, "tipo": c.tipo,
                    })).collect();
                    println!("{}", json!({
                        "nup": resultado.numero,
                        "id_procedimento": resultado.id_procedimento,
                        "visao": match resultado.visao {
                            VisaoDosComentarios::Processo => "P",
                            VisaoDosComentarios::Todos => "T",
                        },
                        "total": resultado.total_declarado,
                        "comentarios": itens,
                        "lista_completa": true,
                        "origem": "web",
                        "requisicoes": resultado.requisicoes,
                    }));
                } else {
                    println!("Comentários do processo {} ({} registros):",
                        resultado.numero, resultado.total_declarado);
                    for item in &resultado.itens {
                        println!("{} | {} | {} | {} | {}", item.id,
                            item.data_hora, item.unidade, item.usuario, item.texto);
                    }
                }
                ExitCode::SUCCESS
            }
            Err(erro) => falha(cli, &erro),
        }
    }))
}

fn falha(cli: &Cli, erro: &Erro) -> ExitCode {
    let causa = crate::erro_publico::caminho_de(erro);
    let mensagem = "a lista completa de comentários não foi comprovada; nenhuma escrita realizada";
    if cli.json {
        println!("{}", json!({
            "codigo": "fonte_invalida", "causa": causa,
            "erro": mensagem, "retentavel": false
        }));
    } else {
        eprintln!("IES: {mensagem} [{}]", causa.join("/"));
    }
    ExitCode::from(1)
}
