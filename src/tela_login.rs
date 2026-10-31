//! Detecção estrutural de formulário de autenticação no HTML recebido.
//! Não faz login, não interpreta senhas e não inicia novas requisições.

/// Shim do contrato. A implementação reconhece formulário SIP ou módulo próprio.
pub(crate) fn e_tela_login(_html: &str) -> bool {
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
