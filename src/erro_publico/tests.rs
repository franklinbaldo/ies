//! O derive gera `From`, status HTTP e o fato de sessão encerrada.
use super::*;

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
enum Interno {
    LinkAusente,
    SessaoEncerrada(#[source] crate::erro_publico::SessaoEncerrada),
    Recusa(#[source] StatusHttpRecusado),
}

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
enum Operacao {
    Tela(#[from] Interno),
    Folha,
    LinkSemHref(#[source] Interno),
}

fn tela() -> Result<(), Interno> {
    Err(Interno::LinkAusente)
}

fn operacao() -> Result<(), Operacao> {
    tela()?;
    Ok(())
}

#[test]
fn interna_gera_from_para_o_interrogacao() {
    assert!(matches!(
        operacao(),
        Err(Operacao::Tela(Interno::LinkAusente))
    ));
}

#[test]
fn sessao_encerrada_atravessa_interna_e_fonte() {
    assert!(
        Operacao::Tela(Interno::SessaoEncerrada(
            crate::erro_publico::SessaoEncerrada
        ))
        .sessao_encerrada()
    );
    assert!(
        Operacao::LinkSemHref(Interno::SessaoEncerrada(
            crate::erro_publico::SessaoEncerrada
        ))
        .sessao_encerrada()
    );
    assert!(!Operacao::Tela(Interno::LinkAusente).sessao_encerrada());
    assert!(!Operacao::Folha.sessao_encerrada());
}

#[test]
fn status_http_vem_da_variante_http_ou_da_composicao() {
    let recusa = || Interno::Recusa(StatusHttpRecusado::sem_mensagem(403));
    assert_eq!(recusa().status_http().map(|h| h.status), Some(403));
    assert_eq!(
        Operacao::Tela(recusa()).status_http().map(|h| h.status),
        Some(403)
    );
    assert_eq!(Operacao::Folha.status_http(), None);
}
