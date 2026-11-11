//! RED/GREEN do vínculo de anexação: HTML SINTÉTICO com a forma observada
//! na `UnB` 4.0.9 (03/11/2026) na árvore do próprio anexado: o `.html` do
//! nó raiz é um literal JS com um único `<a>` cujo `href` é o
//! `procedimento_trabalhar` assinado do pai, com `id_procedimento` do pai e
//! `id_procedimento_anexado` igual à raiz, e cujo texto é o NUP do pai.
//! Ids, números e hashes inventados.
use super::tests::{NUP, arvore, no_documento, no_processo};
use super::*;
use crate::instalacao::Instalacao;

pub(crate) const ORIGEM: &str = "https://sei.exemplo.gov.br/sei";
pub(crate) const NUP_PAI: &str = "00009.000009/2026-09";
pub(crate) const ID_PAI: &str = "90";

fn instalacao() -> Instalacao {
    Instalacao::interpretar(ORIGEM).unwrap()
}

/// `href` do link do pai, como servido (`&` cru).
pub(crate) fn href_do_pai(pai: &str, proprio: &str) -> String {
    format!(
        "controlador.php?acao=procedimento_trabalhar&acao_origem=arvore_visualizar&id_procedimento={pai}&id_procedimento_anexado={proprio}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={}",
        "4".repeat(64)
    )
}

/// O `.html` do nó raiz do anexado na forma observada.
pub(crate) fn html_do_anexado(href: &str, texto: &str) -> String {
    format!(
        "<div style=\"font-size:.875rem;display:inline;\">Processo anexado ao processo <a href=\"{href}\" target=\"_blank\" class=\"ancoraVisualizacaoArvore\">{texto}</a>.</div>"
    )
}

/// Árvore do anexado `id` com o `.html` da raiz trocado por `html`.
pub(crate) fn arvore_do_anexado(id: &str, nup: &str, html: &str) -> String {
    arvore(&[
        no_processo(0, id, nup),
        no_documento(1, "21", id, "0000101"),
    ])
    .replace("Nos[0].html = '';", &format!("Nos[0].html = '{html}';"))
}

fn ler(html: &str) -> Result<Option<VinculoDeAnexacao>, super::ErroArvore> {
    ler_vinculo_de_anexacao(&instalacao(), html, "10")
}

fn padrao() -> String {
    arvore_do_anexado(
        "10",
        NUP,
        &html_do_anexado(&href_do_pai(ID_PAI, "10"), NUP_PAI),
    )
}

#[test]
fn le_o_pai_pelo_link_do_no_raiz() {
    let vinculo = ler(&padrao()).unwrap().unwrap();
    assert_eq!(vinculo.numero, NUP_PAI);
    assert_eq!(vinculo.id_procedimento, ID_PAI);
    assert_eq!(
        vinculo.trabalhar.as_str(),
        format!("{ORIGEM}/{}", href_do_pai(ID_PAI, "10"))
    );
    let debug = format!("{vinculo:?}");
    assert!(!debug.contains(&"4".repeat(64)), "{debug}");
    // A árvore continua lida como a de qualquer processo.
    assert!(ler_arvore(&padrao(), NUP, "10").is_ok());
}

#[test]
fn sem_o_link_o_processo_nao_e_anexado() {
    let sigla = "Processo aberto somente na unidade <a class=\"ancoraSigla\">U</a>.<br />";
    let outro_link = "<a href=\"controlador.php?acao=procedimento_trabalhar&id_procedimento=90&infra_hash=x\">y</a>";
    for html in [
        arvore(&[no_processo(0, "10", NUP)]),
        arvore_do_anexado("10", NUP, sigla),
        arvore_do_anexado("10", NUP, outro_link),
        arvore(&[no_processo(0, "10", NUP)]).replace("Nos[0].html = '';\n", ""),
    ] {
        assert_eq!(ler(&html), Ok(None), "{html}");
    }
}

