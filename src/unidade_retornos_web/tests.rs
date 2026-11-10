use super::*;

const ORIGEM: &str = "https://sei.treinamento.unb.br/sei";
const CONTROLE: &str =
    "https://sei.treinamento.unb.br/sei/controlador.php?acao=procedimento_controlar";

fn instalacao() -> Instalacao {
    Instalacao::interpretar(ORIGEM).unwrap()
}
fn href(complemento: &str) -> String {
    let consulta = if complemento.is_empty() {
        String::new()
    } else {
        format!("&{complemento}")
    };
    format!(
        "controlador.php?acao=retorno_programado_listar&infra_hash={}{}",
        "a".repeat(64),
        consulta
    )
}
fn calendario(ano: u16, mes: u8, hoje: bool) -> String {
    let fim = unidade_retornos::ultimo_dia(ano, mes).unwrap();
    let (a_seguinte, m_seguinte) = mes_vizinho((ano, mes), 1).unwrap();
    let data_hoje = format!("15%2F{mes:02}%2F{ano:04}");
    let classe = if hoje { "diaHoje" } else { "diaUtil" };
    format!(
        "<html><a href='{}'>Ver todo o mes</a>\
         <td class='{classe}'><a href='{}'>15</a></td>\
         <a href='{}'>vizinho</a></html>",
        href(&format!(
            "data_inicial=01%2F{mes:02}%2F{ano:04}&data_final={fim:02}%2F{mes:02}%2F{ano:04}"
        )),
        href(&format!("data_atual={data_hoje}")),
        href(&format!(
            "data_atual=01%2F{m_seguinte:02}%2F{a_seguinte:04}"
        )),
    )
}
fn linha() -> &'static str {
    "<table summary='Aguardando Retorno de Outras Unidades'>\
     <caption>1 registro:</caption>\
     <tr><th>Processo</th><th>Unidade</th><th>Data Programada</th></tr>\
     <tr><td>001/2026</td><td>Unidade 1</td><td>15/10/2026</td></tr></table>\
     <div id='divDevolverAreaTabela'>Nenhum registro encontrado.</div>"
}
fn controle() -> String {
    format!("<a href='{}'>Retorno Programado</a>", href(""))
}
#[test]
fn mes_da_tela_custa_dois_get_apos_o_controle() {
    // Caso Todos tests/test_unidade_listar_retornos_010.py @323eb01.
    let mut chamados = Vec::new();
    let resultado = ler_com(
        CONTROLE,
        &controle(),
        &instalacao(),
        Some((2026, 10)),
        |url| {
            chamados.push(url.to_owned());
            if url.contains("data_inicial=") {
                Ok(linha().to_owned())
            } else {
                Ok(calendario(2026, 10, true))
            }
        },
    )
    .unwrap();
    assert_eq!(resultado.mes, "2026-10");
    assert_eq!(resultado.requisicoes, 3);
    assert_eq!(resultado.itens.len(), 1);
    assert_eq!(resultado.itens[0].numero, "001/2026");
    assert_eq!(chamados.len(), 2);
    assert!(chamados[1].contains("data_inicial=01%2F10%2F2026"));
}
#[test]
fn proximo_mes_usa_um_get_adicional_sem_repetir_pagina() {
    // Caso Todos tests/test_unidade_listar_retornos_010.py @323eb01.
    let mut chamados = Vec::new();
    let resultado = ler_com(
        CONTROLE,
        &controle(),
        &instalacao(),
        Some((2026, 11)),
        |url| {
            chamados.push(url.to_owned());
            if url.contains("data_inicial=") {
                Ok("<div id='divAguardandoAreaTabela'>Nenhum registro encontrado.</div>".into())
            } else if url.contains("data_atual=01%2F11%2F2026") {
                Ok(calendario(2026, 11, false))
            } else {
                Ok(calendario(2026, 10, true))
            }
        },
    )
    .unwrap();
    assert_eq!(resultado.mes, "2026-11");
    assert_eq!(resultado.requisicoes, 4);
    assert_eq!(resultado.itens.len(), 0);
    assert_eq!(chamados.len(), 3);
    assert!(chamados[1].contains("data_atual=01%2F11%2F2026"));
    assert!(chamados[2].contains("data_inicial=01%2F11%2F2026"));
}
#[test]
fn dois_links_mensais_identicos_sao_uma_unica_oferta() {
    // Replay das sete instalacoes: o calendario publica duas ancoras
    // para a MESMA URL assinada de "Ver todo o mes".
    let link = href("data_inicial=01%2F10%2F2026&data_final=31%2F10%2F2026");
    let calendario_duplo = calendario(2026, 10, true).replace(
        "</html>",
        &format!("<a href='{link}'>Ver todo o mes (rodape)</a></html>"),
    );
    let mut chamados = 0;
    let lido = ler_com(
        CONTROLE,
        &controle(),
        &instalacao(),
        Some((2026, 10)),
        |url| {
            chamados += 1;
            if url.contains("data_inicial=") {
                Ok(linha().to_owned())
            } else {
                Ok(calendario_duplo.clone())
            }
        },
    )
    .unwrap();
    assert_eq!(lido.mes, "2026-10");
    assert_eq!(lido.itens.len(), 1);
    assert_eq!(lido.requisicoes, 3);
    assert_eq!(chamados, 2);
}

