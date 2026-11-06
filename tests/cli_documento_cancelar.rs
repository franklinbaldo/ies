//! Interface CLI de cancelar: erros de uso são decididos antes de arquivos,
//! rede ou POST. Números, caminhos e hosts são sintéticos (#273).
use std::process::{Command, Output};

fn sem_sessao(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(args)
        .env_remove("IES_CONFIG_FILE")
        .env("IES_URL_CONTROLE_FILE", "/nao/existe/url")
        .env("IES_COOKIE_FILE", "/nao/existe/cookie")
        .env("IES_ORIGEM_FILE", "/nao/existe/origem")
        .env("IES_INDICE_FILE", "/nao/existe/ies.sqlite3")
        .output()
        .expect("IES deve iniciar")
}
fn json(resultado: &Output) -> serde_json::Value {
    serde_json::from_slice(&resultado.stdout).expect("falha deve ser JSON")
}

#[test]
fn motivo_obrigatorio_e_exigido_pelo_clap_antes_da_sessao() {
    let resultado = sem_sessao(&["documento", "cancelar", "0000101", "--json"]);
    assert_eq!(resultado.status.code(), Some(2));
    assert_eq!(resultado.stdout, b"");
}

#[test]
fn recusa_numero_invalido_sem_tocar_na_sessao() {
    let r = sem_sessao(&[
        "documento",
        "cancelar",
        "abc",
        "--motivo",
        "Teste",
        "--json",
    ]);
    assert_eq!(r.status.code(), Some(2));
    let valor = json(&r);
    assert_eq!(valor["codigo"], "entrada_invalida");
    assert_eq!(valor["subcodigo"], "numero_documento_nao_numerico");
}

#[test]
fn recusa_motivo_tipado_sem_subcodigo_antes_da_sessao() {
    let casos = [
        (
            "   ",
            "o motivo está vazio",
            r#"{"causa":{"tipo":"MotivoVazio"},"codigo":"entrada_invalida","erro":"o motivo está vazio","retentavel":false}"#,
        ),
        (
            "Motivo\ncom controle",
            "o motivo contém caractere de controle",
            r#"{"causa":{"tipo":"CaractereDeControleNoMotivo"},"codigo":"entrada_invalida","erro":"o motivo contém caractere de controle","retentavel":false}"#,
        ),
        (
            "Cópia 🚀",
            "o motivo não é representável em ISO-8859-1",
            r#"{"causa":{"tipo":"MotivoNaoRepresentavelEmLatin1"},"codigo":"entrada_invalida","erro":"o motivo não é representável em ISO-8859-1","retentavel":false}"#,
        ),
    ];
    for (motivo, mensagem, esperado) in casos {
        for confirmar in [false, true] {
            let mut args = vec!["documento", "cancelar", "0000101", "--motivo", motivo];
            if confirmar {
                args.push("--confirmar");
            }
            let humano = sem_sessao(&args);
            assert_eq!(humano.status.code(), Some(2));
            assert_eq!(humano.stdout, b"");
            assert_eq!(
                String::from_utf8(humano.stderr).unwrap(),
                format!("IES: {mensagem}\n")
            );
            args.push("--json");
            let resultado = sem_sessao(&args);
            assert_eq!(resultado.status.code(), Some(2));
            assert_eq!(resultado.stderr, b"");
            assert_eq!(
                String::from_utf8(resultado.stdout).unwrap(),
                format!("{esperado}\n")
            );
        }
    }
}

#[test]
fn motivo_valido_vai_ate_sessao_mas_nao_inventa_configuracao() {
    for confirmar in [false, true] {
        let mut args = vec![
            "documento",
            "cancelar",
            "0000101",
            "--motivo",
            "Teste",
            "--json",
        ];
        if confirmar {
            args.push("--confirmar");
        }
        let r = sem_sessao(&args);
        assert_eq!(r.status.code(), Some(1));
        assert_eq!(json(&r)["codigo"], "fonte_invalida");
    }
}
