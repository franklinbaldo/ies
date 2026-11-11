//! Replay da cassette pública `UnB` da lista de destinos (#244).
//! Somente leitura de ZIP verificado, nunca rede nem segredos.
use super::super::fluxo;
use super::{ZipDoAcervo, conferir_cassette_ate, gravacao, html_da_resposta, ler_pagina};
use crate::acao_do_sei::AcaoDoSei;
use crate::formulario_web::{RespostaDoPost, SubmissaoDerivada};
use crate::tela_html::HtmlServido;
use crate::transporte::{FalhaTransporte, RespostaWeb};
use std::io::Cursor;
use std::path::Path;

struct TransporteGravado {
    passos: Vec<(bool, String)>,
    proximo: usize,
    gets: usize,
    posts: usize,
}
impl TransporteGravado {
    fn resposta(&mut self, post: bool) -> RespostaDoPost<Cursor<Vec<u8>>> {
        let (metodo_post, html) = self
            .passos
            .get(self.proximo)
            .expect("o IES não pode exceder as cinco interações gravadas");
        assert_eq!(
            *metodo_post, post,
            "método do passo {} da cassette",
            self.proximo
        );
        self.proximo += 1;
        if post {
            self.posts += 1;
        } else {
            self.gets += 1;
        }
        RespostaDoPost {
            resposta: RespostaWeb {
                status_code: 200,
                mime: "text/html; charset=utf-8".to_owned(),
                content_length: None,
                localizacao: None,
                corpo: Cursor::new(html.as_bytes().to_vec()),
            },
            localizacao: None,
        }
    }
}
impl fluxo::TransporteLupa for TransporteGravado {
    type Corpo = Cursor<Vec<u8>>;
    fn get(
        &mut self,
        _url: &str,
        _cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        Ok(self.resposta(false))
    }
    fn post(
        &mut self,
        _submissao: SubmissaoDerivada,
        _cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        Ok(self.resposta(true))
    }
}

fn observar_formulario_da_lupa_unb(html: &str, passo: usize) {
    let servido = HtmlServido::de_teste(html);
    let dom = crate::tela_html::interpretar(&servido);
    let form = crate::formulario_web::localizar_formulario(&dom, "frmUnidadeLista")
        .expect("formulario da lupa UnB");
    let mut submits_nomeados = 0;
    let mut submits_sem_nome = 0;
    let mut controles = 0;
    for tag in form
        .children()
        .all(dom.parser())
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        let elemento = tag.name().as_utf8_str();
        if elemento != "input" && elemento != "button" {
            continue;
        }
        controles += 1;
        let tipo = crate::html_atributos::valor_do_atributo(tag, "type").expect("tipo UnB");
        if tipo
            .as_deref()
            .is_some_and(|tipo| tipo.eq_ignore_ascii_case("submit"))
        {
            let nome = crate::html_atributos::valor_do_atributo(tag, "name").expect("nome UnB");
            let id = crate::html_atributos::valor_do_atributo(tag, "id").expect("id UnB");
            let id_publico = id.as_deref().filter(|x| {
                x.len() <= 32 && x.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
            });
            println!(
                "observacao UnB submit passo {passo}: id={:?}, name_btnPesquisar={}, desabilitado={}",
                id_publico.unwrap_or("<sem-id-ou-opaco>"),
                nome.as_deref() == Some("btnPesquisar"),
                crate::html_atributos::atributo_presente(tag, "disabled").expect("disabled UnB")
            );
            if nome.as_deref() == Some("btnPesquisar") {
                submits_nomeados += 1;
            } else if nome.is_none() {
                submits_sem_nome += 1;
            }
        }
    }
    println!(
        "observacao UnB formulario passo {passo}: controles={controles}, submits_nomeados={submits_nomeados}, submits_sem_nome={submits_sem_nome}"
    );
    let lida = ler_pagina(&HtmlServido::de_teste(html))
        .unwrap_or_else(|e| panic!("lupa UnB passo {passo}: {e:?}"));
    assert!(lida.paginas >= 1);
    assert!(lida.pagina < lida.paginas);
    assert!(lida.unidades.len() <= lida.total);
    println!(
        "observacao UnB lupa passo {passo}: paginas={}, linhas={}",
        lida.paginas,
        lida.unidades.len()
    );
}

