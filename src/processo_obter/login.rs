//! Para onde aponta um 302 que não é `procedimento_trabalhar`: a tela de
//! login (`…/login.php`), com ou sem a recusa característica da entrada sem
//! assinatura (`msg=Link sem assinatura`, ADR 0003), ou outro lugar.
//! Nenhuma regex: o caminho é partido à mão e `msg` é decodificado
//! (`%XX` e `+`) só para a comparação exata.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DestinoDoRedirecionamento {
    RecusaSemAssinatura,
    Login,
    Outro,
}

/// A mensagem que a `UnB` 4.0.9 pôs no `Location` de cada recusa medida.
const RECUSA: &str = "Link sem assinatura";

pub(super) fn destino(localizacao: Option<&str>) -> DestinoDoRedirecionamento {
    use DestinoDoRedirecionamento as D;
    let Some(localizacao) = localizacao.filter(|l| !l.is_empty()) else {
        return D::Outro;
    };
    let (caminho, consulta) = localizacao.split_once('?').unwrap_or((localizacao, ""));
    let arquivo = caminho.rsplit('/').next().unwrap_or(caminho);
    if !arquivo.eq_ignore_ascii_case("login.php") {
        return D::Outro;
    }
    if recusa_sem_assinatura(consulta) {
        D::RecusaSemAssinatura
    } else {
        D::Login
    }
}

/// A consulta traz `msg` com a recusa característica da entrada sem
/// assinatura (também conferida no 302 do Controle para o login).
pub(crate) fn recusa_sem_assinatura(consulta: &str) -> bool {
    consulta
        .split('&')
        .filter_map(|par| par.split_once('='))
        .any(|(nome, valor)| nome == "msg" && decodificar(valor).as_deref() == Some(RECUSA))
}

/// `application/x-www-form-urlencoded` → texto; `None` se mal formado ou
/// se os bytes não forem UTF-8.
fn decodificar(valor: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(valor.len());
    let mut resto = valor.bytes();
    while let Some(b) = resto.next() {
        match b {
            b'+' => bytes.push(b' '),
            b'%' => {
                let alto = char::from(resto.next()?).to_digit(16)?;
                let baixo = char::from(resto.next()?).to_digit(16)?;
                bytes.push(u8::try_from(alto * 16 + baixo).ok()?);
            }
            outro => bytes.push(outro),
        }
    }
    String::from_utf8(bytes).ok()
}
