use super::*;
use crate::paginas::FaixaObservada;
use crate::tela_html::HtmlServido;

fn pagina(total: usize, inicio: usize, ids: &[(u32, &str)]) -> String {
    let fim = inicio + ids.len() - 1;
    let mut linhas = String::new();
    for (id, nup) in ids {
        use std::fmt::Write as _;
        write!(
            &mut linhas,
            "<tr id='P{id}'><td>linha</td><td><a class='processoVisualizado' \
             href='controlador.php?acao=procedimento_trabalhar&amp;id_procedimento={id}'>\
             {nup}</a></td><td>Ofício</td></tr>"
        )
        .unwrap();
    }
    format!(
        "<html><body><a id='lnkInfraUnidade'>PPI</a>\
         <input id='hdnTipoVisualizacao' value='D'>\
         <input id='hdnRecebidosHashCriterios' value='r'>\
         <input id='hdnGeradosHashCriterios' value='g'>\
         <table id='tblProcessosDetalhado'><caption>({total} registros - {inicio} a {fim}):</caption>\
         <tr><th></th><th>Processo</th><th>Tipo</th></tr>{linhas}</table></body></html>"
    )
}

#[test]
fn pagina_detalhada_completa_de_um_get_vira_lista_sem_categorias_inventadas() {
    let html = pagina(2, 1, &[(1, "000001/2026"), (2, "000002/2026")]);
    assert!(possui_tabela(&html));
    let resposta = ler_paginas(&[HtmlServido::de_teste(&html)], Some("PPI")).unwrap();
    assert_eq!(resposta.unidade, "PPI");
    assert_eq!(resposta.total_declarado, 2);
    assert_eq!(resposta.processos.len(), 2);
    assert_eq!(resposta.processos[0].id.as_str(), "1");
    assert_eq!(resposta.processos[0].numero, "000001/2026");
    assert!(resposta.processos[0].visualizado);
    assert_eq!(resposta.processos[1].numero, "000002/2026");
}

#[test]
fn booleano_antes_de_id_href_e_class_nao_altera_a_leitura() {
    let original = pagina(2, 1, &[(1, "000001/2026"), (2, "000002/2026")]);
    let html = crate::html_atributos::com_booleano_antes_dos_atributos(&original);
    assert!(possui_tabela(&html));
    let resposta = ler_paginas(&[HtmlServido::de_teste(&html)], Some("PPI")).unwrap();
    assert_eq!(resposta.processos.len(), 2);
    assert!(resposta.processos[0].visualizado);
    assert_eq!(resposta.processos[1].numero, "000002/2026");
}

#[test]
fn duas_paginas_detalhadas_capturadas_unem_intervalos_sem_post() {
    let primeira = HtmlServido::de_teste(&pagina(3, 1, &[(1, "000001/2026"), (2, "000002/2026")]));
    let segunda = HtmlServido::de_teste(&pagina(3, 3, &[(3, "000003/2026")]));
    let leitura = ler_paginas(&[primeira.clone(), segunda], Some("PPI")).unwrap();
    assert_eq!(leitura.total_declarado, 3);
    assert_eq!(leitura.processos.len(), 3);
}

#[test]
fn pagina_parcial_publica_faixa_observada_sem_afirmar_total_lido() {
    let pagina = HtmlServido::de_teste(&pagina(3, 1, &[(1, "000001/2026"), (2, "000002/2026")]));
    let leitura = ler_paginas(&[pagina], Some("PPI")).unwrap();
    assert_eq!(leitura.total_declarado, 3);
    assert_eq!(
        leitura.faixa_observada,
        FaixaObservada::Lidas { inicio: 1, fim: 2 }
    );
    assert_eq!(leitura.processos.len(), 2);
}

#[test]
fn pagina_lembrada_fora_do_inicio_publica_sua_faixa() {
    let pagina = HtmlServido::de_teste(&pagina(3, 3, &[(3, "000003/2026")]));
    let leitura = ler_paginas(&[pagina], Some("PPI")).unwrap();
    assert_eq!(
        leitura.faixa_observada,
        FaixaObservada::Lidas { inicio: 3, fim: 3 }
    );
}

#[test]
fn nao_aceita_faixas_omitidas_ou_sobrepostas() {
    let primeira = HtmlServido::de_teste(&pagina(4, 1, &[(1, "000001/2026"), (2, "000002/2026")]));
    let outra = HtmlServido::de_teste(&pagina(4, 4, &[(4, "000004/2026")]));
    assert_eq!(
        ler_paginas(&[primeira.clone(), outra], Some("PPI")),
        Err(ErroDetalhada::FaixaDescontinua)
    );
    let outra = HtmlServido::de_teste(&pagina(4, 2, &[(2, "000002/2026"), (3, "000003/2026")]));
    assert_eq!(
        ler_paginas(&[primeira.clone(), outra], Some("PPI")),
        Err(ErroDetalhada::FaixaDescontinua)
    );
}

