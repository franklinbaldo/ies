//! Hipótese sintética: catálogo por instalação, cenário e unidade.
use iesei::banco_cassettes::iniciar;
use iesei::documentos_simulados::{TextoPadrao, textos_padrao};
use iesei::instalacao::Instalacao;
use rusqlite::{Connection, params};

#[test]
fn catalogo_preserva_homonimos_e_isola_escopos() {
    let banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let origem = Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
    let outra = Instalacao::interpretar("https://outro.invalid/sei").unwrap();
    for (instalacao, cenario, unidade, id, nome) in [
        (&origem, "c", "U1", "007", "Minuta A & B"),
        (&origem, "c", "U1", "008", "Minuta A & B"),
        (&origem, "c", "U2", "007", "Outra unidade"),
        (&origem, "d", "U1", "007", "Outro cenário"),
        (&outra, "c", "U1", "007", "Outra origem"),
    ] {
        banco
            .execute(
                "INSERT INTO sim_texto_padrao VALUES (?1, ?2, ?3, ?4, ?5)",
                params![instalacao.url_base(), cenario, unidade, id, nome],
            )
            .unwrap();
    }
    let texto = |id: &str, nome: &str| TextoPadrao {
        id: id.into(),
        nome: nome.into(),
    };
    assert_eq!(
        textos_padrao(&banco, &origem, "c", "U1").unwrap(),
        [texto("007", "Minuta A & B"), texto("008", "Minuta A & B")]
    );
    for (instalacao, cenario, unidade, nome) in [
        (&origem, "c", "U2", "Outra unidade"),
        (&origem, "d", "U1", "Outro cenário"),
        (&outra, "c", "U1", "Outra origem"),
    ] {
        assert_eq!(
            textos_padrao(&banco, instalacao, cenario, unidade).unwrap(),
            [texto("007", nome)]
        );
    }
    assert_eq!(
        textos_padrao(&banco, &origem, "c", "U3").unwrap(),
        Vec::<TextoPadrao>::new()
    );
    assert_eq!(banco.total_changes(), 5);
}
