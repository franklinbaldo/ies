//! Regressões da coleta integral de unidades (#244). Fixture sintética
//! reproduz a paginação publicada pela lupa no acervo GO.
use super::*;
use crate::tela_html::HtmlServido;
fn pagina(n: usize, paginas: usize, total: usize, ids: &[&str]) -> String {
    let options = (0..paginas).fold(String::new(), |mut html, i| {
        use std::fmt::Write as _;
        write!(html, "<option value='{i}'>{}</option>", i + 1).unwrap();
        html
    });
    let rows = ids.iter().fold(String::new(), |mut html, id| {
        use std::fmt::Write as _;
        write!(html, "<tr><td><input type='checkbox' value='{id}'></td><td>ABC/{id}</td><td>Descrição</td><td>Órgão</td></tr>").unwrap();
        html
    });
    format!(
        "<form id='frmUnidadeLista' method='post'><input type='hidden' name='hdnInfraPaginaAtual' value='{n}'/><select name='selInfraPaginacaoSuperior'>{options}</select><table class='infraTable'><caption>Lista de Unidades ({total} registros):</caption><tr><th></th><th>Sigla</th><th>Descrição</th><th>Órgão</th></tr>{rows}</table></form>"
    )
}
#[test]
fn total_da_legenda_aceita_faixa_observada_no_go() {
    // Cassette real unidade-listar-destinos-go @2026-10-21.
    assert_eq!(
        total_da_legenda("Lista de Unidades (13671 registros - 1 a 50):"),
        Ok(13671)
    );
    assert_eq!(total_da_legenda("Lista de Unidades (1 registro):"), Ok(1));
    assert_eq!(total_da_legenda("Lista de Unidades (2 registros):"), Ok(2));
    assert_eq!(
        total_da_legenda("Lista de Unidades (x registros):"),
        Err(ErroLupa::Legenda(
            crate::legenda::ErroLegenda::ContagemIlegivel("x".parse::<usize>().unwrap_err())
        ))
    );
}
#[test]
fn paginas_inteiras_sem_repeticao() {
    let p0 = ler_pagina(&HtmlServido::de_teste(&pagina(0, 2, 3, &["1", "2"]))).unwrap();
    let p1 = ler_pagina(&HtmlServido::de_teste(&pagina(1, 2, 3, &["3"]))).unwrap();
    assert_eq!(reunir_paginas(vec![p0, p1]).unwrap().len(), 3);
}
#[test]
fn repeticao_e_total_errado_sao_recusados_antes_do_indice() {
    let p0 = ler_pagina(&HtmlServido::de_teste(&pagina(0, 2, 3, &["1", "2"]))).unwrap();
    let p1 = ler_pagina(&HtmlServido::de_teste(&pagina(1, 2, 3, &["1"]))).unwrap();
    assert_eq!(
        reunir_paginas(vec![p0, p1]),
        Err(ErroLupa::PaginasDivergentes)
    );
    let p0 = ler_pagina(&HtmlServido::de_teste(&pagina(0, 2, 3, &["1", "2"]))).unwrap();
    let p1 = ler_pagina(&HtmlServido::de_teste(&pagina(1, 2, 3, &[]))).unwrap();
    assert_eq!(
        reunir_paginas(vec![p0, p1]),
        Err(ErroLupa::PaginasDivergentes)
    );
}
#[test]
fn nenhuma_unidade_publicada_na_lupa_eh_uma_lista_vazia() {
    // Cassette GO unidade-listar-destinos-go, POST passo 12:
    // frmUnidadeLista e "Nenhum registro encontrado.", sem infraTable.
    let html = HtmlServido::de_teste(
        "<form id='frmUnidadeLista'>
        <input type='hidden' name='hdnInfraPaginaAtual' value='0'>
        <select name='selInfraPaginacaoSuperior'><option value='0'>1</option></select>
        <div>Nenhum registro encontrado.</div>
    </form>",
    );
    let lida = ler_pagina(&html).expect("vazio observado, sem tabela");
    assert_eq!((lida.total, lida.pagina, lida.paginas), (0, 0, 1));
    assert_eq!(lida.unidades, Vec::new());
    assert_eq!(reunir_paginas(vec![lida]).unwrap(), Vec::new());
}
#[test]
fn ausencia_de_tabela_sem_aviso_de_vazio_nao_pode_ser_sucesso() {
    let html = HtmlServido::de_teste(
        "<form id='frmUnidadeLista'>
        <input type='hidden' name='hdnInfraPaginaAtual' value='0'>
        <select name='selInfraPaginacaoSuperior'><option value='0'>1</option></select>
    </form>",
    );
    assert_eq!(ler_pagina(&html).err(), Some(ErroLupa::TabelaForaDaForma));
}
#[test]
fn cabecalho_ausente_nao_pode_virar_lista_vazia() {
    assert!(ler_pagina(&HtmlServido::de_teste("<form id='frmUnidadeLista'></form>")).is_err());
}
#[test]
fn filtros_publicados_precisam_provar_lista_integral() {
    // Cenários de filtro e órgão do Todos tests/test_lupa_unidades_010.py @323eb01.
    let html = pagina(0, 1, 1, &["7"]).replace(
        "<table class='infraTable'>",
        "<select name='selOrgaoUnidade'>
            <option value='null'>Nenhum órgão</option>
            <option value='' selected='selected'>Todos</option>
         </select>
         <input name='txtSiglaUnidade' value=''>
         <input name='txtDescricaoUnidade' value=''>
         <table class='infraTable'>",
    );
    assert_eq!(
        conferir_filtros_de_lista_integral(&HtmlServido::de_teste(&html)),
        Ok(())
    );
    // Uma lista de nove resultados filtrados pode ser internamente
    // consistente, mas não substitui a lista integral de destinos.
    for alterada in [
        html.replace(
            "name='txtSiglaUnidade' value=''",
            "name='txtSiglaUnidade' value='CDE'",
        ),
        html.replace(
            "name='txtDescricaoUnidade' value=''",
            "name='txtDescricaoUnidade' value='gabinete'",
        ),
        html.replace("value='' selected='selected'>Todos", "value=''>Todos"),
        html.replace(
            "value='' selected='selected'>Todos",
            "value='null' selected='selected'>Todos",
        ),
        html.replace(
            "name='txtSiglaUnidade' value=''",
            "name='txtSiglaUnidade' value=''><input name='txtSiglaUnidade' value=''",
        ),
    ] {
        assert_eq!(
            conferir_filtros_de_lista_integral(&HtmlServido::de_teste(&alterada)),
            Err(ErroLupa::PaginaInvalida),
            "não publicar um snapshot quando o filtro não é integral"
        );
    }
}

#[test]
fn paginador_so_admite_indices_contiguos_e_unicos() {
    // GO publica índices de página de base zero; opções duplicadas,
    // omitidas ou saltadas não autorizam persistir um snapshot integral.
    let html = pagina(0, 2, 2, &["7", "8"]);
    for alterada in [
        html.replace(
            "<option value='1'>2</option>",
            "<option value='0'>2</option>",
        ),
        html.replace(
            "<option value='1'>2</option>",
            "<option value='3'>2</option>",
        ),
        html.replace("<option value='1'>2</option>", "<option>2</option>"),
    ] {
        assert_eq!(
            ler_pagina(&HtmlServido::de_teste(&alterada)).err(),
            Some(ErroLupa::PaginaInvalida),
            "paginador corrompido nao pode gerar snapshot completo"
        );
    }
}
