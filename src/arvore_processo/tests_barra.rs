//! RED/GREEN do link das propriedades na barra do nó `PROCESSO`: forma
//! observada na `UnB` 4.0.9, fallback `procedimento_consultar` e recusas.
use super::ler_link_das_propriedades;
use super::tests::{NUP, arvore, no_documento, no_processo};
use crate::instalacao::Instalacao;
use crate::propriedades_processo::tests_apoio::{
    acao_da_barra, barra, com_barra, link_das_propriedades,
};
use crate::propriedades_processo::{ErroPropriedades, FonteDasPropriedades};

const ORIGEM: &str = "https://sei.exemplo.gov.br/sei";

fn instalacao() -> Instalacao {
    Instalacao::interpretar(ORIGEM).unwrap()
}

fn base() -> String {
    arvore(&[
        no_processo(0, "10", NUP),
        no_documento(1, "21", "10", "0000101"),
    ])
}

fn com_acoes(acoes: &[String]) -> String {
    com_barra(&base(), 0, &barra(acoes))
}

fn ler(html: &str) -> Result<Option<(FonteDasPropriedades, String)>, ErroPropriedades> {
    ler_link_das_propriedades(&instalacao(), html, "10")
        .map(|achado| achado.map(|e| (e.fonte, e.as_str().to_owned())))
}

fn alterar(id: &str) -> String {
    acao_da_barra(&link_das_propriedades("procedimento_alterar", id))
}

fn consultar(id: &str) -> String {
    acao_da_barra(&link_das_propriedades("procedimento_consultar", id))
}

fn absoluta(acao: &str) -> String {
    format!("{ORIGEM}/{}", link_das_propriedades(acao, "10"))
}

#[test]
fn le_o_link_de_alterar_da_barra_do_processo() {
    assert_eq!(
        ler(&com_acoes(&[alterar("10")])),
        Ok(Some((
            FonteDasPropriedades::Alterar,
            absoluta("procedimento_alterar")
        )))
    );
}

#[test]
fn sem_alterar_usa_consultar_e_alterar_vence_quando_ha_os_dois() {
    assert_eq!(
        ler(&com_acoes(&[consultar("10")])),
        Ok(Some((
            FonteDasPropriedades::Consultar,
            absoluta("procedimento_consultar")
        )))
    );
    assert_eq!(
        ler(&com_acoes(&[consultar("10"), alterar("10")]))
            .unwrap()
            .map(|(f, _)| f),
        Some(FonteDasPropriedades::Alterar)
    );
}

#[test]
fn barra_vazia_ou_sem_os_links_nao_publica() {
    assert_eq!(ler(&base()), Ok(None));
    assert_eq!(ler(&com_acoes(&[])), Ok(None));
    let sem_linha = base().replace("Nos[0].acoes = '';\n", "");
    assert_eq!(ler(&sem_linha), Ok(None));
}

#[test]
fn entidade_no_href_e_decodificada() {
    let href = link_das_propriedades("procedimento_alterar", "10").replace('&', "&amp;");
    assert_eq!(
        ler(&com_acoes(&[acao_da_barra(&href)]))
            .unwrap()
            .map(|(_, u)| u),
        Some(absoluta("procedimento_alterar"))
    );
}

#[test]
fn so_a_barra_do_processo_e_lida() {
    let html = com_barra(&base(), 1, &barra(&[alterar("10")]));
    assert_eq!(ler(&html), Ok(None));
}

#[test]
fn links_repetidos_sao_recusa() {
    assert_eq!(
        ler(&com_acoes(&[alterar("10"), alterar("10")])),
        Err(ErroPropriedades::LinksDasPropriedadesRepetidosNaBarra)
    );
    assert_eq!(
        ler(&com_acoes(&[consultar("10"), consultar("10")])),
        Err(ErroPropriedades::LinksDasPropriedadesRepetidosNaBarra)
    );
}

#[test]
fn endereco_de_outro_processo_ou_fora_da_regra_e_recusa() {
    assert_eq!(
        ler(&com_acoes(&[alterar("11")])),
        Err(ErroPropriedades::EnderecoDasPropriedadesDeOutroProcesso)
    );
    let curto = link_das_propriedades("procedimento_alterar", "10").replace(&"3".repeat(64), "33");
    assert_eq!(
        ler(&com_acoes(&[acao_da_barra(&curto)])),
        Err(ErroPropriedades::EnderecoDasPropriedadesNaoReconhecido)
    );
    let absoluto = format!(
        "{ORIGEM}/{}",
        link_das_propriedades("procedimento_alterar", "10")
    );
    assert_eq!(
        ler(&com_acoes(&[acao_da_barra(&absoluto)])),
        Err(ErroPropriedades::EnderecoDasPropriedadesNaoReconhecido)
    );
}

#[test]
fn barra_fora_da_forma_ou_repetida_e_recusa() {
    let fora = base().replace("Nos[0].acoes = '';", "Nos[0].acoes = montar();");
    assert_eq!(
        ler(&fora),
        Err(ErroPropriedades::BarraDoProcessoForaDaFormaObservada)
    );
    let repetida = com_acoes(&[alterar("10")]).replacen(
        "Nos[0].src = '';",
        "Nos[0].acoes = '';\nNos[0].src = '';",
        1,
    );
    assert_eq!(
        ler(&repetida),
        Err(ErroPropriedades::BarrasDoProcessoRepetidasNaArvore)
    );
}

#[test]
fn incluir_documento_so_usa_link_assinado_na_barra_do_processo() {
    use super::ler_link_de_incluir_documento;
    let href = format!(
        "controlador.php?acao=documento_escolher_tipo&id_procedimento=10&infra_hash={}",
        "e".repeat(64)
    );
    // O helper barra() já inclui uma ação de documento_escolher_tipo.
    // Para testar uma única URL escolhida, construir a barra sem essa ação-padrão.
    let lido = ler_link_de_incluir_documento(
        &instalacao(),
        &com_barra(&base(), 0, &acao_da_barra(&href)),
        "10",
    )
    .unwrap()
    .unwrap();
    assert!(
        ler_link_de_incluir_documento(&instalacao(), &com_acoes(&[]), "10")
            .unwrap()
            .is_some()
    );
    assert_eq!(lido.as_str(), format!("{ORIGEM}/{href}"));
    assert!(!format!("{lido:?}").contains(&"e".repeat(64)));
    assert!(
        ler_link_de_incluir_documento(&instalacao(), &base(), "10")
            .unwrap()
            .is_none()
    );
}

#[test]
fn incluir_documento_recusa_links_repetidos_e_outro_processo() {
    use super::{ErroLinkIncluirDocumento as ErroPropriedades, ler_link_de_incluir_documento};
    let href = format!(
        "controlador.php?acao=documento_escolher_tipo&id_procedimento=11&infra_hash={}",
        "e".repeat(64)
    );
    assert_eq!(
        ler_link_de_incluir_documento(
            &instalacao(),
            &com_barra(&base(), 0, &acao_da_barra(&href)),
            "10",
        ),
        Err(ErroPropriedades::EnderecoDeOutroProcesso)
    );
    let certo = href.replace("id_procedimento=11", "id_procedimento=10");
    assert_eq!(
        ler_link_de_incluir_documento(&instalacao(), &com_acoes(&[acao_da_barra(&certo)]), "10",),
        Err(ErroPropriedades::LinkRepetido)
    );
    let outro_no = com_barra(&base(), 1, &acao_da_barra(&certo));
    assert_eq!(
        ler_link_de_incluir_documento(&instalacao(), &outro_no, "10"),
        Ok(None)
    );
}
