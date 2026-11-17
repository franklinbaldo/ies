//! RED/GREEN do link do histórico na árvore: as duas formas observadas,
//! ambas, nenhuma, repetidas e URLs fora da conferência. HTML sintético.
use super::tests_apoio::*;
use super::*;
use crate::arvore_processo::tests::{NUP, arvore, no_processo};
use crate::instalacao::Instalacao;
use crate::tela_html::HtmlServido;

const ID: &str = "10";
const ORIGEM: &str = "https://sei.exemplo.gov.br/sei";

fn instalacao() -> Instalacao {
    Instalacao::interpretar(ORIGEM).unwrap()
}

fn base() -> String {
    arvore(&[no_processo(0, ID, NUP)])
}

fn ler(partes: &[(String, String)]) -> Result<EnderecoDoHistorico, super::ErroLinkDoHistorico> {
    ler_link_do_historico(
        &instalacao(),
        &HtmlServido::de_teste(&com_link(&base(), partes)),
        ID,
    )
}

fn esperado() -> String {
    format!("{ORIGEM}/{}", link_do_historico(ID))
}

#[test]
fn forma_unb_le_a_url_da_funcao_servida() {
    let lido = ler(&[forma_unb(&link_do_historico(ID))]).unwrap();
    assert_eq!(lido.as_str(), esperado());
}

#[test]
fn forma_go_le_a_url_do_onclick() {
    let lido = ler(&[forma_go(&link_do_historico(ID))]).unwrap();
    assert_eq!(lido.as_str(), esperado());
}

#[test]
fn onclick_com_amp_e_o_mesmo_endereco() {
    let url = link_do_historico(ID).replace('&', "&amp;");
    assert_eq!(ler(&[forma_go(&url)]).unwrap().as_str(), esperado());
}

#[test]
fn debug_nao_mostra_a_url() {
    let lido = ler(&[forma_go(&link_do_historico(ID))]).unwrap();
    let debug = format!("{lido:?}");
    assert!(!debug.contains(HASH_HISTORICO) && !debug.contains("controlador"));
}

#[test]
fn sem_link_e_recusa() {
    assert_eq!(
        ler_link_do_historico(&instalacao(), &HtmlServido::de_teste(&base()), ID),
        Err(super::ErroLinkDoHistorico::LinkDoHistoricoAusenteNaArvore)
    );
}

#[test]
fn as_duas_formas_juntas_sao_recusa() {
    let url = link_do_historico(ID);
    let (script, _) = forma_unb(&url);
    let (_, corpo) = forma_go(&url);
    assert_eq!(
        ler(&[(script, corpo)]),
        Err(super::ErroLinkDoHistorico::LinksDoHistoricoRepetidosNaArvore)
    );
}

#[test]
fn link_repetido_e_recusa() {
    let url = link_do_historico(ID);
    assert_eq!(
        ler(&[forma_go(&url), forma_go(&url)]),
        Err(super::ErroLinkDoHistorico::LinksDoHistoricoRepetidosNaArvore)
    );
    assert_eq!(
        ler(&[forma_unb(&url), forma_unb(&url)]),
        Err(super::ErroLinkDoHistorico::LinksDoHistoricoRepetidosNaArvore)
    );
}

#[test]
fn onclick_fora_das_formas_e_recusa() {
    let url = link_do_historico(ID);
    let (_, corpo) = forma_go(&url);
    for trocado in [
        corpo.replace("consultarAndamento('", "consultarAndamento(\""),
        corpo.replace("');\"", "')\""),
        corpo.replace("consultarAndamento('", "outraFuncao('"),
    ] {
        assert_eq!(
            ler(&[(String::new(), trocado)]),
            Err(super::ErroLinkDoHistorico::LinkDoHistoricoForaDaFormaObservada)
        );
    }
}

#[test]
fn funcao_sem_a_linha_do_src_e_recusa() {
    let url = link_do_historico(ID);
    let (script, corpo) = forma_unb(&url);
    let trocado = script.replace("parent.document", "window.parent.document");
    assert_eq!(
        ler(&[(trocado, corpo)]),
        Err(super::ErroLinkDoHistorico::LinkDoHistoricoForaDaFormaObservada)
    );
}

#[test]
fn url_fora_da_conferencia_e_recusa() {
    let url = link_do_historico(ID);
    for trocada in [
        url.replace(
            "procedimento_consultar_historico",
            "procedimento_atualizar_andamento",
        ),
        url.replace(HASH_HISTORICO, "abc"),
        url.replace(
            "controlador.php?",
            "https://outro.exemplo/sei/controlador.php?",
        ),
        format!("{url}&acao=procedimento_consultar_historico"),
    ] {
        assert_eq!(
            ler(&[forma_go(&trocada)]),
            Err(super::ErroLinkDoHistorico::EnderecoDoHistoricoNaoReconhecido),
            "{trocada}"
        );
    }
}

#[test]
fn link_de_outro_processo_e_recusa() {
    assert_eq!(
        ler(&[forma_unb(&link_do_historico("11"))]),
        Err(super::ErroLinkDoHistorico::EnderecoDoHistoricoDeOutroProcesso)
    );
}
