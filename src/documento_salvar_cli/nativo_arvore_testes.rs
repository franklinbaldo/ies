//! Navegação sintética pela rota HTTP real do anti-IES; POST pelo builder nativo.
use super::*;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::time::Duration;

fn get_http(
    banco: &rusqlite::Connection,
    instalacao: &Instalacao,
    alvo: &str,
) -> RespostaWeb<Cursor<Vec<u8>>> {
    let escuta = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let endereco = escuta.local_addr().unwrap();
    let bruto = std::thread::scope(|escopo| {
        let cliente = escopo.spawn(|| {
            let mut tcp = TcpStream::connect(endereco).unwrap();
            tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            write!(
                tcp,
                "GET {alvo} HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            )
            .unwrap();
            tcp.shutdown(Shutdown::Write).unwrap();
            let mut bruto = String::new();
            tcp.read_to_string(&mut bruto).unwrap();
            bruto
        });
        let (mut fluxo, _) = escuta.accept().unwrap();
        fluxo
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        anti_ies::atender_http(&mut fluxo, Some((banco, instalacao, "ensaio"))).unwrap();
        drop(fluxo);
        cliente.join().unwrap()
    });
    let (cabecalho, corpo) = bruto.split_once("\r\n\r\n").unwrap();
    let status = cabecalho.split_ascii_whitespace().nth(1).unwrap().parse().unwrap();
    let mime = cabecalho
        .lines()
        .find_map(|linha| linha.strip_prefix("Content-Type: "))
        .unwrap();
    resposta(status, mime, corpo.as_bytes().to_vec(), Some(corpo.len()))
}

fn arvore_http(instalacao: &Instalacao, banco: &rusqlite::Connection) -> HtmlServido {
    let r = get_http(
        banco,
        instalacao,
        "/sei/controlador.php?acao=procedimento_visualizar&id_procedimento=10&id_documento=21",
    );
    assert_eq!(r.status_code, 200);
    decodificar_resposta(r).unwrap()
}

struct EnvioDaArvore<'a> {
    interno: EnvioAntiIes<'a>,
    gets: usize,
}

impl EnvioDoControle for EnvioDaArvore<'_> {
    type Corpo = Cursor<Vec<u8>>;

    fn get(&mut self, url: &str, _: &str) -> Result<RespostaWeb<Self::Corpo>, FalhaTransporte> {
        self.gets += 1;
        let origem = format!("https://{}", self.interno.instalacao.host());
        Ok(get_http(
            self.interno.banco,
            self.interno.instalacao,
            url.strip_prefix(&origem).unwrap(),
        ))
    }

    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<crate::formulario_web::RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        self.interno.post(submissao, cookie)
    }
}

#[test]
fn arvore_anti_ies_respeita_guarda_nativa_antes_do_editor() {
    use iesei::editor_documento::{Oferta, registrar_oferta};
    use std::num::NonZeroU32;

    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let banco = preparar_anti_ies(&instalacao, NonZeroU32::new(1).unwrap(), "<p>Antes</p>");
    let mut envio = EnvioDaArvore {
        interno: EnvioAntiIes {
            banco: &banco,
            instalacao: &instalacao,
            posts: 0,
        },
        gets: 0,
    };
    let mut alvo = alvo_classico();
    for (editavel, assinaturas) in [(false, 0), (true, 1), (true, 0)] {
        registrar_oferta(
            &banco,
            &instalacao,
            "ensaio",
            "21",
            Oferta {
                editavel,
                assinaturas,
            },
        )
        .unwrap();
        alvo.html = arvore_http(&instalacao, &banco);
        let resultado = abrir_editor(&mut envio, &instalacao, &alvo);
        match (editavel, assinaturas) {
            (false, _) => assert!(matches!(
                resultado,
                Err(ErroEditorNativo::LinkDoEditorNaoConferido)
            )),
            (true, 1) => assert!(matches!(
                resultado,
                Err(ErroEditorNativo::DocumentoAssinado)
            )),
            (true, 0) => {
                let editor = resultado.unwrap();
                let protocolo = Protocolo::ler(&editor.html).unwrap();
                assert_eq!(protocolo.versao().unwrap().numero(), 1);
                assert_eq!(protocolo.secoes()[0].html, "<p>Antes</p>");
            }
            _ => unreachable!(),
        }
        if !editavel || assinaturas != 0 {
            assert_eq!(envio.gets, 0);
        }
    }
    assert_eq!(envio.interno.posts, 0);
    let documento = iesei::documento_versoes::ler(&banco, &instalacao, "ensaio", "21")
        .unwrap()
        .unwrap();
    assert_eq!(documento.revisoes.len(), 1);
    assert_eq!(documento.revisoes[0].html_corpo, "<p>Antes</p>");
}

