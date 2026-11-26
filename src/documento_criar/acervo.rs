//! Replay offline do cadastro de documento no acervo público de Goiás (#246).
//! ZIP, membro, origem e integridade são conferidos antes da classificação.
//! Só imprime contagens: nunca HTML, URL assinada ou cookie.
use super::texto_padrao::{ErroTextoPadrao, url_publicada};
use super::{conferir_cadastro, tipos_oferecidos};
use crate::cassette::acervo::gravacao;
use crate::cassette::{ZipDoAcervo, conferir_cassette_ate, html_da_resposta};
use crate::tela_html::HtmlServido;
use std::path::Path;

const CARDS: [&str; 2] = [
    "documento-incluir-go--fa4845e16ba9.md",
    "documento-escritas-go--16f83179ecaa.md",
];

fn tipo_do_cadastro(html: &str) -> Option<String> {
    let servido = HtmlServido::de_teste(html);
    let dom = crate::tela_html::arvore(&servido);
    let rotulos = dom.com_id("lblSerieTitulo").collect::<Vec<_>>();
    let [rotulo] = rotulos.as_slice() else {
        return None;
    };
    if !rotulo.nome().eq_ignore_ascii_case("label") {
        return None;
    }
    let tipo = rotulo
        .texto()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    (!tipo.is_empty()).then_some(tipo)
}

fn oferece_texto_padrao(servido: &HtmlServido) -> bool {
    let dom = crate::tela_html::arvore(servido);
    dom.com_id("frmDocumentoCadastro").any(|form| {
        form.elementos("input").any(|campo| {
            campo.atributo("name") == Some("rdoTextoInicial")
                && campo.atributo("type") == Some("radio")
                && campo.atributo("value") == Some("T")
                && !campo.presente("disabled")
        })
    })
}

fn cache_acervo() -> std::path::PathBuf {
    std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || {
            let home = std::env::var_os("HOME").expect("HOME ou IES_ACERVO_CACHE");
            Path::new(&home).join(".cache/ies")
        },
        std::path::PathBuf::from,
    )
}

#[test]
#[ignore = "wip #246: requer ZIP público conferido em IES_ACERVO_CACHE, sem rede"]
fn replay_documento_criar_go() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cache = cache_acervo();
    let zip = ZipDoAcervo::ler(&cache.join("todos-sei-cassettes-go.zip"))
        .expect("ZIP de Goiás não está no cache do acervo");
    let mut lidas = 0usize;
    let mut recusadas = 0usize;
    let mut nao_aplicaveis = 0usize;
    let mut respostas_alvo = 0usize;
    let mut cadastros = 0usize;
    let mut radio_t = 0usize;
    let mut autocomplete_lido = 0usize;
    let mut controles_recusados = 0usize;
    let mut script_recusado = 0usize;
    for card in CARDS {
        let caminho = raiz.join("cassettes/go").join(card);
        let cassette = conferir_cassette_ate(&caminho, &zip, 120_000_000)
            .expect("integridade ou origem do card inválida");
        assert_eq!(cassette.instalacao, "go");
        for resposta in gravacao::respostas(&cassette.json) {
            if !matches!(
                resposta.operacao,
                "documento_escolher_tipo" | "documento_gerar"
            ) {
                continue;
            }
            respostas_alvo += 1;
            if resposta.status != 200 {
                nao_aplicaveis += 1;
                continue;
            }
            let Ok(html) = html_da_resposta(resposta.interacao) else {
                nao_aplicaveis += 1;
                continue;
            };
            let resultado = match resposta.operacao {
                "documento_escolher_tipo" => {
                    tipos_oferecidos(&HtmlServido::de_teste(&html)).is_ok()
                }
                "documento_gerar" => tipo_do_cadastro(&html).is_some_and(|tipo| {
                    let servido = HtmlServido::de_teste(&html);
                    if conferir_cadastro(&servido, &tipo, None, None).is_err() {
                        return false;
                    }
                    cadastros += 1;
                    // Censo independente da declaração literal do autocomplete:
                    // presença de T no formulário original não prova consulta.
                    let oferecido = oferece_texto_padrao(&servido);
                    radio_t += usize::from(oferecido);
                    match url_publicada(&servido) {
                        Ok(_) => {
                            assert!(oferecido, "autocomplete sem rádio T no cadastro");
                            autocomplete_lido += 1;
                        }
                        Err(ErroTextoPadrao::ControlesNaoOferecidos) => {
                            controles_recusados += 1;
                        }
                        Err(ErroTextoPadrao::AutocompletarNaoDeclarado) => {
                            script_recusado += 1;
                        }
                        Err(erro) => panic!(
                            "classe inesperada no leitor de cadastro: {:?}",
                            std::mem::discriminant(&erro)
                        ),
                    }
                    true
                }),
                _ => unreachable!(),
            };
            if resultado {
                lidas += 1;
            } else {
                recusadas += 1;
            }
        }
    }
    assert!(
        respostas_alvo > 0,
        "cards sem respostas de inclusão de documento"
    );
    assert!(lidas + recusadas > 0, "nenhum HTML candidato no replay");
    assert_eq!(respostas_alvo, lidas + recusadas + nao_aplicaveis);
    // Referência observada no replay GO imutável (run 42118476321):
    // sem estes números um leitor quebrado poderia recusar tudo e passar.
    // As 5 recusas antigas eram listas de tipos com "Nome&nbsp;(Formulário)",
    // lidas desde a correção do ensaio live de 10/11/2026 (PR #635).
    assert_eq!(respostas_alvo, 15, "mudou o conjunto de respostas-alvo");
    assert_eq!(lidas, 8, "regressão na leitura de inclusão/cadastro");
    assert_eq!(recusadas, 0, "mudou a classificação de recusas");
    assert_eq!(nao_aplicaveis, 7, "mudou a classe não aplicável");
    println!(
        "documento.criar GO: {respostas_alvo} respostas; {lidas} lidas, {recusadas} recusadas, {nao_aplicaveis} não aplicáveis"
    );
    // Censo nominal do replay original 45815210177: não permitir que
    // uma regressão no leitor recuse todas as telas e deixe o replay verde.
    assert_eq!(cadastros, 3);
    assert_eq!(radio_t, 3);
    assert_eq!(autocomplete_lido, 3);
    assert_eq!(controles_recusados, 0);
    assert_eq!(script_recusado, 0);
    println!(
        "texto padrão GO (somente cadastro): {cadastros} telas, {radio_t} rádio T, {autocomplete_lido} autocomplete reconhecido, {controles_recusados} controles recusados, {script_recusado} script recusado"
    );
    conferir_css_no_documento_gerado(raiz, &zip);
}

/// Exercita apenas a detecção de CSS no HTML original de impressão. A
/// captura tem a impressão depois de salvar, não a foto da minuta inicial.
fn conferir_css_no_documento_gerado(raiz: &Path, zip: &ZipDoAcervo) {
    let card = raiz.join("cassettes/go/documento-criar-texto-go--5621c99af5dc.md");
    let cassette =
        conferir_cassette_ate(&card, zip, 120_000_000).expect("card GO da criação com texto");
    let mut impressoes = 0;
    for resposta in gravacao::respostas(&cassette.json) {
        if resposta.operacao == "documento_imprimir_web" && resposta.status == 200 {
            let html = html_da_resposta(resposta.interacao).expect("HTML de impressão GO");
            assert!(crate::documento_criar_cli::contem_estilos(
                &HtmlServido::de_teste(&html)
            ));
            impressoes += 1;
        }
    }
    assert_eq!(impressoes, 2);
}
