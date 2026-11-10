//! RED/GREEN da assinatura na árvore (Refs #174): a ação da barra, a caneta
//! nas duas formas (4.0/UFRJ/PEN2 e GO/PEN3/PEN4), a tela do documento e o
//! link `documento_assinar` nas formas 4.0 e 4.1/5.0, na estrutura das
//! cassettes `documento-escritas-*`. HTML sintético; nenhum valor real.
use super::assinatura::{
    Assinante, ErroAssinaturaNaArvore, ErroLinkDeAssinar, ler_assinatura_na_arvore,
    ler_link_de_assinar, tela_do_documento,
};
use super::tests::{NUP, arvore, no_documento, no_processo};
use super::tests_barra_documento::{barra, com_barra};
use super::tests_concluir::absoluta;
use super::tests_tela::instalacao;

const CONTEXTO: &str = "infra_sistema=100000100&infra_unidade_atual=110000001";
pub(crate) const HASH_TELA_DOC: &str =
    "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
pub(crate) const HASH_ASSINAR: &str =
    "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";

/// O `<a>` "Assinar Documento" da barra, como servido.
pub(crate) const ACAO_ASSINAR: &str = "<a href=\"#\" onclick=\"assinarDocumento();\" tabindex=\"453\" ><img  src=\"svg/documento_assinar.svg?11\" alt=\"Assinar Documento\" title=\"Assinar Documento\"/></a>";

/// O `href` assinado da tela do documento (a3 do nó `DOCUMENTO`).
pub(crate) fn link_da_tela_do_documento(processo: &str, documento: &str) -> String {
    format!(
        "controlador.php?acao=arvore_visualizar&acao_origem=procedimento_visualizar&id_procedimento={processo}&id_documento={documento}&{CONTEXTO}&infra_hash={HASH_TELA_DOC}"
    )
}

/// O link da tela "Assinatura de Documento".
pub(crate) fn link_de_assinar(processo: &str, documento: &str) -> String {
    format!(
        "controlador.php?acao=documento_assinar&acao_origem=arvore_visualizar&id_procedimento={processo}&id_documento={documento}&arvore=1&{CONTEXTO}&infra_hash={HASH_ASSINAR}"
    )
}

/// A caneta do documento `id`; `forma40`: a4 `javascript:alert`, a5 `null`.
pub(crate) fn caneta(
    j: usize,
    id: &str,
    nome: &str,
    cargo: &str,
    unidade: &str,
    forma40: bool,
) -> String {
    let (a4, a5) = if forma40 {
        (
            format!("javascript:alert('Assinado por:\\\\n{nome}\\\\n{cargo}\\\\n{unidade}');"),
            "null".to_owned(),
        )
    } else {
        (
            format!(
                "controlador.php?acao=assinatura_listar&acao_origem=procedimento_visualizar&id_documento={id}&{CONTEXTO}&infra_hash={}",
                "e".repeat(64)
            ),
            "\"ifrVisualizacao\"".to_owned(),
        )
    };
    format!(
        "NosAcoes[{j}] = new infraArvoreAcao(\"ASSINATURA\",\"A{id}\",\"{id}\",\"{a4}\",{a5},\"Assinado por:\\n{nome}\\n{cargo}\\n{unidade}\",\"svg/assinatura1.svg?11\",true);"
    )
}

/// Árvore do processo `processo` com o documento `documento` (número
/// `numero`), a tela assinada no a3, a barra `acoes` e as `canetas`.
pub(crate) fn arvore_assinavel(
    processo: &str,
    documento: &str,
    numero: &str,
    acoes: &str,
    canetas: &[String],
) -> String {
    let base = arvore(&[
        no_processo(0, processo, NUP),
        no_documento(1, documento, processo, numero),
    ])
    .replacen(
        "\"about:blank\"",
        &format!("\"{}\"", link_da_tela_do_documento(processo, documento)),
        1,
    );
    let mut html = com_barra(&base, 1, acoes);
    let linhas: String = canetas.iter().fold(String::new(), |mut t, c| {
        t.push_str("    ");
        t.push_str(c);
        t.push('\n');
        t
    });
    html = html.replacen(
        "  if (Nos.length){",
        &format!("{linhas}  if (Nos.length){{"),
        1,
    );
    html
}

