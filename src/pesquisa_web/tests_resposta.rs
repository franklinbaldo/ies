//! RED/GREEN da resposta 200 da pesquisa: barra com separador de milhar,
//! lista, "sem resultado" e formas não reconhecidas. HTML sintético.
use super::*;
use crate::pesquisa_web::tests_fixtures::{
    conteudo_da_lista, formulario, numero, resultado, resultado_numa_pagina, sem_resultado,
};
use crate::tela_html::HtmlServido;

#[test]
fn contagem_aceita_ponto_so_entre_grupos_de_tres_digitos() {
    assert_eq!(ler_contagem("1"), Some(1));
    assert_eq!(ler_contagem("10"), Some(10));
    assert_eq!(ler_contagem("999"), Some(999));
    assert_eq!(ler_contagem("1.234"), Some(1234));
    assert_eq!(ler_contagem("4.567"), Some(4567));
    assert_eq!(ler_contagem("12.345.678"), Some(12_345_678));
    for fora in [
        "", "1234", "1,234", "1.23", "1.2345", "01.234", "012", ".234", "1.", "1..234", "1 234",
        "+1", "-1", "1.234,5", "١٢", "1.ab4",
    ] {
        assert_eq!(ler_contagem(fora), None, "{fora:?}");
    }
}

#[test]
fn contagem_enorme_nao_estoura() {
    assert_eq!(ler_contagem("18.446.744.073.709.551.615"), Some(u64::MAX));
    assert_eq!(ler_contagem("18.446.744.073.709.551.616"), None);
}

#[test]
fn lista_publica_total_faixa_e_numeros_com_linha_da_arvore() {
    let RespostaDaPesquisa::Lista(lista) =
        ler_resposta_html(&HtmlServido::de_teste(&resultado("00001", 1, 10, "1.234"))).unwrap()
    else {
        panic!("esperada lista");
    };
    assert_eq!(lista.total_declarado, 1234);
    assert_eq!(lista.faixa, FaixaDaPesquisa { inicio: 1, fim: 10 });
    assert_eq!(lista.processos.len(), 10);
    assert_eq!(lista.processos[0].numero.as_str(), numero(1));
    assert_eq!(lista.processos[9].numero.as_str(), numero(10));
}

#[test]
fn pagina_posterior_e_lida_com_a_faixa_da_barra() {
    let RespostaDaPesquisa::Lista(lista) = ler_resposta_html(&HtmlServido::de_teste(&resultado(
        "00001", 1231, 1234, "1.234",
    )))
    .unwrap() else {
        panic!("esperada lista");
    };
    assert_eq!(
        lista.faixa,
        FaixaDaPesquisa {
            inicio: 1231,
            fim: 1234
        }
    );
    assert_eq!(lista.processos.len(), 4);
}

#[test]
fn aviso_de_sem_resultado_e_resposta_observada() {
    assert_eq!(
        ler_resposta_html(&HtmlServido::de_teste(&sem_resultado("99999"))),
        Ok(RespostaDaPesquisa::SemResultado)
    );
}

#[test]
fn pagina_sem_tabela_nem_aviso_nao_e_reconhecida() {
    let html = HtmlServido::de_teste(&format!(
        "<html>{}</html>",
        formulario("x", true, "<p>outra</p>")
    ));
    assert_eq!(
        ler_resposta_html(&html),
        Err(ErroRespostaDaPesquisa::RespostaDaPesquisaNaoReconhecida)
    );
}

#[test]
fn tabela_e_aviso_juntos_falham_fechado() {
    let html = HtmlServido::de_teste(&resultado("1", 1, 3, "3").replace(
        "</table>",
        "</table><div class=\"pesquisaSemResultado\">Nenhum</div>",
    ));
    assert_eq!(
        ler_resposta_html(&html),
        Err(ErroRespostaDaPesquisa::TabelaDeResultadoEAvisoDeSemResultadoNaMesmaResposta)
    );
}

#[test]
fn barra_ausente_ou_repetida_e_recusada() {
    let sem = HtmlServido::de_teste(
        &resultado("1", 1, 3, "3").replace("pesquisaBarraD", "pesquisaBarraX"),
    );
    assert_eq!(
        ler_resposta_html(&sem),
        Err(ErroRespostaDaPesquisa::BarraDeContagemDaPesquisaAusente)
    );
    let dupla = HtmlServido::de_teste(&resultado("1", 1, 3, "3").replace(
        "<table border",
        "<div class=\"pesquisaBarraD\">Exibindo 1 - 3 de 3</div><table border",
    ));
    assert_eq!(
        ler_resposta_html(&dupla),
        Err(ErroRespostaDaPesquisa::BarrasDeContagemDaPesquisaRepetidas)
    );
}

