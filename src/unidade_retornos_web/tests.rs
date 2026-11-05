use super::*;

const ORIGEM: &str = "https://sei.treinamento.unb.br/sei";
const CONTROLE: &str = "https://sei.treinamento.unb.br/sei/controlador.php?acao=procedimento_controlar";

fn instalacao() -> Instalacao {
    Instalacao::interpretar(ORIGEM).unwrap()
}
fn href(complemento: &str) -> String {
    format!("controlador.php?acao=retorno_programado_listar&infra_hash={}&{complemento}", "a".repeat(64))
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
        href(&format!("data_inicial=01%2F{mes:02}%2F{ano:04}&data_final={fim:02}%2F{mes:02}%2F{ano:04}")),
        href(&format!("data_atual={data_hoje}")),
        href(&format!("data_atual=01%2F{m_seguinte:02}%2F{a_seguinte:04}")),
    )
}
fn linha() -> &'static str {
    "<table summary='Aguardando Retorno de Outras Unidades'>\
     <caption>1 registro:</caption>\
     <tr><th>Processo</th><th>Unidade</th><th>Data Programada</th></tr>\
     <tr><td>001/2026</td><td>Unidade 1</td><td>15/10/2026</td></tr></table>"
}
fn controle() -> String {
    format!("<a href='{}'>Retorno Programado</a>", href(""))
}
#[test]
fn mes_da_tela_custa_dois_get_apos_o_controle() {
    // Caso Todos tests/test_unidade_listar_retornos_010.py @323eb01.
    let mut chamados = Vec::new();
    let resultado = ler_com(CONTROLE, &controle(), &instalacao(), Some((2026,10)), |url| {
        chamados.push(url.to_owned());
        if url.contains("data_inicial=") {
            Ok(linha().to_owned())
        } else {
            Ok(calendario(2026,10,true))
        }
    }).unwrap();
    assert_eq!(resultado.mes,"2026-10");
    assert_eq!(resultado.requisicoes,3);
    assert_eq!(resultado.itens.len(),1);
    assert_eq!(resultado.itens[0].numero,"001/2026");
    assert_eq!(chamados.len(),2);
    assert!(chamados[1].contains("data_inicial=01%2F10%2F2026"));
}
#[test]
fn proximo_mes_usa_um_get_adicional_sem_repetir_pagina() {
    // Caso Todos tests/test_unidade_listar_retornos_010.py @323eb01.
    let mut chamados = Vec::new();
    let resultado = ler_com(CONTROLE, &controle(), &instalacao(), Some((2026,11)), |url| {
        chamados.push(url.to_owned());
        if url.contains("data_inicial=") {
            Ok("<div id='divAguardandoAreaTabela'>Nenhum registro encontrado.</div>".into())
        } else if url.contains("data_atual=01%2F11%2F2026") {
            Ok(calendario(2026,11,false))
        } else {
            Ok(calendario(2026,10,true))
        }
    }).unwrap();
    assert_eq!(resultado.mes,"2026-11");
    assert_eq!(resultado.requisicoes,4);
    assert_eq!(resultado.itens.len(),0);
    assert_eq!(chamados.len(),3);
    assert!(chamados[1].contains("data_atual=01%2F11%2F2026"));
    assert!(chamados[2].contains("data_inicial=01%2F11%2F2026"));
}
#[test]
fn sem_mes_nao_inventa_o_relogio_do_sei() {
    // Caso Todos tests/test_unidade_listar_retornos_010.py @323eb01.
    let resultado = ler_com(CONTROLE, &controle(), &instalacao(), None, |_| {
        Ok(calendario(2026,10,false))
    });
    assert!(matches!(resultado,Err(ErroLeituraRetornos::CalendarioDosRetornosDesconhecido)));
}
#[test]
fn link_em_outro_host_e_recusado_antes_do_get() {
    let controle = format!("<a href='https://outro.test/sei/{}'>Retornos</a>", href(""));
    let mut chamados = 0;
    let resultado = ler_com(CONTROLE, &controle, &instalacao(),Some((2026,10)), |_| {
        chamados += 1;
        Ok(String::new())
    });
    assert!(matches!(resultado,Err(ErroLeituraRetornos::LinkDeRetornosForaDaInstalacao)));
    assert_eq!(chamados,0);
}
#[test]
fn menu_repetido_nao_dispara_get() {
    let controle = format!("{}{}",controle(),controle());
    let mut chamados = 0;
    let resultado = ler_com(CONTROLE,&controle,&instalacao(),Some((2026,10)), |_| {
        chamados += 1;
        Ok(String::new())
    });
    assert!(matches!(resultado,Err(ErroLeituraRetornos::LinkDeRetornosAmbiguo)));
    assert_eq!(chamados,0);
}
#[test]
fn mes_invalido_recusado_antes_de_acesso_a_sessao() {
    for valor in ["2026-00","2026-13","2026-1","2026/10","abcd-10","0000-10"] {
        assert!(mes_pedido(Some(valor)).is_err(), "{valor}");
    }
    assert!(matches!(mes_pedido(Some("2026-10")),Ok(Some((2026,10)))));
    assert!(matches!(mes_pedido(None),Ok(None)));
}
