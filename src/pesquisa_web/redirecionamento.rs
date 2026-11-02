//! 302 da pesquisa de protocolo, **interpretado e nunca seguido**.
//! Forma observada nas cassettes `processo-busca-*` (`unb`, `pen1`, `ufrj`, `go`),
//! POSTs 6 e 10: `Location` relativo para `procedimento_trabalhar`.
//! Contrato: docs/arquitetura/0002-submeter-formulario-observado.md.
use crate::dominio::IdProcedimento;
use crate::formulario_web::Redirecionamento302;
use crate::instalacao::{Instalacao, host_valido};
use crate::transporte::{conferir_chave_de_consulta, conferir_valor_de_consulta};
use ErroRedirecionamentoDaPesquisa as E;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroRedirecionamentoDaPesquisa {
    LocalizacaoDoRedirecionamentoAusente,
    LocalizacaoDoRedirecionamentoAcimaDoLimite,
    LocalizacaoDoRedirecionamentoNaoEhUrlHttps,
    LocalizacaoDoRedirecionamentoForaDaInstalacao,
    ParametroNaoCanonicoNaLocalizacaoDoRedirecionamento,
    RedirecionamentoParaAcaoDiferenteDeProcedimentoTrabalhar,
    IdProcedimentoAusenteNoRedirecionamento,
    IdProcedimentoRepetidoNoRedirecionamento,
    IdProcedimentoNaoDecimalNoRedirecionamento,
    InfraHashAusenteNoRedirecionamento,
    InfraHashRepetidoNoRedirecionamento,
    InfraHashForaDoFormatoNoRedirecionamento,
    UnidadeAtualAusenteNoRedirecionamento,
    UnidadeAtualRepetidaNoRedirecionamento,
    UnidadeAtualAusenteNaUrlDoEnvio,
    UnidadeAtualRepetidaNaUrlDoEnvio,
    UnidadeAtualDoRedirecionamentoDiferenteDaDoEnvio,
}

impl ErroRedirecionamentoDaPesquisa {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::LocalizacaoDoRedirecionamentoAusente => "localizacao_do_redirecionamento_ausente",
            Self::LocalizacaoDoRedirecionamentoAcimaDoLimite => {
                "localizacao_do_redirecionamento_acima_do_limite"
            }
            Self::LocalizacaoDoRedirecionamentoNaoEhUrlHttps => {
                "localizacao_do_redirecionamento_nao_eh_url_https"
            }
            Self::LocalizacaoDoRedirecionamentoForaDaInstalacao => {
                "localizacao_do_redirecionamento_fora_da_instalacao"
            }
            Self::ParametroNaoCanonicoNaLocalizacaoDoRedirecionamento => {
                "parametro_nao_canonico_na_localizacao_do_redirecionamento"
            }
            Self::RedirecionamentoParaAcaoDiferenteDeProcedimentoTrabalhar => {
                "redirecionamento_para_acao_diferente_de_procedimento_trabalhar"
            }
            Self::IdProcedimentoAusenteNoRedirecionamento => {
                "id_procedimento_ausente_no_redirecionamento"
            }
            Self::IdProcedimentoRepetidoNoRedirecionamento => {
                "id_procedimento_repetido_no_redirecionamento"
            }
            Self::IdProcedimentoNaoDecimalNoRedirecionamento => {
                "id_procedimento_nao_decimal_no_redirecionamento"
            }
            Self::InfraHashAusenteNoRedirecionamento => "infra_hash_ausente_no_redirecionamento",
            Self::InfraHashRepetidoNoRedirecionamento => "infra_hash_repetido_no_redirecionamento",
            Self::InfraHashForaDoFormatoNoRedirecionamento => {
                "infra_hash_fora_do_formato_no_redirecionamento"
            }
            Self::UnidadeAtualAusenteNoRedirecionamento => {
                "unidade_atual_ausente_no_redirecionamento"
            }
            Self::UnidadeAtualRepetidaNoRedirecionamento => {
                "unidade_atual_repetida_no_redirecionamento"
            }
            Self::UnidadeAtualAusenteNaUrlDoEnvio => "unidade_atual_ausente_na_url_do_envio",
            Self::UnidadeAtualRepetidaNaUrlDoEnvio => "unidade_atual_repetida_na_url_do_envio",
            Self::UnidadeAtualDoRedirecionamentoDiferenteDaDoEnvio => {
                "unidade_atual_do_redirecionamento_diferente_da_do_envio"
            }
        }
    }
}

/// O `Location` observado tem algumas centenas de bytes; 8 KiB é o mesmo
/// teto dos arquivos privados de URL assinada.
const LIMITE_DA_LOCALIZACAO: usize = 8_192;

/// Host, caminho e consulta de uma URL `https://` absoluta.
fn partir_https(url: &str) -> Option<(&str, &str, &str)> {
    let (host, resto) = url.strip_prefix("https://")?.split_once('/')?;
    if !host_valido(host) {
        return None;
    }
    let (caminho, consulta) = resto.split_once('?').unwrap_or((resto, ""));
    Some((host, caminho, consulta))
}

fn tem_esquema(referencia: &str) -> bool {
    let fim = referencia.find(['/', '?']).unwrap_or(referencia.len());
    referencia[..fim]
        .split_once(':')
        .is_some_and(|(esquema, _)| {
            !esquema.is_empty()
                && esquema
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'-' | b'.'))
        })
}

