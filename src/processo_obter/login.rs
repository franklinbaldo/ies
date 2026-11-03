//! Para onde aponta um 302 que não é `procedimento_trabalhar`: a tela de
//! login (`…/login.php`), com ou sem a recusa característica da entrada sem
//! assinatura (`msg=Link sem assinatura`, ADR 0003), ou outro lugar.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DestinoDoRedirecionamento {
    RecusaSemAssinatura,
    Login,
    Outro,
}

pub(super) fn destino(localizacao: Option<&str>) -> DestinoDoRedirecionamento {
    let _ = localizacao;
    DestinoDoRedirecionamento::Outro
}