#[test]
fn texto_da_barra_fora_da_forma_observada_e_recusado() {
    for texto in [
        "Mostrando 1 - 3 de 3",
        "Exibindo 1 a 3 de 3",
        "Exibindo 1 - 3 / 3",
        "Exibindo 1 - 3 de 3 itens",
        "Exibindo 1-3 de 3",
    ] {
        let html =
            HtmlServido::de_teste(&resultado("1", 1, 3, "3").replace("Exibindo 1 - 3 de 3", texto));
        assert_eq!(
            ler_resposta_html(&html),
            Err(ErroRespostaDaPesquisa::TextoDaBarraDeContagemForaDoFormato),
            "{texto}"
        );
    }
}

#[test]
fn numero_da_barra_sem_separador_observado_e_recusado() {
    for total in ["1234", "1,234", "1.23"] {
        let html = HtmlServido::de_teste(
            &resultado("1", 1, 10, "1.234").replace("de 1.234", &format!("de {total}")),
        );
        assert_eq!(
            ler_resposta_html(&html),
            Err(ErroRespostaDaPesquisa::NumeroDaBarraDeContagemForaDoFormato),
            "{total}"
        );
    }
}

#[test]
fn faixa_impossivel_e_recusada() {
    for (inicio, fim, total) in [("0", "3", "3"), ("3", "1", "3"), ("1", "3", "2")] {
        let html = HtmlServido::de_teste(&resultado("1", 1, 3, "3").replace(
            "Exibindo 1 - 3 de 3",
            &format!("Exibindo {inicio} - {fim} de {total}"),
        ));
        assert_eq!(
            ler_resposta_html(&html),
            Err(ErroRespostaDaPesquisa::FaixaDaBarraDeContagemImpossivel),
            "{inicio}-{fim}/{total}"
        );
    }
}

/// Goiás 5.0.3 (`processo-busca-go`, passo 4): "Exibindo 1 - 10 de 966" com
/// 9 registros no HTML (27 `tr`, 3 por registro); a linha falta na página.
#[test]
fn menos_linhas_que_a_faixa_declarada_publicam_as_ausentes() {
    let html = HtmlServido::de_teste(
        &format!(
            "<html>{}</html>",
            formulario("1", true, &conteudo_da_lista(1, 9, "966", 10))
        )
        .replace("Exibindo 1 - 9 de", "Exibindo 1 - 10 de"),
    );
    let RespostaDaPesquisa::Lista(lista) = ler_resposta_html(&html).unwrap() else {
        panic!("esperada lista");
    };
    assert_eq!(lista.processos.len(), 9);
    assert_eq!(lista.total_declarado, 966);
    assert_eq!(lista.faixa, FaixaDaPesquisa { inicio: 1, fim: 10 });
    assert_eq!(lista.linhas_ausentes_da_faixa, 1);
}

/// Goiás 5.0.3 (`pesquisa-paginas-go`, passo 8): barra "Exibindo 1 - 10 de
/// 3.300" e `table.pesquisaResultado` vazia.
#[test]
fn tabela_vazia_com_barra_publica_a_faixa_inteira_ausente() {
    let html = resultado("1", 1, 10, "3.300");
    let inicio = html.find("class=\"pesquisaResultado\">").unwrap() + 26;
    let fim = html[inicio..].find("</table>").unwrap() + inicio;
    let vazia = HtmlServido::de_teste(&format!("{} {}", &html[..inicio], &html[fim..]));
    let RespostaDaPesquisa::Lista(lista) = ler_resposta_html(&vazia).unwrap() else {
        panic!("esperada lista");
    };
    assert_eq!(lista.processos.len(), 0);
    assert_eq!(lista.total_declarado, 3300);
    assert_eq!(lista.linhas_ausentes_da_faixa, 10);
}

#[test]
fn mais_linhas_que_a_faixa_sao_recusadas() {
    let html = HtmlServido::de_teste(
        &format!(
            "<html>{}</html>",
            formulario("1", true, &conteudo_da_lista(1, 11, "1.234", 10))
        )
        .replace("Exibindo 1 - 11 de", "Exibindo 1 - 10 de"),
    );
    assert_eq!(
        ler_resposta_html(&html),
        Err(ErroRespostaDaPesquisa::LinhasDaTabelaDiferentesDaFaixaDaBarra)
    );
}

