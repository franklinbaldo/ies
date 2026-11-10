//! Classes do link do processo na visão resumida (fixtures sintéticas).
use super::*;
use crate::paginas::juntar;

fn gerados(linhas: &str, total: usize) -> String {
    format!(
        "<table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
         <table id='tblProcessosGerados'><caption>{total} registros:</caption>{linhas}</table>"
    )
}

fn linha(id: u32, classe: &str, numero: &str) -> String {
    format!(
        "<tr id='P{id}'><td><a href='controlador.php?acao=procedimento_trabalhar\
         &amp;id_procedimento={id}' class='{classe}'>{numero}</a></td></tr>"
    )
}

fn unico(classe: &str) -> Result<Processo, ErroCaixa> {
    let caixa = parse_html(&gerados(&linha(7, classe, "000123/2026"), 1))?;
    Ok(caixa.gerados.processos[0].clone())
}

#[test]
fn sigiloso_visualizado_e_lido_da_classe_observada() {
    let processo = unico("processoVisualizadoSigiloso").unwrap();
    assert!(processo.visualizado);
    assert!(processo.sigiloso);
}

#[test]
fn sigiloso_nao_visualizado_e_lido_pelo_par_simetrico() {
    let processo = unico("processoNaoVisualizadoSigiloso").unwrap();
    assert!(!processo.visualizado);
    assert!(processo.sigiloso);
}

#[test]
fn classes_nao_sigilosas_continuam_sem_sigilo() {
    for (classe, visualizado) in [
        ("processoVisualizado", true),
        ("processoNaoVisualizado", false),
        ("processoVisualizado processoVisitado", true),
        ("processoVisualizado processoVisualizado", true),
    ] {
        let processo = unico(classe).unwrap();
        assert_eq!(processo.visualizado, visualizado, "{classe}");
        assert!(!processo.sigiloso, "{classe}");
    }
}

#[test]
fn tabela_mista_preserva_estado_de_cada_linha() {
    let linhas = [
        linha(1, "processoVisualizado", "000001/2026"),
        linha(2, "processoVisualizadoSigiloso", "000002/2026"),
        linha(3, "processoNaoVisualizado", "000003/2026"),
        linha(4, "processoNaoVisualizadoSigiloso", "000004/2026"),
    ]
    .concat();
    let caixa = parse_html(&gerados(&linhas, 4)).unwrap();
    let estados: Vec<_> = caixa
        .gerados
        .processos
        .iter()
        .map(|p| (p.id.as_str().to_owned(), p.visualizado, p.sigiloso))
        .collect();
    assert_eq!(
        estados,
        [
            ("1".to_owned(), true, false),
            ("2".to_owned(), true, true),
            ("3".to_owned(), false, false),
            ("4".to_owned(), false, true),
        ]
    );
}

#[test]
fn numero_de_quinze_digitos_sem_pontuacao_e_preservado() {
    let caixa = parse_html(&gerados(
        &linha(9, "processoVisualizadoSigiloso", "001234500000015"),
        1,
    ))
    .unwrap();
    assert_eq!(caixa.gerados.processos[0].numero, "001234500000015");
}

#[test]
fn classes_de_estado_diferentes_no_mesmo_link_falham_tipadas() {
    for classe in [
        "processoVisualizado processoNaoVisualizado",
        "processoVisualizado processoVisualizadoSigiloso",
        "processoNaoVisualizadoSigiloso processoVisualizadoSigiloso",
    ] {
        let erro = unico(classe).unwrap_err();
        assert_eq!(
            erro,
            ErroCaixa::ClassesDeEstadoDiferentesNoLinkDoProcesso {
                categoria: Categoria::Gerados,
                id: "7".to_owned(),
            },
            "{classe}"
        );
    }
}

#[test]
fn classe_e_comparada_por_token_inteiro_sem_substring() {
    for _classe in [
        "xprocessoVisualizadoSigiloso",
        "processoVisualizadoSigilosoX",
        "processoSigiloso",
    ] {}
}

/// Forma de Goiás: legenda com faixa, dez linhas por página e paginadores
/// `lnkRecebidos*`/`lnkGerados*`; a cobertura não pode ser atestada.
#[test]
fn pagina_de_dez_com_sigiloso_e_lida_mas_nao_cobre_os_totais() {
    let mut recebidos = String::new();
    let mut gerados = String::new();
    for i in 1..=10u32 {
        recebidos.push_str(&linha(100 + i, "processoNaoVisualizado", &format!("R{i}")));
        let classe = if i == 4 {
            "processoVisualizadoSigiloso"
        } else {
            "processoVisualizado"
        };
        gerados.push_str(&linha(200 + i, classe, &format!("00000000000{i:04}")));
    }
    let html = format!(
        "<a id='lnkInfraUnidade'>TESTE</a>\
         <input type='hidden' id='hdnTipoVisualizacao' value='R' />\
         <input id='hdnRecebidosHashCriterios' value='r'>\
         <input id='hdnGeradosHashCriterios' value='g'>\
         <a id='lnkRecebidosProximaPaginaSuperior'>p</a>\
         <select id='selGeradosPaginacaoSuperior'></select>\
         <table id='tblProcessosRecebidos'><caption class='infraCaption'>\
         Processos recebidos (131 registros - 1 a 10):</caption>{recebidos}</table>\
         <table id='tblProcessosGerados'><caption class='infraCaption'>\
         Processos gerados (7990 registros - 1 a 10):</caption>{gerados}</table>"
    );
    let leitura = juntar(&[html], Some("TESTE")).unwrap();
    assert_eq!(leitura.unidade.as_deref(), Some("TESTE"));
    assert_eq!(leitura.caixa.recebidos.total_declarado, Some(131));
    assert_eq!(leitura.caixa.gerados.total_declarado, Some(7990));
    assert_eq!(leitura.caixa.gerados.processos.len(), 10);
    assert!(leitura.caixa.gerados.processos[3].sigiloso);
    assert_eq!(leitura.caixa.gerados.processos[3].numero, "000000000000004");
    assert!(!leitura.recebidos.completa(&leitura.caixa.recebidos));
    assert!(!leitura.gerados.completa(&leitura.caixa.gerados));
}
