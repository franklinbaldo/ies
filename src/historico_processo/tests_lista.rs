//! RED/GREEN da página do histórico: visão `R`/`P`, legenda e contagem,
//! NUP do título. HTML sintético com a estrutura observada.
use super::tests_apoio::*;
use super::*;
use crate::arvore_processo::tests::NUP;
use crate::historico::ErroHistorico;

#[test]
fn le_a_visao_resumida_com_total_igual_as_linhas() {
    let lidos = ler_andamentos(&pagina_do_historico(NUP, "10", 'R', 3, 3), NUP).unwrap();
    assert_eq!(lidos.tipo, TipoDoHistorico::Resumido);
    assert_eq!(lidos.total_declarado, 3);
    assert_eq!(lidos.itens.len(), 3);
    let primeiro = &lidos.itens[0];
    assert_eq!(primeiro.ocorrido_em.as_deref(), Some("01/01/2026 10:00"));
    assert_eq!(primeiro.unidade.as_deref(), Some("U0"));
    assert_eq!(primeiro.usuario.as_deref(), Some("p0"));
    assert_eq!(primeiro.descricao, "Andamento sintetico 0");
}

#[test]
fn le_a_visao_completa_e_o_singular_da_legenda() {
    let lidos = ler_andamentos(&pagina_do_historico(NUP, "10", 'P', 1, 1), NUP).unwrap();
    assert_eq!(lidos.tipo, TipoDoHistorico::Completo);
    assert_eq!(lidos.total_declarado, 1);
}

#[test]
fn menos_linhas_que_o_total_publica_o_total_sem_alegar_completude() {
    let lidos = ler_andamentos(&pagina_do_historico(NUP, "10", 'P', 49, 20), NUP).unwrap();
    assert_eq!(lidos.total_declarado, 49);
    assert_eq!(lidos.itens.len(), 20);
}

#[test]
fn mais_linhas_que_o_total_e_recusa() {
    assert_eq!(
        ler_andamentos(&pagina_do_historico(NUP, "10", 'R', 2, 3), NUP),
        Err(super::ErroAndamentos::AndamentosAlemDoTotalDeclarado)
    );
}

#[test]
fn pagina_de_outro_nup_diverge_do_pedido() {
    let erro = ler_andamentos(
        &pagina_do_historico("00009.000009/2026-09", "10", 'R', 1, 1),
        NUP,
    )
    .unwrap_err();
    assert_eq!(
        erro,
        super::ErroAndamentos::Historico(ErroHistorico::NumeroDoHistoricoDiferenteDoSolicitado)
    );
    assert!(erro.diverge_do_pedido());
    assert!(!super::ErroAndamentos::LegendaDosAndamentosAusente.diverge_do_pedido());
}

#[test]
fn legenda_ausente_ou_fora_da_forma_e_recusa() {
    let html = pagina_do_historico(NUP, "10", 'R', 1, 1);
    let sem = html.replace(
        "<caption class=\"infraCaption\">Lista de Andamentos (1 registro):</caption>",
        "",
    );
    assert_eq!(
        ler_andamentos(&sem, NUP),
        Err(super::ErroAndamentos::LegendaDosAndamentosAusente)
    );
    for legenda in [
        "Lista de Andamentos (1 registro - 1 a 1):",
        "Lista de Andamentos (um registro):",
        "Andamentos (1 registro):",
        "Lista de Andamentos (1 registro)",
    ] {
        let trocada = html.replace("Lista de Andamentos (1 registro):", legenda);
        assert_eq!(
            ler_andamentos(&trocada, NUP),
            Err(super::ErroAndamentos::LegendaDosAndamentosForaDaFormaObservada),
            "{legenda}"
        );
    }
    let duas = html.replace(
        "</caption>",
        "</caption><caption>Lista de Andamentos (1 registro):</caption>",
    );
    assert_eq!(
        ler_andamentos(&duas, NUP),
        Err(super::ErroAndamentos::LegendaDosAndamentosForaDaFormaObservada)
    );
}

#[test]
fn tipo_ausente_repetido_ou_desconhecido_e_recusa() {
    let html = pagina_do_historico(NUP, "10", 'R', 1, 1);
    let campo =
        "  <input type=\"hidden\" id=\"hdnTipoHistorico\" name=\"hdnTipoHistorico\" value=\"R\" />";
    let nao_publicado = Err(super::ErroAndamentos::TipoDoHistoricoNaoPublicado);
    let sem_formulario = Err(super::ErroAndamentos::FormularioDoTipoNaoLocalizado(
        crate::formulario_web::ErroSubmissaoFormulario::FormularioAusenteNaTela,
    ));
    for (trocado, esperado) in [
        (html.replace(campo, ""), nao_publicado.clone()),
        (
            html.replace(campo, &format!("{campo}{campo}")),
            nao_publicado.clone(),
        ),
        (
            html.replace(campo, &campo.replace("value=\"R\"", "value=\"X\"")),
            nao_publicado,
        ),
        (
            html.replace("frmProcedimentoHistorico\" method", "outroForm\" method"),
            sem_formulario,
        ),
    ] {
        assert_eq!(ler_andamentos(&trocado, NUP), esperado);
    }
}

#[test]
fn historico_completo_e_pedido_como_completo() {
    assert_eq!(TipoDoHistorico::Completo.como_texto(), "completo");
}
