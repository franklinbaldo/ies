//! O `Location` de um 302 do GET do Controle aponta para a entrada do
//! login da própria instalação? Sem regex e sem seguir o 302 (é dado).
//! Entradas observadas (docs/como-o-sei-funciona/sessao-login.md): o SIP
//! `/sip/login.php`, o módulo de Goiás
//! `<base>/modulos/goias/login-sei/{MdGoiasLoginSEI,login}.php` e a raiz
//! `<base>/` que a cadeia do login percorre. Mesmo host (`http://` no
//! mesmo host conta, como no login), caminho exato, sem fragmento e sem a
//! recusa de link sem assinatura.
use super::redirecionamento::resolver_no_host;
use crate::instalacao::Instalacao;
use crate::processo_obter::login::recusa_sem_assinatura;

/// Arquivos do módulo de login de Goiás, sob a base da instalação.
const MODULO_GO: &str = "/modulos/goias/login-sei/";
const ARQUIVOS_DO_MODULO_GO: [&str; 2] = ["MdGoiasLoginSEI.php", "login.php"];
const LOGIN_DO_SIP: &str = "/sip/login.php";

/// `true` só se `localizacao`, resolvida contra `atual` (a URL HTTPS do
/// Controle pedida), é uma entrada do login de `instalacao`.
pub(crate) fn aponta_para_o_login(instalacao: &Instalacao, atual: &str, localizacao: &str) -> bool {
    // O navegador descartaria o fragmento; aqui ele é forma não observada.
    if localizacao.contains('#') {
        return false;
    }
    let Ok(url) = resolver_no_host(atual, localizacao, instalacao.host()) else {
        return false;
    };
    let origem = format!("https://{}", instalacao.host());
    let Some(caminho_e_consulta) = url.strip_prefix(origem.as_str()) else {
        return false;
    };
    let (caminho, consulta) = caminho_e_consulta
        .split_once('?')
        .unwrap_or((caminho_e_consulta, ""));
    let base = instalacao.url_base();
    let base = base.strip_prefix(origem.as_str()).unwrap_or_default();
    let entrada = caminho == LOGIN_DO_SIP
        || caminho.strip_prefix(base) == Some("/")
        || caminho
            .strip_prefix(base)
            .and_then(|resto| resto.strip_prefix(MODULO_GO))
            .is_some_and(|arquivo| ARQUIVOS_DO_MODULO_GO.contains(&arquivo));
    entrada && !recusa_sem_assinatura(consulta)
}