#[test]
fn links_mensais_diferentes_continuam_ambiguos_e_sem_get_criador() {
    // Duas ofertas assinadas diferentes nao sao equivalentes.
    let segundo = href("data_inicial=01%2F10%2F2026&data_final=31%2F10%2F2026&pagina=2");
    let calendario_ambiguo = calendario(2026, 10, true).replace(
        "</html>",
        &format!("<a href='{segundo}'>Outro mes assinado</a></html>"),
    );
    let mut chamados = 0;
    let resultado = ler_com(
        CONTROLE,
        &controle(),
        &instalacao(),
        Some((2026, 10)),
        |_| {
            chamados += 1;
            Ok(calendario_ambiguo.clone())
        },
    );
    assert!(matches!(
        resultado,
        Err(ErroLeituraRetornos::CalendarioDosRetornosDesconhecido)
    ));
    assert_eq!(chamados, 1);
}

#[test]
fn mes_anterior_segue_link_publicado_sem_inventar_url() {
    // Somente se o calendario realmente publicou um href para o primeiro dia.
    let mut chamados = Vec::new();
    let resultado = ler_com(
        CONTROLE,
        &controle(),
        &instalacao(),
        Some((2026, 9)),
        |url| {
            chamados.push(url.to_owned());
            if url.contains("data_inicial=") {
                Ok("<div id='divAguardandoAreaTabela'>Nenhum registro encontrado.</div>".into())
            } else if url.contains("data_atual=01%2F09%2F2026") {
                Ok(calendario(2026, 9, false))
            } else {
                Ok(format!(
                    "{}<a href='{}'>dia do mes anterior</a>",
                    calendario(2026, 10, true),
                    href("data_atual=01%2F09%2F2026")
                ))
            }
        },
    )
    .unwrap();
    assert_eq!(resultado.mes, "2026-09");
    assert_eq!(resultado.requisicoes, 4);
    assert_eq!(resultado.itens.len(), 0);
    assert_eq!(chamados.len(), 3);
    assert!(chamados[1].contains("data_atual=01%2F09%2F2026"));
    assert!(chamados[2].contains("data_inicial=01%2F09%2F2026"));
}
#[test]
fn mes_anterior_sem_link_publicado_falha_fechado_sem_get_extra() {
    let mut chamados = 0;
    let resultado = ler_com(
        CONTROLE,
        &controle(),
        &instalacao(),
        Some((2026, 9)),
        |_| {
            chamados += 1;
            Ok(calendario(2026, 10, true))
        },
    );
    assert!(matches!(
        resultado,
        Err(ErroLeituraRetornos::MesDosRetornosInacessivel)
    ));
    assert_eq!(chamados, 1);
}
#[test]
fn sem_mes_nao_inventa_o_relogio_do_sei() {
    // Caso Todos tests/test_unidade_listar_retornos_010.py @323eb01.
    // Data UTC fixa evita depender do dia de execucao do CI.
    let mut chamados = 0;
    let resultado = ler_com_data_utc(
        CONTROLE,
        &controle(),
        &instalacao(),
        None,
        (2026, 10, 16),
        |url| {
            chamados += 1;
            assert!(!url.contains("data_inicial="));
            Ok(calendario(2026, 10, false))
        },
    );
    assert!(matches!(
        resultado,
        Err(ErroLeituraRetornos::CalendarioDosRetornosDesconhecido)
    ));
    assert_eq!(chamados, 1);
}
#[test]
fn link_em_outro_host_e_recusado_antes_do_get() {
    let controle = format!("<a href='https://outro.test/sei/{}'>Retornos</a>", href(""));
    let mut chamados = 0;
    let resultado = ler_com(CONTROLE, &controle, &instalacao(), Some((2026, 10)), |_| {
        chamados += 1;
        Ok(String::new())
    });
    assert!(matches!(
        resultado,
        Err(ErroLeituraRetornos::LinkDeRetornosForaDaInstalacao)
    ));
    assert_eq!(chamados, 0);
}
#[test]
fn menu_repetido_nao_dispara_get() {
    let controle = format!("{}{}", controle(), controle());
    let mut chamados = 0;
    let resultado = ler_com(CONTROLE, &controle, &instalacao(), Some((2026, 10)), |_| {
        chamados += 1;
        Ok(String::new())
    });
    assert!(matches!(
        resultado,
        Err(ErroLeituraRetornos::LinkDeRetornosAmbiguo)
    ));
    assert_eq!(chamados, 0);
}
#[test]
fn mes_invalido_recusado_antes_de_acesso_a_sessao() {
    for valor in [
        "2026-00", "2026-13", "2026-1", "2026/10", "abcd-10", "0000-10",
    ] {
        assert!(mes_da_cli(valor).is_err(), "{valor}");
    }
    assert!(matches!(mes_da_cli("2026-10"), Ok((2026, 10))));
}

