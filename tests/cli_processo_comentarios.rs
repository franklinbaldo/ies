//! #257: a CLI publica a prévia e recusa texto inválido antes da sessão.
//! A confirmação com texto válido é coberta pela CLI ↔ anti-IES local.
use std::process::Command;

#[test]
fn bdd_257_02_expoe_previa_e_recusa_texto_invalido_sem_sessao() {
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["processo", "--help"])
        .output()
        .expect("CLI deve iniciar");
    assert!(saida.status.success());
    let ajuda = String::from_utf8(saida.stdout).expect("ajuda UTF-8");
    let comandos: Vec<_> = ajuda
        .lines()
        .filter_map(|linha| linha.split_whitespace().next())
        .collect();
    assert!(comandos.contains(&"comentar"), "prévia deve ser pública");
    assert!(
        comandos.contains(&"comentarios"),
        "leitura deve ser pública"
    );

    let bloqueada = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "--json",
            "processo",
            "comentar",
            "00001.000001/2026-01",
            "--texto",
            "   ",
            "--confirmar",
        ])
        .env_remove("IES_ORIGEM_FILE")
        .env_remove("IES_COOKIE_FILE")
        .env_remove("IES_URL_CONTROLE_FILE")
        .output()
        .expect("a CLI não deve depender de sessão para recusar a escrita");
    // Texto vazio é erro de uso do clap: saída 2, sem sessão nem stdout.
    assert_eq!(bloqueada.status.code(), Some(2));
    assert_eq!(bloqueada.stdout, b"");
}
