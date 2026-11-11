//! RED/GREEN da faixa observada: a página que a tela mostra, sem POST.
use super::*;

fn linha(id: u32) -> String {
    format!("<tr id='P{id}'><td><a class='processoVisualizado'>NUP-{id}</a></td></tr>")
}

fn tabela(nome: &str, legenda: &str, ids: &[u32], extra: &str) -> String {
    format!(
        "{extra}<table id='tblProcessos{nome}'><caption>{legenda}</caption>{}</table>",
        ids.iter().map(|id| linha(*id)).collect::<String>()
    )
}

fn pagina(recebidos: &str, gerados: &str) -> String {
    format!(
        "<a id='lnkInfraUnidade'>TESTE</a>\
         <input id='hdnTipoVisualizacao' value='R'>\
         <input id='hdnRecebidosHashCriterios' value='aaaa'>\
         <input id='hdnGeradosHashCriterios' value='bbbb'>\
         <input type='hidden' id='hdnRecebidosPaginaAtual' value='0'>\
         <input type='hidden' id='hdnGeradosPaginaAtual' value='0'>{recebidos}{gerados}"
    )
}

fn vazia(nome: &str) -> String {
    tabela(nome, "0 registros:", &[], "")
}

#[test]
fn pagina_parcial_publica_a_faixa_da_legenda() {
    let rec = tabela(
        "Recebidos",
        "Processos recebidos (4 registros - 1 a 2):",
        &[1, 2],
        "<a id='lnkRecebidosProximaPaginaSuperior'>»</a>",
    );
    let leitura = juntar(&[pagina(&rec, &vazia("Gerados"))], Some("TESTE")).unwrap();
    assert_eq!(
        leitura.recebidos.faixa_observada(),
        Ok(FaixaObservada::Lidas { inicio: 1, fim: 2 })
    );
    assert!(!leitura.recebidos.completa(&leitura.caixa.recebidos));
    assert_eq!(
        leitura.gerados.faixa_observada(),
        Ok(FaixaObservada::SemLinhas)
    );
}

#[test]
fn legenda_sem_intervalo_com_todas_as_linhas_publica_um_ate_total() {
    let rec = tabela("Recebidos", "3 registros:", &[1, 2, 3], "");
    let leitura = juntar(&[pagina(&rec, &vazia("Gerados"))], None).unwrap();
    assert_eq!(
        leitura.recebidos.faixa_observada(),
        Ok(FaixaObservada::Lidas { inicio: 1, fim: 3 })
    );
}

#[test]
fn pagina_lembrada_pelo_servidor_publica_faixa_fora_do_inicio() {
    let rec = tabela(
        "Recebidos",
        "Processos recebidos (4 registros - 3 a 4):",
        &[3, 4],
        "",
    );
    let leitura = juntar(&[pagina(&rec, &vazia("Gerados"))], Some("TESTE")).unwrap();
    assert_eq!(
        leitura.recebidos.faixa_observada(),
        Ok(FaixaObservada::Lidas { inicio: 3, fim: 4 })
    );
    assert!(!leitura.recebidos.completa(&leitura.caixa.recebidos));
}

#[test]
fn paginas_contiguas_fora_do_inicio_somam_faixas() {
    let p2 = tabela("Recebidos", "(6 registros - 3 a 4):", &[3, 4], "");
    let p3 = tabela("Recebidos", "(6 registros - 5 a 6):", &[5, 6], "");
    let paginas = [
        pagina(&p2, &vazia("Gerados")),
        pagina(&p3, &vazia("Gerados")),
    ];
    let leitura = juntar(&paginas, Some("TESTE")).unwrap();
    assert_eq!(
        leitura.recebidos.faixa_observada(),
        Ok(FaixaObservada::Lidas { inicio: 3, fim: 6 })
    );
}

#[test]
fn paginas_com_lacuna_continuam_recusadas() {
    let p1 = tabela("Recebidos", "(6 registros - 1 a 2):", &[1, 2], "");
    let p3 = tabela("Recebidos", "(6 registros - 5 a 6):", &[5, 6], "");
    let paginas = [
        pagina(&p1, &vazia("Gerados")),
        pagina(&p3, &vazia("Gerados")),
    ];
    assert_eq!(
        juntar(&paginas, Some("TESTE")).err(),
        Some(ErroCaixa::FaixaDescontinua {
            categoria: Categoria::Recebidos
        })
    );
}

