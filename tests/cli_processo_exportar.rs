//! #889: binário real, contrato de entrada e falha sem sessão.
//! Não é replay nem E2E de exportação: nenhum servidor é chamado.
use std::process::{Command, Output};

const NUP: &str = "00001.000001/2026-01";

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

#[test]
fn ajuda_de_exportacao_publica_formatos_e_destino() {
    let resultado = sem_sessao(&["processo", "exportar", "--help"]);
    assert!(resultado.status.success());
    let ajuda = String::from_utf8_lossy(&resultado.stdout);
    for campo in ["--formato", "--saida", "--documentos", "pdf", "zip"] {
        assert!(ajuda.contains(campo), "{campo}: {ajuda}");
    }
}

#[test]
fn formatos_validos_sem_sessao_falham_antes_de_rede() {
    for formato in ["pdf", "zip"] {
        let resultado = sem_sessao(&[
            "processo",
            "exportar",
            NUP,
            "--formato",
            formato,
            "--saida",
            "processo.teste",
            "--json",
        ]);
        assert_eq!(resultado.status.code(), Some(1));
        assert_eq!(resultado.stdout, Vec::<u8>::new());
    }
}

#[test]
fn formato_invalido_e_erro_de_uso() {
    let resultado = sem_sessao(&[
        "processo",
        "exportar",
        NUP,
        "--formato",
        "html",
        "--saida",
        "processo.teste",
    ]);
    assert_eq!(resultado.status.code(), Some(2));
    assert_eq!(resultado.stdout, Vec::<u8>::new());
}

#[test]
fn selecao_pdf_recusada_antes_da_sessao() {
    let resultado = sem_sessao(&[
        "processo",
        "exportar",
        NUP,
        "--formato",
        "pdf",
        "--saida",
        "processo.teste",
        "--documentos",
        "050000001",
    ]);
    assert_eq!(resultado.status.code(), Some(1));
    assert_eq!(resultado.stdout, Vec::<u8>::new());
    let erro = String::from_utf8_lossy(&resultado.stderr);
    assert!(erro.contains("SelecaoSomenteZip"), "{erro}");
}
