//! RED/GREEN da barra do `DOCUMENTO` (Refs #88): o link assinado de
//! `documento_versao_listar` e `editarConteudo('N'|'S')`, na forma das
//! cassettes `liberacoes-e-versoes-leitura-*` (sete instalações). HTML
//! sintético; nenhum valor real.
use super::barra_do_documento::{
    BarraDoDocumento, EdicaoNaBarra, ErroBarraDoDocumento as E, ler_barra_do_documento,
};
use super::tests::{NUP, arvore, no_documento, no_processo};
use super::tests_tela::instalacao;
use crate::erro_html::ErroHtml;

pub(crate) const HASH_VERSOES: &str =
    "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

/// O `href` "Versões do Documento" da barra, na forma servida.
pub(crate) fn href_versoes(processo: &str, documento: &str) -> String {
    format!(
        "controlador.php?acao=documento_versao_listar&acao_origem=arvore_visualizar&acao_retorno=arvore_visualizar&id_procedimento={processo}&id_documento={documento}&arvore=1&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={HASH_VERSOES}"
    )
}

/// Barra de documento interno com o link das versões e a edição `edicao`
/// (`Some('N')`, `Some('S')` ou sem o link de edição).
pub(crate) fn barra(processo: &str, documento: &str, edicao: Option<char>) -> String {
    let editar = edicao.map_or_else(String::new, |c| {
        format!(
            "<a href=\"#\" onclick=\"editarConteudo(\\'{c}\\');\" tabindex=\"452\"><img src=\"svg/documento_editar_conteudo.svg?25\" title=\"Editar Conteúdo\"/></a>"
        )
    });
    format!(
        "<a href=\"controlador.php?acao=documento_alterar&id_procedimento={processo}&id_documento={documento}&infra_hash={HASH_VERSOES}\" tabindex=\"451\"><img title=\"Consultar/Alterar Documento\"/></a>{editar}<a href=\"{}\" tabindex=\"455\"><img src=\"svg/documento_versoes.svg?25\" alt=\"Versões do Documento\" title=\"Versões do Documento\"/></a>",
        href_versoes(processo, documento)
    )
}

/// Troca a barra vazia do nó `i` (gerada por `arvore`) por `texto`.
pub(crate) fn com_barra(html: &str, i: usize, texto: &str) -> String {
    html.replacen(
        &format!("Nos[{i}].acoes = '';"),
        &format!("Nos[{i}].acoes = '{texto}';"),
        1,
    )
}

fn base() -> String {
    arvore(&[
        no_processo(0, "10", NUP),
        no_documento(1, "21", "10", "0000101"),
    ])
}

fn ler(html: &str) -> Result<BarraDoDocumento, E> {
    ler_barra_do_documento(&instalacao(), html, "21", "10")
}

#[test]
fn link_das_versoes_e_edicao_nao_assinada_sao_lidos_sem_expor_url() {
    let lida = ler(&com_barra(&base(), 1, &barra("10", "21", Some('N')))).unwrap();
    assert_eq!(lida.edicao, EdicaoNaBarra::NaoAssinado);
    let url = lida.versoes.expect("link publicado");
    assert_eq!(
        url.as_str(),
        format!(
            "https://sei.exemplo.gov.br/sei/{}",
            href_versoes("10", "21")
        )
    );
    assert!(!format!("{url:?}").contains(HASH_VERSOES));
}

#[test]
fn assinado_sem_edicao_e_sem_link() {
    let assinado = ler(&com_barra(&base(), 1, &barra("10", "21", Some('S')))).unwrap();
    assert_eq!(assinado.edicao, EdicaoNaBarra::Assinado);
    assert!(assinado.versoes.is_some());
    let sem_edicao = ler(&com_barra(&base(), 1, &barra("10", "21", None))).unwrap();
    assert_eq!(sem_edicao.edicao, EdicaoNaBarra::NaoPublicada);
    let vazia = ler(&base()).unwrap();
    assert_eq!(
        vazia,
        BarraDoDocumento {
            versoes: None,
            edicao: EdicaoNaBarra::NaoPublicada
        }
    );
    let outro = barra("10", "21", None).replace("editarConteudo", "x");
    let outra_forma = barra("10", "21", Some('X'));
    assert!(
        ler(&com_barra(&base(), 1, &outro))
            .unwrap()
            .versoes
            .is_some()
    );
    assert_eq!(
        ler(&com_barra(&base(), 1, &outra_forma)).unwrap().edicao,
        EdicaoNaBarra::ForaDaForma
    );
}

#[test]
fn link_de_outro_documento_ou_processo_e_repetido_sao_recusas() {
    let de_outro = com_barra(&base(), 1, &barra("10", "22", Some('N')));
    assert_eq!(ler(&de_outro), Err(E::EnderecoDasVersoesDeOutroDocumento));
    let de_outro_processo = com_barra(&base(), 1, &barra("11", "21", Some('N')));
    assert_eq!(
        ler(&de_outro_processo),
        Err(E::EnderecoDasVersoesDeOutroDocumento)
    );
    let dois = format!("{0}{0}", barra("10", "21", None));
    assert_eq!(
        ler(&com_barra(&base(), 1, &dois)),
        Err(E::Html(ErroHtml::repetido(
            "link_das_versoes_repetido_na_barra"
        )))
    );
    let sem_hash = barra("10", "21", None).replace(
        &format!("&infra_hash={HASH_VERSOES}\" tabindex=\"455"),
        "\" tabindex=\"455",
    );
    assert_eq!(
        ler(&com_barra(&base(), 1, &sem_hash)),
        Err(E::Html(ErroHtml::valor("link_das_versoes_nao_reconhecido")))
    );
}

#[test]
fn barra_repetida_e_concatenacao_vazia() {
    let html = com_barra(&base(), 1, &barra("10", "21", Some('N')));
    let concat = html.replacen(
        "Nos[1].src = '';",
        "Nos[1].acoes = Nos[1].acoes.concat('');\nNos[1].src = '';",
        1,
    );
    assert!(ler(&concat).unwrap().versoes.is_some());
    let repetida = html.replacen(
        "Nos[1].src = '';",
        "Nos[1].acoes = '';\nNos[1].src = '';",
        1,
    );
    assert_eq!(ler(&repetida), Err(E::BarrasRepetidasNaArvore));
    // A barra do PROCESSO (nó 0) não é a do documento.
    let na_raiz = com_barra(&base(), 0, &barra("10", "21", Some('N')));
    assert_eq!(ler(&na_raiz).unwrap().versoes, None);
}
