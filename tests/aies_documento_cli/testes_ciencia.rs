//! BDD-271-02: a CLI real registra ciência do documento no anti-IES e relê o efeito.
//! Sintético: não equivale a confronto independente com o SEI.
use super::*;

#[test]
fn bdd_271_02_ciencia_documental_uma_vez_com_lista_e_sqlite() {
    let pasta = Pasta::nova();
    let args = [
        "--json",
        "documento",
        "ciente",
        DOC,
    ];
    let previa = executar(&pasta, &args, 4);
    assert_eq!(
        previa.status.code(),
        Some(0),
        "prévia: stdout={} stderr={}",
        String::from_utf8_lossy(&previa.stdout),
        String::from_utf8_lossy(&previa.stderr)
    );
    let antes = json(&previa);
    assert_eq!(antes["confirmado"], false);
    assert_eq!(antes["verificado"], false);
    assert_eq!(antes["requisicoes"], 4);

    let banco = Connection::open(pasta.0.join("anti-ies.sqlite3")).unwrap();
    let qtde: i64 = banco.query_row("SELECT COUNT(*) FROM ciencia_documental WHERE origem=?1 AND cenario=?2 AND id_documento='42'",
        rusqlite::params![ORIGEM, CENARIO], |r| r.get(0)).unwrap();
    assert_eq!(qtde, 0, "prévia não grava ciência");
    drop(banco);

    let confirmado = executar(
        &pasta,
        &[
            "--json",
            "documento",
            "ciente",
            DOC,
            "--confirmar",
        ],
        5,
    );
    assert_eq!(
        confirmado.status.code(),
        Some(0),
        "confirmar: stdout={} stderr={}",
        String::from_utf8_lossy(&confirmado.stdout),
        String::from_utf8_lossy(&confirmado.stderr)
    );
    let feito = json(&confirmado);
    assert_eq!(feito["confirmado"], true);
    assert_eq!(feito["verificado"], true);
    assert_eq!(feito["numero"], DOC);
    assert_eq!(feito["id_documento"], "42");
    assert_eq!(feito["processo"]["id"], "7");
    assert_eq!(feito["requisicoes"], 5);
    let atividade = feito["id_atividade"].as_str().expect("atividade observada");

    // Oráculo independente da árvore e do leitor HTML: estado gravado no SQLite.
    let banco = Connection::open(pasta.0.join("anti-ies.sqlite3")).unwrap();
    let registros: Vec<String> = {
        let mut query = banco.prepare("SELECT id_atividade FROM ciencia_documental WHERE origem=?1 AND cenario=?2 AND id_documento='42' AND id_procedimento='7'").unwrap();
        query
            .query_map(rusqlite::params![ORIGEM, CENARIO], |r| {
                r.get::<_, String>(0)
            })
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    assert_eq!(registros, vec![atividade.to_owned()]);
}