/// Como a lista `UnB` publica o `btnNovo` (observação, sem asserção).
fn observar_btn_novo_unb(html: &HtmlServido) {
    let dom = crate::tela_html::interpretar(html);
    for no in
        crate::html_atributos::nos_com_id(&dom, "btnNovo").expect("id do btnNovo sem ambiguidade")
    {
        let tag = no
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .expect("btnNovo é elemento HTML");
        let href = crate::html_atributos::valor_do_atributo(tag, "href").expect("href válido");
        let onclick =
            crate::html_atributos::valor_do_atributo(tag, "onclick").expect("onclick válido");
        println!(
            "observacao UnB btnNovo: tag={}, href_url={}, onclick_url={}, onclick_location={}, onclick_urls={}, alvo_cadastro={}",
            tag.name().as_utf8_str(),
            href.as_deref()
                .is_some_and(|x| x.contains("controlador.php?")),
            onclick
                .as_deref()
                .is_some_and(|x| x.contains("controlador.php?")),
            onclick.as_deref().is_some_and(|x| x.contains("location")),
            onclick
                .as_deref()
                .map_or(0, |x| x.matches("controlador.php?").count()),
            onclick
                .as_deref()
                .is_some_and(|x| x.contains("acao=grupo_unidade_cadastrar"))
        );
    }
}

#[test]
#[ignore = "exige ZIP público UnB em IES_ACERVO_CACHE; executado no CI"]
fn observacao_dos_links_da_lupa_unb_no_acervo() {
    // Pré-RED da #244: gravação existente, sem acesso ao SEI.
    // Não imprimir URLs assinadas, hashes, cookies nem HTML do acervo.
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    let card = raiz.join("cassettes/unb/unidade-destinos-no-indice-unb--57497bc11c4f.md");
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || {
            Path::new(&std::env::var_os("HOME").expect("HOME ou IES_ACERVO_CACHE"))
                .join(".cache/ies")
        },
        std::path::PathBuf::from,
    );
    let zip = ZipDoAcervo::ler(&cache.join("lote-5b6af062ae69d8c7.zip"))
        .expect("ZIP público UnB ausente");
    let conferida = conferir_cassette_ate(&card, &zip, 120_000_000)
        .expect("cassette UnB não confere hashes ou origem");
    assert_eq!(conferida.instalacao, "unb");
    let mut get = 0;
    let mut post = 0;
    let origem = "https://sei.treinamento.unb.br/sei";
    let url_tela = "https://sei.treinamento.unb.br/sei/controlador.php";
    for resposta in gravacao::respostas(&conferida.json) {
        if resposta.metodo == "POST" {
            post += 1;
        } else {
            assert_eq!(resposta.metodo, "GET");
            get += 1;
        }
        assert_eq!(resposta.status, 200, "passo {}", resposta.passo);
        if resposta.operacao == "unidade_selecionar_envio_processo" {
            let html = html_da_resposta(resposta.interacao)
                .unwrap_or_else(|_| panic!("lupa UnB ilegível no passo {}", resposta.passo));
            observar_formulario_da_lupa_unb(&html, resposta.passo);
            continue;
        }
        let proxima = if resposta.operacao == "procedimento_controlar" {
            Some(AcaoDoSei::GrupoUnidadeListar)
        } else if resposta.operacao == "grupo_unidade_listar" {
            Some(AcaoDoSei::GrupoUnidadeCadastrar)
        } else if resposta.operacao == "grupo_unidade_cadastrar" {
            Some(AcaoDoSei::UnidadeSelecionarEnvioProcesso)
        } else {
            None
        };
        let Some(proxima) = proxima else { continue };
        let html = html_da_resposta(resposta.interacao).map_or_else(
            |_| panic!("resposta UnB ilegível no passo {}", resposta.passo),
            |html| HtmlServido::de_teste(&html),
        );
        if resposta.operacao == "grupo_unidade_listar" {
            observar_btn_novo_unb(&html);
        }
        let candidatos =
            fluxo::candidatos(&html, proxima).expect("controle UnB precisa ser analisável");
        let distintos: std::collections::HashSet<_> = candidatos
            .iter()
            .filter(|u| u.contains(&format!("acao={}", proxima.como_na_url())))
            .collect();
        let resultado = fluxo::destino(&html, url_tela, origem, proxima);
        let estado = match resultado {
            Ok(_) => "unico",
            Err(fluxo::FalhaDaLupa::LinkAmbiguo) => "ambiguo",
            Err(fluxo::FalhaDaLupa::LinkAusente) => "ausente",
            Err(_) => "outra_recusa",
        };
        println!(
            "observacao UnB passo {}: proxima={proxima:?}, candidatos={}, distintos={}, estado={estado}",
            resposta.passo,
            candidatos.len(),
            distintos.len()
        );
        assert_eq!(
            estado, "unico",
            "cassette UnB deve publicar um destino autorizado"
        );
    }
    assert_eq!((get, post), (4, 1), "apenas 5 respostas gravadas, sem rede");
}

