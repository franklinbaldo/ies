//! RED/GREEN da ação de reabrir na barra e da URL dela na tela do nó, com
//! fixtures SINTÉTICAS na estrutura observada: a barra com
//! `reabrirProcesso()` (sete instalações), a função de três linhas (`UnB`
//! 4.0.9, PEN1 4.0.12) e a linha `var linkReabrirProcesso` (UFRJ 4.1.5,
//! PEN2 4.1.5, GO 5.0.3, PEN3 e PEN4 5.0.4). Hosts, hashes e ids inventados.
use super::reabrir::{
    ErroEnderecoDeReabrir, FormaDaReabertura, ler_endereco_de_reabrir, ler_reabrir_da_barra,
};
use super::tests::{NUP, arvore, no_processo};
use super::tests_concluir::{
    ACAO_FUNCAO, absoluta, arvore_para_concluir, link_da_tela_do_no, pela_tela, tela_do_no,
};
use super::tests_tela::instalacao;
use crate::erro_html::ErroHtml;
use crate::propriedades_processo::tests_apoio::com_barra;

pub(crate) const HASH_REABRIR: &str =
    "3333333333333333333333333333333333333333333333333333333333333333";
const CONTEXTO: &str = "infra_sistema=100000100&infra_unidade_atual=110000001";

/// O `<a>` de reabrir, como as sete instalações o servem.
pub(crate) const ACAO_REABRIR: &str = "<a href=\"#\" onclick=\"reabrirProcesso();\" tabindex=\"451\" ><img src=\"svg/processo_reabrir.svg?11\" alt=\"Reabrir Processo\" title=\"Reabrir Processo\" /></a>";

/// A URL de reabrir publicada na tela do nó (nas duas formas).
pub(crate) fn link_de_reabrir(id: &str) -> String {
    format!(
        "controlador.php?acao=procedimento_reabrir&acao_origem=arvore_visualizar&id_procedimento={id}&atualizar_arvore=1&{CONTEXTO}&infra_hash={HASH_REABRIR}"
    )
}

/// Tela do nó da forma 4.0, com `reabrirProcesso()` na forma de três linhas.
pub(crate) fn tela_do_no_pela_funcao(id: &str) -> String {
    let fixa = format!(
        "controlador.php?acao=procedimento_reabrir&acao_origem=arvore_visualizar&id_procedimento={id}&atualizar_arvore=1&{CONTEXTO}&infra_hash={}",
        "7".repeat(64)
    );
    tela_do_no(id)
        .replace(&fixa, &link_de_reabrir(id))
        .replace("Confirma reabertura", "Confirma reabetura")
}

/// Tela do nó da forma 4.1/5.0: as variáveis `link…` numa linha cada, sem
/// as funções (servidas num `.js` à parte).
pub(crate) fn tela_do_no_pela_variavel(id: &str) -> String {
    format!(
        "<html><head><title>SEI - </title>\n<script type=\"text/javascript\" charset=\"iso-8859-1\" src=\"js/arvore_visualizar.js?4.1.5\"></script>\n\
         <script type=\"text/javascript\">\n\
         var linkExcluirProcesso = 'controlador.php?acao=procedimento_excluir&acao_origem=arvore_visualizar&id_procedimento={id}&{CONTEXTO}&infra_hash={h}';\n\
         var linkRemoverSobrestamentoProcesso = 'controlador.php?acao=procedimento_remover_sobrestamento&acao_origem=arvore_visualizar&id_procedimento={id}&atualizar_arvore=1&{CONTEXTO}&infra_hash={h}';\n\
         var linkReabrirProcesso = '{url}';\n\
         var linkExcluirDocumento = 'controlador.php?acao=documento_excluir&acao_origem=arvore_visualizar&id_procedimento={id}&id_documento=&atualizar_arvore=1&{CONTEXTO}&infra_hash={h}';\n\
         </script></head><body></body></html>",
        h = "7".repeat(64),
        url = link_de_reabrir(id),
    )
}

fn barra(acoes: &[String]) -> String {
    arvore_para_concluir("10", NUP, "", acoes)
}

#[test]
fn reabrir_na_barra_leva_a_tela_do_no_da_raiz() {
    let html = barra(&[ACAO_REABRIR.to_owned()]);
    let lido = ler_reabrir_da_barra(&instalacao(), &html, "10")
        .unwrap()
        .expect("barra oferece reabrir");
    assert_eq!(
        lido.tela_do_no.as_str(),
        absoluta(&link_da_tela_do_no("10"))
    );
}

