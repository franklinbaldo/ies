//! O derive gera o `From` que o `?` usa.

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
enum Interno {
    LinkAusente,
}

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
enum Operacao {
    Tela(#[from] Interno),
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
