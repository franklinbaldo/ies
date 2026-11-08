//! `ies processo enviar NUP --para SIGLA [--manter-aberto] [--confirmar]`
//! (contracts/cli/processo-enviar.md): uso inválido sai com 2 antes de
//! qualquer arquivo; prévia sem POST de escrita; com `--confirmar`, um POST
//! conferido por leitura; efeito incerto, 3. A saída nunca traz URL,
//! `infra_hash` ou cookie.
use crate::processo::NumeroProcesso;
use crate::processo_concluir_cli::{
    com_sessao, erro_de_uso, falha::FalhaDaSituacao, falha_do_envio_publica, lista_humana,
};
use crate::processo_enviar::{ErroEnviar, PedidoDoEnvio, ProcessoEnviado, enviar_processo};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, ProcessoAcao, saida_erro};
#[cfg(test)]
use serde_json::Value;
use serde_json::json;
use std::process::ExitCode;

mod humano;
pub(crate) use humano::humano;

// A confirmação de remoção continua fechada em instalações reais.
// A bancada E2E da #333 pode exercitar o mesmo código somente com origem
// .test reservada e proxy HTTP numericamente limitado ao loopback.
fn ensaio_local_habilitado() -> bool {
    std::env::var("IES_ANTI_IES_TESTE").as_deref() == Ok("1")
}

fn destino_do_ensaio_local(origem: &str) -> bool {
    origem == "https://sei.test/sei"
        && std::env::var("https_proxy")
            .ok()
            .and_then(|proxy| {
                proxy
                    .strip_prefix("http://127.0.0.1:")
                    .and_then(|porta| porta.parse::<u16>().ok())
            })
            .is_some_and(|porta| porta != 0)
}

/// `Some` só para `ies processo enviar`.
pub(crate) fn tentar_enviar(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao:
            Some(ProcessoAcao::Enviar {
                numero,
                para,
                manter_aberto,
                remover_anotacoes,
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
    // A implementação já tem ensaios sintéticos, mas não passou pelo ciclo
    // de escrita e releitura numa instalação de treinamento (#333).
    // Bloquear a confirmação pública antes da sessão; manter a prévia livre.
    if *remover_anotacoes && *confirmar && !ensaio_local_habilitado() {
        let erro = ErroEnviar::RemocaoDeAnotacoesSemVerificacao;
        return Some(saida_erro::erro_fonte_http(
            cli.json,
            erro.subcodigo(),
            erro.mensagem(),
            None,
        ));
    }
    Some(enviar(
        cli,
        &nup,
        sigla,
        *manter_aberto,
        *remover_anotacoes,
        *confirmar,
    ))
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
    remover_anotacoes: bool,
    confirmar: bool,
) -> ExitCode {
    com_sessao(cli, |sessao, indice| {
        // Opt-in não autoriza tocar em uma instalação real, nem sequer GET.
        // A recusa ocorre antes de construir o pedido ou abrir a rede.
        if remover_anotacoes && confirmar && !destino_do_ensaio_local(sessao.origem) {
            let erro = ErroEnviar::RemocaoDeAnotacoesSemVerificacao;
            return saida_erro::erro_fonte_http(
                cli.json,
                erro.subcodigo(),
                erro.mensagem(),
                None,
            );
        }
        let pedido = PedidoDoEnvio {
            nup,
            unidade: cli.unidade.as_deref(),
            para,
            manter_aberto,
            remover_anotacoes,
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
                if cli.json {
                    if let Err(falha) = crate::saida::imprimir_json_compacto(
                        &json_de_falha_com_sugestoes(&erro, sessao, indice, para),
                    ) {
                        eprintln!("IES: falha ao emitir JSON: {falha}");
                    }
                } else {
                    for linha in linhas_da_falha(&erro) {
                        eprintln!("{linha}");
                    }
                    let sugestoes = sugestoes_guardadas(&erro, sessao, indice, para);
                    if let Some(mensagem) = texto_das_sugestoes(&sugestoes) {
                        eprintln!("{mensagem}");
                    }
                }
                ExitCode::from(erro.saida())
            }
        }
    })
}

/// Sugestões locais não são oferta do SEI nem autorizam uma escrita.
/// Não faz pedidos HTTP e somente é consultado para a falha pertinente.
fn sugestoes_guardadas(
    erro: &ErroEnviar,
    sessao: &crate::processo_obter::SessaoDoProcesso<'_>,
    indice: &crate::indice_local::IndiceLocal,
    para: &str,
) -> Vec<String> {
    if !matches!(erro, ErroEnviar::DestinoNaoOferecidoPeloSei { .. }) {
        return Vec::new();
    }
    let instalacao = crate::instalacao::Instalacao::interpretar(sessao.origem);
    let ativa = crate::indice_local::unidade_dos_links(sessao.url_do_controle);
    match (instalacao, ativa) {
        (Some(instalacao), Some(ativa)) => {
            indice.sugestoes_de_unidades_guardadas(&instalacao.url_base(), &ativa, para)
        }
        _ => Vec::new(),
    }
}

/// Projeção de falha para agentes; não cria sugestões nem as converte
/// de volta a valores Rust para a saída humana.
fn json_de_falha_com_sugestoes<'a>(
    erro: &'a ErroEnviar,
    sessao: &crate::processo_obter::SessaoDoProcesso<'_>,
    indice: &crate::indice_local::IndiceLocal,
    para: &str,
) -> FalhaDaSituacao<'a> {
    let mut valor = json_de_falha(erro);
    valor.sugestoes_guardadas = sugestoes_guardadas(erro, sessao, indice, para);
    valor
}

