//! Superfície das listas independentes da caixa (Refs #57):
//! `listar-processos [--pagina N]` (visão D), `recebidos`/`gerados` e `colunas`
//! (inclusive `--definir`, cujos erros de uso saem antes de ler a configuração).
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
        &[
            "unidade",
            "listar-processos",
            "--categoria",
            "gerados",
            "--pagina",
            "2",
        ][..],
        // Removida: as capturas não se juntam à troca de visão.
        &["unidade", "listar-processos", "--pagina-arquivo", "p.html"][..],
        &["unidade", "listar-processos", "--pagina", "0"][..],
        &["unidade", "listar-processos", "--pagina", "x"][..],
        &["unidade", "colunas", "--pagina", "2"][..],
        &["unidade", "colunas", "--confirmar"][..],
        &["unidade", "colunas", "--definir"][..],
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

/// Falha fechada antes da rede: saída 1, stdout vazio, erro tipado no stderr.
fn falha_sem_sessao(args: &[&str]) {
    let saida = sem_sessao(args);
    assert_eq!(saida.status.code(), Some(1), "{args:?}");
    assert_eq!(saida.stdout, b"", "{args:?}");
    let erro = String::from_utf8_lossy(&saida.stderr);
    assert!(erro.starts_with("Error: "), "{args:?}: {erro}");
}

#[test]
fn listas_sem_sessao_falham_antes_de_qualquer_requisicao() {
    for args in [
        &["unidade", "recebidos", "--json"][..],
        &["unidade", "gerados", "--json"][..],
        &["unidade", "recebidos", "--pagina", "2", "--json"][..],
        &["unidade", "listar-processos", "--json"][..],
        &["unidade", "listar-processos", "--pagina", "2", "--json"][..],
        &["unidade", "colunas", "--json"][..],
        &["unidade", "colunas", "--definir", "todas", "--json"][..],
        &[
            "unidade",
            "colunas",
            "--definir",
            "tipo,atribuicao",
            "--confirmar",
            "--json",
        ][..],
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
        falha_sem_sessao(args);
    }
}

#[test]
fn ajuda_da_unidade_lista_os_tres_comandos_da_caixa() {
    let saida = sem_sessao(&["unidade", "--help"]);
    assert_eq!(saida.status.code(), Some(0));
    let texto = String::from_utf8(saida.stdout).unwrap();
    for comando in [
        "listar-processos",
        "recebidos",
        "gerados",
        "atual",
        "colunas",
    ] {
        assert!(texto.contains(comando), "{comando}: {texto}");
    }
    let listar = sem_sessao(&["unidade", "listar-processos", "--help"]);
    let texto = String::from_utf8(listar.stdout).unwrap();
    assert!(!texto.contains("--pagina-arquivo"), "{texto}");
    assert!(texto.contains("--saida"), "{texto}");
    assert!(!texto.contains("--categoria"), "{texto}");
    assert!(texto.contains("--pagina <N>"), "{texto}");
    for lista in ["recebidos", "gerados"] {
        let ajuda = sem_sessao(&["unidade", lista, "--help"]);
        let texto = String::from_utf8(ajuda.stdout).unwrap();
        assert!(texto.contains("--pagina <N>"), "{lista}: {texto}");
        assert!(texto.contains("--saida"), "{lista}: {texto}");
        assert!(!texto.contains("--pagina-arquivo"), "{lista}: {texto}");
    }
}

/// Erro de uso de `--definir`: saída 2 antes de ler a configuração local
/// (os arquivos não existem), logo sem requisição.
#[test]
fn definir_com_lista_invalida_e_uso_invalido_sem_requisicao() {
    for lista in ["", "tipo,,anotacao", "tipo,processo", "Tipo", "tipo,tipo"] {
        for confirmar in [false, true] {
            let mut args = vec!["unidade", "colunas", "--definir", lista, "--json"];
            if confirmar {
                args.push("--confirmar");
            }
            let saida = sem_sessao(&args);
            assert_eq!(saida.status.code(), Some(2), "{args:?}");
            assert_eq!(saida.stdout, b"", "{args:?}");
        }
        let humano = sem_sessao(&["unidade", "colunas", "--definir", lista]);
        assert_eq!(humano.status.code(), Some(2), "{lista}");
        assert!(humano.stdout.is_empty(), "{lista}");
        let _texto = String::from_utf8(humano.stderr).unwrap();
    }
}

#[test]
fn ajuda_de_colunas_mostra_definir_e_confirmar() {
    let saida = sem_sessao(&["unidade", "colunas", "--help"]);
    assert_eq!(saida.status.code(), Some(0));
    let texto = String::from_utf8(saida.stdout).unwrap();
    assert!(texto.contains("--definir <todas|LISTA>"), "{texto}");
    assert!(texto.contains("--confirmar"), "{texto}");
}
