//! Leitores de produção sobre o HTML original do acervo; sem POST ao SEI.
use super::{Formato, endereco_download, submissao};
use crate::cassette::acervo::gravacao;
use crate::cassette::{ZipDoAcervo, archive_filename, conferir_cassette_ate, html_da_resposta};
use crate::formulario_web::{TelaServida, localizar_formulario};
use crate::tela_html::{HtmlServido, arvore};
use std::path::{Path, PathBuf};

fn replay(formato: Formato, nomes: &[&str], acao: &str, formulario: &str) {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("cassettes/unb");
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || PathBuf::from(std::env::var_os("HOME").expect("HOME")).join(".cache/ies"),
        PathBuf::from,
    );
    let mut formularios = 0;
    let mut downloads = 0;
    let mut selecao = 0;
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
                let post = submissao(&origem, &tela, &id, formato, None, None)
                    .expect("derivação de formulário original");
                let campos: Vec<_> = form_urlencoded::parse(post.corpo()).collect();
                assert!(campos.iter().any(|(n, v)| n == "rdoTipo" && v == "T"));
                assert!(campos.iter().any(|(n, v)| n == "hdnFlagGerar" && v == "1"));
                assert!(!campos.iter().any(|(n, _)| n.starts_with("btn")));
                let dom = arvore(&html);
                let form = localizar_formulario(&dom, formulario).unwrap();
                let apenas = form.descendentes().any(|e| {
                    e.nome() == "input"
                        && e.atributo("type") == Some("radio")
                        && e.atributo("name") == Some("rdoTipo")
                        && e.atributo("value") == Some("A")
                        && !e.presente("disabled")
                });
                let oculto = form.descendentes().any(|e| {
                    e.nome() == "input"
                        && e.atributo("type") == Some("hidden")
                        && e.atributo("name") == Some("hdnDocumentosApenas")
                });
                let lista = form.descendentes().any(|e| {
                    e.nome() == "select"
                        && e.atributo("name") == Some("selDocumentosApenas")
                        && e.presente("multiple")
                });
                if matches!(formato, Formato::Zip) && apenas && oculto && lista {
                    // Derivação sobre o formulário original, com escolha fictícia.
                    let post = submissao(&origem, &tela, &id, formato, None, Some("123±Anexo"))
                        .expect("modalidade A sobre HTML original");
                    let corpo = String::from_utf8_lossy(post.corpo());
                    assert!(corpo.contains("rdoTipo=A"));
                    assert!(
                        corpo.contains("hdnDocumentosApenas=123%B1Anexo")
                            || corpo.contains("hdnDocumentosApenas=123%C2%B1Anexo")
                    );
                    selecao += 1;
                }
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
    if matches!(formato, Formato::Zip) {
        assert!(selecao > 0, "nenhum formulário original oferece seleção A");
    }
    println!(
        "exportação {formato:?}: {formularios} formulários, {downloads} links e {selecao} seletores A originais; wire POST e anti-IES não comprovados"
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
    // A captura seletiva acima de 100 é uma fonte independente do ZIP integral.
    replay(
        Formato::Zip,
        &["processo-arquivo-apenas-alem-de-100-unb--dff8c4a20a0d.md"],
        "procedimento_gerar_zip",
        "frmProcedimentoZip",
    );
}

