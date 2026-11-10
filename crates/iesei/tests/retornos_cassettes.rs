use iesei::banco_cassettes::iniciar;
use iesei::instalacao::Instalacao;
use iesei::retornos_cassettes::{Mes, Retorno, Situacao, guardar, ler};
use rusqlite::Connection;

fn instalacao() -> Instalacao {
    Instalacao::interpretar("https://sei.exemplo/sei").unwrap()
}

fn mes() -> Mes {
    Mes {
        instalacao: instalacao(),
        data_inicial: "01%2F10%2F2026".into(),
        data_final: "31%2F10%2F2026".into(),
        itens: vec![Retorno {
            situacao: Situacao::Aguardando,
            numero: "NUP-1".into(),
            unidade: "TESTE".into(),
            data_programada: "15/10/2026".into(),
            retornado_em: None,
            prazo_restante: Some("2 dias".into()),
        }],
    }
}

#[test]
fn reconstroi_meses_e_isola_instalacao_e_periodo() {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let versao: i64 = banco
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    // A extensão do esquema derivado não altera a identidade dos retornos.
    assert_eq!(versao, 37);
    let m = mes();
    assert!(
        ler(
            &banco,
            &instalacao(),
            "cartao",
            &m.data_inicial,
            &m.data_final
        )
        .unwrap()
        .is_none()
    );
    guardar(&mut banco, "cartao", &m).unwrap();
    guardar(&mut banco, "cartao", &m).unwrap();
    assert_eq!(
        ler(
            &banco,
            &instalacao(),
            "cartao",
            &m.data_inicial,
            &m.data_final
        )
        .unwrap(),
        Some(m.clone())
    );
    let linhas: i64 = banco
        .query_row("SELECT count(*) FROM retorno_linha", [], |r| r.get(0))
        .unwrap();
    assert_eq!(linhas, 1);
    let outra = Instalacao::interpretar("https://outra.exemplo/sei").unwrap();
    assert!(
        ler(&banco, &outra, "cartao", &m.data_inicial, &m.data_final)
            .unwrap()
            .is_none()
    );
    assert!(
        ler(
            &banco,
            &instalacao(),
            "outro",
            &m.data_inicial,
            &m.data_final
        )
        .unwrap()
        .is_none()
    );
}

#[test]
fn vazio_comprovado_e_diferente_de_mes_desconhecido() {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let mut m = mes();
    m.itens.clear();
    guardar(&mut banco, "vazio", &m).unwrap();
    assert_eq!(
        ler(
            &banco,
            &instalacao(),
            "vazio",
            &m.data_inicial,
            &m.data_final
        )
        .unwrap(),
        Some(m)
    );
    let m = mes();
    assert!(
        ler(
            &banco,
            &instalacao(),
            "outro",
            &m.data_inicial,
            &m.data_final
        )
        .unwrap()
        .is_none()
    );
}
