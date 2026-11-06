use super::*;

#[test]
fn calendario_deslocado_dois_meses_adiante_obedece_links_e_contagem() {
    // Navegacao sintetica baseada nos links data_atual observados no acervo.
    // O mes vem do diaHoje servido, nao do relogio UTC usado como pista.
    let mut pedidos = Vec::new();
    let lido = ler_com_data_utc(
        CONTROLE,
        &controle(),
        &instalacao(),
        None,
        (2026, 11, 5),
        |url| {
            pedidos.push(url.to_owned());
            if url.contains("data_inicial=") {
                Ok(linha().to_owned())
            } else if url.contains("data_atual=01%2F11%2F2026") {
                Ok(calendario(2026, 11, true))
            } else if url.contains("data_atual=01%2F10%2F2026") {
                Ok(calendario(2026, 10, false))
            } else {
                Ok(calendario(2026, 9, false))
            }
        },
    )
    .unwrap();
    assert_eq!(lido.mes, "2026-11");
    assert_eq!(lido.requisicoes, 5); // Controle + calendario + 2 saltos + mes.
    assert_eq!(pedidos.len(), 4);
    assert!(pedidos[1].contains("data_atual=01%2F10%2F2026"));
    assert!(pedidos[2].contains("data_atual=01%2F11%2F2026"));
    assert!(pedidos[3].contains("data_inicial=01%2F11%2F2026"));
}

#[test]
fn calendario_deslocado_dois_meses_atras_exige_ambos_links_publicados() {
    // O teste disponibiliza, explicitamente, cada URL assinada de retorno.
    // Nao se construiu URL de navegacao por adivinhacao.
    let janeiro = format!(
        "{}<a href='{}'>anterior</a>",
        calendario(2027, 1, false),
        href("data_atual=01%2F12%2F2026")
    );
    let dezembro = format!(
        "{}<a href='{}'>anterior</a>",
        calendario(2026, 12, false),
        href("data_atual=01%2F11%2F2026")
    );
    let mut pedidos = Vec::new();
    let lido = ler_com_data_utc(
        CONTROLE,
        &controle(),
        &instalacao(),
        None,
        (2026, 11, 5),
        |url| {
            pedidos.push(url.to_owned());
            if url.contains("data_inicial=") {
                Ok(linha().to_owned())
            } else if url.contains("data_atual=01%2F12%2F2026") {
                Ok(dezembro.clone())
            } else if url.contains("data_atual=01%2F11%2F2026") {
                Ok(calendario(2026, 11, true))
            } else {
                Ok(janeiro.clone())
            }
        },
    )
    .unwrap();
    assert_eq!(lido.mes, "2026-11");
    assert_eq!(lido.requisicoes, 5);
    assert_eq!(pedidos.len(), 4);
    assert!(pedidos[1].contains("data_atual=01%2F12%2F2026"));
    assert!(pedidos[2].contains("data_atual=01%2F11%2F2026"));
    assert!(pedidos[3].contains("data_inicial=01%2F11%2F2026"));
}