// A fonte fornece os nomes, a ordem, a principal e o conteúdo da revisão N.
// Identidade, oferta e edição N+1 são sintéticas; não reproduzem POST real.
fn conferir_projecao_multissecao(html: &HtmlServido) {
    use iesei::editor_documento::{Secao as SecaoPersistida, ler_secoes, registrar_secoes};
    use std::num::NonZeroU32;

    let original = Protocolo::ler(html).unwrap();
    let principal = original.principal().unwrap();
    let versao = NonZeroU32::new(original.versao().unwrap().numero()).unwrap();
    let secoes: Vec<_> = original
        .secoes()
        .iter()
        .map(|s| SecaoPersistida {
            nome: s.nome.clone(),
            principal: s.nome == principal,
            html: s.html.clone(),
        })
        .collect();
    assert!(secoes.len() > 1);
    let corpo = &secoes.iter().find(|s| s.principal).unwrap().html;
    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let banco = preparar_anti_ies(&instalacao, versao, corpo);
    registrar_secoes(&banco, &instalacao, "ensaio", "21", versao, &secoes).unwrap();
    let origem = instalacao.url_base();
    let url =
        format!("{origem}/controlador.php?acao=editor_montar&id_procedimento=10&id_documento=21");
    let mut envio = EnvioDaArvore {
        interno: EnvioAntiIes {
            banco: &banco,
            instalacao: &instalacao,
            posts: 0,
        },
        gets: 0,
    };
    let mut alvo = alvo_classico();
    alvo.html = arvore_http(&instalacao, &banco);
    let editor = abrir_editor(&mut envio, &instalacao, &alvo).unwrap();
    let projetado = Protocolo::ler(&editor.html).unwrap();
    assert_eq!(
        projetado.versao().map(VersaoDocumento::numero),
        Some(versao.get())
    );
    assert!(projetado.principal() == Some(principal));
    assert_eq!(projetado.secoes().len(), secoes.len());
    assert!(
        projetado
            .secoes()
            .iter()
            .zip(&secoes)
            .all(|(a, b)| a.nome == b.nome && a.html == b.html),
        "projeção alterou ordem ou conteúdo"
    );
    assert!(ler_secoes(&banco, &instalacao, "ensaio", "21", versao).unwrap() == secoes);

    let cookie = crate::transporte::sessao::CookieDaSessao::sintetico("PHPSESSID=SINTETICO");
    let sessao = SessaoDoProcesso {
        url_do_controle: &url,
        origem: &origem,
        cookie: &cookie,
    };
    let alvo_edicao = AlvoDaEdicao {
        instalacao: &instalacao,
        sessao: &sessao,
        unidade: None,
        documento: &alvo,
    };
    let novo = "<p>IES: edição sintética multisseção</p>";
    let nova = NonZeroU32::new(versao.get() + 1).unwrap();
    let aceite = enviar_classico(
        &mut envio,
        &alvo_edicao,
        &editor,
        projetado.secoes(),
        principal,
        novo,
    );
    let relido = reabrir_apos_aceite(aceite, nova.get(), || {
        abrir_editor(&mut envio, &instalacao, &alvo)
    })
        .unwrap()
        .unwrap();
    assert_eq!(envio.interno.posts, 1);
    let fresco = Protocolo::ler(&relido.html).unwrap();
    assert!(releitura::confere(&original, &fresco, nova.get(), novo));
    assert!(ler_secoes(&banco, &instalacao, "ensaio", "21", versao).unwrap() == secoes);
    let mut esperadas = secoes;
    for s in &mut esperadas {
        s.html = if s.principal {
            novo.into()
        } else {
            html_para_envio(&s.html)
        };
    }
    assert!(ler_secoes(&banco, &instalacao, "ensaio", "21", nova).unwrap() == esperadas);
    assert_eq!(fresco.secoes().len(), esperadas.len());
}

#[test]
fn projecao_e_post_multissecao_com_vizinhas_vazias() {
    conferir_projecao_multissecao(&formulario_classico(&"a".repeat(64)));
}

/// Confronta GETs originais com a projeção e uma transição local sintética.
/// Usa apenas loopback HTTP e transporte em processo; não executa o binário CLI.
#[test]
#[ignore = "requer ZIPs originais conferidos em IES_ACERVO_CACHE"]
fn replay_editor_nativo_no_acervo_projecao_multissecao() {
    use crate::cassette::acervo::gravacao;
    use crate::cassette::{ZipDoAcervo, archive_filename, conferir_cassette_ate, html_da_resposta};
    use std::path::{Path, PathBuf};

    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cache = std::env::var_os("IES_ACERVO_CACHE").map_or_else(
        || PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/ies"),
        PathBuf::from,
    );
    for relativo in [
        "cassettes/unb/documento-escritas-unb--36a904749e20.md",
        "cassettes/ufrj/documento-escritas-ufrj--2241d2a8f71b.md",
    ] {
        let card = raiz.join(relativo);
        let nome = archive_filename(&card).unwrap();
        let zip = ZipDoAcervo::ler(&cache.join(nome)).unwrap();
        let captura = conferir_cassette_ate(&card, &zip, 120_000_000).unwrap();
        let mut total = 0;
        for resposta in gravacao::respostas(&captura.json) {
            if resposta.operacao == "editor_montar"
                && resposta.metodo == "GET"
                && resposta.status == 200
            {
                let pagina = html_da_resposta(resposta.interacao).unwrap();
                conferir_projecao_multissecao(&HtmlServido::de_teste(&pagina));
                total += 1;
            }
        }
        assert_eq!(total, 3);
    }
}
