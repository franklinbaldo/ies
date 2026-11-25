//! RED/GREEN da colheita: páginas SINTÉTICAS do Controle nas visões R e D
//! (estrutura observada: `tr#P{id}` com o link do número, menu
//! `protocolo_pesquisar`, `form#frmProtocoloPesquisaRapida`), com linhas
//! válidas e inválidas, e o conteúdo **exato** do índice depois. Hosts,
//! hashes, ids e números inventados; sem rede.
use super::*;
use crate::acao_do_sei::AcaoDoSei;
use crate::configuracao::ArquivosDeAcesso;
use crate::tela_html::HtmlServido;
use std::fmt::Write as _;
use std::path::PathBuf;

pub(crate) const ORIGEM: &str = "https://sei.exemplo.gov.br/sei";
const CONTEXTO: &str = "infra_sistema=100000100&amp;infra_unidade_atual=110000001";
pub(crate) const UNIDADE_ID: &str = "110000001";
pub(crate) const COOKIE: &str = "PHPSESSID=SINTETICO";
pub(crate) const MIME: &str = "text/html; charset=iso-8859-1";

pub(crate) fn hash(c: char) -> String {
    c.to_string().repeat(64)
}

pub(crate) fn instalacao() -> Instalacao {
    Instalacao::interpretar(ORIGEM).unwrap()
}

pub(crate) fn url_controle() -> String {
    format!(
        "{ORIGEM}/controlador.php?acao=procedimento_controlar&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={}",
        hash('a')
    )
}

/// `href` de `tr#P{id}`, com entidades como a fonte traz.
pub(crate) fn href_da_linha(id: &str, c: char) -> String {
    format!(
        "controlador.php?acao=procedimento_trabalhar&amp;acao_origem=procedimento_controlar&amp;acao_retorno=procedimento_controlar&amp;id_procedimento={id}&amp;{CONTEXTO}&amp;infra_hash={}",
        hash(c)
    )
}

/// A URL absoluta que a colheita grava para esse `href`.
pub(crate) fn trabalho(id: &str, c: char) -> String {
    format!("{ORIGEM}/{}", href_da_linha(id, c).replace("&amp;", "&"))
}

fn href_da_pesquisa() -> String {
    format!(
        "controlador.php?acao=protocolo_pesquisar&amp;acao_origem=procedimento_controlar&amp;{CONTEXTO}&amp;infra_hash={}",
        hash('e')
    )
}

fn action_rapida() -> String {
    format!(
        "controlador.php?acao=protocolo_pesquisa_rapida&amp;{CONTEXTO}&amp;infra_hash={}",
        hash('b')
    )
}

/// Menu da pesquisa e pesquisa rápida, como no topo de toda tela.
pub(crate) fn menu() -> String {
    format!(
        "<a id=\"lnkInfraUnidade\">UNIDADE TESTE</a>\
         <a link=\"protocolo_pesquisar\" href=\"{}\">Pesquisa</a>\
         <form id=\"frmProtocoloPesquisaRapida\" method=\"post\" action=\"{}\">\
         <input type=\"text\" id=\"txtPesquisaRapida\" name=\"txtPesquisaRapida\" />\
         <img onclick=\"document.getElementById('frmProtocoloPesquisaRapida').submit();\" /></form>",
        href_da_pesquisa(),
        action_rapida()
    )
}

fn linhas_r(linhas: &[(&str, &str, Option<String>)]) -> String {
    let mut html = String::new();
    for (id, numero, href) in linhas {
        let href = href
            .as_deref()
            .map(|h| format!(" href=\"{h}\""))
            .unwrap_or_default();
        let _ = write!(
            html,
            "<tr id=\"P{id}\"><td><a{href} class=\"processoVisualizado\">{numero}</a></td></tr>"
        );
    }
    html
}

/// Controle na visão R: `recebidos` e `gerados` como (id, número, href).
pub(crate) fn pagina_r(
    recebidos: &[(&str, &str, Option<String>)],
    gerados: &[(&str, &str, Option<String>)],
) -> String {
    format!(
        "<html><body>{}\
         <table id=\"tblProcessosRecebidos\"><caption>{} registros:</caption>{}</table>\
         <table id=\"tblProcessosGerados\"><caption>{} registros:</caption>{}</table></body></html>",
        menu(),
        recebidos.len(),
        linhas_r(recebidos),
        gerados.len(),
        linhas_r(gerados)
    )
}