#[test]
fn sem_mes_encontra_dia_hoje_em_mes_anterior_ao_aberto() {
    // Caso Todos tests/test_unidade_listar_retornos_010.py @323eb01:
    // a tela abre no ultimo mes visitado, nao necessariamente no mes de hoje.
    let mut pedidos = Vec::new();
    let lido = ler_com_data_utc(
        CONTROLE,
        &controle(),
        &instalacao(),
        None,
        (2026, 10, 16),
        |url| {
            pedidos.push(url.to_owned());
            if url.contains("data_inicial=") {
                Ok(linha().into())
            } else if url.contains("data_atual=01%2F10%2F2026") {
                Ok(calendario(2026, 10, true))
            } else {
                Ok(format!(
                    "{}<a href='{}'>anterior</a>",
                    calendario(2026, 11, false),
                    href("data_atual=01%2F10%2F2026")
                ))
            }
        },
    )
    .unwrap();
    assert_eq!(lido.mes, "2026-10");
    assert_eq!(lido.itens.len(), 1);
    assert_eq!(lido.requisicoes, 4);
    assert_eq!(pedidos.len(), 3);
    assert!(pedidos[1].contains("data_atual=01%2F10%2F2026"));
    assert!(pedidos[2].contains("data_inicial=01%2F10%2F2026"));
}

#[test]
fn sem_mes_na_virada_utc_nao_converte_hoje_da_maquina_em_hoje_do_sei() {
    // Caso Todos tests/test_unidade_listar_retornos_010.py @323eb01:
    // UTC ja e novembro, mas a celula diaHoje do SEI esta em outubro.
    let mut pedidos = Vec::new();
    let lido = ler_com_data_utc(
        CONTROLE,
        &controle(),
        &instalacao(),
        None,
        (2026, 11, 1),
        |url| {
            pedidos.push(url.to_owned());
            if url.contains("data_inicial=") {
                Ok(linha().into())
            } else if url.contains("data_atual=01%2F10%2F2026") {
                Ok(calendario(2026, 10, true))
            } else {
                Ok(format!(
                    "{}<a href='{}'>anterior</a>",
                    calendario(2026, 11, false),
                    href("data_atual=01%2F10%2F2026")
                ))
            }
        },
    )
    .unwrap();
    assert_eq!(lido.mes, "2026-10");
    assert_eq!(lido.requisicoes, 4);
    assert_eq!(pedidos.len(), 3);
}

