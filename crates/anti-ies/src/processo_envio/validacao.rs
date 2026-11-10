//! Modal de revalidação hipotética, projetada de estado tipado no SQLite.
//! Não é reprodução do HTML/POST de Goiás: a cassette não contém esta ação.
use super::{Estado, Resposta, campo_unico, html, ler, pares, redirecionado, resposta_html, unico};
use iesei::instalacao::Instalacao;
use iesei::validacao_acesso::{self, Validacao};
use rusqlite::Connection;

/// Simula uma única validação de sessão, sem armazenar senhas reais.
#[must_use]
pub fn responder(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    let Some(id) = unico(alvo, "id_procedimento")
        .filter(|id| id.bytes().all(|b| b.is_ascii_digit()))
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    if !matches!(ler(banco, instalacao, cenario, id), Ok(Some(_))) {
        return Resposta::nova(Estado::DadosInsuficientes);
    }
    let situacao = match validacao_acesso::ler(banco, instalacao, cenario, id) {
        Ok(Some(situacao)) => situacao,
        _ => return Resposta::nova(Estado::DadosInsuficientes),
    };
    if metodo == "GET" {
        if situacao != Validacao::Exigida {
            return Resposta::nova(Estado::RequisicaoInvalida);
        }
        let hash = crate::protocolo_sintetico::INFRA_HASH;
        let pagina = format!(
            "<html><body><form id='frmValidarAcesso' method='post' action='controlador.php?acao=usuario_validar_acesso&id_procedimento={}&infra_hash={hash}'><input type='password' name='pwdSenha'><button type='submit' name='sbmValidar' value='Validar'>Validar</button></form></body></html>",
            html(id),
        );
        return resposta_html(pagina);
    }
    if metodo != "POST" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Some(campos) = pares(corpo) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    if situacao != Validacao::Exigida
        || !campo_unico(&campos, "pwdSenha").is_some_and(|senha| !senha.is_empty())
        || campo_unico(&campos, "sbmValidar") != Some("Validar")
    {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    match validacao_acesso::guardar(banco, instalacao, cenario, id, Validacao::Realizada) {
        Ok(()) => redirecionado("usuario_validar_acesso", id),
        Err(_) => Resposta::nova(Estado::DadosInsuficientes),
    }
}
