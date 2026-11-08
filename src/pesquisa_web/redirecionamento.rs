//! 302 da pesquisa de protocolo, **interpretado e nunca seguido**.
//! Forma observada nas cassettes `processo-busca-*` (`unb`, `pen1`, `ufrj`, `go`),
//! POSTs 6 e 10: `Location` relativo para `procedimento_trabalhar`.
//! Contrato: docs/arquitetura/0002-submeter-formulario-observado.md.
use crate::dominio::IdProcedimento;
use crate::formulario_web::Redirecionamento302;
use crate::instalacao::Instalacao;
use crate::sessao_login::redirecionamento::{ErroDeOrigem, esquema_http, sufixo_da_mesma_origem};
use crate::transporte::{conferir_chave_de_consulta, conferir_valor_de_consulta};
use crate::url_observada::{esquema_e_resto, partes_https, pasta, tem_esquema};
use ErroRedirecionamentoDaPesquisa as E;

#[derive(Clone, Copy, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroRedirecionamentoDaPesquisa {
    LocalizacaoDoRedirecionamentoAusente,
    LocalizacaoDoRedirecionamentoAcimaDoLimite,
    LocalizacaoDoRedirecionamentoNaoEhUrlHttps,
    LocalizacaoDoRedirecionamentoForaDaInstalacao,
    ParametroNaoCanonicoNaLocalizacaoDoRedirecionamento,
    RedirecionamentoParaAcaoDiferenteDeProcedimentoTrabalhar,
    AcaoDeOrigemAusenteNoRedirecionamento,
    AcaoDeOrigemRepetidaNoRedirecionamento,
    AcaoDeOrigemDoRedirecionamentoDiferenteDaPesquisa,
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
    InfraSistemaAusenteNoRedirecionamento,
    InfraSistemaRepetidoNoRedirecionamento,
    InfraSistemaNaoDecimalNoRedirecionamento,
    InfraSistemaAusenteNaUrlDoEnvio,
    InfraSistemaRepetidoNaUrlDoEnvio,
    InfraSistemaNaoDecimalNaUrlDoEnvio,
    InfraSistemaDoRedirecionamentoDiferenteDoEnvio,
    /// `acao_origem` presente onde a forma observada não o traz (entrada por id).
    AcaoDeOrigemInesperadaNoRedirecionamento,
    /// O id endereçado não é o pedido (entrada por id, linha do Controle).
    IdProcedimentoDoRedirecionamentoDiferenteDoPedido,
    /// A entrada com documento respondeu sem o `id_documento` pedido, uma vez.
    IdDocumentoDoRedirecionamentoDiferenteDoPedido,
    /// Pesquisa em documentos: o 302 do número SEI traz `id_documento`
    /// decimal; vazio (a forma do processo) ou ausente não é documento.
    IdDocumentoAusenteNoRedirecionamento,
    IdDocumentoRepetidoNoRedirecionamento,
    IdDocumentoNaoDecimalNoRedirecionamento,
}

impl ErroRedirecionamentoDaPesquisa {}

/// URL absoluta de `procedimento_trabalhar` já conferida (mesma instalação,
/// uma `acao`, chaves canônicas, `infra_hash` de 64 hex). É assinada:
/// nunca vai para `Debug`, saída ou log; só `ies processo NUP` a usa num GET.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct UrlDeTrabalho(String);

impl UrlDeTrabalho {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for UrlDeTrabalho {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("UrlDeTrabalho(<redigido>)")
    }
}

/// O que o 302 da pesquisa endereça: o id interno e a URL validada.
#[derive(Debug)]
pub(crate) struct ProcessoRedirecionado {
    pub(crate) id: IdProcedimento,
    pub(crate) trabalho: UrlDeTrabalho,
}