#[test]
fn link_na_linha_que_traz_a_declaracao_seguinte() {
    let html = padrao().replace(
        "';\nNos[1] = new infraArvoreNo(\"DOCUMENTO\"",
        "';Nos[1] = new infraArvoreNo(\"DOCUMENTO\"",
    );
    assert!(html.contains("</div>';Nos[1] = new"));
    assert_eq!(ler(&html).unwrap().unwrap().id_procedimento, ID_PAI);
}

#[test]
fn link_do_pai_fora_da_forma_e_recusa() {
    let href = href_do_pai(ID_PAI, "10");
    let com_href = |h: &str| arvore_do_anexado("10", NUP, &html_do_anexado(h, NUP_PAI));
    let com_texto = |t: &str| arvore_do_anexado("10", NUP, &html_do_anexado(&href, t));
    let forma = super::ErroArvore::VinculoDeAnexacaoForaDaFormaObservada;
    let recusa = super::ErroArvore::EnderecoDoVinculoDeAnexacaoRecusado(
        crate::arvore_processo::endereco::EnderecoRecusado::NaoReconhecido,
    );
    let casos = [
        (com_href(&href_do_pai(ID_PAI, "11")), Err(forma.clone())),
        (com_href(&href_do_pai("10", "10")), Err(forma.clone())),
        (
            com_href(&href.replace(
                "acao=procedimento_trabalhar",
                "acao=procedimento_visualizar",
            )),
            Err(recusa.clone()),
        ),
        (
            com_href(&href.replace(&"4".repeat(64), &"4".repeat(63))),
            Err(recusa.clone()),
        ),
        (com_href(&format!("{ORIGEM}/{href}")), Err(recusa.clone())),
        (
            com_href(&href.replace("&infra_sistema", "&id_procedimento=91&infra_sistema")),
            Err(forma.clone()),
        ),
        (
            com_href(&href.replace(
                "&infra_sistema",
                "&id_procedimento_anexado=10&infra_sistema",
            )),
            Err(forma.clone()),
        ),
        (
            com_href(&href.replace("id_procedimento=90", "id_procedimento=9a")),
            Err(forma.clone()),
        ),
        (com_texto(""), Err(forma.clone())),
        (com_texto(&format!(" {NUP_PAI}")), Err(forma.clone())),
        (com_texto(&format!("{NUP_PAI}&amp;")), Err(forma.clone())),
        (com_texto(&format!("<b>{NUP_PAI}</b>")), Err(forma.clone())),
    ];
    for (html, esperado) in casos {
        assert_eq!(ler(&html), esperado, "{html}");
    }
}

#[test]
fn links_ou_html_repetidos_e_html_nao_literal_sao_recusa() {
    let link = html_do_anexado(&href_do_pai(ID_PAI, "10"), NUP_PAI);
    let dois = arvore_do_anexado("10", NUP, &format!("{link}{link}"));
    assert_eq!(
        ler(&dois),
        Err(super::ErroArvore::VinculosDeAnexacaoRepetidos)
    );
    let duas_linhas = padrao().replace(
        "Nos[0].src = '';",
        &format!("Nos[0].src = '';\nNos[0].html = '{link}';"),
    );
    assert_eq!(
        ler(&duas_linhas),
        Err(super::ErroArvore::VinculosDeAnexacaoRepetidos)
    );
    let nao_literal = arvore(&[no_processo(0, "10", NUP)])
        .replace("Nos[0].html = '';", "Nos[0].html = montar();");
    assert!(matches!(
        ler(&nao_literal),
        Err(super::ErroArvore::HtmlDoNoDoProcessoIlegivel(_))
    ));
}

#[test]
fn vinculo_fora_da_forma_nao_e_divergencia_do_pedido() {
    for erro in [
        super::ErroArvore::VinculoDeAnexacaoForaDaFormaObservada,
        super::ErroArvore::VinculosDeAnexacaoRepetidos,
        super::ErroArvore::HtmlDoNoDoProcessoForaDaFormaObservada,
    ] {
        assert!(!erro.diverge_do_pedido());
    }
}
