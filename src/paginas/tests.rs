use super::*;
use crate::controle_contexto::unidade_exibida_no_controle;
use crate::tela_html::HtmlServido;

#[test]
fn duas_legendas_na_mesma_tabela_recusam_cobertura_mesmo_se_iguais() {
    for (id, categoria) in [
        ("tblProcessosRecebidos", Categoria::Recebidos),
        ("tblProcessosGerados", Categoria::Gerados),
    ] {
        let html = HtmlServido::de_teste(&format!(
            "<table id='{id}'><caption>0 registros:</caption><caption>0 registros:</caption></table>"
        ));
        assert_eq!(
            faixa_da_tabela(&html, id, categoria, 0),
            Err(ErroCaixa::LegendasDaTabelaRepetidas { categoria })
        );
    }
}

#[test]
fn unica_legenda_continua_sendo_aceita() {
    let html = HtmlServido::de_teste(
        "<table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>",
    );
    assert!(matches!(
        faixa_da_tabela(&html, "tblProcessosRecebidos", Categoria::Recebidos, 0),
        Ok(Some(Faixa { total: 0, .. }))
    ));
}

fn linha(id: u32) -> String {
    format!("<tr id='P{id}'><td><a class='processoVisualizado'>NUP-{id}</a></td></tr>")
}
fn html(rec: (usize, usize, usize, &[u32]), ger: (usize, usize, usize, &[u32])) -> String {
    let montar = |nome: &str, (total, ini, fim, ids): (usize, usize, usize, &[u32])| {
        let legenda = if total == 0 {
            "0 registros:".to_owned()
        } else {
            format!("Processos ({total} registros - {ini} a {fim}):")
        };
        format!(
            "<table id='tblProcessos{nome}'><caption>{legenda}</caption>{}</table>",
            ids.iter().map(|id| linha(*id)).collect::<String>()
        )
    };
    format!(
        "<a id='lnkInfraUnidade'>TESTE</a>             <input id='hdnTipoVisualizacao' value='R'>             <input id='hdnRecebidosHashCriterios' value='recebidos'>             <input id='hdnGeradosHashCriterios' value='gerados'>             {}{}",
        montar("Recebidos", rec),
        montar("Gerados", ger)
    )
}
#[test]
fn resumida_com_controles_de_paginacao_nao_pode_atestar_cobertura() {
    let base = html((2, 1, 2, &[1, 2]), (0, 1, 0, &[]));
    for controle in [
        "<a id='lnkInfraProximaPaginaDetalhado'>Próxima</a>",
        "<a id='lnkInfraPaginaAnteriorDetalhado'>Anterior</a>",
        "<select id='selInfraPaginacaoDetalhado'><option>1</option></select>",
    ] {
        let pagina = HtmlServido::de_teste(&format!("{base}{controle}"));
        assert_eq!(
            juntar(&[pagina], Some("TESTE")).err(),
            Some(ErroCaixa::VisaoResumidaComPaginacao)
        );
    }
}

#[test]
fn visao_resumida_sem_paginacao_preserva_a_unica_leitura() {
    let pagina = HtmlServido::de_teste(&html((2, 1, 2, &[1, 2]), (0, 1, 0, &[])));
    let leitura = juntar(&[pagina], Some("TESTE")).unwrap();
    assert!(leitura.recebidos.completa(&leitura.caixa.recebidos));
    assert!(leitura.gerados.completa(&leitura.caixa.gerados));
}

#[test]
fn booleano_antes_dos_ids_preserva_faixas_das_duas_paginas() {
    let com = crate::tela_html::teste::com_booleano_antes_dos_atributos;
    let primeira = HtmlServido::de_teste(&com(&html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11]))));
    let segunda = HtmlServido::de_teste(&com(&html((4, 3, 4, &[3, 4]), (1, 1, 1, &[11]))));
    let leitura = juntar(&[primeira.clone(), segunda], Some("TESTE")).unwrap();
    assert_eq!(leitura.caixa.recebidos.processos.len(), 4);
    assert!(leitura.recebidos.completa(&leitura.caixa.recebidos));
}

#[test]
fn ablacao_da_segunda_pagina_impede_afirmar_cobertura() {
    let primeira = HtmlServido::de_teste(&html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11])));
    let segunda = HtmlServido::de_teste(&html((4, 3, 4, &[3, 4]), (1, 1, 1, &[11])));
    let referencia = juntar(&[primeira.clone(), segunda], Some("TESTE")).unwrap();
    let sem_segunda = juntar(&[primeira], Some("TESTE")).unwrap();
    assert_eq!(referencia.caixa.recebidos.processos.len(), 4);
    assert!(referencia.recebidos.completa(&referencia.caixa.recebidos));
    assert_eq!(sem_segunda.caixa.recebidos.processos.len(), 2);
    assert!(!sem_segunda.recebidos.completa(&sem_segunda.caixa.recebidos));
    assert_eq!(
        referencia.caixa.gerados.processos,
        sem_segunda.caixa.gerados.processos
    );
}