/// Apresentação humana só a partir de dados tipados do índice local.
fn texto_das_sugestoes(sugestoes: &[String]) -> Option<String> {
    if sugestoes.is_empty() {
        return None;
    }
    Some(format!(
        "IES: sugestões guardadas (não confirmadas pelo SEI): {}",
        sugestoes.join(", ")
    ))
}

/// Mantém a ordem histórica: mensagem, situação, falha de escrita,
/// verificação. Não constrói nem interpreta `serde_json::Value`.
fn linhas_da_falha(erro: &ErroEnviar) -> Vec<String> {
    use ErroEnviar as E;
    let mut linhas = vec![format!("IES: {} [{}]", erro.mensagem(), erro.subcodigo())];
    let aberto_em = match erro {
        E::ProcessoNaoAbertoNaUnidadeDaSessao { aberto_em }
        | E::UnidadeDaSessaoAusenteDaSituacao { aberto_em }
        | E::EnvioNaoPublicadoNaBarra { aberto_em }
        | E::ProcessoJaAbertoNoDestino { aberto_em }
        | E::EnvioNaoAplicado { aberto_em, .. }
        | E::SituacaoDaOrigemDiferenteDaPedidaAposEnvio { aberto_em, .. }
        | E::EfeitoIncertoNoEnvio { aberto_em, .. } => Some(aberto_em.as_slice()),
        _ => None,
    };
    let depois = match erro {
        E::EnvioNaoAplicado { depois, .. }
        | E::SituacaoDaOrigemDiferenteDaPedidaAposEnvio { depois, .. } => Some(depois.as_slice()),
        E::EfeitoIncertoNaRemocaoDeAnotacoes { depois } => depois.as_deref(),
        _ => None,
    };
    for (campo, lista) in [("aberto_em", aberto_em), ("depois", depois)] {
        if let Some(lista) = lista {
            let siglas: Vec<_> = lista.iter().map(String::as_str).collect();
            linhas.push(format!("IES: {campo}: {}", lista_humana(&siglas)));
        }
    }
    let falha_do_envio = match erro {
        E::EnvioNaoAplicado { falha_do_envio, .. }
        | E::SituacaoDaOrigemDiferenteDaPedidaAposEnvio { falha_do_envio, .. }
        | E::EfeitoIncertoNoEnvio { falha_do_envio, .. } => falha_do_envio.as_ref(),
        _ => None,
    };
    if let Some(falha) = falha_do_envio {
        linhas.push(format!("IES: resposta da escrita: {}", falha.subcodigo()));
    }
    if let E::EfeitoIncertoNoEnvio { verificacao, .. } = erro
        && let Some(motivo) = verificacao.motivo()
    {
        linhas.push(format!("IES: verificação: {motivo}"));
    }
    if let E::EfeitoIncertoNoEnvio {
        anotacao_depois: Some(false),
        ..
    } = erro
    {
        linhas.push("IES: anotacao_depois: removida (releitura antes da tramitação)".to_owned());
    }
    linhas
}

