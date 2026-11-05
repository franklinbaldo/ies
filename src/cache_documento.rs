//! Cache de documento por instalação, número SEI, versão e SHA-256.
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn nome_conhecido_e_desconhecido() {
        assert_eq!(nome(Some(2), "aabbccddeeff001122"), "v2-aabbccddeeff0011.html");
        assert_eq!(nome(None, "aabbccddeeff001122"), "x-aabbccddeeff0011.html");
    }

    #[test]
    fn cache_idempotente_em_xdg_temporario() {
        let home = std::env::temp_dir().join(format!("ies-cache-red-{}", std::process::id()));
        let alvo = caminho_em(&home, "sei.exemplo", "0011519", Some(2), "aabbccddeeff001122", "html");
        assert_eq!(alvo, home.join("ies/sei.exemplo/0011519/v2-aabbccddeeff0011.html"));
        let bytes = b"<html><body>teste</body></html>";
        let alvo = gravar_em(&home, "sei.exemplo", "0011519", Some(2), "html", bytes).unwrap();
        let primeira = std::fs::metadata(&alvo).unwrap().modified().unwrap();
        let segunda = gravar_em(&home, "sei.exemplo", "0011519", Some(2), "html", bytes).unwrap();
        assert_eq!(alvo, segunda);
        assert_eq!(std::fs::metadata(&alvo).unwrap().modified().unwrap(), primeira);
        std::fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn sem_cache_nao_escreve() {
        assert!(!habilitado(true, None));
        assert!(!habilitado(false, Some("0")));
        assert!(habilitado(false, None));
    }
}
