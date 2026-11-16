//! Gerenciamento seguro de credenciais pelo cofre nativo do SO (Keyring).
//! Utiliza escopo determinístico canônico por instalação: "ies:<instancia>".

use keyring::Entry;

/// Prefixo do serviço exclusivo do IES no Keyring.
pub const SERVICO_PREFIXO: &str = "ies";

/// Extrai a instância / host limpo da URL de origem do SEI.
pub fn instancia_da_origem(origem: &str) -> String {
    let sem_esquema = if let Some(pos) = origem.find("://") {
        &origem[pos + 3..]
    } else {
        origem
    };
    let host = sem_esquema.split('/').next().unwrap_or(sem_esquema);
    host.split(':')
        .next()
        .unwrap_or(host)
        .trim()
        .to_ascii_lowercase()
}

/// Serviço determinístico canônico escopado pela instalação do SEI.
/// Exemplo: "ies:sei.sistemas.ro.gov.br" ou "ies:sei.treinamento.unb.br".
pub fn servico_do_cofre(origem: &str) -> String {
    let inst = instancia_da_origem(origem);
    if inst.is_empty() {
        SERVICO_PREFIXO.to_string()
    } else {
        format!("{SERVICO_PREFIXO}:{inst}")
    }
}

/// Salva a senha com segurança no cofre nativo sob o serviço canônico da instalação.
pub fn salvar_senha(usuario: &str, origem: &str, senha: &str) -> Result<(), String> {
    let senha = senha.trim();
    if senha.is_empty() {
        return Err("A senha não pode ser vazia".to_string());
    }
    let user = usuario.trim();
    if user.is_empty() {
        return Err("O usuário não pode ser vazio".to_string());
    }
    let servico = servico_do_cofre(origem);
    let entry = Entry::new(&servico, user)
        .map_err(|e| format!("Falha ao acessar cofre do sistema: {e}"))?;
    entry
        .set_password(senha)
        .map_err(|e| format!("Falha ao gravar senha no cofre: {e}"))?;
    Ok(())
}

/// Obtém a senha armazenada com segurança no cofre.
pub fn obter_senha(usuario: &str, origem: &str) -> Option<String> {
    let user = usuario.trim();
    if user.is_empty() {
        return None;
    }
    let servico = servico_do_cofre(origem);
    let entry = Entry::new(&servico, user).ok()?;
    match entry.get_password() {
        Ok(s) if !s.trim().is_empty() => Some(s),
        _ => None,
    }
}

/// Remove a credencial do cofre nativo.
pub fn excluir_senha(usuario: &str, origem: &str) -> Result<(), String> {
    let user = usuario.trim();
    if user.is_empty() {
        return Err("Usuário não informado para exclusão".to_string());
    }
    let servico = servico_do_cofre(origem);
    let entry = Entry::new(&servico, user)
        .map_err(|e| format!("Falha ao acessar cofre do sistema: {e}"))?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("Falha ao excluir credencial do cofre: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deriva_servico_corretamente() {
        let orig = "https://sei.treinamento.unb.br/sei";
        assert_eq!(instancia_da_origem(orig), "sei.treinamento.unb.br");
        assert_eq!(servico_do_cofre(orig), "ies:sei.treinamento.unb.br");

        let orig2 = "https://sei.sistemas.ro.gov.br";
        assert_eq!(servico_do_cofre(orig2), "ies:sei.sistemas.ro.gov.br");

        let orig3 = "http://localhost:8080/sei";
        assert_eq!(servico_do_cofre(orig3), "ies:localhost");
    }

    #[test]
    #[ignore = "requer cofre de credenciais nativo do SO"]
    fn salva_obtem_e_exclui_senha_no_keyring_real() {
        let u = "teste_usuario_ci";
        let orig = "https://sei.teste.gov.br";
        let res = salvar_senha(u, orig, "minha_senha_secreta");
        assert!(res.is_ok());
        let obtida = obter_senha(u, orig);
        assert_eq!(obtida.as_deref(), Some("minha_senha_secreta"));
        let excl = excluir_senha(u, orig);
        assert!(excl.is_ok());
        assert_eq!(obter_senha(u, orig), None);
    }
}
