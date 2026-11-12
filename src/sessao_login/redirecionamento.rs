//! Interpretador explícito de `Location` do login, sem regex e sem
//! normalizar caminhos. Só o host da instalação é aceito. Regra observada
//! (UFRJ e GO, 02/11/2026): `http://` no mesmo host é promovido a
//! `https://`; a URL HTTP nunca é pedida.
use super::ErroLogin;
use crate::instalacao::host_valido;
pub(crate) use crate::url_observada::caminho_absoluto as caminho_da_url;
use crate::url_observada::{esquema_e_resto, resto_http_ou_https, tem_prefixo_de_esquema};

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
    let resto_absoluto = resto_http_ou_https(referencia).or_else(|| referencia.strip_prefix("//"));
    let caminho_e_consulta = if let Some(resto) = resto_absoluto {
        let depois = sufixo_da_mesma_origem(resto, host, esquema_http(referencia)).map_err(
            |erro| match erro {
                ErroDeOrigem::AutoridadeInvalida => Ilegivel,
                ErroDeOrigem::OutroHost => ErroLogin::RedirecionamentoParaForaDaInstalacao,
            },
        )?;
        match depois {
            "" => "/".to_owned(),
            depois if depois.starts_with('?') => format!("/{depois}"),
            depois => depois.to_owned(),
        }
    } else if tem_prefixo_de_esquema(referencia) {
        return Err(Ilegivel);
    } else if referencia.starts_with('/') {
        referencia.to_owned()
    } else if referencia.starts_with('?') {
        format!("{caminho_atual}{referencia}")
    } else {
        // `caminho_da_url` começa por `/`: a pasta, com a barra final, é
        // tudo até a última `/` e nunca falta.
        let diretorio = caminho_atual.trim_end_matches(|c| c != '/');
        format!("{diretorio}{referencia}")
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

pub(crate) enum ErroDeOrigem {
    AutoridadeInvalida,
    OutroHost,
}

/// Valida a autoridade de uma referência absoluta ou iniciada por `//`.
/// O chamador identifica o esquema e continua responsável pelo caminho.
pub(crate) fn sufixo_da_mesma_origem<'a>(
    resto: &'a str,
    host: &str,
    http: bool,
) -> Result<&'a str, ErroDeOrigem> {
    let (autoridade, sufixo) = resto.split_at(resto.find(['/', '?']).unwrap_or(resto.len()));
    let autoridade = if http {
        autoridade.strip_suffix(":80").unwrap_or(autoridade)
    } else {
        autoridade
    };
    if !host_valido(autoridade) {
        return Err(ErroDeOrigem::AutoridadeInvalida);
    }
    if !autoridade.eq_ignore_ascii_case(host) {
        return Err(ErroDeOrigem::OutroHost);
    }
    Ok(sufixo)
}

pub(crate) fn esquema_http(referencia: &str) -> bool {
    esquema_e_resto(referencia).is_some_and(|(esquema, _)| esquema.eq_ignore_ascii_case("http"))
}
