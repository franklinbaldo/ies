//! RED/GREEN da ação de concluir na barra e da função `concluirProcesso()`
//! na tela do nó, com fixtures SINTÉTICAS na estrutura observada (UFRJ
//! 4.1.5 e GO/PEN 5.0 para o link; `UnB` 4.0.9 e PEN1 4.0.12 para a
//! função). Hosts, hashes e ids inventados.
use super::concluir::{
    ConcluirNaBarra, ErroConcluirNaBarra, ErroFuncaoDeConcluir, FuncaoForaDaForma,
    ler_concluir_da_barra, ler_funcao_de_concluir,
};
use super::tests::{NUP, arvore, no_documento, no_processo};
use super::tests_tela::instalacao;
use crate::propriedades_processo::tests_apoio::{acao_da_barra, com_barra};

pub(crate) const HASH_CONCLUIR: &str =
    "5555555555555555555555555555555555555555555555555555555555555555";
pub(crate) const HASH_RAIZ: &str =
    "6666666666666666666666666666666666666666666666666666666666666666";
pub(crate) const HASH_FUNCAO: &str =
    "8888888888888888888888888888888888888888888888888888888888888888";
const CONTEXTO: &str = "infra_sistema=100000100&infra_unidade_atual=110000001";
const ORIGEM: &str = "https://sei.exemplo.gov.br/sei";

/// O link relativo da tela "Conclusão de Processo", como a barra o publica.
pub(crate) fn link_de_concluir(id: &str) -> String {
    format!(
        "controlador.php?acao=procedimento_concluir&acao_origem=arvore_visualizar&acao_retorno=arvore_visualizar&id_procedimento={id}&arvore=1&{CONTEXTO}&infra_hash={HASH_CONCLUIR}"
    )
}

/// O `href` assinado da tela do nó (`a3` do `Nos[0]` `PROCESSO`).
pub(crate) fn link_da_tela_do_no(id: &str) -> String {
    format!(
        "controlador.php?acao=arvore_visualizar&acao_origem=procedimento_visualizar&id_procedimento={id}&{CONTEXTO}&infra_hash={HASH_RAIZ}"
    )
}

/// A URL do corpo de `concluirProcesso()` (forma 4.0).
pub(crate) fn link_da_funcao(id: &str) -> String {
    format!(
        "controlador.php?acao=procedimento_concluir&acao_origem=arvore_visualizar&id_procedimento={id}&atualizar_arvore=1&{CONTEXTO}&infra_hash={HASH_FUNCAO}"
    )
}

pub(crate) fn absoluta(relativa: &str) -> String {
    format!("{ORIGEM}/{relativa}")
}

/// O `<a>` da forma 4.0, como servido.
pub(crate) const ACAO_FUNCAO: &str = "<a href=\"#\" onclick=\"concluirProcesso();\" tabindex=\"451\" ><img src=\"svg/processo_concluir.svg?11\" alt=\"Concluir Processo\" title=\"Concluir Processo\" /></a>";

/// Uma ação vizinha qualquer, para a barra não ter só a conclusão (não a
/// de enviar, que `processo enviar` lê: os testes dele a põem uma vez).
fn vizinha() -> String {
    acao_da_barra(&format!(
        "controlador.php?acao=procedimento_enviar_email&acao_origem=arvore_visualizar&id_procedimento=10&arvore=1&{CONTEXTO}&infra_hash={}",
        "4".repeat(64)
    ))
}

/// Árvore do processo `id` com a raiz assinada, `html` na situação e a
/// barra com `acoes` (entre vizinhas).
pub(crate) fn arvore_para_concluir(
    id: &str,
    nup: &str,
    situacao: &str,
    acoes: &[String],
) -> String {
    let base = arvore(&[
        no_processo(0, id, nup),
        no_documento(1, "21", id, "0000101"),
    ])
    .replace(
        &format!("\"controlador.php?acao=arvore_visualizar&id_procedimento={id}\""),
        &format!("\"{}\"", link_da_tela_do_no(id)),
    )
    .replace("Nos[0].html = '';", &format!("Nos[0].html = '{situacao}';"));
    let mut barra = vizinha();
    for acao in acoes {
        barra.push_str(acao);
    }
    barra.push_str(&vizinha());
    com_barra(&base, 0, &barra)
}

