//! `ies processo marcar`: prévia Web idempotente da atribuição/retirada.
//! Um comando de escrita ainda não observado nunca envia requisição modificadora.
//! A sessão, URL assinada e cookie ficam só nas peças privadas existentes.
use crate::indice_local::IndiceLocal;
use crate::marcadores_processo::MarcadoresLidos;
use crate::processo::NumeroProcesso;
use crate::processo_marcar::{Decisao, ErroMarcar, Operacao, Rotulo, decidir, interpretar_rotulo, validar_texto};
use crate::processo_obter::{
    EnvioDoProcesso, Marcadores, PedidoDeAndamentos, PedidoDoProcesso, SessaoDoProcesso,
    obter_processo,
};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao, processo_concluir_cli, saida_erro};
use serde_json::json;
use std::process::ExitCode;

/// Despacho específico, sem tocar nos outros subcomandos.
pub(crate) fn tentar_marcar(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao:
            Some(ProcessoAcao::Marcar {
                numero,
                marcador,
                retirar,
                texto,
                confirmar,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    let nup = match NumeroProcesso::novo(numero) {
        Ok(nup) => nup,
        Err(erro) => {
            return Some(erro_de_uso(
                cli,
                erro.subcodigo(),
                "número de processo inválido",
            ));
        }
    };
    let (bruto, operacao) = if let Some(rotulo) = marcador {
        (rotulo.as_str(), Operacao::Atribuir)
    } else if let Some(rotulo) = retirar {
        (rotulo.as_str(), Operacao::Retirar)
    } else {
        // O clap exige exatamente um dos dois, sem tentar abrir a sessão.
        return Some(erro_de_uso(
            cli,
            "marcador_nao_informado",
            "informe --marcador ou --retirar",
        ));
    };
    let rotulo = match interpretar_rotulo(bruto) {
        Ok(rotulo) => rotulo,
        Err(erro) => return Some(erro_de_uso(cli, erro.subcodigo(), erro.mensagem())),
    };
    if let Some(texto) = texto
        && let Err(erro) = validar_texto(texto)
    {
        return Some(erro_de_uso(cli, erro.subcodigo(), erro.mensagem()));
    }
    Some(processo_concluir_cli::com_sessao(cli, |sessao, indice| {
        previa(
            cli, sessao, indice, &nup, &rotulo, operacao, *confirmar, &mut EnvioHttps,
        )
    }))
}

fn erro_de_uso(cli: &Cli, subcodigo: &str, mensagem: &str) -> ExitCode {
    if cli.json {
        println!(
            "{}",
            json!({
                "codigo": "entrada_invalida", "subcodigo": subcodigo,
                "erro": mensagem, "retentavel": false
            })
        );
    } else {
        eprintln!("IES: {mensagem} [{subcodigo}]");
    }
    ExitCode::from(2)
}

fn falha(cli: &Cli, erro: ErroMarcar, requisicoes: u8) -> ExitCode {
    if cli.json {
        println!(
            "{}",
            json!({
                "codigo": "fonte_invalida", "subcodigo": erro.subcodigo(),
                "erro": erro.mensagem(), "retentavel": false,
                "requisicoes": requisicoes
            })
        );
    } else {
        eprintln!(
            "IES: {} [{}]; requisições: {}",
            erro.mensagem(),
            erro.subcodigo(),
            requisicoes
        );
    }
    ExitCode::from(1)
}

fn marcadores_observados(marcadores: &Marcadores) -> Result<&MarcadoresLidos, ErroMarcar> {
    match marcadores {
        Marcadores::Lidos(lista) => Ok(lista),
        _ => Err(ErroMarcar::MarcadoresNaoLidos),
    }
}

fn previa<E: EnvioDoProcesso>(
    cli: &Cli,
    sessao: &SessaoDoProcesso<'_>,
    indice: &IndiceLocal,
    nup: &NumeroProcesso,
    rotulo: &Rotulo,
    operacao: Operacao,
    confirmar: bool,
    envio: &mut E,
) -> ExitCode {
    let pedido = PedidoDoProcesso {
        nup,
        unidade: cli.unidade.as_deref(),
        andamentos: PedidoDeAndamentos::Omitidos,
        propriedades: false,
        marcadores: true,
        // Uma escrita depende da unidade publicada pela tela do processo.
        unidade_exigida: true,
    };
    let aberto = match obter_processo(sessao, &pedido, indice, envio) {
        Ok(aberto) => aberto,
        Err(erro) => {
            return saida_erro::erro_fonte_http(
                cli.json,
                erro.subcodigo(),
                erro.mensagem(),
                erro.status_http(),
            );
        }
    };
    let n = aberto.requisicoes;
    let Some(unidade) = aberto.unidade.as_deref() else {
        return falha(cli, ErroMarcar::UnidadeDaSessaoNaoPublicada, n);
    };
    let lidos = match marcadores_observados(&aberto.marcadores) {
        Ok(lidos) => lidos,
        Err(erro) => return falha(cli, erro, n),
    };
    let decisao = match decidir(lidos, rotulo, operacao) {
        Ok(decisao) => decisao,
        Err(erro) => return falha(cli, erro, n),
    };
    if confirmar && decisao == Decisao::PreviaDeRetirada {
        // Ainda sem prova dos campos e da resposta do POST, nunca escrever.
        return falha(cli, ErroMarcar::RemocaoSemFormularioObservado, n);
    }
    let nada_a_fazer = decisao == Decisao::NadaAFazer;
    let acao = match operacao {
        Operacao::Atribuir => "atribuir",
        Operacao::Retirar => "retirar",
    };
    if cli.json {
        println!(
            "{}",
            json!({
                "nup": nup.as_str(),
                "id_procedimento": aberto.arvore.processo.id,
                "unidade": unidade,
                "marcador": format!("{}#{}", rotulo.id, rotulo.nome),
                "acao": acao,
                "origem": "web",
                "confirmado": confirmar,
                "alterado": false,
                "nada_a_fazer": nada_a_fazer,
                "retirada_possivel": !nada_a_fazer,
                "requisicoes": n,
            })
        );
    } else if nada_a_fazer {
        println!(
            "Processo {}: nada a fazer com {} ({}). Requisições: {}.",
            nup.as_str(),
            rotulo.id,
            acao,
            n
        );
    } else {
        println!(
            "Prévia: retirar marcador {}#{} do processo {} (unidade {}). Requisições: {}. A escrita ainda não está disponível.",
            rotulo.id,
            rotulo.nome,
            nup.as_str(),
            unidade,
            n
        );
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn cli_exige_um_rotulo_e_nao_permite_duas_operacoes() {
        assert!(Cli::try_parse_from(["ies", "processo", "marcar", "NUP"]).is_err());
        assert!(
            Cli::try_parse_from([
                "ies",
                "processo",
                "marcar",
                "NUP",
                "--marcador",
                "1#A",
                "--retirar",
                "2#B"
            ])
            .is_err()
        );
    }

    #[test]
    fn cli_aceita_texto_so_para_atribuicao() {
        assert!(
            Cli::try_parse_from([
                "ies",
                "processo",
                "marcar",
                "NUP",
                "--marcador",
                "1#A",
                "--texto",
                "Olá"
            ])
            .is_ok()
        );
        assert!(
            Cli::try_parse_from([
                "ies",
                "processo",
                "marcar",
                "NUP",
                "--retirar",
                "1#A",
                "--texto",
                "X"
            ])
            .is_err()
        );
    }
}