#[test]
#[ignore = "exige ZIP público UnB em IES_ACERVO_CACHE; executado no CI"]
fn replay_do_fluxo_completo_unb() {
    // Cinco respostas reais gravadas (4 GET + 1 POST), sem rede.
    // Reproduz os HTMLs na ordem da cassette, com cookie apenas sintético.
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    let card = raiz.join("cassettes/unb/unidade-destinos-no-indice-unb--57497bc11c4f.md");
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || {
            Path::new(&std::env::var_os("HOME").expect("HOME ou IES_ACERVO_CACHE"))
                .join(".cache/ies")
        },
        std::path::PathBuf::from,
    );
    let zip = ZipDoAcervo::ler(&cache.join("lote-5b6af062ae69d8c7.zip"))
        .expect("ZIP público UnB ausente");
    let conferida = conferir_cassette_ate(&card, &zip, 120_000_000)
        .expect("cassette UnB não confere hashes ou origem");
    assert_eq!(conferida.instalacao, "unb");
    let passos: Vec<_> = gravacao::respostas(&conferida.json)
        .into_iter()
        .map(|r| {
            assert_eq!(r.status, 200);
            (
                r.metodo == "POST",
                html_da_resposta(r.interacao).expect("HTML público da cassette deve ser legível"),
            )
        })
        .collect();
    assert_eq!(passos.len(), 5);
    let mut transporte = TransporteGravado {
        passos,
        proximo: 0,
        gets: 0,
        posts: 0,
    };
    let mut requisicoes = 0;
    let resultado = fluxo::ler_integral_com_contador(
        &mut transporte,
        "https://sei.treinamento.unb.br/sei/controlador.php?acao=procedimento_controlar&infra_hash=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "https://sei.treinamento.unb.br/sei",
        "PHPSESSID=SINTETICO",
        &mut requisicoes,
    );
    println!(
        "fluxo UnB: resultado={}, GET={}, POST={}, tentativas={}",
        if resultado.is_ok() { "ok" } else { "recusado" },
        transporte.gets,
        transporte.posts,
        requisicoes
    );
    assert!(
        resultado.is_ok(),
        "o replay UnB completo deve ser aceito: {:?}",
        resultado.err()
    );
    assert_eq!((transporte.gets, transporte.posts, requisicoes), (4, 1, 5));
}