#[test]
fn lista_completa_nao_tem_linhas_ausentes() {
    let RespostaDaPesquisa::Lista(lista) =
        ler_resposta_html(&HtmlServido::de_teste(&resultado("00001", 1, 10, "1.234"))).unwrap()
    else {
        panic!("esperada lista");
    };
    assert_eq!(lista.linhas_ausentes_da_faixa, 0);
}

#[test]
fn linha_invalida_da_tabela_e_erro_de_resultado() {
    let html = HtmlServido::de_teste(
        &resultado("1", 1, 3, "3")
            .replace("acao=procedimento_trabalhar", "acao=documento_imprimir_web"),
    );
    assert_eq!(
        ler_resposta_html(&html),
        Err(ErroRespostaDaPesquisa::Resultado(
            ErroResultadoPesquisa::AcaoDoLinkNaoLida(
                crate::consulta_do_controlador::ErroChaveDaConsulta::AcaoDesconhecida
            )
        ))
    );
}

#[test]
fn inicio_e_fim_acima_de_999_tambem_exigem_o_separador() {
    // Só o total acima de 999 foi observado (quatro dígitos); posições sem o
    // separador falham fechado em vez de virar outra leitura.
    let html = HtmlServido::de_teste(
        &resultado("1", 1231, 1234, "1.234").replace("1.231 - 1.234", "1231 - 1234"),
    );
    assert_eq!(
        ler_resposta_html(&html),
        Err(ErroRespostaDaPesquisa::NumeroDaBarraDeContagemForaDoFormato)
    );
}

#[test]
fn aviso_de_sem_resultado_com_barra_de_contagem_e_ambiguo() {
    let html = sem_resultado("99999").replace(
        "<div class=\"pesquisaSemResultado\">",
        "<div class=\"pesquisaBarraD\">Exibindo 1 - 3 de 3</div><div class=\"pesquisaSemResultado\">",
    );
    assert_ne!(
        html,
        sem_resultado("99999"),
        "fixture sem o marcador esperado"
    );
    assert_eq!(
        ler_resposta_html(&HtmlServido::de_teste(&html)),
        Err(ErroRespostaDaPesquisa::AvisoDeSemResultadoEBarraDeContagemNaMesmaResposta)
    );
}

#[test]
fn lista_numa_pagina_le_n_resultados_como_faixa_inteira() {
    for n in [2, 9] {
        let RespostaDaPesquisa::Lista(lista) = ler_resposta_html(&HtmlServido::de_teste(
            &resultado_numa_pagina("00001", n, &format!("{n} resultados")),
        ))
        .unwrap() else {
            panic!("esperada lista");
        };
        assert_eq!(lista.total_declarado, n);
        assert_eq!(lista.faixa, FaixaDaPesquisa { inicio: 1, fim: n });
        assert_eq!(lista.processos.len() as u64, n);
    }
}

#[test]
fn barra_numa_pagina_fora_da_forma_observada_e_recusada() {
    for barra in [
        "1 resultado",
        "2 resultado",
        "resultados",
        "2 resultados encontrados",
        "2 Resultados",
    ] {
        assert_eq!(
            ler_resposta_html(&HtmlServido::de_teste(&resultado_numa_pagina(
                "00001", 2, barra
            ))),
            Err(ErroRespostaDaPesquisa::TextoDaBarraDeContagemForaDoFormato),
            "{barra:?}"
        );
    }
    assert_eq!(
        ler_resposta_html(&HtmlServido::de_teste(&resultado_numa_pagina(
            "00001",
            2,
            "02 resultados"
        ))),
        Err(ErroRespostaDaPesquisa::NumeroDaBarraDeContagemForaDoFormato)
    );
    assert_eq!(
        ler_resposta_html(&HtmlServido::de_teste(&resultado_numa_pagina(
            "00001",
            2,
            "0 resultados"
        ))),
        Err(ErroRespostaDaPesquisa::FaixaDaBarraDeContagemImpossivel)
    );
    // "N resultados" não escreve a faixa: a contagem tem de bater.
    assert_eq!(
        ler_resposta_html(&HtmlServido::de_teste(&resultado_numa_pagina(
            "00001",
            2,
            "3 resultados"
        ))),
        Err(ErroRespostaDaPesquisa::LinhasDaTabelaDiferentesDaFaixaDaBarra)
    );
}