/// Barra interna com edição `edicao` e a ação de assinar.
pub(crate) fn barra_assinavel(processo: &str, documento: &str, edicao: Option<char>) -> String {
    format!("{}{ACAO_ASSINAR}", barra(processo, documento, edicao))
}

/// Tela do documento na forma 4.0 (função de três linhas).
pub(crate) fn tela_do_documento_40(processo: &str, documento: &str) -> String {
    format!(
        "<html><head><title>SEI - Visualizar Árvore</title><script type=\"text/javascript\">\n\
         function cienciaDocumento(){{\n  //if (confirm('Confirma ciência no documento?')){{\n   location.href = 'controlador.php?acao=documento_ciencia&id_documento={documento}&infra_hash={h}';\n  //}}\n}}\n\
           function assinarDocumento(){{\n    parent.infraAbrirJanelaModal('{url}',600,450);\n  }}\n\
         </script></head><body></body></html>",
        h = "7".repeat(64),
        url = link_de_assinar(processo, documento),
    )
}

/// Tela do documento na forma 4.1/5.0 (variável).
pub(crate) fn tela_do_documento_50(processo: &str, documento: &str) -> String {
    format!(
        "<html><head><title>SEI - Visualizar Árvore</title><script type=\"text/javascript\">\n\
         var linkCienciaDocumento = 'controlador.php?acao=documento_ciencia&id_documento={documento}&infra_hash={h}';\n\
         var linkAssinarDocumento = '{url}';\n\
         var linkDocumentoEnviarEmail = 'controlador.php?acao=documento_enviar_email&id_documento={documento}&infra_hash={h}';\n\
         </script></head><body></body></html>",
        h = "7".repeat(64),
        url = link_de_assinar(processo, documento),
    )
}

fn assinante(nome: &str, cargo: &str, unidade: &str) -> Assinante {
    Assinante {
        nome: nome.into(),
        cargo: cargo.into(),
        unidade: unidade.into(),
    }
}

#[test]
fn barra_publica_assinar_sem_caneta() {
    let html = arvore_assinavel(
        "10",
        "21",
        "0000101",
        &barra_assinavel("10", "21", Some('N')),
        &[],
    );
    let lida = ler_assinatura_na_arvore(&html, "21").unwrap();
    assert!(lida.publicada);
    assert_eq!(lida.assinantes, Vec::<Assinante>::new());
}

#[test]
fn caneta_nas_duas_formas_traz_o_assinante() {
    for forma40 in [true, false] {
        let canetas = [caneta(
            0,
            "21",
            "Usuário Teste",
            "Assessor(a)",
            "TESTE",
            forma40,
        )];
        let html = arvore_assinavel(
            "10",
            "21",
            "0000101",
            &barra_assinavel("10", "21", Some('S')),
            &canetas,
        );
        let lida = ler_assinatura_na_arvore(&html, "21").unwrap();
        assert_eq!(
            lida.assinantes,
            vec![assinante("Usuário Teste", "Assessor(a)", "TESTE")],
            "{forma40}"
        );
    }
}

#[test]
fn caneta_de_outro_documento_nao_conta() {
    let canetas = [caneta(0, "22", "Outro", "Cargo", "TESTE", true)];
    let html = arvore_assinavel(
        "10",
        "21",
        "0000101",
        &barra_assinavel("10", "21", None),
        &canetas,
    );
    assert_eq!(
        ler_assinatura_na_arvore(&html, "21").unwrap().assinantes,
        Vec::<Assinante>::new()
    );
}

