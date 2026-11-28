//! Regressão do estado SQLite do anti-IES (#589), com dados sintéticos.
use super::{CENARIO, Marcador, Observacao, catalogo_marcadores, estado, marcadores};

fn conferir_apendice(ids: &[&str], retirados: &[&str], posicao_esperada: i64) {
    let (mut banco, origem) = estado(true, true);
    semear_vizinhos(&banco, &origem.url_base());
    let itens: Vec<Marcador> = ids
        .iter()
        .map(|id| Marcador {
            id: (*id).into(),
            nome: format!("Marcador {id}"),
            cor: "azul".into(),
            texto: Some(format!("Texto {id}")),
            usuario: Some("aluno1".into()),
            data_hora: "04/11/2026 09:00".into(),
        })
        .collect();
    marcadores::registrar(
        &mut banco,
        &origem,
        CENARIO,
        "10",
        "TESTE",
        &Observacao {
            total_declarado: itens.len(),
            itens: itens.clone(),
        },
    )
    .unwrap();
    let catalogo = catalogo_marcadores::ler(&banco, &origem, CENARIO, "TESTE").unwrap();
    assert!(catalogo.is_some());
    for id in retirados {
        marcadores::retirar(&banco, &origem, CENARIO, "10", "TESTE", id).unwrap();
    }
    let sobreviventes: Vec<_> = itens
        .into_iter()
        .filter(|m| !retirados.contains(&m.id.as_str()))
        .collect();
    let apos_retirada = marcadores::ler(&banco, &origem, CENARIO, "10", "TESTE")
        .unwrap()
        .unwrap();
    assert_eq!(apos_retirada.itens, sobreviventes);
    assert_eq!(apos_retirada.total_declarado, sobreviventes.len());

    marcadores::atribuir(
        &banco,
        &origem,
        CENARIO,
        "10",
        "TESTE",
        "501",
        "Nova revisão",
    )
    .unwrap();
    let finalizado = marcadores::ler(&banco, &origem, CENARIO, "10", "TESTE")
        .unwrap()
        .unwrap();
    let mut esperados = sobreviventes;
    esperados.push(Marcador {
        id: "501".into(),
        nome: "Revisão".into(),
        cor: "azul".into(),
        texto: Some("Nova revisão".into()),
        usuario: None,
        data_hora: "01/01/2026 00:00".into(),
    });
    assert_eq!(finalizado.total_declarado, esperados.len());
    assert_eq!(finalizado.itens, esperados);
    // O leitor ordena, mas não expõe a posição: conferir também o índice físico.
    let posicao: i64 = banco
        .query_row(
            "SELECT posicao FROM processo_marcador_linha
             WHERE origem=?1 AND cenario=?2 AND id_procedimento='10'
               AND unidade='TESTE' AND id_marcador='501'",
            rusqlite::params![origem.url_base(), CENARIO],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(posicao, posicao_esperada);
    let posicoes_vizinhas = banco
        .prepare(
            "SELECT posicao FROM processo_marcador_linha WHERE id_marcador='599' ORDER BY posicao",
        )
        .unwrap()
        .query_map([], |r| r.get::<_, i64>(0))
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
    assert_eq!(posicoes_vizinhas, vec![10, 11, 12, 13]);
    assert_eq!(
        catalogo_marcadores::ler(&banco, &origem, CENARIO, "TESTE").unwrap(),
        catalogo
    );
}

#[test]
fn retirada_intermediaria_nao_colide_com_a_posicao_do_sobrevivente() {
    conferir_apendice(&["502", "503", "504"], &["503"], 3);
}

#[test]
fn atribuicao_apos_duas_retiradas_preserva_ordem_de_apendice() {
    conferir_apendice(&["502", "503", "504", "505"], &["503", "504"], 4);
}

// Uma testemunha para cada chave da coorte, com índice maior que o alvo.
fn semear_vizinhos(banco: &rusqlite::Connection, origem: &str) {
    for (outra_origem, cenario, id, unidade, posicao) in [
        ("https://outra.test/sei", CENARIO, "10", "TESTE", 10),
        (origem, "outro-cenario", "10", "TESTE", 11),
        (origem, CENARIO, "11", "TESTE", 12),
        (origem, CENARIO, "10", "OUTRA", 13),
    ] {
        if unidade != "OUTRA" {
            banco
                .execute(
                    "INSERT INTO sim_processo
                     (origem, cenario, id_procedimento, nup, unidade)
                     VALUES (?1, ?2, ?3, '00001.000002/2026-01', ?4)",
                    rusqlite::params![outra_origem, cenario, id, unidade],
                )
                .unwrap();
        }
        banco
            .execute(
                "INSERT INTO processo_marcadores_observados
                 (origem, cenario, id_procedimento, unidade, total_declarado)
                 VALUES (?1, ?2, ?3, ?4, 1)",
                rusqlite::params![outra_origem, cenario, id, unidade],
            )
            .unwrap();
        banco
            .execute(
                "INSERT INTO processo_marcador_linha
                 (origem, cenario, id_procedimento, unidade, posicao,
                  id_marcador, nome, cor, data_hora)
                 VALUES (?1, ?2, ?3, ?4, ?5, '599', 'Vizinho', 'azul', '04/11/2026 09:00')",
                rusqlite::params![outra_origem, cenario, id, unidade, posicao],
            )
            .unwrap();
    }
}

#[test]
fn atribuicao_em_observacao_vazia_comeca_em_zero() {
    conferir_apendice(&[], &[], 0);
}

#[test]
fn retirada_do_sufixo_reutiliza_a_posicao_livre() {
    conferir_apendice(&["502", "503", "504"], &["504"], 2);
}
