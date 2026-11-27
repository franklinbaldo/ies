//! #876: hipótese sintética de prévia, não replay do POST original.
use super::{CENARIO, Command, ORIGEM, executar, fluxo::executar_fluxo, json};
use rusqlite::{Connection, params};

#[test]
fn criar_bloco_previa_nao_grava_nem_adivinha_identidade() {
    // RED de gramática antes do TLS; não mascarar a lacuna com timeout.
    let ajuda = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["bloco", "criar", "--help"])
        .output()
        .unwrap();
    assert!(
        ajuda.status.success(),
        "gramática bloco criar ausente: {}",
        String::from_utf8_lossy(&ajuda.stderr)
    );
    // A descrição já existe no SQLite. A prévia não pode adotar seu ID.
    let (saida, pedidos) = executar(false, None, true);
    assert!(pedidos.iter().all(|p| p.starts_with("GET ")));
    let valor = json(&saida);
    assert_eq!(valor["unidade"], "TESTE");
    assert_eq!(valor["tipo_bloco"], "assinatura");
    assert_eq!(valor["descricao"], "Despachos de teste");
    assert_eq!(valor["grupo_padrao"], "null");
    assert_eq!(valor["confirmado"], false);
    assert_eq!(valor["alterado"], false);
    assert!(valor.get("bloco").is_none());
    assert!(!valor.to_string().contains("infra_hash"));
    assert!(!valor.to_string().contains("PHPSESSID"));
}

pub(super) fn conferir_sqlite(banco: &Connection) {
    let (total, inicio, fim): (i64, i64, i64) = banco
        .query_row(
            "SELECT total,inicio,fim FROM sim_bloco_pagina
             WHERE origem=?1 AND cenario=?2 AND tipo='assinatura' AND pagina=1",
            params![ORIGEM, CENARIO],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!((total, inicio, fim), (3, 1, 3));
    let mut stmt = banco
        .prepare(
            "SELECT numero,estado,descricao FROM sim_bloco_linha
             WHERE origem=?1 AND cenario=?2 AND tipo='assinatura' AND pagina=1
             ORDER BY posicao",
        )
        .unwrap();
    let linhas: Vec<(String, String, String)> = stmt
        .query_map(params![ORIGEM, CENARIO], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(
        linhas,
        vec![
            ("2195".to_owned(), "Gerado".to_owned(), "Despachos de teste".to_owned()),
            (
                "2196".to_owned(),
                "Disponibilizado".to_owned(),
                "Assinaturas pendentes".to_owned()
            ),
            ("2197".to_owned(), "Gerado".to_owned(), "Despachos de teste".to_owned()),
        ]
    );
}

#[test]
fn criar_bloco_confirmado_preserva_homonimo_e_persiste_novo_id() {
    let (saida, pedidos) = executar_fluxo(false, None, true, true, true);
    let valor = json(&saida);
    assert_eq!(valor["confirmado"], true);
    assert_eq!(valor["alterado"], true);
    assert_eq!(valor["bloco"]["numero"], "2197");
    assert_eq!(valor["bloco"]["descricao"], "Despachos de teste");
    assert_eq!(valor["bloco"]["estado"], "Gerado");
    assert!(pedidos[3].starts_with("POST "));
    assert!(pedidos[3].contains("acao=bloco_assinatura_cadastrar"));
    assert!(pedidos[5].contains("acao=bloco_assinatura_listar"));
}

#[test]
fn criar_bloco_confirmar_recusa_lista_parcial_sem_post() {
    let (saida, pedidos) = executar_fluxo(false, None, true, false, true);
    assert!(!saida.status.success());
    assert!(pedidos.iter().all(|p| p.starts_with("GET ")));
}