#[test]
fn sem_mes_sem_dia_hoje_nos_candidatos_recusa_sem_consultar_lista() {
    // Relogio local e apenas pista de navegacao. Nao publica mes suposto.
    let mut pedidos = Vec::new();
    let resultado = ler_com_data_utc(
        CONTROLE,
        &controle(),
        &instalacao(),
        None,
        (2026, 11, 1),
        |url| {
            pedidos.push(url.to_owned());
            if url.contains("data_atual=01%2F10%2F2026") {
                Ok(calendario(2026, 10, false))
            } else {
                Ok(format!(
                    "{}<a href='{}'>anterior</a>",
                    calendario(2026, 11, false),
                    href("data_atual=01%2F10%2F2026")
                ))
            }
        },
    );
    assert!(matches!(
        resultado,
        Err(ErroLeituraRetornos::CalendarioDosRetornosDesconhecido)
    ));
    assert_eq!(pedidos.len(), 2);
    assert!(!pedidos.iter().any(|u| u.contains("data_inicial=")));
}

#[test]
fn primeiro_dia_duplicado_com_url_identica_nao_inventa_ambiguidade() {
    // O oraculo Todos deduplica por URL assinada, nao por quantidade de ancoras.
    let anterior = href("data_atual=01%2F10%2F2026");
    let mut pedidos = Vec::new();
    let lido = ler_com_data_utc(
        CONTROLE,
        &controle(),
        &instalacao(),
        Some((2026, 10)),
        (2026, 11, 5),
        |url| {
            pedidos.push(url.to_owned());
            if url.contains("data_inicial=") {
                Ok(linha().into())
            } else if url.contains("data_atual=01%2F10%2F2026") {
                Ok(calendario(2026, 10, false))
            } else {
                Ok(format!(
                    "{}<a href='{anterior}'>prev 1</a><a href='{anterior}'>prev 2</a>",
                    calendario(2026, 11, false)
                ))
            }
        },
    )
    .unwrap();
    assert_eq!(lido.mes, "2026-10");
    assert_eq!(lido.requisicoes, 4);
    assert_eq!(pedidos.len(), 3);
}

#[test]
fn calendario_utc_cobre_virada_de_ano_e_bissexto_sem_dependencia_de_fuso() {
    assert_eq!(data_utc_de_dias(0), Some((1970, 1, 1)));
    assert_eq!(data_utc_de_dias(11_016), Some((2000, 2, 29)));
    assert_eq!(data_utc_de_dias(20_757), Some((2026, 10, 31)));
    assert_eq!(data_utc_de_dias(20_758), Some((2026, 11, 1)));
    assert_eq!(data_utc_de_dias(20_762), Some((2026, 11, 5)));
}
#[test]
fn dois_dias_hoje_conflitantes_nao_publicam_mes() {
    let duplo = calendario(2026, 11, true).replace(
        "</html>",
        &format!(
            "<td class='diaHoje'><a href='{}'>outro dia</a></td></html>",
            href("data_atual=16%2F11%2F2026")
        ),
    );
    let mut pedidos = 0;
    let erro = ler_com_data_utc(
        CONTROLE,
        &controle(),
        &instalacao(),
        None,
        (2026, 11, 5),
        |_| {
            pedidos += 1;
            Ok(duplo.clone())
        },
    );
    assert!(matches!(
        erro,
        Err(ErroLeituraRetornos::CalendarioDosRetornosDesconhecido)
    ));
    assert_eq!(pedidos, 1);
}
#[test]
fn primeiro_dia_duplicado_com_url_divergente_falha_antes_do_get() {
    let primeiro = href("data_atual=01%2F10%2F2026");
    let outro = href("data_atual=01%2F10%2F2026&pagina=2");
    let tela = format!(
        "{}<a href='{primeiro}'>prev 1</a><a href='{outro}'>prev 2</a>",
        calendario(2026, 11, false)
    );
    let mut pedidos = 0;
    let erro = ler_com_data_utc(
        CONTROLE,
        &controle(),
        &instalacao(),
        Some((2026, 10)),
        (2026, 11, 5),
        |_| {
            pedidos += 1;
            Ok(tela.clone())
        },
    );
    assert!(matches!(
        erro,
        Err(ErroLeituraRetornos::CalendarioDosRetornosDesconhecido)
    ));
    assert_eq!(pedidos, 1);
}

#[path = "tests/saltos.rs"]
mod saltos;
