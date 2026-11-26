//! Leitores de produção sobre o HTML original do acervo; sem POST ao SEI.
use super::{Formato, endereco_download, submissao};
use crate::cassette::acervo::gravacao;
use crate::cassette::{ZipDoAcervo, archive_filename, conferir_cassette_ate, html_da_resposta};
use crate::formulario_web::TelaServida;
use crate::tela_html::HtmlServido;
use std::path::{Path, PathBuf};

fn replay(formato: Formato, nomes: &[&str], acao: &str, formulario: &str) {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("cassettes/unb");
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || PathBuf::from(std::env::var_os("HOME").expect("HOME")).join(".cache/ies"),
        PathBuf::from,
    );
    let mut formularios = 0;
    let mut downloads = 0;
    for nome in nomes {
        let card = raiz.join(nome);
        let zip = ZipDoAcervo::ler(&cache.join(archive_filename(&card).unwrap())).unwrap();
        let original = conferir_cassette_ate(&card, &zip, 120_000_000).unwrap();
        assert_eq!(original.instalacao, "unb");
        let origem = gravacao::instalacao(&original.json).unwrap();
        for r in gravacao::respostas(&original.json) {
            if r.operacao != acao || r.status != 200 {
                continue;
            }
            let Some(id) = gravacao::id_procedimento(r.interacao) else {
                continue;
            };
            let html = HtmlServido::de_teste(&html_da_resposta(r.interacao).unwrap());
            if r.metodo == "GET" && html.as_str().contains(formulario) {
                let url = gravacao::url_sem_consulta(&original.json, r.interacao);
                let codificacao =
                    crate::pagina_do_controle::codificacao_da_tela(&gravacao::mime(r.interacao))
                        .expect("charset original");
                let tela = TelaServida {
                    url: &url,
                    html: &html,
                    codificacao,
                };
                let post = submissao(&origem, &tela, &id, formato, None)
                    .expect("derivação de formulário original");
                let campos: Vec<_> = form_urlencoded::parse(post.corpo()).collect();
                assert!(campos.iter().any(|(n, v)| n == "rdoTipo" && v == "T"));
                assert!(campos.iter().any(|(n, v)| n == "hdnFlagGerar" && v == "1"));
                assert!(!campos.iter().any(|(n, _)| n.starts_with("btn")));
                formularios += 1;
            }
            if r.metodo == "POST"
                && html
                    .as_str()
                    .contains("document.getElementById('ifrDownload').src")
            {
                assert!(
                    endereco_download(&origem, &html, &id).is_ok(),
                    "URL original de download deve ser reconhecida sem imprimir valores"
                );
                downloads += 1;
            }
        }
    }
    assert!(formularios > 0, "nenhum formulário original confrontado");
    assert!(
        downloads > 0,
        "nenhuma resposta original com download confrontada"
    );
    println!(
        "exportação {formato:?}: {formularios} formulários e {downloads} links originais; POST real e anti-IES não comprovados"
    );
}

#[test]
#[ignore = "ZIP público conferido no cache; sem rede"]
fn replay_de_processo_exportar_no_acervo() {
    replay(
        Formato::Pdf,
        &[
            "processo-arquivo-pdf-unb--216a198480de.md",
            "processo-arquivo-documentos-unb--4462ca5487d7.md",
        ],
        "procedimento_gerar_pdf",
        "frmProcedimentoPdf",
    );
    replay(
        Formato::Zip,
        &["documento-lote-pelo-zip-unb--05402e0f931b.md"],
        "procedimento_gerar_zip",
        "frmProcedimentoZip",
    );
}