#[test]
fn paginador_da_tabela_com_legenda_sem_intervalo_e_recusado() {
    for (controle, categoria, nome) in [
        (
            "<a id='lnkRecebidosProximaPaginaInferior'>»</a>",
            Categoria::Recebidos,
            "Recebidos",
        ),
        (
            "<select id='selGeradosPaginacaoSuperior'><option value='0'>1</option></select>",
            Categoria::Gerados,
            "Gerados",
        ),
    ] {
        let alvo = tabela(nome, "2 registros:", &[1, 2], controle);
        let html = if nome == "Recebidos" {
            pagina(&alvo, &vazia("Gerados"))
        } else {
            pagina(&vazia("Recebidos"), &alvo)
        };
        assert_eq!(
            juntar(&[html], Some("TESTE")).err(),
            Some(ErroCaixa::PaginadorDaTabelaSemFaixaNaLegenda { categoria }),
            "{nome}"
        );
    }
}

#[test]
fn faixa_publicavel_exige_total_e_intervalo_ou_todas_as_linhas() {
    let casos = [
        (
            tabela("Recebidos", "Processos recebidos", &[1], ""),
            ErroCaixa::TotalNaoDeclaradoNaLegenda {
                categoria: Categoria::Recebidos,
            },
        ),
        (
            tabela("Recebidos", "5 registros:", &[1, 2], ""),
            ErroCaixa::LegendaSemFaixaComTotalDiferenteDasLinhas {
                categoria: Categoria::Recebidos,
            },
        ),
    ];
    for (rec, esperado) in casos {
        // Legenda sem total falha já na leitura; as demais na publicação.
        let resultado = juntar(&[pagina(&rec, &vazia("Gerados"))], None).and_then(|leitura| {
            conferir_faixa_publicavel(Categoria::Recebidos, &leitura.recebidos)
        });
        assert_eq!(resultado, Err(esperado));
    }
}

#[test]
fn faixa_publicavel_aceita_parcial_vazia_e_integral() {
    let rec = tabela("Recebidos", "(4 registros - 1 a 2):", &[1, 2], "");
    let leitura = juntar(&[pagina(&rec, &vazia("Gerados"))], None).unwrap();
    for (categoria, cobertura) in [
        (Categoria::Recebidos, &leitura.recebidos),
        (Categoria::Gerados, &leitura.gerados),
    ] {
        assert_eq!(conferir_faixa_publicavel(categoria, cobertura), Ok(()));
    }
}

#[test]
fn faixa_na_pagina_le_total_e_intervalo_de_uma_tabela() {
    let rec = tabela("Recebidos", "(114 registros - 101 a 102):", &[1, 2], "");
    let html = pagina(&rec, &tabela("Gerados", "2 registros:", &[7, 8], ""));
    assert_eq!(
        faixa_na_pagina(&html, Categoria::Recebidos),
        Ok(Some(FaixaNaPagina {
            total: 114,
            inicio: 101,
            fim: 102
        }))
    );
    assert_eq!(
        faixa_na_pagina(&html, Categoria::Gerados),
        Ok(Some(FaixaNaPagina {
            total: 2,
            inicio: 1,
            fim: 2
        }))
    );
    let zero = pagina(&vazia("Recebidos"), &vazia("Gerados"));
    assert_eq!(faixa_na_pagina(&zero, Categoria::Recebidos), Ok(None));
}

#[test]
fn legenda_no_singular_comprova_a_faixa_de_um_registro() {
    // UFRJ, 04/11/2026: `Processos recebidos (1 registro):`, uma linha.
    let rec = tabela("Recebidos", "Processos recebidos (1 registro):", &[1], "");
    let leitura = juntar(&[pagina(&rec, &vazia("Gerados"))], None).unwrap();
    assert_eq!(
        leitura.recebidos.faixa_observada(),
        Ok(FaixaObservada::Lidas { inicio: 1, fim: 1 })
    );
    assert_eq!(
        conferir_faixa_publicavel(Categoria::Recebidos, &leitura.recebidos),
        Ok(())
    );
    let parcial = tabela("Recebidos", "(1 registro - 1 a 1):", &[1], "");
    let html = pagina(&parcial, &vazia("Gerados"));
    assert_eq!(
        faixa_na_pagina(&html, Categoria::Recebidos),
        Ok(Some(FaixaNaPagina {
            total: 1,
            inicio: 1,
            fim: 1
        }))
    );
}
