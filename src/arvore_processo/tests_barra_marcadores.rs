//! RED/GREEN do link dos marcadores (`andamento_marcador_gerenciar`) na
//! barra do nó `PROCESSO`: forma observada na `UnB` 4.0.9, GO e PEN1, e
//! recusas, como os outros links assinados.
use super::ler_link_dos_marcadores;
use super::tests::{NUP, arvore, no_documento, no_processo};
use crate::instalacao::Instalacao;
use crate::marcadores_processo::tests_apoio::{HASH_MARCADORES, link_dos_marcadores};
use crate::propriedades_processo::tests_apoio::{acao_da_barra, barra, com_barra};

const ORIGEM: &str = "https://sei.exemplo.gov.br/sei";

fn base() -> String {
    arvore(&[
        no_processo(0, "10", NUP),
        no_documento(1, "21", "10", "0000101"),
    ])
}

fn com_acoes(acoes: &[String]) -> String {
    com_barra(&base(), 0, &barra(acoes))
}

fn ler(html: &str) -> Result<Option<String>, crate::marcadores_processo::ErroMarcadores> {
    let instalacao = Instalacao::interpretar(ORIGEM).unwrap();
    ler_link_dos_marcadores(&instalacao, html, "10")
        .map(|achado| achado.map(|e| e.as_str().to_owned()))
}

fn gerenciar(id: &str) -> String {
    acao_da_barra(&link_dos_marcadores(id))
}

#[test]
fn le_o_link_de_gerenciar_marcador_da_barra_do_processo() {
    assert_eq!(
        ler(&com_acoes(&[gerenciar("10")])),
        Ok(Some(format!("{ORIGEM}/{}", link_dos_marcadores("10"))))
    );
    let com_entidade = acao_da_barra(&link_dos_marcadores("10").replace('&', "&amp;"));
    assert!(ler(&com_acoes(&[com_entidade])).unwrap().is_some());
}

#[test]
fn barra_sem_o_link_nao_publica_e_nao_e_recusa() {
    assert_eq!(ler(&base()), Ok(None));
    assert_eq!(ler(&com_acoes(&[])), Ok(None));
    assert_eq!(
        ler(&com_barra(&base(), 1, &barra(&[gerenciar("10")]))),
        Ok(None)
    );
}

#[test]
fn link_repetido_de_outro_processo_ou_fora_da_regra_e_recusa() {
    assert_eq!(
        ler(&com_acoes(&[gerenciar("10"), gerenciar("10")])),
        Err(crate::marcadores_processo::ErroMarcadores::LinksDosMarcadoresRepetidosNaBarra)
    );
    assert_eq!(
        ler(&com_acoes(&[gerenciar("11")])),
        Err(crate::marcadores_processo::ErroMarcadores::EnderecoDosMarcadoresDeOutroProcesso)
    );
    let curto = link_dos_marcadores("10").replace(HASH_MARCADORES, "66");
    assert_eq!(
        ler(&com_acoes(&[acao_da_barra(&curto)])),
        Err(crate::marcadores_processo::ErroMarcadores::EnderecoDosMarcadoresNaoReconhecido)
    );
    let absoluto = format!("{ORIGEM}/{}", link_dos_marcadores("10"));
    assert_eq!(
        ler(&com_acoes(&[acao_da_barra(&absoluto)])),
        Err(crate::marcadores_processo::ErroMarcadores::EnderecoDosMarcadoresNaoReconhecido)
    );
    let duas_acoes =
        link_dos_marcadores("10").replace("&arvore=1", "&acao=andamento_marcador_gerenciar");
    assert_eq!(
        ler(&com_acoes(&[acao_da_barra(&duas_acoes)])),
        Err(crate::marcadores_processo::ErroMarcadores::EnderecoDosMarcadoresNaoReconhecido)
    );
}

#[test]
fn barra_fora_da_forma_ou_repetida_e_recusa() {
    let fora = base().replace("Nos[0].acoes = '';", "Nos[0].acoes = montar();");
    assert_eq!(
        ler(&fora),
        Err(crate::marcadores_processo::ErroMarcadores::BarraDoProcessoForaDaFormaObservada)
    );
    let repetida = com_acoes(&[gerenciar("10")]).replacen(
        "Nos[0].src = '';",
        "Nos[0].acoes = '';\nNos[0].src = '';",
        1,
    );
    assert_eq!(
        ler(&repetida),
        Err(crate::marcadores_processo::ErroMarcadores::BarrasDoProcessoRepetidasNaArvore)
    );
}
