//! Superfície das listas independentes da caixa (Refs #57):
//! `listar-processos` sem `--categoria`/`--pagina`, e `recebidos`/`gerados`.
//! Nenhum teste abre sessão nem acessa o SEI.
use std::process::{Command, Output};

fn sem_sessao(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(args)
        .env_remove("IES_CONFIG_FILE")
        .env("IES_URL_CONTROLE_FILE", "/nao/existe/url")
        .env("IES_COOKIE_FILE", "/nao/existe/cookie")
        .env("IES_ORIGEM_FILE", "/nao/existe/origem")
        .output()
        .expect("IES deve iniciar")
}

fn uso_invalido(args: &[&str]) {
    let saida = sem_sessao(args);
    assert_eq!(saida.status.code(), Some(2), "{args:?}");
    assert!(saida.stdout.is_empty(), "{args:?}");
}

#[test]
fn opcoes_removidas_de_listar_processos_sao_erro_de_uso() {
    for args in [
        &["unidade", "listar-processos", "--categoria", "gerados"][..],
        &[
            "unidade",
            "listar-processos",
            "--categoria",
            "recebidos",
            "--json",
        ][..],
        &["unidade", "listar-processos", "--pagina", "2", "--json"][..],
        &[
            "unidade",
            "listar-processos",
            "--categoria",
            "gerados",
            "--pagina",
            "2",
        ][..],
    ] {
        uso_invalido(args);
    }
}

#[test]
fn pagina_das_listas_exige_inteiro_positivo_e_nao_aceita_paginas_privadas() {
    for lista in ["recebidos", "gerados"] {
        for extra in [
            &["--pagina", "0"][..],
            &["--pagina", "-1"][..],
            &["--pagina", "x"][..],
            &["--pagina-arquivo", "p.html"][..],
            &["--categoria", "gerados"][..],
        ] {
            let mut args = vec!["unidade", lista];
            args.extend_from_slice(extra);
            uso_invalido(&args);
        }
    }
}

fn subcodigo_sem_sessao(args: &[&str]) -> String {
    let saida = sem_sessao(args);
    assert_eq!(saida.status.code(), Some(1), "{args:?}");
    let valor: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(valor["codigo"], "fonte_invalida", "{args:?}");
    valor["subcodigo"].as_str().unwrap().to_owned()
}

#[test]
fn listas_sem_sessao_falham_antes_de_qualquer_requisicao() {
    for args in [
        &["unidade", "recebidos", "--json"][..],
        &["unidade", "gerados", "--json"][..],
        &["unidade", "recebidos", "--pagina", "2", "--json"][..],
        &[
            "--unidade",
            "PPI",
            "unidade",
            "gerados",
            "--pagina",
            "3",
            "--json",
        ][..],
    ] {
        assert_eq!(
            subcodigo_sem_sessao(args),
            "arquivo_privado_ausente_ou_inacessivel",
            "{args:?}"
        );
    }
}

#[test]
fn ajuda_da_unidade_lista_os_tres_comandos_da_caixa() {
    let saida = sem_sessao(&["unidade", "--help"]);
    assert_eq!(saida.status.code(), Some(0));
    let texto = String::from_utf8(saida.stdout).unwrap();
    for comando in ["listar-processos", "recebidos", "gerados", "atual"] {
        assert!(texto.contains(comando), "{comando}: {texto}");
    }
    let listar = sem_sessao(&["unidade", "listar-processos", "--help"]);
    let texto = String::from_utf8(listar.stdout).unwrap();
    assert!(texto.contains("--pagina-arquivo"), "{texto}");
    assert!(texto.contains("--saida"), "{texto}");
    assert!(!texto.contains("--categoria"), "{texto}");
    assert!(!texto.contains("--pagina <N>"), "{texto}");
    for lista in ["recebidos", "gerados"] {
        let ajuda = sem_sessao(&["unidade", lista, "--help"]);
        let texto = String::from_utf8(ajuda.stdout).unwrap();
        assert!(texto.contains("--pagina <N>"), "{lista}: {texto}");
        assert!(texto.contains("--saida"), "{lista}: {texto}");
        assert!(!texto.contains("--pagina-arquivo"), "{lista}: {texto}");
    }
}
