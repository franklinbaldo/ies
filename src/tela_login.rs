//! Detecção estrutural de formulário de autenticação no HTML recebido.
//! Não faz login, não interpreta senhas e não inicia novas requisições.

/// Shim do contrato. A implementação reconhece formulário SIP ou módulo próprio.
pub(crate) fn e_tela_login(html: &crate::tela_html::HtmlServido) -> bool {
    let dom = crate::tela_html::arvore(html);
    let forms = dom.elementos("form");
    for form in forms {
        // Atributo repetido vale o primeiro, como no navegador (#808).
        let identificador = |chave| {
            form.atributo(chave)
                .is_some_and(|valor| valor.eq_ignore_ascii_case("frmLogin"))
        };
        let action_login = form
            .atributo("action")
            .is_some_and(|valor| valor.to_ascii_lowercase().contains("login.php"));
        if !identificador("id") && !identificador("name") && !action_login {
            continue;
        }
        // Analisa só filhos do formulário identificado: campos de outro
        // form ou do restante da página não comprovam autenticação.
        if form.elementos("input").any(|item| {
            ["name", "id"].into_iter().any(|campo| {
                item.atributo(campo)
                    .is_some_and(|valor| valor.eq_ignore_ascii_case("txtUsuario"))
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
        let html = crate::tela_html::HtmlServido::de_teste(
            "<form id='frmLogin'><input name='txtUsuario'></form>",
        );
        assert!(e_tela_login(&html));
    }

    #[test]
    fn booleano_antes_do_id_e_do_name_nao_esconde_o_login() {
        for html in [
            "<form novalidate id='frmLogin'><input required name='txtUsuario'></form>",
            "<form novalidate name='frmLogin'><input autofocus id='txtUsuario'></form>",
            "<form novalidate action='login.php'><input required name='txtUsuario'></form>",
        ] {
            assert!(
                e_tela_login(&crate::tela_html::HtmlServido::de_teste(html)),
                "{html}"
            );
        }
    }

    #[test]
    fn formulario_de_modulo_proprio_com_usuario_aninhado_eh_login() {
        let html = crate::tela_html::HtmlServido::de_teste(
            "<form name='frmLogin'><div><input name='txtUsuario'></div></form>",
        );
        assert!(e_tela_login(&html));
    }

    #[test]
    fn action_de_login_com_usuario_eh_login_sem_id() {
        let html = crate::tela_html::HtmlServido::de_teste(
            "<form action='/sip/login.php'><input id='txtUsuario'></form>",
        );
        assert!(e_tela_login(&html));
    }

    #[test]
    fn formulario_de_assinatura_ou_usuario_fora_do_form_nao_eh_login() {
        let assinatura = "<form id='frmAssinar'><input name='pwdSenha'></form>";
        let separado = "<input name='txtUsuario'><form id='frmLogin'></form>";
        assert!(!e_tela_login(&crate::tela_html::HtmlServido::de_teste(
            assinatura
        )));
        assert!(!e_tela_login(&crate::tela_html::HtmlServido::de_teste(
            separado
        )));
        assert!(!e_tela_login(&crate::tela_html::HtmlServido::de_teste("")));
    }
}
