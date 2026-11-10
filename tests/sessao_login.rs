//! `ies login` pela borda da CLI, sem rede: cada caso falha antes do
//! primeiro pedido (sem arquivos de sessão gravados, nenhum GET de
//! conferência acontece, e a falha das credenciais vem antes da raiz).
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn privado(caminho: &Path, conteudo: &str, modo: u32) {
    std::fs::write(caminho, conteudo).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(caminho, std::fs::Permissions::from_mode(modo)).unwrap();
    }
    #[cfg(not(unix))]
    let _ = modo;
}

struct Pasta(PathBuf);

impl Pasta {
    fn nova(nome: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("ies-cli-login-{nome}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        privado(
            &dir.join("origem"),
            "https://sei.exemplo.gov.br/sei\n",
            0o600,
        );
        Self(dir)
    }

    fn ies(&self, args: &[&str], credenciais: Option<&Path>) -> Output {
        let mut comando = Command::new(env!("CARGO_BIN_EXE_ies"));
        comando
            .args(args)
            .env_remove("IES_CONFIG_FILE")
            .env("IES_URL_CONTROLE_FILE", self.0.join("url-controle"))
            .env("IES_COOKIE_FILE", self.0.join("cookie"))
            .env("IES_ORIGEM_FILE", self.0.join("origem"));
        match credenciais {
            Some(caminho) => comando.env("IES_CREDENCIAIS_FILE", caminho),
            None => comando.env_remove("IES_CREDENCIAIS_FILE"),
        };
        comando.output().unwrap()
    }
}

impl Drop for Pasta {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn json(saida: &Output) -> serde_json::Value {
    serde_json::from_slice(&saida.stdout).unwrap()
}

#[test]
fn ajuda_publica_forcar_e_nao_aceita_credencial_por_argumento() {
    let ajuda = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["login", "--help"])
        .output()
        .unwrap();
    assert!(ajuda.status.success());
    let texto = String::from_utf8_lossy(&ajuda.stdout);
    assert!(texto.contains("--forcar"), "{texto}");
    let senha = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["login", "--senha", "x"])
        .output()
        .unwrap();
    assert_eq!(senha.status.code(), Some(2));
}

#[test]
fn sem_configuracao_falha_tipada() {
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["--json", "login"])
        .env_remove("IES_CONFIG_FILE")
        .env_remove("IES_URL_CONTROLE_FILE")
        .output()
        .unwrap();
    assert_eq!(saida.status.code(), Some(1));
    assert_eq!(json(&saida)["subcodigo"], "url_do_controle_sem_caminho");
}

#[test]
fn sem_arquivo_de_credenciais_nada_e_pedido_nem_gravado() {
    let pasta = Pasta::nova("sem-credenciais");
    let saida = pasta.ies(&["--json", "login"], None);
    assert_eq!(saida.status.code(), Some(1));
    let valor = json(&saida);
    assert_eq!(valor["codigo"], "fonte_invalida");
    assert_eq!(valor["subcodigo"], "caminho_das_credenciais_nao_informado");
    assert!(!pasta.0.join("cookie").exists());
    assert!(!pasta.0.join("url-controle").exists());
}

#[test]
fn credenciais_inseguras_ou_incompletas_sao_recusadas_sem_rede() {
    let pasta = Pasta::nova("credenciais");
    let arquivo = pasta.0.join("credenciais");
    privado(&arquivo, "usuario=u\nsenha=SEGREDO\norgao=O\n", 0o644);
    let saida = pasta.ies(&["--json", "login"], Some(&arquivo));
    assert_eq!(
        json(&saida)["subcodigo"],
        "arquivo_privado_com_permissoes_inseguras"
    );
    privado(&arquivo, "usuario=u\nsenha=SEGREDO\n", 0o600);
    let saida = pasta.ies(&["--json", "login", "--forcar"], Some(&arquivo));
    assert_eq!(json(&saida)["subcodigo"], "campo_das_credenciais_ausente");
    for texto in [&saida.stdout, &saida.stderr] {
        assert!(!String::from_utf8_lossy(texto).contains("SEGREDO"));
    }
    let humana = pasta.ies(&["login"], Some(&arquivo));
    assert_eq!(humana.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&humana.stderr).contains("[campo_das_credenciais_ausente]"));
}