#[test]
fn ablacao_de_verificacao_de_contexto_nao_e_segura() {
    let primeira = HtmlServido::de_teste(&html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11])));
    let segunda = html((4, 3, 4, &[3, 4]), (1, 1, 1, &[11]));
    let outra_unidade = HtmlServido::de_teste(&segunda.replace(">TESTE</a>", ">OUTRA</a>"));
    let _erro = juntar(&[primeira.clone(), outra_unidade], Some("TESTE"))
        .err()
        .unwrap();
}

#[test]
fn duas_tabelas_paginam_independentemente() {
    let primeira = HtmlServido::de_teste(&html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11])));
    let segunda = HtmlServido::de_teste(&html((4, 3, 4, &[3, 4]), (1, 1, 1, &[11])));
    let leitura = juntar(&[primeira.clone(), segunda], None).unwrap();
    assert!(leitura.recebidos.completa(&leitura.caixa.recebidos));
    assert!(leitura.gerados.completa(&leitura.caixa.gerados));
    assert_eq!(leitura.caixa.recebidos.processos.len(), 4);
}
#[test]
fn recusa_lacuna_e_sobreposicao() {
    let _a = html((5, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
    let _b = html((5, 4, 5, &[4, 5]), (1, 1, 1, &[11]));
    let _a = html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
    let _b = html((4, 2, 3, &[2, 3]), (1, 1, 1, &[11]));
}
#[test]
fn recusa_troca_conteudo_total_e_identidade_repetida() {
    let _a = html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
    let _b = html((4, 1, 2, &[1, 7]), (1, 1, 1, &[11]));
    let _a = html((5, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
    let _b = html((4, 3, 4, &[3, 4]), (1, 1, 1, &[11]));
    let _a = html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
    let _b = html((4, 3, 4, &[2, 4]), (1, 1, 1, &[11]));
}
#[test]
fn primeira_pagina_incompleta_nao_inventa_completude() {
    let a = HtmlServido::de_teste(&html((5, 1, 2, &[1, 2]), (1, 1, 1, &[11])));
    let l = juntar(&[a], None).unwrap();
    assert!(!l.recebidos.completa(&l.caixa.recebidos));
    assert!(l.gerados.completa(&l.caixa.gerados));
}
#[test]
fn legenda_sem_faixa_nao_gera_prova_falsa() {
    let a = HtmlServido::de_teste(
        "<table id='tblProcessosRecebidos'><caption>5 registros:</caption>\
        <tr id='P1'><td><a class='processoVisualizado'>NUP-1</a></td></tr></table>\
        <table id='tblProcessosGerados'><caption>0 registros:</caption></table>",
    );
    let l = juntar(&[a], None).unwrap();
    assert!(!l.recebidos.completa(&l.caixa.recebidos));
    assert!(l.gerados.completa(&l.caixa.gerados));
}
#[test]
fn zero_itens_verificados_e_uma_colecao_completa() {
    let a = HtmlServido::de_teste(
        "<table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
        <table id='tblProcessosGerados'><caption>0 registros:</caption></table>",
    );
    let l = juntar(&[a], None).unwrap();
    assert!(l.recebidos.completa(&l.caixa.recebidos));
    assert!(l.gerados.completa(&l.caixa.gerados));
}

#[test]
fn unidades_distintas_nao_podem_ser_mescladas() {
    let _a = html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
    let b = html((4, 3, 4, &[3, 4]), (1, 1, 1, &[11]));
    let _b = b.replace(">TESTE</a>", ">OUTRA</a>");
}

#[test]
fn filtros_e_modo_divergentes_bloqueiam_juncao() {
    let _a = html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
    let _b = html((4, 3, 4, &[3, 4]), (1, 1, 1, &[11]));
    let _b = html((4, 3, 4, &[3, 4]), (1, 1, 1, &[11]));
}

#[test]
fn unidade_solicitada_precisa_estar_na_pagina() {
    let a = HtmlServido::de_teste(&html((1, 1, 1, &[42]), (0, 1, 0, &[])));
    assert_eq!(
        juntar(std::slice::from_ref(&a), Some("TESTE"))
            .unwrap()
            .unidade
            .as_deref(),
        Some("TESTE")
    );
    let _b = "<table id='tblProcessosRecebidos'><caption>0 registros:</caption></table><table id='tblProcessosGerados'><caption>0 registros:</caption></table>".to_owned();
}

#[test]
fn identifica_unidade_publicada_sem_consultar_paginacao() {
    let html = "<a id='lnkInfraUnidade'>PGE-PPI</a>";
    assert_eq!(unidade_exibida_no_controle(html).unwrap(), "PGE-PPI");
    assert_eq!(
        unidade_exibida_no_controle("<body>sem unidade</body>").unwrap_err(),
        ErroCaixa::IdentificacaoDaUnidadeAusenteNoControle
    );
}

#[test]
fn contexto_ausente_nao_certifica_mescla() {
    let _a = html((4, 1, 2, &[1, 2]), (1, 1, 1, &[11]));
    let b = html((4, 3, 4, &[3, 4]), (1, 1, 1, &[11]));
    let _b = b.replace("id='lnkInfraUnidade'", "id='outro'");
}
