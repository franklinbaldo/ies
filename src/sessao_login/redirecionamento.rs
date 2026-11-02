//! Interpretador explícito de `Location` do login, sem regex e sem
//! normalizar caminhos. Só o host da instalação é aceito. Regra observada
//! (UFRJ e GO, 02/11/2026): `http://` no mesmo host é promovido a
//! `https://`; a URL HTTP nunca é pedida.
use super::ErroLogin;
use crate::instalacao::host_valido;

/// Próxima URL HTTPS de um salto: `localizacao` resolvida contra `atual`
/// (URL HTTPS absoluta já aceita) e conferida contra `host`.
pub(crate) fn resolver_no_host(
    atual: &str,
    localizacao: &str,
    host: &str,
) -> Result<String, ErroLogin> {
    use ErroLogin::LocalizacaoDeRedirecionamentoNaoInterpretavel as Ilegivel;
    let referencia = localizacao.trim();
    if referencia.is_empty()
        || !referencia.bytes().all(|b| (33..=126).contains(&b))
        || referencia.contains('\\')
    {
        return Err(Ilegivel);
    }
    // O navegador não envia o fragmento.
    let referencia = referencia.split('#').next().unwrap_or_default();
    let caminho_atual = caminho_da_url(atual).ok_or(Ilegivel)?;
    let resto_absoluto = absoluta(referencia).or_else(|| referencia.strip_prefix("//"));
    let caminho_e_consulta = if let Some(resto) = resto_absoluto {
        let fim = resto.find(['/', '?']).unwrap_or(resto.len());
        let autoridade = &resto[..fim];
        if autoridade.contains(['@', ':']) || !host_valido(autoridade) {
            return Err(Ilegivel);
        }
        if !autoridade.eq_ignore_ascii_case(host) {
            return Err(ErroLogin::RedirecionamentoParaForaDaInstalacao);
        }
        match &resto[fim..] {
            "" => "/".to_owned(),
            depois if depois.starts_with('?') => format!("/{depois}"),
            depois => depois.to_owned(),
        }
    } else if tem_esquema(referencia) {
        return Err(Ilegivel);
    } else if referencia.starts_with('/') {
        referencia.to_owned()
    } else if referencia.starts_with('?') {
        format!("{caminho_atual}{referencia}")
    } else {
        let pasta = caminho_atual
            .rsplit_once('/')
            .map_or("", |(pasta, _)| pasta);
        format!("{pasta}/{referencia}")
    };
    let caminho = caminho_e_consulta.split('?').next().unwrap_or_default();
    if caminho
        .split('/')
        .skip(1)
        .any(|segmento| segmento == "." || segmento == "..")
    {
        return Err(Ilegivel);
    }
    Ok(format!(
        "https://{}{caminho_e_consulta}",
        host.to_ascii_lowercase()
    ))
}

/// O resto depois de `https://` ou `http://` (esquema sem distinção de
/// caixa); `http://` só chega aqui para ser promovido no mesmo host.
fn absoluta(referencia: &str) -> Option<&str> {
    let (esquema, resto) = referencia.split_once("://")?;
    (esquema.eq_ignore_ascii_case("https") || esquema.eq_ignore_ascii_case("http")).then_some(resto)
}

fn tem_esquema(referencia: &str) -> bool {
    let fim = referencia.find(['/', '?']).unwrap_or(referencia.len());
    referencia[..fim]
        .split_once(':')
        .is_some_and(|(esquema, _)| !esquema.is_empty())
}

/// Caminho (com `/` inicial) de uma URL HTTPS absoluta, sem a consulta.
pub(crate) fn caminho_da_url(url: &str) -> Option<&str> {
    let resto = url.strip_prefix("https://")?;
    let inicio = resto.find('/')?;
    let caminho = &resto[inicio..];
    Some(caminho.split('?').next().unwrap_or(caminho))
}