// Fonte independente: Todos test_pasta_mista_gravada_010.py.
// A árvore de destino contém 20 nós em PASTA1, o segundo fechado.
// O teste observa a árvore e contrasta a seleção real, não um POST gravado.
#[test]
#[ignore = "ZIP público conferido no cache; sem rede"]
fn replay_elegibilidade_da_exportacao_no_acervo() {
    use crate::arvore_processo::{
        ConteudoNaArvore, barra_do_no, ler_arvore, ler_conteudo_na_arvore, nup_declarado_na_arvore,
        tela_do_documento,
    };

    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("cassettes");
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || PathBuf::from(std::env::var_os("HOME").expect("HOME")).join(".cache/ies"),
        PathBuf::from,
    );
    let mut resultados = Vec::new();
    for (instalacao, nome) in [
        ("unb", "processo-pasta-mista-unb--dffe2d9379eb.md"),
        ("ufrj", "processo-pasta-mista-ufrj--65861ad4af07.md"),
    ] {
        let card = raiz.join(instalacao).join(nome);
        let zip = ZipDoAcervo::ler(&cache.join(archive_filename(&card).unwrap())).unwrap();
        let original = conferir_cassette_ate(&card, &zip, 120_000_000).unwrap();
        assert_eq!(original.instalacao, instalacao);
        let origem = gravacao::instalacao(&original.json).unwrap();
        let mut observado = false;
        for r in gravacao::respostas(&original.json) {
            if r.operacao != "procedimento_visualizar"
                || r.status != 200
                || gravacao::parametro(r.interacao, "abrir_pastas") != ["1"]
            {
                continue;
            }
            let Some(id) = gravacao::id_procedimento(r.interacao) else {
                continue;
            };
            let Ok(html) = html_da_resposta(r.interacao) else {
                continue;
            };
            let Some(nup) = nup_declarado_na_arvore(&html) else {
                continue;
            };
            let Ok(lida) = ler_arvore(&html, &nup, &id) else {
                continue;
            };
            let pasta: Vec<_> = lida
                .documentos
                .iter()
                .filter(|d| d.pasta.as_deref() == Some("PASTA1"))
                .collect();
            if pasta.len() != 20 {
                continue;
            }
            let fechado = pasta[1];
            if !matches!(
                ler_conteudo_na_arvore(&origem, &html, &fechado.id),
                Ok(ConteudoNaArvore::Vazio)
            ) || !barra_do_no(&html, &fechado.id)
                .unwrap()
                .is_some_and(|barra| barra.as_str().is_empty())
                || tela_do_documento(&origem, &html, &fechado.id, &id).is_some()
            {
                continue;
            }
            assert!(!fechado.natureza.cancelamento.cancelado());
            let numero = fechado.numero_sei.clone().expect("número do nó fechado");
            let vizinho = pasta[0];
            assert!(matches!(
                ler_conteudo_na_arvore(&origem, &html, &vizinho.id),
                Ok(ConteudoNaArvore::Visualizar(_))
            ));
            let positivo = vizinho.numero_sei.clone().expect("número do vizinho");
            assert!(
                super::selecao_da_arvore(
                    &lida.documentos,
                    std::slice::from_ref(&positivo),
                    &html,
                    &origem,
                )
                .is_ok()
            );
            let recusou = matches!(
                super::selecao_da_arvore(
                    &lida.documentos,
                    std::slice::from_ref(&numero),
                    &html,
                    &origem,
                ),
                Err(super::ErroSelecao::DocumentoSelecionadoInacessivel)
            );
            let recusou_mista = matches!(
                super::selecao_da_arvore(&lida.documentos, &[positivo, numero], &html, &origem),
                Err(super::ErroSelecao::DocumentoSelecionadoInacessivel)
            );
            resultados.push((instalacao, recusou, recusou_mista));
            observado = true;
            break;
        }
        assert!(observado, "{instalacao}: árvore de destino não localizada");
    }
    assert_eq!(resultados.len(), 2);
    for (instalacao, isolada, mista) in resultados {
        assert!(isolada && mista, "{instalacao}: seleção aceitou nó fechado");
    }
}

// Censo nominal do diretório central original, sem abrir membros nem
// publicar nomes. Não demonstra por si só a modalidade do POST.
#[test]
#[ignore = "ZIP público conferido no cache; sem rede"]
fn replay_extensoes_do_zip_original_unb() {
    use std::io::Cursor;

    let card = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("cassettes/unb/documento-lote-pelo-zip-unb--05402e0f931b.md");
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || PathBuf::from(std::env::var_os("HOME").expect("HOME")).join(".cache/ies"),
        PathBuf::from,
    );
    let zip = ZipDoAcervo::ler(&cache.join(archive_filename(&card).unwrap())).unwrap();
    let original = conferir_cassette_ate(&card, &zip, 120_000_000).unwrap();
    assert_eq!(original.instalacao, "unb");
    let mut downloads = 0;
    let mut duplicados = false;
    let (mut pdf, mut html, mut outros, mut diretorios) = (0, 0, 0, 0);
    for r in gravacao::respostas(&original.json) {
        if r.operacao != "exibir_arquivo" || r.metodo != "GET" || r.status != 200 {
            continue;
        }
        let corpo = gravacao::corpo(r.interacao);
        let arquivo = zip::ZipArchive::new(Cursor::new(corpo))
            .unwrap_or_else(|_| panic!("download original não contém ZIP legível"));
        downloads += 1;
        duplicados |= arquivo.has_duplicate_names();
        for nome in arquivo.file_names() {
            let nome = nome.unwrap_or_else(|_| panic!("nome original ilegível"));
            let nome = nome.to_ascii_lowercase();
            if nome.ends_with('/') {
                diretorios += 1;
            } else if nome.ends_with(".pdf") {
                pdf += 1;
            } else if nome.ends_with(".html") || nome.ends_with(".htm") {
                html += 1;
            } else {
                outros += 1;
            }
        }
    }
    assert_eq!(downloads, 1, "download original esperado");
    assert!(pdf + html + outros > 0, "nenhum arquivo observado");
    println!(
        "ZIP original UnB (nomes brutos distintos): pdf={pdf} html={html} outros={outros} diretorios={diretorios} duplicados={duplicados}"
    );
}
