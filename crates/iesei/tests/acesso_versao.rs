use iesei::acesso_versao::{self, AcessoVersao};
use iesei::banco_cassettes::iniciar;
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::num::NonZeroU32;

#[test]
fn restricao_de_versao_isolada_por_cenario_instalacao_e_numero() {
    let db = Connection::open_in_memory().unwrap();
    iniciar(&db).unwrap();
    let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
    let outra = Instalacao::interpretar("https://outro.test/sei").unwrap();
    let um = NonZeroU32::new(1).unwrap();
    let dois = NonZeroU32::new(2).unwrap();

    acesso_versao::restringir(&db, &origem, "ensaio", "42", um).unwrap();
    acesso_versao::restringir(&db, &origem, "ensaio", "42", um).unwrap();
    assert_eq!(
        acesso_versao::consultar(&db, &origem, "ensaio", "42", um).unwrap(),
        AcessoVersao::Recusado
    );
    for (instalacao, cenario, id, versao) in [
        (&origem, "ensaio", "42", dois),
        (&origem, "outro", "42", um),
        (&origem, "ensaio", "43", um),
        (&outra, "ensaio", "42", um),
    ] {
        assert_eq!(
            acesso_versao::consultar(&db, instalacao, cenario, id, versao).unwrap(),
            AcessoVersao::Acessivel
        );
    }
    let total: i64 = db
        .query_row("SELECT count(*) FROM versao_inacessivel", [], |r| r.get(0))
        .unwrap();
    assert_eq!(total, 1);
}
