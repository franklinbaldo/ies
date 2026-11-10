//! RED/GREEN do link do conteúdo (`Nos[i].src`) e da tela aberta por
//! número SEI (Refs #60). HTML sintético com a forma das cassettes
//! `documento-numero-e-busca-*`: `documento_visualizar` com `id_documento`
//! (interno), `documento_download_anexo` com `id_anexo` (externo) e `''`
//! (sem acesso ou cancelado). Nenhum valor real.
use super::tests::{NUP, arvore, no_documento, no_processo};
use super::tests_tela::{instalacao, tela_do_processo};
use super::*;
use crate::erro_html::ErroHtml;

pub(crate) const HASH_CONTEUDO: &str =
    "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

/// `Nos[i].src` de um documento interno, na forma servida.
pub(crate) fn src_visualizar(id: &str) -> String {
    format!(
        "controlador.php?acao=documento_visualizar&acao_origem=procedimento_visualizar&id_documento={id}&arvore=1&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={HASH_CONTEUDO}"
    )
}

/// `Nos[i].src` de um documento externo.
pub(crate) fn src_anexo() -> String {
    format!(
        "controlador.php?acao=documento_download_anexo&acao_origem=procedimento_visualizar&id_anexo=901&arvore=1&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={HASH_CONTEUDO}"
    )
}

/// Troca o `.src` vazio do nó `i` (gerado por `arvore`) por `src`.
pub(crate) fn com_src(html: &str, i: usize, src: &str) -> String {
    html.replacen(
        &format!("Nos[{i}].src = '';"),
        &format!("Nos[{i}].src = '{src}';"),
        1,
    )
}

/// `src` da árvore na tela aberta pela pesquisa rápida de um número SEI.
pub(crate) fn src_da_arvore_do_documento(processo: &str, documento: &str) -> String {
    format!(
        "controlador.php?acao=procedimento_visualizar&acao_origem=procedimento_trabalhar&acao_retorno=principal&id_procedimento={processo}&id_documento={documento}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={}",
        super::tests_tela::HASH_ARVORE
    )
}

fn base() -> String {
    arvore(&[
        no_processo(0, "10", NUP),
        no_documento(1, "21", "10", "0000101"),
        no_documento(2, "22", "10", "0000102"),
    ])
}

fn ler(html: &str, id: &str) -> Result<ConteudoNaArvore, ErroConteudoNaArvore> {
    ler_conteudo_na_arvore(&instalacao(), html, id)
}

#[test]
fn link_de_documento_interno_conferido_e_redigido() {
    let html = com_src(&base(), 2, &src_visualizar("22"));
    let ConteudoNaArvore::Visualizar(endereco) = ler(&html, "22").unwrap() else {
        panic!("esperado documento_visualizar");
    };
    assert_eq!(
        endereco.as_str(),
        format!("https://sei.exemplo.gov.br/sei/{}", src_visualizar("22"))
    );
    assert!(!format!("{endereco:?}").contains(HASH_CONTEUDO));
}

#[test]
fn src_vazio_e_download_de_anexo() {
    let html = com_src(&base(), 2, &src_anexo());
    assert_eq!(ler(&html, "21"), Ok(ConteudoNaArvore::Vazio));
    let Ok(ConteudoNaArvore::DownloadDeAnexo(endereco)) = ler(&html, "22") else {
        panic!("esperado documento_download_anexo");
    };
    assert_eq!(
        endereco.as_str(),
        format!("https://sei.exemplo.gov.br/sei/{}", src_anexo())
    );
    assert!(!format!("{endereco:?}").contains(HASH_CONTEUDO));
}

/// Fatia 3: o link do anexo é conferido como o do interno, com um único
/// `id_anexo` decimal no lugar do `id_documento` (o link não traz o id do
/// documento; a ligação é o nó da árvore).
#[test]
fn link_do_anexo_fora_da_forma_e_recusado() {
    let com = |src: String| ler(&com_src(&base(), 2, &src), "22");
    for ruim in [
        src_anexo().replace("&id_anexo=901", ""),
        src_anexo().replace("id_anexo=901", "id_anexo=901&id_anexo=902"),
        src_anexo().replace("id_anexo=901", "id_anexo=9a1"),
        src_anexo().replace("id_anexo=901", "id_anexo="),
        src_anexo().replace(HASH_CONTEUDO, "x"),
        src_anexo().replace("controlador.php?", "controlador_externo.php?"),
    ] {
        assert_eq!(
            com(ruim.clone()),
            Err(ErroConteudoNaArvore::Html(ErroHtml::valor(
                "endereco_do_conteudo_nao_reconhecido"
            ))),
            "{ruim}"
        );
    }
}

