//! O `Location` de um 302 de escrita, **lido como dado e nunca seguido**:
//! a consulta de `controlador.php` quando ele aponta para o mesmo
//! controlador do POST.
//!
//! Formas aceitas: a relativa `controlador.php?…` (a gravada nos 302 de
//! `procedimento_enviar` e `procedimento_concluir` das sete instalações,
//! GO inclusive) e a absoluta `http://` ou `https://` no **host do POST**,
//! na mesma pasta (o Todos mediu no GO o `Location` de escritas em
//! `http://` no mesmo host, issue #766; as cassettes do GO só o gravam
//! no `infra_trocar_unidade` e no `procedimento_controlar`). Outro host,
//! outra pasta, porta (salvo `:80` em `http://`, a padrão), credenciais ou
//! outro esquema: nenhuma consulta.
use super::leitura::UrlHttps;
use crate::sessao_login::redirecionamento::sufixo_da_mesma_origem;

/// A consulta depois de `controlador.php?`, se `localizacao` endereça o
/// controlador de `url_do_envio` (URL HTTPS do POST).
pub(crate) fn consulta_do_controlador<'a>(
    localizacao: &'a str,
    url_do_envio: &str,
) -> Option<&'a str> {
    if let Some(consulta) = localizacao.strip_prefix("controlador.php?") {
        return Some(consulta);
    }
    let post = UrlHttps::interpretar(url_do_envio)?;
    if !localizacao.bytes().all(|b| (33..=126).contains(&b)) || localizacao.contains(['#', '\\']) {
        return None;
    }
    let (esquema, resto) = localizacao.split_once("://")?;
    if !(esquema.eq_ignore_ascii_case("http") || esquema.eq_ignore_ascii_case("https")) {
        return None;
    }
    // A mesma regra de autoridade do login e da pesquisa (#201).
    let http = esquema.eq_ignore_ascii_case("http");
    let Ok(depois) = sufixo_da_mesma_origem(resto, post.host, http) else {
        return None;
    };
    let caminho_e_consulta = depois.strip_prefix('/')?;
    let (caminho, consulta) = caminho_e_consulta.split_once('?')?;
    let pasta = post.caminho.rsplit_once('/').map(|(pasta, _)| pasta)?;
    (caminho.strip_prefix(pasta) == Some("/controlador.php")).then_some(consulta)
}

#[cfg(test)]
#[path = "tests_redirecionamento.rs"]
mod tests;
