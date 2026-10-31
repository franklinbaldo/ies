//! Detecção estrutural de formulário de autenticação no HTML recebido.
//! Não faz login, não interpreta senhas e não inicia novas requisições.

/// Shim do contrato. A implementação reconhece formulário SIP ou módulo próprio.
pub(crate) fn e_tela_login(html: &str) -> bool {
    let Ok(dom) = tl::parse(html, tl::ParserOptions::default()) else {
        return false;
    };
    let parser = dom.parser();
    let Some(forms) = dom.query_selector("form") else {
        return false;
    };
    for form in forms {
        let Some(tag) = form.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let attrs = tag.attributes();
        let identificador = |chave| {
            attrs
                .get(chave)
                .flatten()
                .is_some_and(|valor| valor.as_utf8_str().eq_ignore_ascii_case("frmLogin"))
        };
        let action_login = attrs.get("action").flatten().is_some_and(|valor| {
            valor
                .as_utf8_str()
                .to_ascii_lowercase()
                .contains("login.php")
        });
        if !identificador("id") && !identificador("name") && !action_login {
            continue;
        }
        // Analisa só filhos do formulário identificado: campos de outro
        // form ou do restante da página não comprovam autenticação.
        let interior = tag.inner_html(parser);
        let Ok(filhos) = tl::parse(&interior, tl::ParserOptions::default()) else {
            continue;
        };
        let filhos_parser = filhos.parser();
        if filhos.query_selector("input").is_some_and(|mut inputs| {
            inputs.any(|item| {
                item.get(filhos_parser)
                    .and_then(tl::Node::as_tag)
                    .is_some_and(|input| {
                        ["name", "id"].into_iter().any(|campo| {
                            input.attributes().get(campo).flatten().is_some_and(|valor| {
                                valor.as_utf8_str().eq_ignore_ascii_case("txtUsuario")
                            })
                        })
                    })
            })
        }) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::e_tela_login;

    #[test]
    fn sip_com_usuario_e_form_identificado_eh_login() {
        let html = "<form id='frmLogin'><input name='txtUsuario'></form>";
        assert!(e_tela_login(html));
    }

    #[test]
    fn formulario_de_modulo_proprio_com_usuario_aninhado_eh_login() {
        let html = "<form name='frmLogin'><div><input name='txtUsuario'></div></form>";
        assert!(e_tela_login(html));
    }

    #[test]
    fn action_de_login_com_usuario_eh_login_sem_id() {
        let html = "<form action='/sip/login.php'><input id='txtUsuario'></form>";
        assert!(e_tela_login(html));
    }

    #[test]
    fn formulario_de_assinatura_ou_usuario_fora_do_form_nao_eh_login() {
        let assinatura = "<form id='frmAssinar'><input name='pwdSenha'></form>";
        let separado = "<input name='txtUsuario'><form id='frmLogin'></form>";
        assert!(!e_tela_login(assinatura));
        assert!(!e_tela_login(separado));
        assert!(!e_tela_login(""));
    }
}