#[test]
fn barra_sem_reabrir_nao_oferece() {
    for acoes in [vec![], vec![pela_tela("10")], vec![ACAO_FUNCAO.to_owned()]] {
        assert_eq!(
            ler_reabrir_da_barra(&instalacao(), &barra(&acoes), "10"),
            Ok(None)
        );
    }
    let vazia = arvore(&[no_processo(0, "10", NUP)]);
    assert_eq!(ler_reabrir_da_barra(&instalacao(), &vazia, "10"), Ok(None));
}

#[test]
fn reabrir_repetido_ou_raiz_sem_tela_do_no() {
    let dois = barra(&[ACAO_REABRIR.to_owned(), ACAO_REABRIR.to_owned()]);
    assert_eq!(
        ler_reabrir_da_barra(&instalacao(), &dois, "10"),
        Err(ErroHtml::repetido("funcoes_de_reabrir_repetidas_na_barra"))
    );
    let raiz_crua = com_barra(&arvore(&[no_processo(0, "10", NUP)]), 0, ACAO_REABRIR);
    assert_eq!(
        ler_reabrir_da_barra(&instalacao(), &raiz_crua, "10"),
        Err(ErroHtml::valor("endereco_da_tela_do_no_nao_reconhecido"))
    );
}

#[test]
fn url_de_reabrir_pela_funcao_na_tela_do_no() {
    let (url, forma) =
        ler_endereco_de_reabrir(&instalacao(), &tela_do_no_pela_funcao("10"), "10").unwrap();
    assert_eq!(url.as_str(), absoluta(&link_de_reabrir("10")));
    assert_eq!(forma, FormaDaReabertura::Funcao);
    assert!(!format!("{url:?}").contains(HASH_REABRIR));
}

#[test]
fn url_de_reabrir_pela_variavel_na_tela_do_no() {
    let (url, forma) =
        ler_endereco_de_reabrir(&instalacao(), &tela_do_no_pela_variavel("10"), "10").unwrap();
    assert_eq!(url.as_str(), absoluta(&link_de_reabrir("10")));
    assert_eq!(forma, FormaDaReabertura::Variavel);
}

#[test]
fn url_de_reabrir_ausente_fora_da_forma_repetida_ou_recusada() {
    let ler = |html: &str| ler_endereco_de_reabrir(&instalacao(), html, "10");
    let funcao = tela_do_no_pela_funcao("10");
    let variavel = tela_do_no_pela_variavel("10");
    let sem = variavel.replace("var linkReabrirProcesso = ", "var linkOutraCoisa = ");
    assert_eq!(
        ler(&sem),
        Err(ErroEnderecoDeReabrir::Html(ErroHtml::ausente(
            "endereco_de_reabrir_ausente_na_tela_do_no"
        )))
    );
    let ativo = funcao.replace(
        "//if (confirm('Confirma reabetura",
        "if (confirm('Confirma reabetura",
    );
    // O `confirm` ativo foi observado na `UnB` 4.0.9 (retirada do
    // sobrestamento, live de 10/11/2026): a mesma forma, aceita.
    assert!(ler(&ativo).is_ok());
    let por_aspas = variavel.replace(
        &format!("var linkReabrirProcesso = '{}';", link_de_reabrir("10")),
        &format!("var linkReabrirProcesso = \"{}\";", link_de_reabrir("10")),
    );
    assert_eq!(
        ler(&por_aspas),
        Err(ErroEnderecoDeReabrir::Html(ErroHtml::valor(
            "endereco_de_reabrir_fora_da_forma_observada"
        )))
    );
    let linha = format!("var linkReabrirProcesso = '{}';\n", link_de_reabrir("10"));
    let duas = variavel.replace("</script>", &format!("{linha}</script>"));
    assert_eq!(
        ler(&duas),
        Err(ErroEnderecoDeReabrir::Html(ErroHtml::repetido(
            "enderecos_de_reabrir_repetidos_na_tela_do_no"
        )))
    );
    let as_duas_formas = funcao.replace("</script>", &format!("{linha}</script>"));
    assert_eq!(
        ler(&as_duas_formas),
        Err(ErroEnderecoDeReabrir::Html(ErroHtml::repetido(
            "enderecos_de_reabrir_repetidos_na_tela_do_no"
        )))
    );
    let outro = variavel.replace(
        "id_procedimento=10&atualizar",
        "id_procedimento=11&atualizar",
    );
    assert_eq!(
        ler(&outro),
        Err(ErroEnderecoDeReabrir::EnderecoDaReaberturaDeOutroProcesso)
    );
    let concluir = variavel.replace(
        &link_de_reabrir("10"),
        &link_de_reabrir("10").replace("procedimento_reabrir", "procedimento_concluir"),
    );
    assert_eq!(
        ler(&concluir),
        Err(ErroEnderecoDeReabrir::Html(ErroHtml::valor(
            "endereco_da_reabertura_nao_reconhecido"
        )))
    );
}
