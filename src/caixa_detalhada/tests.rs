use super::*;

fn pagina(total: usize, inicio: usize, ids: &[(u32, &str)]) -> String {
    let fim = inicio + ids.len() - 1;
    let linhas = ids.iter().map(|(id, nup)| {
        format!(
            "<tr id='P{id}'><td>linha</td><td><a class='processoVisualizado' \
             href='controlador.php?acao=procedimento_trabalhar&amp;id_procedimento={id}'>\
             {nup}</a></td><td>Ofício</td></tr>"
        )
    }).collect::<String>();
    format!(
        "<html><body><a id='lnkInfraUnidade'>PPI</a>\
         <input id='hdnTipoVisualizacao' value='D'>\
         <input id='hdnRecebidosHashCriterios' value='r'>\
         <input id='hdnGeradosHashCriterios' value='g'>\
         <table id='tblProcessosDetalhado'><caption>({total} registros - {inicio} a {fim}):</caption>\
         <tr><th>Seleção</th><th>Processo</th><th>Tipo</th></tr>{linhas}</table></body></html>"
    )
}

#[test]
fn pagina_detalhada_completa_de_um_get_vira_lista_sem_categorias_inventadas() {
    let html = pagina(2, 1, &[(1, "000001/2026"), (2, "000002/2026")]);
    assert!(possui_tabela(&html));
    let resposta = ler_paginas(&[html], Some("PPI")).unwrap();
    assert_eq!(resposta.unidade, "PPI");
    assert_eq!(resposta.total_declarado, 2);
    assert_eq!(resposta.processos.len(), 2);
    assert_eq!(resposta.processos[0].id.as_str(), "1");
    assert_eq!(resposta.processos[0].numero, "000001/2026");
    assert_eq!(resposta.processos[0].visualizado, Some(true));
    assert_eq!(resposta.processos[1].numero, "000002/2026");
}

#[test]
fn duas_paginas_detalhadas_capturadas_unem_intervalos_sem_post() {
    let primeira = pagina(3, 1, &[(1, "000001/2026"), (2, "000002/2026")]);
    let segunda = pagina(3, 3, &[(3, "000003/2026")]);
    let leitura = ler_paginas(&[primeira, segunda], Some("PPI")).unwrap();
    assert_eq!(leitura.total_declarado, 3);
    assert_eq!(leitura.processos.len(), 3);
}

#[test]
fn nao_aceita_pagina_inicial_parcial_como_total() {
    let pagina = pagina(3, 1, &[(1, "000001/2026"), (2, "000002/2026")]);
    assert_eq!(ler_paginas(&[pagina], Some("PPI")), Err(ErroDetalhada::PaginasNaoObtidas));
}

#[test]
fn nao_aceita_faixas_omitidas_ou_sobrepostas() {
    let primeira = pagina(4, 1, &[(1, "000001/2026"), (2, "000002/2026")]);
    let outra = pagina(4, 4, &[(4, "000004/2026")]);
    assert_eq!(ler_paginas(&[primeira.clone(), outra], Some("PPI")), Err(ErroDetalhada::FaixaDescontinua));
    let outra = pagina(4, 2, &[(2, "000002/2026"), (3, "000003/2026")]);
    assert_eq!(ler_paginas(&[primeira, outra], Some("PPI")), Err(ErroDetalhada::FaixaDescontinua));
}

#[test]
fn troca_de_unidade_e_duplicidade_sao_erros_tipados() {
    let primeira = pagina(3, 1, &[(1, "000001/2026"), (2, "000002/2026")]);
    let segunda = pagina(3, 3, &[(3, "000003/2026")]);
    let outra = segunda.replace(">PPI</a>", ">OUTRA</a>");
    assert_eq!(ler_paginas(&[primeira.clone(), outra], Some("PPI")), Err(ErroDetalhada::ContextoDiferente));
    let repetida = segunda.replace("id='P3'", "id='P1'");
    assert_eq!(ler_paginas(&[primeira, repetida], Some("PPI")), Err(ErroDetalhada::IdentificadorRepetido));
}

#[test]
fn nao_cria_processo_de_celula_sem_link_reconhecido() {
    let original = pagina(1, 1, &[(1, "000001/2026")]);
    let sem_link = original.replace("acao=procedimento_trabalhar", "acao=outra_acao");
    assert_eq!(ler_paginas(&[sem_link], Some("PPI")), Err(ErroDetalhada::LinkDoProcessoAusente));
}

#[test]
fn nao_atribui_detalhada_a_visao_resumida() {
    let original = pagina(1, 1, &[(1, "000001/2026")]);
    let outra = original.replace("value='D'", "value='R'");
    assert_eq!(ler_paginas(&[outra], Some("PPI")), Err(ErroDetalhada::VisualizacaoIncorreta));
}