pub(crate) fn pela_tela(id: &str) -> String {
    acao_da_barra(&link_de_concluir(id))
}

/// Tela do nó (forma 4.0) com as funções vizinhas e `concluirProcesso()`
/// na forma de três linhas.
pub(crate) fn tela_do_no(id: &str) -> String {
    format!(
        "<html><head><title>SEI - Visualizar Árvore</title><script type=\"text/javascript\">\n\
         function excluirProcesso(){{\n  if (confirm('Confirma exclusão do processo?')){{\n   location.href = 'controlador.php?acao=procedimento_excluir&id_procedimento={id}&{CONTEXTO}&infra_hash={h}';\n  }}\n}}\n\
         function concluirProcesso(){{\n  //if (confirm('Confirma conclusão do processo?')){{\n   location.href = '{url}';\n  //}}\n}}\n\
         function reabrirProcesso(){{\n  //if (confirm('Confirma reabertura do processo?')){{\n   location.href = 'controlador.php?acao=procedimento_reabrir&acao_origem=arvore_visualizar&id_procedimento={id}&atualizar_arvore=1&{CONTEXTO}&infra_hash={h}';\n  //}}\n}}\n\
         </script></head><body></body></html>",
        h = "7".repeat(64),
        url = link_da_funcao(id),
    )
}

#[test]
fn link_da_tela_de_conclusao_na_barra() {
    let html = arvore_para_concluir("10", NUP, "", &[pela_tela("10")]);
    assert_eq!(
        ler_concluir_da_barra(&instalacao(), &html, "10")
            .unwrap()
            .map(|c| match c {
                ConcluirNaBarra::PelaTela(url) => url.as_str().to_owned(),
                ConcluirNaBarra::PelaFuncao { .. } => panic!("forma errada"),
            }),
        Some(absoluta(&link_de_concluir("10")))
    );
}

#[test]
fn funcao_na_barra_leva_a_tela_do_no_da_raiz() {
    let html = arvore_para_concluir("10", NUP, "", &[ACAO_FUNCAO.to_owned()]);
    let Some(ConcluirNaBarra::PelaFuncao { tela_do_no }) =
        ler_concluir_da_barra(&instalacao(), &html, "10").unwrap()
    else {
        panic!("forma errada");
    };
    assert_eq!(tela_do_no.as_str(), absoluta(&link_da_tela_do_no("10")));
    assert!(!format!("{tela_do_no:?}").contains(HASH_RAIZ));
}

#[test]
fn barra_sem_conclusao_ou_sem_barra_nao_oferece() {
    let sem = arvore_para_concluir("10", NUP, "", &[]);
    assert_eq!(ler_concluir_da_barra(&instalacao(), &sem, "10"), Ok(None));
    let vazia = arvore(&[no_processo(0, "10", NUP)]);
    assert_eq!(ler_concluir_da_barra(&instalacao(), &vazia, "10"), Ok(None));
}

#[test]
fn formas_repetidas_e_enderecos_recusados() {
    let ler = |acoes: &[String]| {
        ler_concluir_da_barra(
            &instalacao(),
            &arvore_para_concluir("10", NUP, "", acoes),
            "10",
        )
    };
    let erro = Err(ErroConcluirNaBarra::FormasDeConcluirRepetidasNaBarra);
    assert_eq!(ler(&[pela_tela("10"), ACAO_FUNCAO.to_owned()]), erro);
    assert_eq!(ler(&[pela_tela("10"), pela_tela("10")]), erro);
    assert_eq!(ler(&[ACAO_FUNCAO.to_owned(), ACAO_FUNCAO.to_owned()]), erro);
    assert_eq!(
        ler(&[pela_tela("11")]),
        Err(ErroConcluirNaBarra::EnderecoDaConclusaoDeOutroProcesso)
    );
    let sem_hash = acao_da_barra(&link_de_concluir("10").replace(HASH_CONCLUIR, "abc"));
    assert_eq!(
        ler(&[sem_hash]),
        Err(ErroConcluirNaBarra::EnderecoDaConclusaoNaoReconhecido)
    );
    // Forma 4.0 com a raiz sem `href` assinado (a forma da fixture comum).
    let raiz_crua = com_barra(&arvore(&[no_processo(0, "10", NUP)]), 0, ACAO_FUNCAO);
    assert_eq!(
        ler_concluir_da_barra(&instalacao(), &raiz_crua, "10"),
        Err(ErroConcluirNaBarra::EnderecoDaTelaDoNoNaoReconhecido)
    );
}

