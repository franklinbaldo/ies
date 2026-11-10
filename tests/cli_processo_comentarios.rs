//! #257: a CLI publica a prévia de comentário mas não habilita a escrita.
//! O --confirmar deve ser recusado antes de ler credenciais ou fazer rede.
use std::process::Command;

#[test]
fn bdd_257_02_expoe_previa_mas_recusa_confirmar_sem_sessao() {
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
    assert!(comandos.contains(&"comentarios"), "leitura deve ser pública");

    let bloqueada = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "--json", "processo", "comentar", "00001.000001/2026-01",
            "--texto", "Comentário seguro", "--confirmar",
        ])
        .env_remove("IES_ORIGEM_FILE")
        .env_remove("IES_COOKIE_FILE")
        .env_remove("IES_URL_CONTROLE_FILE")
        .output()
        .expect("a CLI não deve depender de sessão para recusar a escrita");
    assert_eq!(bloqueada.status.code(), Some(1));
    let resposta: serde_json::Value = serde_json::from_slice(&bloqueada.stdout).unwrap();
    assert_eq!(resposta["codigo"], "operacao_recusada");
    assert!(resposta["erro"].as_str().unwrap().contains("Nenhuma requisição"));
}
