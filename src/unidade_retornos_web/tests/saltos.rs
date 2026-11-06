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


#[test]
fn virada_utc_para_ano_novo_somente_publica_o_dia_hoje_servido() {
    // Na virada de ano, o SEI pode estar em 01/01 quando ainda e 31/12 UTC.
    // Nao usar dezembro do relogio como se fosse a data observada.
    let janeiro_com_hoje = calendario(2027, 1, true)
        .replace("data_atual=15%2F01%2F2027", "data_atual=01%2F01%2F2027");
    let mut pedidos = Vec::new();
    let lido = ler_com_data_utc(
        CONTROLE,
        &controle(),
        &instalacao(),
        None,
        (2026, 12, 31),
        |url| {
            pedidos.push(url.to_owned());
            if url.contains("data_inicial=") {
                Ok(linha().to_owned())
            } else if url.contains("data_atual=01%2F01%2F2027") {
                Ok(janeiro_com_hoje.clone())
            } else {
                Ok(calendario(2026, 12, false))
            }
        },
    )
    .unwrap();
    assert_eq!(lido.mes, "2027-01");
    assert_eq!(lido.requisicoes, 4);
    assert_eq!(lido.itens.len(), 1);
    assert_eq!(pedidos.len(), 3);
    assert!(pedidos[1].contains("data_atual=01%2F01%2F2027"));
    assert!(pedidos[2].contains("data_inicial=01%2F01%2F2027"));
}

#[test]
fn virada_de_ano_sem_link_de_janeiro_recusa_sem_consulta_mensal() {
    // A pista UTC nunca autoriza construir uma URL assinada que nao
    // apareceu no calendario; nem permite publicar uma lista de dezembro.
    let ancora = format!(
        "<a href='{}'>vizinho</a>",
        href("data_atual=01%2F01%2F2027")
    );
    let dezembro_sem_janeiro = calendario(2026, 12, false).replace(&ancora, "");
    let mut pedidos = Vec::new();
    let resultado = ler_com_data_utc(
        CONTROLE,
        &controle(),
        &instalacao(),
        None,
        (2026, 12, 31),
        |url| {
            pedidos.push(url.to_owned());
            Ok(dezembro_sem_janeiro.clone())
        },
    );
    assert!(matches!(
        resultado,
        Err(ErroLeituraRetornos::MesDosRetornosInacessivel)
    ));
    assert_eq!(pedidos.len(), 1);
    assert!(!pedidos.iter().any(|url| url.contains("data_inicial=")));
}
