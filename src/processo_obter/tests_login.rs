//! RED/GREEN da leitura do `Location` que vai para a tela de login.
use super::login::{DestinoDoRedirecionamento, destino};

#[test]
fn recusa_caracteristica_da_entrada_sem_assinatura() {
    for localizacao in [
        "https://sei.exemplo.gov.br/sip/login.php?sigla_orgao_sistema=ORG&sigla_sistema=SEI&msg=Link%20sem%20assinatura",
        "https://sei.exemplo.gov.br/sip/login.php?msg=Link+sem+assinatura&sigla_sistema=SEI",
        "/sip/login.php?msg=Link%20sem%20assinatura",
        "login.php?msg=Link sem assinatura",
    ] {
        assert_eq!(
            destino(Some(localizacao)),
            DestinoDoRedirecionamento::RecusaSemAssinatura,
            "{localizacao}"
        );
    }
}

#[test]
fn login_sem_a_mensagem_e_sessao_vencida() {
    for localizacao in [
        "https://sei.exemplo.gov.br/sip/login.php?sigla_orgao_sistema=ORG&sigla_sistema=SEI",
        "https://sei.exemplo.gov.br/sip/login.php?msg=Sess%C3%A3o%20expirada",
        "https://sei.exemplo.gov.br/sip/LOGIN.PHP",
        // A mensagem precisa ser exatamente a observada.
        "https://sei.exemplo.gov.br/sip/login.php?msg=Link%20sem%20assinatura%20nenhuma",
        "https://sei.exemplo.gov.br/sip/login.php?msg=Link%2",
    ] {
        assert_eq!(
            destino(Some(localizacao)),
            DestinoDoRedirecionamento::Login,
            "{localizacao}"
        );
    }
}

#[test]
fn outros_destinos() {
    for localizacao in [
        None,
        Some(""),
        Some("https://sei.exemplo.gov.br/sei/inicio.php"),
        Some("https://sei.exemplo.gov.br/sei/controlador.php?acao=login.php"),
        Some("https://sei.exemplo.gov.br/sei/naologin.php?msg=Link%20sem%20assinatura"),
    ] {
        assert_eq!(
            destino(localizacao),
            DestinoDoRedirecionamento::Outro,
            "{localizacao:?}"
        );
    }
}