/// De onde veio a URL de `procedimento_trabalhar`: cada forma observada
/// tem seu `acao_origem`, sua chave do id e sua referência de comparação.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OrigemDoTrabalho<'a> {
    /// 302 da pesquisa avançada (`processo buscar`): `acao_origem=protocolo_pesquisar`.
    Pesquisa,
    /// 302 da pesquisa rápida (cassettes de GO): `acao_origem=protocolo_pesquisa_rapida`
    /// e o id em `id_protocolo`.
    PesquisaRapida,
    /// Busca rápida de número SEI: a cassette publica `id_documento`,
    /// não o `id_protocolo` da consulta por NUP.
    PesquisaRapidaDocumento,
    /// 302 da entrada sem assinatura (ADR 0003): sem `acao_origem`, mesmo id;
    /// a URL pedida não tem unidade nem sistema para comparar.
    EntradaPorId { id: &'a str },
    /// A mesma entrada com `id_documento` (ADR 0003): sem `acao_origem`, os
    /// dois ids iguais aos pedidos.
    EntradaPorIdComDocumento { id: &'a str, documento: &'a str },
    /// `href` da linha `tr#P{id}` do Controle: `acao_origem=procedimento_controlar`.
    Controle { id: &'a str },
}

impl OrigemDoTrabalho<'_> {
    fn acao_de_origem(self) -> Option<&'static str> {
        match self {
            Self::Pesquisa => Some("protocolo_pesquisar"),
            Self::PesquisaRapida | Self::PesquisaRapidaDocumento => {
                Some("protocolo_pesquisa_rapida")
            }
            Self::EntradaPorId { .. } | Self::EntradaPorIdComDocumento { .. } => None,
            Self::Controle { .. } => Some("procedimento_controlar"),
        }
    }
    fn chave_do_id(self) -> &'static str {
        match self {
            Self::PesquisaRapida => "id_protocolo",
            Self::PesquisaRapidaDocumento => "id_documento",
            _ => "id_procedimento",
        }
    }
}

/// O `Location` observado tem algumas centenas de bytes; 8 KiB é o mesmo
/// teto dos arquivos privados de URL assinada.
const LIMITE_DA_LOCALIZACAO: usize = 8_192;

/// Host, caminho e consulta de uma URL `https://` absoluta.
fn partir_https(url: &str) -> Option<(&str, &str, &str)> {
    let partes = partes_https(url)?;
    Some((partes.host, partes.caminho, partes.consulta.unwrap_or("")))
}

