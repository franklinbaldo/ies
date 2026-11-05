//! Prévia Web de atribuição por login exato; escrita bloqueada até a
//! sequência POST único e releitura estar coberta por teste de transporte.
use crate::arvore_processo::AbertoEm;
use crate::indice_local::IndiceLocal;
use crate::processo::NumeroProcesso;
use crate::processo_atribuir::{ler_tela, planejar};
use crate::processo_concluir::escrita::{Contador, FalhaDaTela, leitura, ler_cookie, obter_tela};
use crate::processo_concluir_cli::{com_sessao, erro_de_uso};
use crate::processo_obter::{SessaoDoProcesso, abrir_para_escrever};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn tentar_atribuir(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao:
            Some(ProcessoAcao::Atribuir {
                numero,
                usuario,
                remover,
                confirmar,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    Some(match NumeroProcesso::novo(numero) {
        Ok(nup) => {
            if *confirmar {
                // Ainda falta provar POST e releitura; nunca testar escrita no escuro.
                falhar(cli.json, "escrita_de_atribuicao_ainda_nao_verificada")
            } else {
                com_sessao(cli, |sessao, indice| {
                    previa(
                        cli,
                        sessao,
                        indice,
                        &nup,
                        if *remover { None } else { usuario.as_deref() },
                    )
                })
            }
        }
        Err(erro) => erro_de_uso(cli.json, erro.subcodigo()),
    })
}

fn falhar(json_mode: bool, subcodigo: &str) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "atribuicao_recusada",
                "subcodigo": subcodigo,
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: atribuição recusada [{subcodigo}]");
    }
    ExitCode::from(1)
}

fn previa(
    cli: &Cli,
    sessao: &SessaoDoProcesso<'_>,
    indice: &IndiceLocal,
    nup: &NumeroProcesso,
    usuario: Option<&str>,
) -> ExitCode {
    let mut envio = EnvioHttps;
    let mut rede = Contador::novo(&mut envio);
    let pedido = leitura(nup, cli.unidade.as_deref(), true);
    let (aberto, acoes) = match abrir_para_escrever(sessao, &pedido, indice, &mut rede) {
        Ok(v) => v,
        Err(erro) => return falhar(cli.json, erro.subcodigo()),
    };
    let Some(unidade) = aberto.unidade.as_deref() else {
        return falhar(cli.json, "unidade_da_sessao_nao_publicada_na_abertura");
    };
    let atual = match &aberto.arvore.aberto_em {
        AbertoEm::Unidades(unidades) => unidades
            .iter()
            .find(|u| u.sigla == unidade)
            .map(|u| u.atribuido_a.as_ref().map_or("", |a| a.sigla.as_str())),
        AbertoEm::Sobrestado(_) | AbertoEm::NaoLido(_) => None,
    };
    if atual.is_none() {
        return falhar(cli.json, "situacao_da_atribuicao_nao_observada");
    }
    let link = match acoes.atribuir {
        Ok(Some(link)) => link,
        Ok(None) => return falhar(cli.json, "atribuicao_nao_oferecida_na_barra"),
        Err(erro) => return falhar(cli.json, erro.subcodigo()),
    };
    let cookie = match ler_cookie(sessao) {
        Ok(v) => v,
        Err(erro) => return falhar(cli.json, erro.subcodigo()),
    };
    let (html, _) = match obter_tela(&mut rede, &cookie, link.as_str()) {
        Ok(v) => v,
        Err(FalhaDaTela::Redirecionada) => {
            return falhar(cli.json, "tela_da_atribuicao_redirecionada");
        }
        Err(FalhaDaTela::SessaoEncerrada) => {
            return falhar(cli.json, "sessao_encerrada_na_atribuicao");
        }
        Err(FalhaDaTela::Transporte(erro)) => return falhar(cli.json, erro.subcodigo()),
    };
    let tela = match ler_tela(&html, &aberto.arvore.processo.id) {
        Ok(v) => v,
        Err(erro) => return falhar(cli.json, erro.subcodigo()),
    };
    let valor = match planejar(&tela, usuario, atual) {
        Ok(v) => v,
        Err(erro) => return falhar(cli.json, erro.subcodigo()),
    };
    if cli.json {
        println!(
            "{}",
            json!({
                "nup": nup.as_str(),
                "id_procedimento": aberto.arvore.processo.id,
                "unidade": unidade,
                "usuario_atual": atual,
                "usuario_pedido": usuario,
                "remover": usuario.is_none(),
                "valor_da_opcao": valor,
                "confirmado": false,
                "alterado": false,
                "origem": "web",
                "requisicoes": rede.feitas()
            })
        );
    } else {
        println!("Prévia: processo {}, unidade {unidade}.", nup.as_str());
        if let Some(login) = usuario {
            println!("Atribuir a {login}; opção verificada na tela.");
        } else {
            println!("Remover a atribuição observada.");
        }
        println!("Requisições: {}", rede.feitas());
    }
    ExitCode::SUCCESS
}
