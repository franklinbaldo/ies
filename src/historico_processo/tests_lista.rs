//! RED/GREEN da página do histórico: visão `R`/`P`, legenda e contagem,
//! NUP do título. HTML sintético com a estrutura observada.
use super::ErroAndamentos as E;
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
        Err(E::AndamentosAlemDoTotalDeclarado)
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
        E::Historico(ErroHistorico::NumeroDoHistoricoDiferenteDoSolicitado)
    );
    assert!(erro.diverge_do_pedido());
    assert!(!E::LegendaDosAndamentosAusente.diverge_do_pedido());
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
        Err(E::LegendaDosAndamentosAusente)
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
            Err(E::LegendaDosAndamentosForaDaFormaObservada),
            "{legenda}"
        );
    }
    let duas = html.replace(
        "</caption>",
        "</caption><caption>Lista de Andamentos (1 registro):</caption>",
    );
    assert_eq!(
        ler_andamentos(&duas, NUP),
        Err(E::LegendaDosAndamentosForaDaFormaObservada)
    );
}

#[test]
fn tipo_ausente_repetido_ou_desconhecido_e_recusa() {
    let html = pagina_do_historico(NUP, "10", 'R', 1, 1);
    let campo =
        "  <input type=\"hidden\" id=\"hdnTipoHistorico\" name=\"hdnTipoHistorico\" value=\"R\" />";
    for trocado in [
        html.replace(campo, ""),
        html.replace(campo, &format!("{campo}{campo}")),
        html.replace(campo, &campo.replace("value=\"R\"", "value=\"X\"")),
        html.replace("frmProcedimentoHistorico\" method", "outroForm\" method"),
    ] {
        assert_eq!(
            ler_andamentos(&trocado, NUP),
            Err(E::TipoDoHistoricoNaoPublicado)
        );
    }
}

#[test]
fn subcodigos_estaveis() {
    let nomes: Vec<_> = [
        E::LegendaDosAndamentosAusente,
        E::LegendaDosAndamentosForaDaFormaObservada,
        E::AndamentosAlemDoTotalDeclarado,
        E::TipoDoHistoricoNaoPublicado,
        E::CharsetDoHistoricoNaoSuportado,
        E::RegraDeVerHistoricoNaoObservada,
        E::VisaoDoHistoricoDiferenteDaPedida,
        E::Historico(ErroHistorico::NumeroDoHistoricoDiferenteDoSolicitado),
    ]
    .iter()
    .map(|e| e.subcodigo())
    .collect();
    assert_eq!(
        nomes,
        [
            "legenda_dos_andamentos_ausente",
            "legenda_dos_andamentos_fora_da_forma_observada",
            "andamentos_alem_do_total_declarado",
            "tipo_do_historico_nao_publicado",
            "charset_do_historico_nao_suportado",
            "regra_de_ver_historico_nao_observada",
            "visao_do_historico_diferente_da_pedida",
            "numero_do_historico_diferente_do_solicitado",
        ]
    );
    assert_eq!(TipoDoHistorico::Resumido.como_texto(), "resumido");
    assert_eq!(TipoDoHistorico::Completo.como_texto(), "completo");
}