/// Resolve como o navegador resolveria, sem normalizar `..`: o caminho
/// resultante precisa ser exatamente o controlador da instalação.
fn resolver(localizacao: &str, url_do_envio: &str, aceita_http: bool) -> Result<String, E> {
    if !localizacao.bytes().all(|b| (33..=126).contains(&b)) || localizacao.contains(['#', '\\']) {
        return Err(E::LocalizacaoDoRedirecionamentoNaoEhUrlHttps);
    }
    let (host, caminho, _) =
        partir_https(url_do_envio).ok_or(E::LocalizacaoDoRedirecionamentoNaoEhUrlHttps)?;
    let http = aceita_http && esquema_http(localizacao);
    let resto_absoluto = localizacao
        .strip_prefix("https://")
        .or_else(|| localizacao.strip_prefix("//"))
        .or_else(|| {
            http.then_some(localizacao)
                .and_then(esquema_e_resto)
                .map(|(_, resto)| resto)
        });
    Ok(if let Some(resto) = resto_absoluto {
        let sufixo = sufixo_da_mesma_origem(resto, host, http).map_err(|erro| match erro {
            ErroDeOrigem::OutroHost => E::LocalizacaoDoRedirecionamentoForaDaInstalacao,
            ErroDeOrigem::AutoridadeInvalida => E::LocalizacaoDoRedirecionamentoNaoEhUrlHttps,
        })?;
        format!("https://{host}{sufixo}")
    } else if localizacao.starts_with('/') {
        format!("https://{host}{localizacao}")
    } else if tem_esquema(localizacao) {
        return Err(E::LocalizacaoDoRedirecionamentoNaoEhUrlHttps);
    } else {
        match pasta(caminho) {
            Some(diretorio) => format!("https://{host}/{diretorio}/{localizacao}"),
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

fn parametros_do_envio(url_do_envio: &str) -> Vec<(&str, &str)> {
    let consulta = partir_https(url_do_envio).map_or("", |(_, _, consulta)| consulta);
    consulta
        .split('&')
        .filter_map(|i| i.split_once('='))
        .collect()
}

fn unidade_do_envio(url_do_envio: &str) -> Result<String, E> {
    match valores(&parametros_do_envio(url_do_envio), "infra_unidade_atual").as_slice() {
        [] => Err(E::UnidadeAtualAusenteNaUrlDoEnvio),
        [unidade] => Ok((*unidade).to_owned()),
        _ => Err(E::UnidadeAtualRepetidaNaUrlDoEnvio),
    }
}

fn decimal(valor: &str) -> bool {
    !valor.is_empty() && valor.bytes().all(|b| b.is_ascii_digit())
}

/// O POST observado é o `action` assinado do formulário, que traz
/// `infra_sistema`; sem ele não há com o que comparar: falha fechada.
fn sistema_do_envio(url_do_envio: &str) -> Result<String, E> {
    match valores(&parametros_do_envio(url_do_envio), "infra_sistema").as_slice() {
        [] => Err(E::InfraSistemaAusenteNaUrlDoEnvio),
        [sistema] if decimal(sistema) => Ok((*sistema).to_owned()),
        [_] => Err(E::InfraSistemaNaoDecimalNaUrlDoEnvio),
        _ => Err(E::InfraSistemaRepetidoNaUrlDoEnvio),
    }
}

/// Extrai o `id_procedimento` e a URL resolvida do 302. Nenhum GET aqui:
/// `processo buscar` descarta a URL; só `ies processo NUP` a abre, porque
/// abrir o processo altera o estado da unidade (docs/decisoes/processo-obter.md).
pub(crate) fn interpretar_redirecionamento(
    redirecionamento: Redirecionamento302,
    instalacao: &Instalacao,
) -> Result<ProcessoRedirecionado, ErroRedirecionamentoDaPesquisa> {
    let processo = conferir_trabalho(
        redirecionamento.localizacao(),
        redirecionamento.url_do_envio(),
        instalacao,
        OrigemDoTrabalho::Pesquisa,
    )?;
    // O `Location` bruto é descartado; só a forma resolvida e conferida sai.
    drop(redirecionamento);
    Ok(processo)
}

/// Confere uma referência a `procedimento_trabalhar` (o `Location` de um 302
/// ou o `href` de uma linha do Controle), resolvida contra `url_base` (a URL
/// pedida ou a da tela), na forma observada para `origem`.
pub(crate) fn conferir_trabalho(
    localizacao: Option<&str>,
    url_base: &str,
    instalacao: &Instalacao,
    origem: OrigemDoTrabalho<'_>,
) -> Result<ProcessoRedirecionado, ErroRedirecionamentoDaPesquisa> {
    let localizacao = localizacao
        .filter(|l| !l.is_empty())
        .ok_or(E::LocalizacaoDoRedirecionamentoAusente)?;
    if localizacao.len() > LIMITE_DA_LOCALIZACAO {
        return Err(E::LocalizacaoDoRedirecionamentoAcimaDoLimite);
    }
    let aceita_http = !matches!(origem, OrigemDoTrabalho::Controle { .. });
    let absoluta = resolver(localizacao, url_base, aceita_http)?;
    let (host, caminho, consulta) =
        partir_https(&absoluta).ok_or(E::LocalizacaoDoRedirecionamentoNaoEhUrlHttps)?;
    if !instalacao.autoriza_controlador(host, caminho) {
        return Err(E::LocalizacaoDoRedirecionamentoForaDaInstalacao);
    }
    let pares = parametros(consulta)?;
    if valores(&pares, "acao") != ["procedimento_trabalhar"] {
        return Err(E::RedirecionamentoParaAcaoDiferenteDeProcedimentoTrabalhar);
    }
    match (
        valores(&pares, "acao_origem").as_slice(),
        origem.acao_de_origem(),
    ) {
        ([], None) => {}
        ([], Some(_)) => return Err(E::AcaoDeOrigemAusenteNoRedirecionamento),
        ([_, ..], None) => return Err(E::AcaoDeOrigemInesperadaNoRedirecionamento),
        ([observada], Some(esperada)) if *observada == esperada => {}
        ([_], Some(_)) => return Err(E::AcaoDeOrigemDoRedirecionamentoDiferenteDaPesquisa),
        (_, Some(_)) => return Err(E::AcaoDeOrigemRepetidaNoRedirecionamento),
    }
    let documento = matches!(origem, OrigemDoTrabalho::PesquisaRapidaDocumento);
    let id = match valores(&pares, origem.chave_do_id()).as_slice() {
        [] | [""] if documento => return Err(E::IdDocumentoAusenteNoRedirecionamento),
        [] | [""] => return Err(E::IdProcedimentoAusenteNoRedirecionamento),
        [id] if documento => {
            IdProcedimento::novo(id).map_err(|_| E::IdDocumentoNaoDecimalNoRedirecionamento)?
        }
        [id] => {
            IdProcedimento::novo(id).map_err(|_| E::IdProcedimentoNaoDecimalNoRedirecionamento)?
        }
        _ if documento => return Err(E::IdDocumentoRepetidoNoRedirecionamento),
        _ => return Err(E::IdProcedimentoRepetidoNoRedirecionamento),
    };
    if let OrigemDoTrabalho::EntradaPorId { id: pedido }
    | OrigemDoTrabalho::Controle { id: pedido }
    | OrigemDoTrabalho::EntradaPorIdComDocumento { id: pedido, .. } = origem
        && id.as_str() != pedido
    {
        return Err(E::IdProcedimentoDoRedirecionamentoDiferenteDoPedido);
    }
    if let OrigemDoTrabalho::EntradaPorIdComDocumento { documento, .. } = origem
        && valores(&pares, "id_documento") != [documento]
    {
        return Err(E::IdDocumentoDoRedirecionamentoDiferenteDoPedido);
    }
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
    let sem_referencia = matches!(
        origem,
        OrigemDoTrabalho::EntradaPorId { .. } | OrigemDoTrabalho::EntradaPorIdComDocumento { .. }
    );
    if !sem_referencia && unidade != unidade_do_envio(url_base)? {
        return Err(E::UnidadeAtualDoRedirecionamentoDiferenteDaDoEnvio);
    }
    let sistema = match valores(&pares, "infra_sistema").as_slice() {
        [] => return Err(E::InfraSistemaAusenteNoRedirecionamento),
        [sistema] if decimal(sistema) => *sistema,
        [_] => return Err(E::InfraSistemaNaoDecimalNoRedirecionamento),
        _ => return Err(E::InfraSistemaRepetidoNoRedirecionamento),
    };
    // Comparação exata, sem normalizar zeros à esquerda.
    if !sem_referencia && sistema != sistema_do_envio(url_base)? {
        return Err(E::InfraSistemaDoRedirecionamentoDiferenteDoEnvio);
    }
    Ok(ProcessoRedirecionado {
        id,
        trabalho: UrlDeTrabalho(absoluta),
    })
}

/// O `id_documento` da URL de trabalho já conferida (302 da pesquisa em
/// documentos): exatamente um, decimal e não vazio.
pub(crate) fn documento_do_trabalho(
    trabalho: &UrlDeTrabalho,
) -> Result<String, ErroRedirecionamentoDaPesquisa> {
    let consulta = partir_https(trabalho.as_str()).map_or("", |(_, _, consulta)| consulta);
    let pares = parametros(consulta)?;
    match valores(&pares, "id_documento").as_slice() {
        [] | [""] => Err(E::IdDocumentoAusenteNoRedirecionamento),
        [id] if decimal(id) => Ok((*id).to_owned()),
        [_] => Err(E::IdDocumentoNaoDecimalNoRedirecionamento),
        _ => Err(E::IdDocumentoRepetidoNoRedirecionamento),
    }
}

#[cfg(test)]
#[path = "tests_redirecionamento.rs"]
mod tests;
