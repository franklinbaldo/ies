//! Regressão técnica para a relação bilateral usada no BDD-258-01/03.
//! Não comprova paridade com cassettes nem execução em SEI real.
use iesei::banco_cassettes::iniciar;
use iesei::instalacao::Instalacao;
use iesei::processos::{ProcessoSimulado, guardar};
use iesei::relacionamentos::{listar, observar, registrar};
use rusqlite::{Connection, params};

#[test]
fn registrar_relacao_inversa_nao_cria_segundo_vinculo() {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let origem = Instalacao::interpretar("https://sei.test/sei").unwrap();
    let cenario = "simetria";
    for (id, numero) in [("10", "P-10"), ("22", "P-22")] {
        guardar(
            &mut banco,
            &origem,
            cenario,
            &ProcessoSimulado {
                id: id.into(),
                numero: numero.into(),
                unidade_origem: "TESTE".into(),
                aberto_em: vec!["TESTE".into()],
                anotacao_origem: None,
                destinos: vec![],
            },
        )
        .unwrap();
        observar(&banco, &origem, cenario, id).unwrap();
    }
    registrar(&banco, &origem, cenario, "10", "22").unwrap();
    registrar(&banco, &origem, cenario, "22", "10").unwrap();
    let registros: i64 = banco
        .query_row(
            "SELECT COUNT(*) FROM processo_relacao WHERE origem=?1 AND cenario=?2",
            params![origem.url_base(), cenario],
            |linha| linha.get(0),
        )
        .unwrap();
    assert_eq!(registros, 1, "um único vínculo bilateral, não duas arestas direcionais");
    assert_eq!(listar(&banco, &origem, cenario, "10").unwrap().unwrap().len(), 1);
    assert_eq!(listar(&banco, &origem, cenario, "22").unwrap().unwrap().len(), 1);
}