#[test]
fn troca_de_unidade_e_duplicidade_sao_erros_tipados() {
    let primeira = HtmlServido::de_teste(&pagina(3, 1, &[(1, "000001/2026"), (2, "000002/2026")]));
    let segunda = pagina(3, 3, &[(3, "000003/2026")]);
    let outra = HtmlServido::de_teste(&segunda.replace(">PPI</a>", ">OUTRA</a>"));
    assert_eq!(
        ler_paginas(&[primeira.clone(), outra], Some("PPI")),
        Err(ErroDetalhada::ContextoDiferente)
    );
    let repetida = HtmlServido::de_teste(
        &segunda
            .replace("id='P3'", "id='P1'")
            .replace("id_procedimento=3", "id_procedimento=1"),
    );
    assert_eq!(
        ler_paginas(&[primeira.clone(), repetida], Some("PPI")),
        Err(ErroDetalhada::IdentificadorDetalhadoRepetido)
    );
}

#[test]
fn nao_cria_processo_de_celula_sem_link_reconhecido() {
    let original = pagina(1, 1, &[(1, "000001/2026")]);
    let sem_link =
        HtmlServido::de_teste(&original.replace("acao=procedimento_trabalhar", "acao=outra_acao"));
    assert_eq!(
        ler_paginas(&[sem_link], Some("PPI")),
        Err(ErroDetalhada::LinkDoProcessoDetalhadoAusente)
    );
}

#[test]
fn nao_atribui_detalhada_a_visao_resumida() {
    let original = pagina(1, 1, &[(1, "000001/2026")]);
    let outra = HtmlServido::de_teste(&original.replace("value='D'", "value='R'"));
    assert_eq!(
        ler_paginas(&[outra], Some("PPI")),
        Err(ErroDetalhada::VisualizacaoIncorreta)
    );
}

#[test]
fn paginacao_detalhada_sem_intervalo_explicito_nao_prova_cobertura() {
    let original = pagina(1, 1, &[(1, "000001/2026")]);
    let sem_faixa = original.replace("registros - 1 a 1", "registros");
    for marcador in [
        "<input id='hdnDetalhadoPaginaAtual' value='2'>",
        "<select id='selInfraPaginacaoDetalhado'><option>2</option></select>",
    ] {
        let html =
            HtmlServido::de_teste(&sem_faixa.replace("</body>", &format!("{marcador}</body>")));
        assert_eq!(
            ler_paginas(&[html], Some("PPI")),
            Err(ErroDetalhada::FaixaSemIntervaloComPaginacao)
        );
    }
    assert_eq!(
        ler_paginas(&[HtmlServido::de_teste(&sem_faixa)], Some("PPI"))
            .unwrap()
            .total_declarado,
        1
    );
}

/// UFRJ 4.1.5 (04/11/2026): D de uma página publica o índice oculto
/// `hdnDetalhadoPaginaAtual=0`, sem paginador e sem faixa na legenda.
#[test]
fn indice_oculto_zero_sem_paginador_nao_e_paginacao() {
    let original = pagina(1, 1, &[(1, "000001/2026")]);
    let sem_faixa = original.replace("registros - 1 a 1", "registros");
    let zero = sem_faixa.replace(
        "</body>",
        "<input type='hidden' id='hdnDetalhadoPaginaAtual' value='0'></body>",
    );
    let lida = ler_paginas(
        std::slice::from_ref(&HtmlServido::de_teste(&zero)),
        Some("PPI"),
    )
    .unwrap();
    assert_eq!(lida.total_declarado, 1);
    for paginador in [
        "<a id='lnkDetalhadoProximaPaginaSuperior'>»</a>",
        "<select id='selDetalhadoPaginacaoSuperior'><option>1</option></select>",
    ] {
        let html = HtmlServido::de_teste(&zero.replace("</body>", &format!("{paginador}</body>")));
        assert_eq!(
            ler_paginas(&[html], Some("PPI")),
            Err(ErroDetalhada::FaixaSemIntervaloComPaginacao),
            "{paginador}"
        );
    }
}

fn com_classe(classe: &str) -> Result<LeituraDetalhada, ErroDetalhada> {
    let original = pagina(1, 1, &[(1, "001234500000015")]);
    ler_paginas(
        &[HtmlServido::de_teste(&original.replace(
            "class='processoVisualizado'",
            &format!("class='{classe}'"),
        ))],
        Some("PPI"),
    )
}

#[test]
fn detalhada_le_sigilo_da_classe_e_preserva_numero_opaco() {
    for (classe, visualizado, sigiloso) in [
        ("processoVisualizado", true, false),
        ("processoNaoVisualizado", false, false),
        ("processoVisualizadoSigiloso", true, true),
        ("processoNaoVisualizadoSigiloso", false, true),
    ] {
        let leitura = com_classe(classe).unwrap();
        let processo = &leitura.processos[0];
        assert_eq!(processo.visualizado, visualizado, "{classe}");
        assert_eq!(processo.sigiloso, sigiloso, "{classe}");
        assert_eq!(processo.numero, "001234500000015");
    }
}

#[test]
fn detalhada_sem_classe_reconhecida_recusa_em_vez_de_inventar_estado() {
    assert_eq!(
        com_classe("outraClasse").map(|leitura| leitura.processos.len()),
        Err(ErroDetalhada::EstadoDoLinkDoProcessoAusente)
    );
}

#[test]
fn detalhada_recusa_classes_de_estado_diferentes_no_link() {
    for classe in [
        "processoVisualizado processoNaoVisualizado",
        "processoNaoVisualizado processoNaoVisualizadoSigiloso",
    ] {
        let erro = com_classe(classe).unwrap_err();
        assert!(matches!(
            erro,
            ErroDetalhada::ClassesDeEstadoDiferentesNoLink(_)
        ));
    }
}
