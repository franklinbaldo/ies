//! Identidade TLS efêmera, exclusivamente para o proxy local dos testes.
use native_tls::Identity;
use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) struct Pasta(pub(super) PathBuf);
impl Pasta {
    pub(super) fn nova() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let caminho =
            std::env::temp_dir().join(format!("aies-criar-{}-{nanos}", std::process::id()));
        fs::create_dir(&caminho).unwrap();
        Self(caminho)
    }
    pub(super) fn privado(&self, nome: &str, conteudo: &str) -> PathBuf {
        let arquivo = self.0.join(nome);
        fs::write(&arquivo, conteudo).unwrap();
        fs::set_permissions(&arquivo, fs::Permissions::from_mode(0o600)).unwrap();
        arquivo
    }
}
impl Drop for Pasta {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

pub(super) fn identidade(pasta: &Path) -> (PathBuf, Identity) {
    let cert = pasta.join("ca.pem");
    let chave = pasta.join("chave.pem");
    let arquivo = pasta.join("cert.p12");
    assert!(
        Command::new("openssl")
            .args([
                "req",
                "-x509",
                "-newkey",
                "rsa:2048",
                "-nodes",
                "-sha256",
                "-days",
                "1",
                "-subj",
                "/CN=sei.test",
                "-addext",
                "subjectAltName=DNS:sei.test",
                "-keyout",
            ])
            .arg(&chave)
            .arg("-out")
            .arg(&cert)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("openssl")
            .args(["pkcs12", "-export", "-passout", "pass:teste", "-inkey",])
            .arg(&chave)
            .arg("-in")
            .arg(&cert)
            .arg("-out")
            .arg(&arquivo)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap()
            .success()
    );
    (
        cert,
        Identity::from_pkcs12(&fs::read(arquivo).unwrap(), "teste").unwrap(),
    )
}
