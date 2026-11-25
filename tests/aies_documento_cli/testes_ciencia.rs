//! BDD-271-02: a CLI real registra ciência do documento no anti-IES e relê o efeito.
//! Sintético: não equivale a confronto independente com o SEI.
use super::*;

#[test]
fn bdd_271_02_ciencia_documental_uma_vez_com_lista_e_sqlite() {
    let pasta = Pasta::nova();
    let args = ["--json", "documento", "ciente", DOC];
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

    // Nova invocação: 4 abertura + 1 tela + 1 aceite + 4 releitura + 1 lista.
    let confirmado = executar(
        &pasta,
        &["--json", "documento", "ciente", DOC, "--confirmar"],
        11,
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
    assert_eq!(feito["requisicoes"], 11);
    assert!(
        feito["usuario"].is_null(),
        "sem identidade de sessão não se inventa nome"
    );
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
    // A ciência é única por documento: nova invocação não pode enviar
    // outro GET potencialmente mutador nem atribuir uma atividade nova.
    // A bancada encerra após os quatro pedidos de abertura do documento.
    let repeticao = executar(
        &pasta,
        &["--json", "documento", "ciente", DOC, "--confirmar"],
        4,
    );
    falha(&repeticao, "CienciaNaoOferecida");
    let banco = Connection::open(pasta.0.join("anti-ies.sqlite3")).unwrap();
    let registros_depois: Vec<String> = banco
        .prepare("SELECT id_atividade FROM ciencia_documental WHERE origem=?1 AND cenario=?2 AND id_documento='42' AND id_procedimento='7'")
        .unwrap()
        .query_map(rusqlite::params![ORIGEM, CENARIO], |r| r.get::<_, String>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(registros_depois, vec![atividade.to_owned()]);
}

/// O BDD-271-02 requer identidade documental: a ciência de um documento
/// não autoriza atribuir sua atividade ao outro documento do mesmo processo.
/// Ensaio sintético; não observa usuário de uma sessão real nem prova SEI.
#[test]
fn bdd_271_02_ciencias_de_documentos_distintos_nao_se_confundem() {
    const OUTRO_DOC: &str = "0011520";
    let pasta = Pasta::nova();
    // Inicializa o SQLite pela mesma CLI que será exercitada no ensaio.
    let previa = executar(&pasta, &["--json", "documento", "ciente", DOC], 4);
    assert_eq!(previa.status.code(), Some(0));
    let caminho = pasta.0.join("anti-ies.sqlite3");
    let mut banco = Connection::open(&caminho).unwrap();
    documento_versoes::guardar(
        &mut banco,
        CENARIO,
        &DocumentoSimulado {
            instalacao: Instalacao::interpretar(ORIGEM).unwrap(),
            id_documento: "43".into(),
            id_procedimento: "7".into(),
            numero_processo: "00001.000001/2026-01".into(),
            numero: OUTRO_DOC.into(),
            tipo: "Despacho".into(),
            revisoes: vec![Revisao {
                versao: NonZeroU32::new(1).unwrap(),
                usuario: "Teste".into(),
                unidade: "TESTE".into(),
                modificacao: "09/11/2026".into(),
                html_corpo: "<p>Segundo documento sintético</p>".into(),
            }],
        },
    )
    .unwrap();
    drop(banco);

    let primeira = executar(
        &pasta,
        &["--json", "documento", "ciente", DOC, "--confirmar"],
        11,
    );
    assert_eq!(
        primeira.status.code(),
        Some(0),
        "primeiro: stdout={} stderr={}",
        String::from_utf8_lossy(&primeira.stdout),
        String::from_utf8_lossy(&primeira.stderr)
    );
    let primeira = json(&primeira);
    assert_eq!(primeira["verificado"], true);
    let atividade_primeira = primeira["id_atividade"].as_str().unwrap().to_owned();

    // A atividade recém-registrada no documento 42 NÃO aparece no 43.
    let banco = Connection::open(&caminho).unwrap();
    let pendente: i64 = banco
        .query_row(
            "SELECT COUNT(*) FROM ciencia_documental
             WHERE origem=?1 AND cenario=?2 AND id_documento='43'",
            rusqlite::params![ORIGEM, CENARIO],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(pendente, 0);
    drop(banco);
    let previa_outro = executar(&pasta, &["--json", "documento", "ciente", OUTRO_DOC], 4);
    assert_eq!(
        previa_outro.status.code(),
        Some(0),
        "outro documento: {}",
        String::from_utf8_lossy(&previa_outro.stderr)
    );
    assert_eq!(json(&previa_outro)["verificado"], false);

    // Uma confirmação própria precisa criar uma segunda atividade no mesmo
    // processo, lida da lista CD43 e registrada separadamente no SQLite.
    let segunda = executar(
        &pasta,
        &["--json", "documento", "ciente", OUTRO_DOC, "--confirmar"],
        11,
    );
    assert_eq!(
        segunda.status.code(),
        Some(0),
        "segundo: stdout={} stderr={}",
        String::from_utf8_lossy(&segunda.stdout),
        String::from_utf8_lossy(&segunda.stderr)
    );
    let segunda = json(&segunda);
    assert_eq!(segunda["id_documento"], "43");
    assert_eq!(segunda["processo"]["id"], "7");
    assert_eq!(segunda["verificado"], true);
    let atividade_segunda = segunda["id_atividade"].as_str().unwrap();
    assert_ne!(atividade_primeira, atividade_segunda);

    let banco = Connection::open(&caminho).unwrap();
    let pares: Vec<(String, String)> = banco
        .prepare(
            "SELECT id_documento, id_atividade FROM ciencia_documental
             WHERE origem=?1 AND cenario=?2 ORDER BY id_documento",
        )
        .unwrap()
        .query_map(rusqlite::params![ORIGEM, CENARIO], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(
        pares,
        vec![
            ("42".into(), atividade_primeira),
            ("43".into(), atividade_segunda.to_owned())
        ]
    );
}

/// BDD-271-02, lacuna de autoria: a revisão ("Ana"/"Bia") não prova
/// quem praticou a ciência. A identidade desta sessão é explicitamente
/// preparada no SQLite sintético e não deriva de cookie ou credencial.
#[test]
fn bdd_271_02_ciencia_guarda_o_usuario_da_sessao_simulada() {
    let pasta = Pasta::nova();
    let previa = executar(&pasta, &["--json", "documento", "ciente", DOC], 4);
    assert_eq!(previa.status.code(), Some(0));
    let caminho = pasta.0.join("anti-ies.sqlite3");
    let banco = Connection::open(&caminho).unwrap();
    // DML de uma sessão conhecida, nunca DDL de fixture no teste.
    // O CI RED anterior confirmou a ausência da tabela (PR #571).
    banco
        .execute(
            "INSERT INTO sessao_usuario (origem, cenario, usuario)
         VALUES (?1, ?2, ?3)",
            rusqlite::params![ORIGEM, CENARIO, "agente-de-teste"],
        )
        .unwrap();
    drop(banco);

    let confirmado = executar(
        &pasta,
        &["--json", "documento", "ciente", DOC, "--confirmar"],
        11,
    );
    assert_eq!(
        confirmado.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&confirmado.stderr)
    );
    let resultado = json(&confirmado);
    assert_eq!(resultado["verificado"], true);
    // O usuário deve vir da Lista de Ciências relida pela CLI, não apenas do SQL.
    assert_eq!(resultado["usuario"], "agente-de-teste");
    let id = resultado["id_atividade"].as_str().unwrap();
    let banco = Connection::open(&caminho).unwrap();
    let (atividade, usuario): (String, Option<String>) = banco
        .query_row(
            "SELECT id_atividade, usuario FROM ciencia_documental
             WHERE origem=?1 AND cenario=?2 AND id_documento='42'",
            rusqlite::params![ORIGEM, CENARIO],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(atividade, id);
    assert_eq!(usuario.as_deref(), Some("agente-de-teste"));
    assert_ne!(
        usuario.as_deref(),
        Some("Bia"),
        "autor da revisão não é autor da ciência"
    );
}
/// BDD-271-03: o GET mutador persistiu uma ciência, mas a Lista relida
/// ainda não a publicou. Não inferir fracasso nem reenviar a escrita.
/// Hipótese de consistência eventual do anti-IES; NÃO é paridade de cassette.
#[test]
fn bdd_271_03_ciencia_persistida_sem_prova_na_lista_falha_sem_retry() {
    let pasta = Pasta::nova();
    let previa = executar(&pasta, &["--json", "documento", "ciente", DOC], 4);
    assert_eq!(previa.status.code(), Some(0));
    assert_eq!(json(&previa)["verificado"], false);

    // O modo de observação é declarado na migração canônica, sem DDL em teste.
    // O documento continua gravável; somente a lista relida omite sua atividade.
    let caminho = pasta.0.join("anti-ies.sqlite3");
    let banco = Connection::open(&caminho).unwrap();
    banco
        .execute(
            "INSERT INTO ciencia_documental_lista_invisivel
         (origem, cenario, id_documento) VALUES (?1, ?2, '42')",
            rusqlite::params![ORIGEM, CENARIO],
        )
        .unwrap();
    drop(banco);

    // O mesmo orçamento de 11 GETs do aceite verificável: o mutador é
    // enviado uma única vez, seguido da árvore e da Lista independentes.
    let resultado = executar(
        &pasta,
        &["--json", "documento", "ciente", DOC, "--confirmar"],
        11,
    );
    falha(&resultado, "NaoConfirmado");

    let banco = Connection::open(&caminho).unwrap();
    let atividade: String = banco
        .query_row(
            "SELECT id_atividade FROM ciencia_documental
             WHERE origem=?1 AND cenario=?2 AND id_documento='42'",
            rusqlite::params![ORIGEM, CENARIO],
            |linha| linha.get(0),
        )
        .unwrap();
    assert_eq!(atividade, "42457", "a escrita chegou ao SQLite");
    drop(banco);

    // Outra invocação (não um retry automático) encontra o botão ausente,
    // não reemite o GET mutador e não cria segunda atividade.
    let repeticao = executar(
        &pasta,
        &["--json", "documento", "ciente", DOC, "--confirmar"],
        4,
    );
    assert_eq!(repeticao.status.code(), Some(1));
    let banco = Connection::open(&caminho).unwrap();
    let total: i64 = banco
        .query_row(
            "SELECT COUNT(*) FROM ciencia_documental
             WHERE origem=?1 AND cenario=?2 AND id_documento='42'
               AND id_atividade=?3",
            rusqlite::params![ORIGEM, CENARIO, atividade],
            |linha| linha.get(0),
        )
        .unwrap();
    assert_eq!(total, 1, "o efeito original permanece único");
}
