use iesei::acesso_sessao::{self, AcessoSessao};
use iesei::banco_cassettes::iniciar;
use iesei::instalacao::Instalacao;
use rusqlite::Connection;

#[test]
fn sessao_recusada_e_idempotente_e_isolada_por_origem_e_cenario() {
    let banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
    let outra = Instalacao::interpretar("https://outro.test/sei").unwrap();
    assert_eq!(
        acesso_sessao::consultar(&banco, &origem, "ensaio").unwrap(),
        AcessoSessao::Permitido
    );
    acesso_sessao::bloquear(&banco, &origem, "ensaio").unwrap();
    acesso_sessao::bloquear(&banco, &origem, "ensaio").unwrap();
    assert_eq!(
        acesso_sessao::consultar(&banco, &origem, "ensaio").unwrap(),
        AcessoSessao::Recusado
    );
    assert_eq!(
        acesso_sessao::consultar(&banco, &outra, "ensaio").unwrap(),
        AcessoSessao::Permitido
    );
    assert_eq!(
        acesso_sessao::consultar(&banco, &origem, "outro").unwrap(),
        AcessoSessao::Permitido
    );
    let linhas: i64 = banco
        .query_row("SELECT count(*) FROM sessao_bloqueada", [], |r| r.get(0))
        .unwrap();
    assert_eq!(linhas, 1);
}