#[test]
fn recusas_do_link_do_conteudo() {
    let outro = com_src(&base(), 2, &src_visualizar("21"));
    assert_eq!(
        ler(&outro, "22"),
        Err(ErroConteudoNaArvore::EnderecoDoConteudoDeOutroDocumento)
    );
    let sem_hash = com_src(
        &base(),
        2,
        &src_visualizar("22").replace(HASH_CONTEUDO, "x"),
    );
    assert_eq!(
        ler(&sem_hash, "22"),
        Err(ErroConteudoNaArvore::Html(ErroHtml::valor(
            "endereco_do_conteudo_nao_reconhecido"
        )))
    );
    let outra_acao = com_src(
        &base(),
        2,
        &src_visualizar("22").replace("documento_visualizar", "documento_imprimir_web"),
    );
    assert_eq!(
        ler(&outra_acao, "22"),
        Err(ErroConteudoNaArvore::Html(ErroHtml::valor(
            "endereco_do_conteudo_nao_reconhecido"
        )))
    );
    let fora = base().replace("Nos[2].src = '';", "Nos[2].src = 'a' + b;");
    assert_eq!(
        ler(&fora, "22"),
        Err(ErroConteudoNaArvore::Html(ErroHtml::valor(
            "link_do_conteudo_fora_da_forma_observada"
        )))
    );
    let sem = base().replace("Nos[2].src = '';", "");
    assert_eq!(
        ler(&sem, "22"),
        Err(ErroConteudoNaArvore::Html(ErroHtml::ausente(
            "link_do_conteudo_ausente_na_arvore"
        )))
    );
    let dois = base().replace("Nos[2].src = '';", "Nos[2].src = '';\nNos[2].src = '';");
    assert_eq!(
        ler(&dois, "22"),
        Err(ErroConteudoNaArvore::Html(ErroHtml::repetido(
            "link_do_conteudo_repetido_na_arvore"
        )))
    );
    assert_eq!(
        ler(&base(), "99"),
        Err(ErroConteudoNaArvore::Html(ErroHtml::ausente(
            "documento_ausente_da_arvore"
        )))
    );
    // O id do nó PROCESSO não é documento.
    assert_eq!(
        ler(&base(), "10"),
        Err(ErroConteudoNaArvore::Html(ErroHtml::ausente(
            "documento_ausente_da_arvore"
        )))
    );
}

#[test]
fn tela_do_documento_devolve_o_processo_da_arvore() {
    let src = src_da_arvore_do_documento("10", "22");
    let html = tela_do_processo("U", std::slice::from_ref(&src));
    let (tela, processo) = ler_tela_do_documento(&instalacao(), &html, "22", None).unwrap();
    assert_eq!(processo, "10");
    assert_eq!(tela.unidade, "U");
    assert_eq!(
        tela.arvore.as_str(),
        format!("https://sei.exemplo.gov.br/sei/{src}")
    );
}

#[test]
fn tela_que_nao_seleciona_o_documento_pedido_e_recusada() {
    let ler = |src: String| {
        let html = tela_do_processo("U", &[src]);
        ler_tela_do_documento(&instalacao(), &html, "22", None).map(|(_, p)| p)
    };
    assert_eq!(
        ler(src_da_arvore_do_documento("10", "21")),
        Err(ErroTelaDoProcesso::EnderecoDaArvoreDeOutroDocumento)
    );
    // A tela de um processo (id_documento vazio): o número não era documento.
    assert_eq!(
        ler(super::tests_tela::src_da_arvore("10")),
        Err(ErroTelaDoProcesso::EnderecoDaArvoreDeOutroDocumento)
    );
    let dois = src_da_arvore_do_documento("10", "22").replace(
        "id_procedimento=10",
        "id_procedimento=10&id_procedimento=11",
    );
    assert_eq!(
        ler(dois),
        Err(ErroTelaDoProcesso::Html(ErroHtml::valor(
            "endereco_da_arvore_nao_reconhecido"
        )))
    );
}

#[test]
fn nup_declarado_sai_do_no_do_processo() {
    assert_eq!(nup_declarado_na_arvore(&base()).as_deref(), Some(NUP));
}
