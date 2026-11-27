//! Regressão do estado SQLite do anti-IES (#589), com dados sintéticos.
use super::{CENARIO, Marcador, Observacao, catalogo_marcadores, estado, marcadores};

fn conferir_apendice(ids: &[&str], retirados: &[&str]) {
    let (mut banco, origem) = estado(true, true);
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

    marcadores::atribuir(&banco, &origem, CENARIO, "10", "TESTE", "501", "Nova revisão")
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
    assert_eq!(
        catalogo_marcadores::ler(&banco, &origem, CENARIO, "TESTE").unwrap(),
        catalogo
    );
}

#[test]
fn retirada_intermediaria_nao_colide_com_a_posicao_do_sobrevivente() {
    conferir_apendice(&["502", "503", "504"], &["503"]);
}

#[test]
fn atribuicao_apos_duas_retiradas_preserva_ordem_de_apendice() {
    conferir_apendice(&["502", "503", "504", "505"], &["503", "504"]);
}
