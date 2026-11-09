//! BDD-271-01/03: ciencia do processo por CLI real, TLS local e SQLite.
//! Hipotese sintetica; cassettes e treinamento sao provas separadas.
use super::*;
use iesei::processos::{self, ProcessoSimulado};

const NUP_CIENTE: &str = "00001.000001/2026-01";

fn preparar_processo(pasta: &Pasta) {
    // Usa o bootstrap do anti-IES ja existente, sem DDL artificial.
    let inicial = executar(pasta, &["--json", "documento", "ciente", DOC], 4);
    assert_eq!(inicial.status.code(), Some(0));
    let mut banco = Connection::open(pasta.0.join("anti-ies.sqlite3")).unwrap();
    let instalacao = Instalacao::interpretar(ORIGEM).unwrap();
    processos::guardar(
        &mut banco,
        &instalacao,
        CENARIO,
        &ProcessoSimulado {
            id: "7".into(),
            numero: NUP_CIENTE.into(),
            unidade_origem: "TESTE".into(),
            aberto_em: vec!["TESTE".into()],
            anotacao_origem: None,
            destinos: vec![],
        },
    ).unwrap();
    banco.execute(
        "INSERT INTO sessao_usuario (origem, cenario, usuario) VALUES (?1, ?2, 'agente-de-teste')",
        rusqlite::params![ORIGEM, CENARIO],
    ).unwrap();
}

#[test]
fn bdd_271_01_processo_ciencia_previa_unica_e_lista_distinta() {
    let pasta = Pasta::nova();
    preparar_processo(&pasta);
    let previa = executar(&pasta, &["--json", "processo", "ciente", NUP_CIENTE], 3);
    assert_eq!(previa.status.code(), Some(0), "previa stdout={} stderr={}",
        String::from_utf8_lossy(&previa.stdout), String::from_utf8_lossy(&previa.stderr));
    assert_eq!(json(&previa)["verificado"], false);
    let banco = Connection::open(pasta.0.join("anti-ies.sqlite3")).unwrap();
    let antes: i64 = banco.query_row("SELECT COUNT(*) FROM ciencia_processo", [], |r| r.get(0)).unwrap();
    assert_eq!(antes, 0, "previa nao escreve");
    drop(banco);

    // Tres GETs de abertura, um da tela, um mutador, nova arvore e uma Lista.
    let resultado = executar(&pasta,
        &["--json", "processo", "ciente", NUP_CIENTE, "--confirmar"], 7);
    assert_eq!(resultado.status.code(), Some(0), "confirmacao stdout={} stderr={}",
        String::from_utf8_lossy(&resultado.stdout), String::from_utf8_lossy(&resultado.stderr));
    let saida = json(&resultado);
    assert_eq!(saida["verificado"], true);
    assert_eq!(saida["numero"], NUP_CIENTE);
    assert_eq!(saida["id_procedimento"], "7");
    assert_eq!(saida["requisicoes"], 7);
    assert_eq!(saida["usuario"], "agente-de-teste");
    let atividade = saida["id_atividade"].as_str().expect("atividade confirmada");
    let banco = Connection::open(pasta.0.join("anti-ies.sqlite3")).unwrap();
    let (id, usuario): (String, Option<String>) = banco.query_row(
        "SELECT id_atividade, usuario FROM ciencia_processo WHERE origem=?1 AND cenario=?2 AND id_procedimento='7'",
        rusqlite::params![ORIGEM,CENARIO], |r| Ok((r.get(0)?,r.get(1)?))).unwrap();
    assert_eq!(id, atividade);
    assert_eq!(usuario.as_deref(), Some("agente-de-teste"));
    let documentos: i64 = banco.query_row("SELECT COUNT(*) FROM ciencia_documental", [], |r| r.get(0)).unwrap();
    assert_eq!(documentos, 0, "ciencia de processo nao e de documento");
    drop(banco);
    let repeticao = executar(&pasta,
        &["--json", "processo", "ciente", NUP_CIENTE, "--confirmar"], 3);
    assert_eq!(repeticao.status.code(), Some(1), "sem segundo GET mutador");
}

#[test]
fn bdd_271_03_processo_lista_invisivel_e_efeito_incerto_sem_retry() {
    let pasta = Pasta::nova();
    preparar_processo(&pasta);
    let banco = Connection::open(pasta.0.join("anti-ies.sqlite3")).unwrap();
    banco.execute(
        "INSERT INTO ciencia_processo_lista_invisivel (origem,cenario,id_procedimento) VALUES (?1,?2,'7')",
        rusqlite::params![ORIGEM,CENARIO]).unwrap();
    drop(banco);
    let resultado = executar(&pasta,
        &["--json", "processo", "ciente", NUP_CIENTE, "--confirmar"], 7);
    assert_eq!(resultado.status.code(), Some(3), "stdout={} stderr={}",
        String::from_utf8_lossy(&resultado.stdout), String::from_utf8_lossy(&resultado.stderr));
    let saida = json(&resultado);
    assert_eq!(saida["codigo"], "efeito_incerto");
    assert_eq!(saida["retentavel"], false);
    assert_eq!(saida["requisicoes"], 7);
    let banco = Connection::open(pasta.0.join("anti-ies.sqlite3")).unwrap();
    let atividade: String = banco.query_row(
        "SELECT id_atividade FROM ciencia_processo WHERE origem=?1 AND cenario=?2 AND id_procedimento='7'",
        rusqlite::params![ORIGEM,CENARIO], |r| r.get(0)).unwrap();
    drop(banco);
    let repeticao = executar(&pasta,
        &["--json", "processo", "ciente", NUP_CIENTE, "--confirmar"], 3);
    assert_eq!(repeticao.status.code(), Some(1));
    let banco = Connection::open(pasta.0.join("anti-ies.sqlite3")).unwrap();
    let atividades: Vec<String> = banco.prepare(
        "SELECT id_atividade FROM ciencia_processo WHERE origem=?1 AND cenario=?2 AND id_procedimento='7'")
        .unwrap().query_map(rusqlite::params![ORIGEM,CENARIO], |r| r.get(0))
        .unwrap().map(Result::unwrap).collect();
    assert_eq!(atividades, vec![atividade], "nenhuma escrita duplicada");
}