/// Controle na visão D (`tblProcessosDetalhado`, coluna Processo).
fn pagina_d(linhas: &[(&str, &str, String)]) -> String {
    let mut html = String::new();
    for (id, numero, href) in linhas {
        let _ = write!(
            html,
            "<tr id='P{id}'><td>linha</td><td><a class='processoVisualizado' href='{href}'>{numero}</a></td><td>Ofício</td></tr>"
        );
    }
    let n = linhas.len();
    format!(
        "<html><body>{}<input id='hdnTipoVisualizacao' value='D'>\
         <input id='hdnRecebidosHashCriterios' value='r'><input id='hdnGeradosHashCriterios' value='g'>\
         <table id='tblProcessosDetalhado'><caption>Processos ({n} registros - 1 a {n}):</caption>\
         <tr><th></th><th>Processo</th><th>Tipo</th></tr>{html}</table></body></html>",
        menu()
    )
}

pub(crate) struct Pasta(pub(crate) PathBuf);

impl Drop for Pasta {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub(crate) fn indice() -> (Pasta, IndiceLocal) {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static N: AtomicUsize = AtomicUsize::new(0);
    let pasta = std::env::temp_dir().join(format!(
        "ies-colheita-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&pasta);
    std::fs::create_dir_all(&pasta).unwrap();
    crate::indice_local::tests::definir_modo(&pasta, 0o700);
    let indice = IndiceLocal::em(pasta.join("ies.sqlite3"));
    (Pasta(pasta), indice)
}

pub(crate) fn geracao() -> String {
    crate::sessao_login::geracao_da_sessao(COOKIE)
}

fn colher(indice: &IndiceLocal, html: &str, mime: Option<&str>) {
    Colheita::do_cookie(indice, &instalacao(), COOKIE, &url_controle())
        .colher(&HtmlServido::de_teste(html), mime);
}

pub(crate) fn processo(nup: &str, id: &str) -> [String; 3] {
    [ORIGEM.to_owned(), nup.to_owned(), id.to_owned()]
}

pub(crate) fn link(acao: AcaoDoSei, objeto: &str, url: &str) -> [String; 6] {
    [
        ORIGEM.to_owned(),
        geracao(),
        UNIDADE_ID.to_owned(),
        acao.como_na_url().to_owned(),
        objeto.to_owned(),
        url.to_owned(),
    ]
}

/// Os dois links da página (menu e pesquisa rápida), já absolutos.
pub(crate) fn links_do_menu() -> [[String; 6]; 2] {
    [
        link(
            AcaoDoSei::ProtocoloPesquisaRapida,
            "",
            &format!("{ORIGEM}/{}", action_rapida().replace("&amp;", "&")),
        ),
        link(
            AcaoDoSei::ProtocoloPesquisar,
            "",
            &format!("{ORIGEM}/{}", href_da_pesquisa().replace("&amp;", "&")),
        ),
    ]
}

#[test]
fn visao_r_grava_so_as_linhas_conferidas_e_os_links_do_menu() {
    let (_pasta, indice) = indice();
    let visualizar =
        href_da_linha("12", 'c').replace("procedimento_trabalhar", "procedimento_visualizar");
    let estrangeiro = format!(
        "https://outro.exemplo.gov.br/sei/{}",
        href_da_linha("13", 'c')
    );
    let absoluto = format!("{ORIGEM}/{}", href_da_linha("17", 'c'));
    let outra_unidade = href_da_linha("19", 'c').replace("110000001", "110000002");
    let hash_curto = href_da_linha("20", 'c').replace(&hash('c'), "cccc");
    let sem_origem =
        href_da_linha("21", 'c').replace("acao_origem=procedimento_controlar&amp;", "");
    let html = pagina_r(
        &[
            ("10", "NUP-A", Some(href_da_linha("10", 'c'))),
            ("12", "NUP-C", Some(visualizar)),
            ("13", "NUP-D", Some(estrangeiro)),
            ("14", "NUP-E", Some(href_da_linha("99", 'c'))),
            ("15", "NUP-F", Some(href_da_linha("15", 'c'))),
            ("16", "NUP-F", Some(href_da_linha("16", 'c'))),
            ("17", "NUP-G", Some(absoluto)),
            ("18", "NUP-H", None),
        ],
        &[
            ("11", "NUP-B", Some(href_da_linha("11", 'd'))),
            ("19", "NUP-I", Some(outra_unidade)),
            ("20", "NUP-J", Some(hash_curto)),
            ("21", "NUP-K", Some(sem_origem)),
        ],
    );
    colher(&indice, &html, Some(MIME));
    let (processos, links) = indice.conteudo();
    assert_eq!(
        processos,
        [processo("NUP-A", "10"), processo("NUP-B", "11")]
    );
    let [rapida, pesquisa] = links_do_menu();
    assert_eq!(
        links,
        [
            link(AcaoDoSei::ProcedimentoTrabalhar, "10", &trabalho("10", 'c')),
            link(AcaoDoSei::ProcedimentoTrabalhar, "11", &trabalho("11", 'd')),
            rapida,
            pesquisa,
        ]
    );
}

#[test]
fn visao_d_grava_as_linhas_conferidas() {
    let (_pasta, indice) = indice();
    let visualizar =
        href_da_linha("31", 'c').replace("procedimento_trabalhar", "procedimento_visualizar");
    let hash_curto = href_da_linha("32", 'c').replace(&hash('c'), "cc");
    let html = pagina_d(&[
        ("30", "NUP-D1", href_da_linha("30", 'f')),
        ("31", "NUP-D2", visualizar),
        ("32", "NUP-D3", hash_curto),
        ("33", "NUP-D4", href_da_linha("33", 'f')),
    ]);
    colher(&indice, &html, Some(MIME));
    let (processos, links) = indice.conteudo();
    assert_eq!(
        processos,
        [processo("NUP-D1", "30"), processo("NUP-D4", "33")]
    );
    let [rapida, pesquisa] = links_do_menu();
    assert_eq!(
        links,
        [
            link(AcaoDoSei::ProcedimentoTrabalhar, "30", &trabalho("30", 'f')),
            link(AcaoDoSei::ProcedimentoTrabalhar, "33", &trabalho("33", 'f')),
            rapida,
            pesquisa,
        ]
    );
}

#[test]
fn id_em_duas_linhas_pula_as_duas() {
    let (_pasta, indice) = indice();
    let html = pagina_r(
        &[("40", "NUP-X", Some(href_da_linha("40", 'c')))],
        &[
            ("40", "NUP-Y", Some(href_da_linha("40", 'c'))),
            ("41", "NUP-Z", Some(href_da_linha("41", 'c'))),
        ],
    );
    colher(&indice, &html, Some(MIME));
    let (processos, links) = indice.conteudo();
    assert_eq!(processos, [processo("NUP-Z", "41")]);
    assert_eq!(
        links[0],
        link(AcaoDoSei::ProcedimentoTrabalhar, "41", &trabalho("41", 'c'))
    );
    assert_eq!(links.len(), 3);
}

#[test]
fn pagina_recusada_pelo_leitor_so_alimenta_o_menu() {
    let (_pasta, indice) = indice();
    // D com link para outro id: o leitor da D recusa a página inteira.
    let html = pagina_d(&[
        ("50", "NUP-1", href_da_linha("50", 'c')),
        ("51", "NUP-2", href_da_linha("52", 'c')),
    ]);
    colher(&indice, &html, Some(MIME));
    let (processos, links) = indice.conteudo();
    assert_eq!(processos.len(), 0);
    assert_eq!(links, links_do_menu());
}

#[test]
fn sem_mime_a_pesquisa_rapida_nao_e_colhida() {
    let (_pasta, indice) = indice();
    let html = pagina_r(&[("10", "NUP-A", Some(href_da_linha("10", 'c')))], &[]);
    colher(&indice, &html, None);
    let (_, links) = indice.conteudo();
    let acoes: Vec<_> = links.iter().map(|l| l[3].as_str()).collect();
    assert_eq!(
        acoes,
        [
            AcaoDoSei::ProcedimentoTrabalhar.como_na_url(),
            AcaoDoSei::ProtocoloPesquisar.como_na_url()
        ]
    );
}

#[test]
fn url_do_controle_sem_unidade_nao_grava_nada() {
    let (_pasta, indice) = indice();
    let html = HtmlServido::de_teste(&pagina_r(
        &[("10", "NUP-A", Some(href_da_linha("10", 'c')))],
        &[],
    ));
    let url = url_controle().replace("&infra_unidade_atual=110000001", "");
    Colheita::do_cookie(&indice, &instalacao(), COOKIE, &url).colher(&html, Some(MIME));
    assert_eq!(indice.conteudo(), (Vec::new(), Vec::new()));
}

#[test]
fn nova_colheita_substitui_o_par_que_divergiu() {
    let (_pasta, indice) = indice();
    indice.registrar_processo(&instalacao(), "NUP-A", "9");
    let html = pagina_r(&[("10", "NUP-A", Some(href_da_linha("10", 'c')))], &[]);
    colher(&indice, &html, Some(MIME));
    assert_eq!(indice.conteudo().0, [processo("NUP-A", "10")]);
}

#[test]
fn indice_desligado_e_nenhuma_nao_fazem_nada() {
    let html = pagina_r(&[("10", "NUP-A", Some(href_da_linha("10", 'c')))], &[]);
    let desligado = IndiceLocal::desligado();
    colher(&desligado, &html, Some(MIME));
    Colheita::nenhuma().colher(&HtmlServido::de_teste(&html), Some(MIME));
    assert!(!desligado.ligado());
}

/// A colheita do retrato coerente lido dos arquivos (#222); falha de
/// leitura: nenhuma colheita.
fn da_sessao<'i>(indice: &'i IndiceLocal, arquivos: &ArquivosDeAcesso) -> Colheita<'i> {
    crate::transporte::sessao::ReferenciasDaSessao::ler(
        &arquivos.url_controle,
        &arquivos.origem,
        &arquivos.cookie,
    )
    .map_or_else(
        |_| Colheita::nenhuma(),
        |sessao| Colheita::das_referencias(indice, &sessao),
    )
}

fn arquivos_da_colheita(pasta: &Pasta) -> ArquivosDeAcesso {
    let arquivos = ArquivosDeAcesso {
        origem: pasta.0.join("origem"),
        url_controle: pasta.0.join("controle"),
        cookie: pasta.0.join("sessao"),
    };
    gravar(&arquivos.origem, ORIGEM);
    gravar(&arquivos.url_controle, &url_controle());
    gravar(&arquivos.cookie, &["sessao", "=primeira"].concat());
    arquivos
}

fn gravar(caminho: &std::path::Path, texto: &str) {
    std::fs::write(caminho, texto).unwrap();
    crate::indice_local::tests::definir_modo(caminho, 0o600);
}

// Caso de todos tests/test_sessao_global_contrato.py @323eb01
#[test]
fn nova_leitura_reflete_a_geracao_atual() {
    let (pasta, indice) = indice();
    let arquivos = arquivos_da_colheita(&pasta);
    let primeira = da_sessao(&indice, &arquivos).alvo.unwrap();
    gravar(&arquivos.cookie, &["sessao", "=segunda"].concat());
    let segunda = da_sessao(&indice, &arquivos).alvo.unwrap();
    let geracoes = std::collections::HashSet::from([primeira.geracao, segunda.geracao]);
    assert_eq!(geracoes.len(), 2);
}

// Caso de todos tests/v07/test_security_boundaries.py @323eb01
#[test]
fn colheita_recusa_origem_divergente_e_cookie_invalido() {
    let (pasta, indice) = indice();
    let arquivos = arquivos_da_colheita(&pasta);
    gravar(&arquivos.origem, "https://outro.exemplo.gov.br/sei");
    assert!(da_sessao(&indice, &arquivos).alvo.is_none());

    gravar(&arquivos.origem, ORIGEM);
    gravar(&arquivos.cookie, "linha\ninvalida");
    assert!(da_sessao(&indice, &arquivos).alvo.is_none());
}

// Caso de todos tests/test_sessao_em_arquivo_010.py @323eb01
#[cfg(unix)]
#[test]
fn colheita_recusa_arquivo_de_sessao_com_permissao_aberta() {
    let (pasta, indice) = indice();
    let arquivos = arquivos_da_colheita(&pasta);
    crate::indice_local::tests::definir_modo(&arquivos.cookie, 0o644);
    assert!(da_sessao(&indice, &arquivos).alvo.is_none());
}

#[test]
fn observacao_da_unidade_so_nasce_de_controle_verificado() {
    let (pasta, indice) = indice();
    let html = pagina_r(&[], &[]);
    colher(&indice, &html, Some(MIME));
    let banco = rusqlite::Connection::open(pasta.0.join("ies.sqlite3")).unwrap();
    let unidade = iesei::sqlite::consultar_unidade(&banco, &instalacao(), &geracao())
        .unwrap()
        .unwrap();
    assert_eq!(unidade.unidade.as_str(), "UNIDADE TESTE");
    assert_eq!(unidade.visao, iesei::observacao::VisaoDoControle::Resumida);

    let outra_geracao = hash('f');
    assert!(
        iesei::sqlite::consultar_unidade(&banco, &instalacao(), &outra_geracao)
            .unwrap()
            .is_none()
    );
    let linhas: i64 = banco
        .query_row("SELECT count(*) FROM controle_unidade", [], |r| r.get(0))
        .unwrap();
    assert_eq!(linhas, 1);
}

#[test]
fn tela_sem_tabelas_nao_persiste_identidade_da_unidade() {
    let (pasta, indice) = indice();
    colher(&indice, "<a id='lnkInfraUnidade'>FALSA</a>", Some(MIME));
    // Forçar a criação do banco via operação já suportada.
    indice.registrar_entrada_por_id(&instalacao(), true);
    let banco = rusqlite::Connection::open(pasta.0.join("ies.sqlite3")).unwrap();
    assert!(
        iesei::sqlite::consultar_unidade(&banco, &instalacao(), &geracao())
            .unwrap()
            .is_none()
    );
}