/// JSON do sucesso (prévia, ou enviado e conferido).
/// O contrato externo de destino continua isolado do tipo de domínio.
#[derive(serde::Serialize)]
struct DestinoPublico<'a> {
    descricao: &'a str,
    id_unidade: &'a str,
    sigla: &'a str,
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "quatro campos independentes exigidos pelo contrato JSON"
)]
#[derive(serde::Serialize)]
struct ResultadoEnvio<'a> {
    aberto_em: &'a [String],
    alterado: bool,
    confirmado: bool,
    depois: Option<&'a [String]>,
    depois_esperado: Vec<String>,
    destino: DestinoPublico<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    falha_do_envio: Option<crate::saida_erro::FalhaPublica>,
    id_procedimento: &'a str,
    manter_aberto: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    remover_anotacoes: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    anotacao_antes: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    anotacao_depois: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metodo_remocao: Option<crate::processo_enviar::MetodoRemocao>,
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
        remover_anotacoes: feito.remover_anotacoes.then_some(true),
        anotacao_antes: feito.anotacao_antes,
        anotacao_depois: feito.anotacao_depois,
        metodo_remocao: feito.metodo_remocao,
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
            .map(crate::processo_concluir_cli::falha_do_envio_publica),
    }
}

/// Compatibilidade com testes do contrato, sem Value no fluxo de produção.
#[cfg(test)]
pub(crate) fn json_de_sucesso(feito: &ProcessoEnviado) -> Value {
    serde_json::to_value(resultado_envio(feito))
        .expect("a projeção tipada do envio só possui valores JSON")
}

/// JSON da falha depois da abertura.
pub(crate) fn json_de_falha(erro: &ErroEnviar) -> FalhaDaSituacao<'_> {
    let mut valor = FalhaDaSituacao::nova(
        erro.codigo(),
        erro.subcodigo(),
        erro.mensagem(),
        erro.status_http(),
    );
    match erro {
        ErroEnviar::SituacaoDoProcessoNaoLidaAntesDeEnviar { motivo } => {
            valor.motivo = Some(motivo.como_texto());
        }
        ErroEnviar::ProcessoNaoAbertoNaUnidadeDaSessao { aberto_em }
        | ErroEnviar::UnidadeDaSessaoAusenteDaSituacao { aberto_em }
        | ErroEnviar::EnvioNaoPublicadoNaBarra { aberto_em }
        | ErroEnviar::ProcessoJaAbertoNoDestino { aberto_em } => {
            valor.aberto_em = Some(aberto_em);
        }
        ErroEnviar::DestinoNaoOferecidoPeloSei { oferecidos } => {
            valor.oferecidos = Some(oferecidos);
        }
        ErroEnviar::VariosDestinosComASigla { candidatos } => {
            valor.candidatos = Some(candidatos);
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
            valor.aberto_em = Some(aberto_em);
            valor.depois = Some(depois);
            valor.falha_do_envio = falha_do_envio.as_ref().map(falha_do_envio_publica);
        }
        ErroEnviar::EfeitoIncertoNaRemocaoDeAnotacoes { depois } => {
            valor.depois = depois.as_deref();
        }
        ErroEnviar::EfeitoIncertoNoEnvio {
            aberto_em,
            anotacao_depois,
            falha_do_envio,
            verificacao,
        } => {
            valor.aberto_em = Some(aberto_em);
            valor.anotacao_depois = *anotacao_depois;
            valor.verificacao = Some(verificacao.motivo());
            valor.falha_do_envio = falha_do_envio.as_ref().map(falha_do_envio_publica);
        }
        _ => {}
    }
    valor
}

#[cfg(test)]
#[path = "processo_enviar_cli/tests.rs"]
mod tests;
