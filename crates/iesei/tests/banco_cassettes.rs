use iesei::banco_cassettes::{ControleResumido, Linha, guardar, iniciar, ler};
use iesei::instalacao::Instalacao;
use iesei::observacao::{FaixaObservada, TabelaObservada, UnidadeExibida};
use rusqlite::Connection;

fn origem() -> Instalacao {
    Instalacao::interpretar("https://sei.exemplo/sei").unwrap()
}

fn controle() -> ControleResumido {
    ControleResumido {
        instalacao: origem(),
        unidade: UnidadeExibida::nova("TESTE").unwrap(),
        recebidos: TabelaObservada::nova(
            Some(20),
            FaixaObservada::nova(1, 1),
            vec![Linha {
                id: "7".into(),
                numero: "N-7".into(),
                visualizado: true,
                sigiloso: false,
            }],
        )
        .unwrap(),
        gerados: TabelaObservada::nova(Some(0), None, vec![]).unwrap(),
    }
}

#[test]
fn banco_efemero_guarda_observado_sem_inventar_ausentes() {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let version: i64 = banco
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    // Migração 0003 acrescenta o estado compartilhado de documentos simulados.
    assert_eq!(version, 12);
    assert!(ler(&banco, &origem(), "cartao").unwrap().is_none());

    let observado = controle();
    guardar(&mut banco, "cartao", &observado).unwrap();
    guardar(&mut banco, "cartao", &observado).unwrap();
    assert_eq!(ler(&banco, &origem(), "cartao").unwrap(), Some(observado));
    let contagem: i64 = banco
        .query_row("SELECT count(*) FROM controle_linha", [], |r| r.get(0))
        .unwrap();
    assert_eq!(contagem, 1);
    assert!(ler(&banco, &origem(), "outro").unwrap().is_none());

    let outra = Instalacao::interpretar("https://outro.exemplo/sei").unwrap();
    assert!(ler(&banco, &outra, "cartao").unwrap().is_none());
}

#[test]
fn parcial_nao_pode_virar_controle_disponivel() {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let mut observado = controle();
    observado.recebidos = TabelaObservada::nova(Some(20), None, vec![]).unwrap();
    assert!(guardar(&mut banco, "cartao", &observado).is_err());
    assert!(ler(&banco, &origem(), "cartao").unwrap().is_none());
}

#[test]
fn nenhuma_pagina_ficticia_sobrevive_a_reconstrucao() {
    let mut primeiro = Connection::open_in_memory().unwrap();
    iniciar(&primeiro).unwrap();
    guardar(&mut primeiro, "cartao", &controle()).unwrap();
    let segundo = Connection::open_in_memory().unwrap();
    iniciar(&segundo).unwrap();
    assert!(ler(&segundo, &origem(), "cartao").unwrap().is_none());
}
