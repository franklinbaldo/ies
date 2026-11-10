//! Endereços de navegação observados nas telas de criação; sem seguir
//! redirecionamentos automaticamente e sem inferir IDs ou URLs assinadas.
use super::ErroCriar;
use crate::instalacao::Instalacao;

pub(super) fn id_do_link(link: &str) -> Option<&str> {
    let mut ids =
        link.split('?').nth(1)?.split('&').filter(|par| {
            par.split_once('=').map_or(*par, |(nome, _)| nome) == "id_tipo_procedimento"
        });
    let primeiro = ids.next()?.strip_prefix("id_tipo_procedimento=")?;
    (ids.next().is_none() && !primeiro.is_empty() && primeiro.bytes().all(|b| b.is_ascii_digit()))
        .then_some(primeiro)
}

/// Exige exatamente uma ocorrência do parâmetro acao, com valor literal
/// esperado. Uma segunda ação diferente também torna a URL ambígua.
fn acao_unica(consulta: &str, esperada: &str) -> bool {
    let mut acoes = consulta.split('&').filter_map(|par| {
        let (nome, valor) = par.split_once('=').unwrap_or((par, ""));
        (nome == "acao").then_some(valor)
    });
    acoes.next() == Some(esperada) && acoes.next().is_none()
}

pub(super) fn resolver_link(
    instalacao: &Instalacao,
    base: &str,
    referencia: &str,
    acao: &str,
) -> Result<String, ErroCriar> {
    if !referencia.starts_with("controlador.php?")
        || referencia.contains(['#', '\\', '\''])
        || !referencia.bytes().all(|b| (33..=126).contains(&b))
    {
        return Err(ErroCriar::Formulario);
    }
    let base = base.split('?').next().ok_or(ErroCriar::Formulario)?;
    let (pasta, _) = base.rsplit_once('/').ok_or(ErroCriar::Formulario)?;
    let url = format!("{pasta}/{referencia}");
    let partes = crate::url_observada::UrlHttps::interpretar(&url).ok_or(ErroCriar::Formulario)?;
    if !instalacao.autoriza_controlador(partes.host, partes.caminho) {
        return Err(ErroCriar::Formulario);
    }
    if !acao_unica(partes.consulta.unwrap_or_default(), acao) {
        return Err(ErroCriar::Formulario);
    }
    Ok(url)
}

pub(super) fn url_do_redirecionamento(
    instalacao: &Instalacao,
    localizacao: &str,
    url_envio: &str,
    acao: &str,
) -> Result<String, ErroCriar> {
    let absoluta = if localizacao.starts_with("https://") {
        localizacao.to_owned()
    } else {
        resolver_link(instalacao, url_envio, localizacao, acao)?
    };
    let partes =
        crate::url_observada::UrlHttps::interpretar(&absoluta).ok_or(ErroCriar::EfeitoIncerto)?;
    if !instalacao.autoriza_controlador(partes.host, partes.caminho) {
        return Err(ErroCriar::EfeitoIncerto);
    }
    if !acao_unica(partes.consulta.unwrap_or_default(), acao) {
        return Err(ErroCriar::EfeitoIncerto);
    }
    Ok(absoluta)
}

pub(super) fn retorno_da_criacao(
    instalacao: &Instalacao,
    localizacao: &str,
    url_envio: &str,
) -> Result<(String, String), ErroCriar> {
    // A `UnB` termina o Location com a âncora da árvore (`#ID-<id>`); o
    // fragmento não vai ao servidor e fica fora da validação.
    let localizacao = localizacao
        .split_once('#')
        .map_or(localizacao, |(url, _)| url);
    let url = url_do_redirecionamento(instalacao, localizacao, url_envio, "procedimento_trabalhar")
        .map_err(|_| ErroCriar::EfeitoIncerto)?;
    let consulta = crate::url_observada::UrlHttps::interpretar(&url)
        .and_then(|p| p.consulta)
        .ok_or(ErroCriar::EfeitoIncerto)?;
    let mut ids = consulta
        .split('&')
        .filter(|par| par.split_once('=').map_or(*par, |(nome, _)| nome) == "id_procedimento");
    let primeiro = ids.next().ok_or(ErroCriar::EfeitoIncerto)?;
    if ids.next().is_some() {
        return Err(ErroCriar::EfeitoIncerto);
    }
    let id = primeiro
        .strip_prefix("id_procedimento=")
        .filter(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
        .ok_or(ErroCriar::EfeitoIncerto)?
        .to_owned();
    Ok((url, id))
}