#[test]
fn onclick_diferente_nao_e_a_funcao_de_concluir() {
    let outra = ACAO_FUNCAO.replace("concluirProcesso();", "reabrirProcesso();");
    assert_eq!(
        ler_concluir_da_barra(
            &instalacao(),
            &arvore_para_concluir("10", NUP, "", &[outra]),
            "10"
        ),
        Ok(None)
    );
}

#[test]
fn url_da_funcao_na_tela_do_no() {
    let url = ler_funcao_de_concluir(&instalacao(), &tela_do_no("10"), "10").unwrap();
    assert_eq!(url.as_str(), absoluta(&link_da_funcao("10")));
    assert!(!format!("{url:?}").contains(HASH_FUNCAO));
}

#[test]
fn funcao_ausente_fora_da_forma_ou_repetida() {
    let ler = |html: &str| ler_funcao_de_concluir(&instalacao(), html, "10");
    let tela = tela_do_no("10");
    let sem = tela.replace("function concluirProcesso(){", "function outraCoisa(){");
    assert_eq!(
        ler(&sem),
        Err(ErroFuncaoDeConcluir::FuncaoDeConcluirAusenteNaTelaDoNo)
    );
    let ativo = tela.replace(
        "//if (confirm('Confirma conclusão",
        "if (confirm('Confirma conclusão",
    );
    // O `confirm` ativo foi observado na `UnB` 4.0.9 (retirada do
    // sobrestamento, live de 10/11/2026): a mesma forma, aceita.
    assert!(ler(&ativo).is_ok());
    let duas = tela.replace("</script>", &format!("{}</script>", funcao_extra()));
    assert_eq!(
        ler(&duas),
        Err(ErroFuncaoDeConcluir::FuncoesDeConcluirRepetidasNaTelaDoNo)
    );
    let fora = tela.replace(
        "//if (confirm('Confirma conclusão",
        "//if (window.confirm('Confirma conclusão",
    );
    let erro = ler(&fora).unwrap_err();
    assert_eq!(
        erro,
        ErroFuncaoDeConcluir::FuncaoDeConcluirForaDaFormaObservada(FuncaoForaDaForma)
    );
    assert_eq!(
        std::error::Error::source(&erro).and_then(|c| c.downcast_ref::<FuncaoForaDaForma>()),
        Some(&FuncaoForaDaForma)
    );
    let outro = tela.replace(
        &format!("id_procedimento=10&atualizar_arvore=1&{CONTEXTO}&infra_hash={HASH_FUNCAO}"),
        &format!("id_procedimento=11&atualizar_arvore=1&{CONTEXTO}&infra_hash={HASH_FUNCAO}"),
    );
    assert_eq!(
        ler(&outro),
        Err(ErroFuncaoDeConcluir::EnderecoDaConclusaoDeOutroProcesso)
    );
    let reabrir = tela.replace(
        &link_da_funcao("10"),
        &link_da_funcao("10").replace("procedimento_concluir", "procedimento_reabrir"),
    );
    assert_eq!(
        ler(&reabrir),
        Err(ErroFuncaoDeConcluir::EnderecoDaConclusaoNaoReconhecido)
    );
}

fn funcao_extra() -> String {
    format!(
        "function concluirProcesso(){{\n  //if (confirm('Confirma conclusão do processo?')){{\n   location.href = '{}';\n  //}}\n}}\n",
        link_da_funcao("10")
    )
}
