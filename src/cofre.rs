//! Gerenciamento seguro de credenciais pelo cofre nativo do SO (Keyring).
//! Utiliza escopo determinístico por instalação: "ies:<instancia>".

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
    host.split(':').next().unwrap_or(host).trim().to_ascii_lowercase()
}

/// Serviço determinístico escopado pela instalação do SEI.
/// Exemplo: "ies:sei.sistemas.ro.gov.br" ou "ies:sei.treinamento.unb.br".
pub fn servico_do_cofre(origem: &str) -> String {
    let inst = instancia_da_origem(origem);
    if inst.is_empty() {
        SERVICO_PREFIXO.to_string()
    } else {
        format!("{SERVICO_PREFIXO}:{inst}")
    }
}

/// Deriva a chave legada de armazenamento no formato `usuario@instancia`.
pub fn chave_do_cofre(usuario: &str, origem: &str) -> String {
    let inst = instancia_da_origem(origem);
    if inst.is_empty() {
        usuario.trim().to_string()
    } else {
        format!("{}@{}", usuario.trim(), inst)
    }
}

/// Salva a senha com segurança no cofre nativo sob o serviço escopado da instalação.
pub fn salvar_senha(usuario: &str, origem: &str, senha: &str) -> Result<(), String> {
    let user = usuario.trim();
    let servico = servico_do_cofre(origem);

    let entry = Entry::new(&servico, user)
        .map_err(|e| format!("Falha ao acessar cofre do sistema: {e}"))?;
    entry
        .set_password(senha)
        .map_err(|e| format!("Falha ao gravar senha no cofre: {e}"))?;

    let chave_legada = chave_do_cofre(user, origem);
    if let Ok(entry_legada) = Entry::new(SERVICO_PREFIXO, &chave_legada) {
        let _ = entry_legada.set_password(senha);
    }
    Ok(())
}

/// Obtém a senha armazenada com segurança no cofre.
pub fn obter_senha(usuario: &str, origem: &str) -> Option<String> {
    let user = usuario.trim();
    let servico = servico_do_cofre(origem);

    if let Ok(entry) = Entry::new(&servico, user) {
        if let Ok(pwd) = entry.get_password() {
            return Some(pwd);
        }
    }
    if let Ok(entry) = Entry::new(&servico, &servico) {
        if let Ok(pwd) = entry.get_password() {
            return Some(pwd);
        }
    }
    let chave_legada = chave_do_cofre(user, origem);
    if let Ok(entry) = Entry::new(SERVICO_PREFIXO, &chave_legada) {
        if let Ok(pwd) = entry.get_password() {
            return Some(pwd);
        }
    }
    if let Ok(entry) = Entry::new(&format!("{chave_legada}.{SERVICO_PREFIXO}"), &chave_legada) {
        if let Ok(pwd) = entry.get_password() {
            return Some(pwd);
        }
    }
    None
}

/// Remove a credencial do cofre.
pub fn excluir_senha(usuario: &str, origem: &str) -> Result<(), String> {
    let user = usuario.trim();
    let servico = servico_do_cofre(origem);
    let chave_legada = chave_do_cofre(user, origem);

    let _ = Entry::new(&servico, user).and_then(|e| e.delete_credential());
    let _ = Entry::new(SERVICO_PREFIXO, &chave_legada).and_then(|e| e.delete_credential());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deriva_servico_e_chave_corretamente() {
        let orig = "https://sei.treinamento.unb.br/sei";
        assert_eq!(instancia_da_origem(orig), "sei.treinamento.unb.br");
        assert_eq!(servico_do_cofre(orig), "ies:sei.treinamento.unb.br");
        assert_eq!(chave_do_cofre("aluno1", orig), "aluno1@sei.treinamento.unb.br");

        let orig2 = "https://sei.sistemas.ro.gov.br";
        assert_eq!(servico_do_cofre(orig2), "ies:sei.sistemas.ro.gov.br");
        assert_eq!(chave_do_cofre("76450694220", orig2), "76450694220@sei.sistemas.ro.gov.br");
    }

    #[test]
    fn salva_e_obtem_senha_escopada() {
        let u = "teste_usuario";
        let orig = "https://sei.teste.gov.br";
        let res = salvar_senha(u, orig, "minha_senha_secreta");
        assert!(res.is_ok());
        let obtida = obter_senha(u, orig);
        assert_eq!(obtida.as_deref(), Some("minha_senha_secreta"));
        let _ = excluir_senha(u, orig);
    }
}
