//! Entrada pública de documento criar (#246).
//! A confirmação deriva um único POST do cadastro e só reconhece o efeito
//! depois de releitura da árvore. Não há retry.
use crate::documento_criar::{ErroPreviaCriacao, PedidoCriacao, PreviaCriacao, executar_criacao};
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::Contador;
use crate::processo_concluir_cli::com_sessao;
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Documento, DocumentoAcao, Domain, saida_erro};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn tentar_criar(cli: &Cli) -> Option<ExitCode> {
    let Domain::Documento(Documento {
        acao:
            Some(DocumentoAcao::Criar {
                processo,
                tipo,
                descricao,
                nivel,
                confirmar,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    let nup = match NumeroProcesso::novo(processo) {
        Ok(nup) => nup,
        Err(erro) => return Some(entrada_invalida(cli.json, erro.subcodigo())),
    };
    if tipo.trim().is_empty() {
        return Some(entrada_invalida(cli.json, "tipo_de_documento_vazio"));
    }
    Some(com_sessao(cli, |sessao, indice| {
        let pedido = PedidoCriacao {
            nup: &nup,
            unidade: cli.unidade.as_deref(),
            tipo,
            descricao: descricao.as_deref(),
            nivel: nivel.as_deref(),
        };
        // A contagem externa sobrevive ao Result::Err, inclusive depois do POST.
        let mut envio = EnvioHttps;
        let mut contagem = Contador::novo(&mut envio);
        match executar_criacao(sessao, &pedido, indice, &mut contagem, *confirmar) {
            Ok(previa) => publicar_previa(cli.json, &previa),
            Err(erro) => publicar_erro(cli.json, &erro, contagem.feitas()),
        }
    }))
}

fn entrada_invalida(json_mode: bool, subcodigo: &str) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "entrada_invalida",
                "subcodigo": subcodigo,
                "erro": "parâmetro de criação de documento inválido",
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: parâmetro de criação de documento inválido [{subcodigo}]");
    }
    ExitCode::from(2)
}

fn publicar_previa(json_mode: bool, previa: &PreviaCriacao) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "nup": previa.nup,
                "tipo": previa.tipo,
                "unidade": previa.unidade,
                "origem": "web",
                "confirmado": previa.confirmado,
                "alterado": previa.confirmado,
                "numero_sei": previa.numero_sei,
                "requisicoes": previa.requisicoes
            })
        );
    } else {
        if previa.confirmado {
            println!(
                "Documento {} criado no processo {} (unidade {}), número SEI {}.",
                previa.tipo,
                previa.nup,
                previa.unidade,
                previa.numero_sei.as_deref().unwrap_or("não observado")
            );
        } else {
            println!(
                "Prévia: criar documento {} no processo {} (unidade {}).",
                previa.tipo, previa.nup, previa.unidade
            );
            println!("Nenhum documento foi criado.");
        }
        println!("Requisições: {}", previa.requisicoes);
    }
    ExitCode::SUCCESS
}

fn erro_incerto_json(erro: &ErroPreviaCriacao, requisicoes: u8) -> serde_json::Value {
    let mut saida = json!({
        "codigo": "efeito_incerto",
        "subcodigo": erro.subcodigo(),
        "erro": "escrita enviada; não é possível comprovar o efeito por leitura",
        "retentavel": false,
        "requisicoes": requisicoes
    });
    if let Some(causa) = erro.causa_envio() {
        saida["causa_envio"] = json!(causa.subcodigo());
        if let Some(http) = causa.status_http() {
            if http.status != 200 {
                saida["status_http"] = json!(http.status);
            }
            if let Some(mensagem) = &http.mensagem_do_servidor {
                saida["mensagem_do_servidor"] = json!(mensagem);
            }
        }
    }
    saida
}

fn publicar_erro(json_mode: bool, erro: &ErroPreviaCriacao, requisicoes: u8) -> ExitCode {
    if matches!(
        erro,
        ErroPreviaCriacao::EfeitoIncerto | ErroPreviaCriacao::EfeitoIncertoNoEnvio(_)
    ) {
        if json_mode {
            println!("{}", erro_incerto_json(erro, requisicoes));
        } else {
            eprintln!("IES: efeito incerto na criação do documento; não repita --confirmar");
            eprintln!("Requisições: {requisicoes}");
            if let Some(causa) = erro.causa_envio() {
                eprintln!(
                    "{}",
                    saida_erro::linha_humana(
                        causa.subcodigo(),
                        "falha observada no envio do documento",
                        causa.status_http(),
                    )
                );
            }
        }
        return ExitCode::from(3);
    }
    saida_erro::erro_fonte_http(
        json_mode,
        erro.subcodigo(),
        "criação do documento não pôde ser conferida",
        erro.status_http(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn efeito_incerto_publica_requisicoes_realmente_tentadas() {
        let saida = erro_incerto_json(&ErroPreviaCriacao::EfeitoIncerto, 7);
        assert_eq!(saida["codigo"], "efeito_incerto");
        assert_eq!(
            saida["requisicoes"], 7,
            "null não informa a contagem já observada"
        );
        assert_eq!(saida["retentavel"], false);
    }
}
