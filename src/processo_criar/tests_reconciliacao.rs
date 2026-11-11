//! RED sintético #245: dois retratos completos do Controle, sem enviar HTTP.
//! Os números são artificiais; IDs e NUPs nunca são derivados um do outro.
use super::*;
use crate::tela_html::HtmlServido;
use std::fmt::Write as _;

fn tela(registros: &[(&str, &str)], total: usize, inicio: usize) -> String {
    let mut linhas = String::new();
    for (id, numero) in registros {
        write!(
            &mut linhas,
            "<tr id='P{id}'><td><a class='processoVisualizado'>{numero}</a></td></tr>"
        )
        .expect("escrita em String nao falha");
    }
    let legenda = if total == 0 {
        "0 registros:".to_owned()
    } else {
        format!(
            "{total} registros - {inicio} a {}):",
            inicio + registros.len().saturating_sub(1)
        )
    };
    format!(
        "<html><body><a id='lnkInfraUnidade'>TESTE</a>\
         <table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
         <table id='tblProcessosGerados'><caption>{legenda}</caption>{linhas}</table>\
         </body></html>"
    )
}

const NUP_A: &str = "00000.000001/2026-01";
const NUP_B: &str = "00000.000002/2026-02";

#[test]
fn novo_id_com_nup_observado_e_somente_um_candidato() {
    let antes = FotografiaDosGerados::da_tela(
        &HtmlServido::de_teste(&tela(&[("10", NUP_A)], 1, 1)),
        "TESTE",
    )
    .unwrap();
    let depois = FotografiaDosGerados::da_tela(
        &HtmlServido::de_teste(&tela(&[("10", NUP_A), ("11", NUP_B)], 2, 1)),
        "TESTE",
    )
    .unwrap();
    assert_eq!(
        antes.novos(&depois).unwrap(),
        vec![CandidatoGerado {
            id: "11".to_owned(),
            nup: NUP_B.to_owned(),
        }]
    );
}

#[test]
fn zero_antes_e_um_depois_nao_inventa_id() {
    let antes =
        FotografiaDosGerados::da_tela(&HtmlServido::de_teste(&tela(&[], 0, 1)), "TESTE").unwrap();
    let depois = FotografiaDosGerados::da_tela(
        &HtmlServido::de_teste(&tela(&[("17", NUP_B)], 1, 1)),
        "TESTE",
    )
    .unwrap();
    assert_eq!(
        antes.novos(&depois).unwrap(),
        vec![CandidatoGerado {
            id: "17".to_owned(),
            nup: NUP_B.to_owned(),
        }]
    );
}

#[test]
fn nenhuma_entrada_nova_e_apenas_ausencia_observada() {
    let html = HtmlServido::de_teste(&tela(&[("10", NUP_A)], 1, 1));
    let antes = FotografiaDosGerados::da_tela(&html, "TESTE").unwrap();
    let depois = FotografiaDosGerados::da_tela(&html, "TESTE").unwrap();
    assert_eq!(antes.novos(&depois).unwrap(), Vec::<CandidatoGerado>::new());
}

#[test]
fn dois_ids_novos_sao_preservados_sem_escolher_um_arbitrariamente() {
    let antes =
        FotografiaDosGerados::da_tela(&HtmlServido::de_teste(&tela(&[], 0, 1)), "TESTE").unwrap();
    let depois = FotografiaDosGerados::da_tela(
        &HtmlServido::de_teste(&tela(&[("10", NUP_A), ("11", NUP_B)], 2, 1)),
        "TESTE",
    )
    .unwrap();
    let encontrados = antes.novos(&depois).unwrap();
    assert_eq!(encontrados.len(), 2);
    assert_eq!(encontrados[0].nup, NUP_A);
    assert_eq!(encontrados[1].nup, NUP_B);
}

#[test]
fn legenda_paginada_recusada_antes_de_atribuir_ausencia() {
    let parcial = HtmlServido::de_teste(&tela(&[("10", NUP_A)], 5, 1));
    assert_eq!(
        FotografiaDosGerados::da_tela(&parcial, "TESTE"),
        Err(ErroFotografia::CoberturaIncompleta)
    );
    // Página 2 também não prova todos os Gerados.
    let segunda = HtmlServido::de_teste(&tela(&[("10", NUP_A)], 5, 2));
    assert_eq!(
        FotografiaDosGerados::da_tela(&segunda, "TESTE"),
        Err(ErroFotografia::CoberturaIncompleta)
    );
}

#[test]
fn unidade_e_id_ambiguo_falham_fechados() {
    let original = HtmlServido::de_teste(&tela(&[("10", NUP_A)], 1, 1));
    assert!(FotografiaDosGerados::da_tela(&original, "OUTRA").is_err());
    let duplicado = HtmlServido::de_teste(&tela(&[("10", NUP_A), ("10", NUP_B)], 2, 1));
    assert!(FotografiaDosGerados::da_tela(&duplicado, "TESTE").is_err());
}

#[test]
fn identidade_preexistente_alterada_nao_vira_candidato_novo() {
    let antes = FotografiaDosGerados::da_tela(
        &HtmlServido::de_teste(&tela(&[("10", NUP_A)], 1, 1)),
        "TESTE",
    )
    .unwrap();
    let depois = FotografiaDosGerados::da_tela(
        &HtmlServido::de_teste(&tela(&[("10", NUP_B)], 1, 1)),
        "TESTE",
    )
    .unwrap();
    assert_eq!(
        antes.novos(&depois),
        Err(ErroFotografia::IdentidadeAlterada)
    );
}

#[test]
fn desaparecimento_de_id_preexistente_nao_prova_que_post_nao_aplicou() {
    let antes = FotografiaDosGerados::da_tela(
        &HtmlServido::de_teste(&tela(&[("10", NUP_A)], 1, 1)),
        "TESTE",
    )
    .unwrap();
    let depois =
        FotografiaDosGerados::da_tela(&HtmlServido::de_teste(&tela(&[], 0, 1)), "TESTE").unwrap();
    assert_eq!(
        antes.novos(&depois),
        Err(ErroFotografia::IdentidadeAnteriorAusente)
    );
}

#[test]
fn nup_repetido_em_ids_distintos_nao_pode_virar_duas_identidades() {
    // A legenda declara duas linhas, mas ambas apontam para o mesmo NUP.
    // Na reconciliação de Gerados não há como tratar dois IDs como dois
    // processos independentes, mesmo com cobertura posicional completa.
    let ambigua = HtmlServido::de_teste(&tela(&[("10", NUP_A), ("11", NUP_A)], 2, 1));
    assert_eq!(
        FotografiaDosGerados::da_tela(&ambigua, "TESTE"),
        Err(ErroFotografia::NupRepetido)
    );
}