/// Resolve como o navegador resolveria, sem normalizar `..`: o caminho
/// resultante precisa ser exatamente o controlador da instalação.
fn resolver(localizacao: &str, url_do_envio: &str) -> Result<String, E> {
    if !localizacao.bytes().all(|b| (33..=126).contains(&b)) || localizacao.contains(['#', '\\']) {
        return Err(E::LocalizacaoDoRedirecionamentoNaoEhUrlHttps);
    }
    let (host, caminho, _) =
        partir_https(url_do_envio).ok_or(E::LocalizacaoDoRedirecionamentoNaoEhUrlHttps)?;
    Ok(if localizacao.starts_with("https://") {
        localizacao.to_owned()
    } else if let Some(resto) = localizacao.strip_prefix("//") {
        format!("https://{resto}")
    } else if localizacao.starts_with('/') {
        format!("https://{host}{localizacao}")
    } else if tem_esquema(localizacao) {
        return Err(E::LocalizacaoDoRedirecionamentoNaoEhUrlHttps);
    } else {
        match caminho.rsplit_once('/') {
            Some((pasta, _)) => format!("https://{host}/{pasta}/{localizacao}"),
            None => format!("https://{host}/{localizacao}"),
        }
    })
}

/// Valores por nome de parâmetro, com nomes canônicos e sem `;`.
fn parametros(consulta: &str) -> Result<Vec<(&str, &str)>, E> {
    consulta
        .split('&')
        .map(|item| {
            let (nome, valor) = item
                .split_once('=')
                .ok_or(E::ParametroNaoCanonicoNaLocalizacaoDoRedirecionamento)?;
            conferir_chave_de_consulta(nome)
                .and_then(|()| conferir_valor_de_consulta(valor))
                .map_err(|_| E::ParametroNaoCanonicoNaLocalizacaoDoRedirecionamento)?;
            Ok((nome, valor))
        })
        .collect()
}

fn valores<'a>(parametros: &[(&str, &'a str)], nome: &str) -> Vec<&'a str> {
    parametros
        .iter()
        .filter(|(n, _)| *n == nome)
        .map(|(_, v)| *v)
        .collect()
}

fn unidade_do_envio(url_do_envio: &str) -> Result<String, E> {
    let consulta = partir_https(url_do_envio).map_or("", |(_, _, consulta)| consulta);
    let pares: Vec<_> = consulta
        .split('&')
        .filter_map(|i| i.split_once('='))
        .collect();
    match valores(&pares, "infra_unidade_atual").as_slice() {
        [] => Err(E::UnidadeAtualAusenteNaUrlDoEnvio),
        [unidade] => Ok((*unidade).to_owned()),
        _ => Err(E::UnidadeAtualRepetidaNaUrlDoEnvio),
    }
}

/// Extrai o `id_procedimento` do 302 e descarta a URL assinada (consumida).
/// Nunca emite GET ao destino: abrir o processo pode registrar recebimento.
pub(crate) fn interpretar_redirecionamento(
    redirecionamento: Redirecionamento302,
    instalacao: &Instalacao,
) -> Result<IdProcedimento, ErroRedirecionamentoDaPesquisa> {
    let localizacao = redirecionamento
        .localizacao()
        .filter(|l| !l.is_empty())
        .ok_or(E::LocalizacaoDoRedirecionamentoAusente)?;
    if localizacao.len() > LIMITE_DA_LOCALIZACAO {
        return Err(E::LocalizacaoDoRedirecionamentoAcimaDoLimite);
    }
    let absoluta = resolver(localizacao, redirecionamento.url_do_envio())?;
    let (host, caminho, consulta) =
        partir_https(&absoluta).ok_or(E::LocalizacaoDoRedirecionamentoNaoEhUrlHttps)?;
    if !instalacao.autoriza_controlador(host, caminho) {
        return Err(E::LocalizacaoDoRedirecionamentoForaDaInstalacao);
    }
    let pares = parametros(consulta)?;
    if valores(&pares, "acao") != ["procedimento_trabalhar"] {
        return Err(E::RedirecionamentoParaAcaoDiferenteDeProcedimentoTrabalhar);
    }
    let id = match valores(&pares, "id_procedimento").as_slice() {
        [] | [""] => return Err(E::IdProcedimentoAusenteNoRedirecionamento),
        [id] => {
            IdProcedimento::novo(id).map_err(|_| E::IdProcedimentoNaoDecimalNoRedirecionamento)?
        }
        _ => return Err(E::IdProcedimentoRepetidoNoRedirecionamento),
    };
    match valores(&pares, "infra_hash").as_slice() {
        [] => return Err(E::InfraHashAusenteNoRedirecionamento),
        [hash] if hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()) => {}
        [_] => return Err(E::InfraHashForaDoFormatoNoRedirecionamento),
        _ => return Err(E::InfraHashRepetidoNoRedirecionamento),
    }
    let unidade = match valores(&pares, "infra_unidade_atual").as_slice() {
        [] => return Err(E::UnidadeAtualAusenteNoRedirecionamento),
        [unidade] => *unidade,
        _ => return Err(E::UnidadeAtualRepetidaNoRedirecionamento),
    };
    if unidade != unidade_do_envio(redirecionamento.url_do_envio())? {
        return Err(E::UnidadeAtualDoRedirecionamentoDiferenteDaDoEnvio);
    }
    // Location e URL assinados descartados aqui, antes de devolver o id.
    drop(redirecionamento);
    Ok(id)
}

#[cfg(test)]
#[path = "tests_redirecionamento.rs"]
mod tests;