#[test]
fn caneta_fora_da_forma_ou_repetida_e_recusa() {
    let dois = caneta(0, "21", "A", "Cargo", "TESTE", true)
        .replace("\\nTESTE\",", "\\nTESTE\\nB\\nCargo\\nTESTE\",");
    let html = arvore_assinavel(
        "10",
        "21",
        "0000101",
        &barra_assinavel("10", "21", None),
        &[dois],
    );
    assert_eq!(
        ler_assinatura_na_arvore(&html, "21"),
        Err(ErroAssinaturaNaArvore::AssinaturasDoDocumentoForaDaFormaObservada)
    );
    let duas = [
        caneta(0, "21", "A", "Cargo", "TESTE", true),
        caneta(1, "21", "B", "Cargo", "TESTE", false),
    ];
    let html = arvore_assinavel(
        "10",
        "21",
        "0000101",
        &barra_assinavel("10", "21", None),
        &duas,
    );
    assert_eq!(
        ler_assinatura_na_arvore(&html, "21"),
        Err(ErroAssinaturaNaArvore::AssinaturasDoDocumentoRepetidasNaArvore)
    );
}

#[test]
fn barra_sem_assinar_ou_com_duas() {
    let html = arvore_assinavel("10", "21", "0000101", &barra("10", "21", Some('N')), &[]);
    assert!(!ler_assinatura_na_arvore(&html, "21").unwrap().publicada);
    let dupla = format!("{}{ACAO_ASSINAR}", barra_assinavel("10", "21", None));
    let html = arvore_assinavel("10", "21", "0000101", &dupla, &[]);
    assert_eq!(
        ler_assinatura_na_arvore(&html, "21"),
        Err(ErroAssinaturaNaArvore::AcoesDeAssinarRepetidasNaBarra)
    );
}

#[test]
fn tela_do_documento_e_o_a3_conferido() {
    let html = arvore_assinavel(
        "10",
        "21",
        "0000101",
        &barra_assinavel("10", "21", None),
        &[],
    );
    let url = tela_do_documento(&instalacao(), &html, "21", "10").unwrap();
    assert_eq!(
        url.as_str(),
        absoluta(&link_da_tela_do_documento("10", "21"))
    );
    assert!(tela_do_documento(&instalacao(), &html, "21", "11").is_none());
    assert!(tela_do_documento(&instalacao(), &html, "22", "10").is_none());
}

#[test]
fn link_de_assinar_nas_duas_formas() {
    for tela in [
        tela_do_documento_40("10", "21"),
        tela_do_documento_50("10", "21"),
    ] {
        let url = ler_link_de_assinar(&instalacao(), &tela, "21", "10").unwrap();
        assert_eq!(url.as_str(), absoluta(&link_de_assinar("10", "21")));
        assert_eq!(
            ler_link_de_assinar(&instalacao(), &tela, "22", "10"),
            Err(ErroLinkDeAssinar::EnderecoDeAssinarDeOutroDocumento)
        );
    }
}

#[test]
fn link_de_assinar_ausente_repetido_ou_fora_da_forma() {
    let sem = tela_do_documento_50("10", "21").replace("var linkAssinarDocumento", "var linkOutro");
    assert_eq!(
        ler_link_de_assinar(&instalacao(), &sem, "21", "10"),
        Err(ErroLinkDeAssinar::LinkDeAssinarAusenteNaTelaDoDocumento)
    );
    let dois = tela_do_documento_50("10", "21").replace(
        "</script>",
        &format!(
            "var linkAssinarDocumento = '{}';\n</script>",
            link_de_assinar("10", "21")
        ),
    );
    assert_eq!(
        ler_link_de_assinar(&instalacao(), &dois, "21", "10"),
        Err(ErroLinkDeAssinar::LinksDeAssinarRepetidosNaTelaDoDocumento)
    );
    let fora = tela_do_documento_40("10", "21").replace(",600,450);", ",700,450);");
    assert_eq!(
        ler_link_de_assinar(&instalacao(), &fora, "21", "10"),
        Err(ErroLinkDeAssinar::LinkDeAssinarForaDaFormaObservada)
    );
    let sem_hash =
        tela_do_documento_50("10", "21").replace(&format!("&infra_hash={HASH_ASSINAR}"), "");
    assert_eq!(
        ler_link_de_assinar(&instalacao(), &sem_hash, "21", "10"),
        Err(ErroLinkDeAssinar::EnderecoDeAssinarNaoReconhecido)
    );
}
