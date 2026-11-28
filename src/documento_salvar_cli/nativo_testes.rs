use super::*;
use crate::formulario_web::SubmissaoDerivada;
use crate::transporte::RespostaWeb;
use crate::transporte::testes_http::resposta;
use std::cell::Cell;
use std::io::Cursor;

#[test]
fn formulario_classico_envia_lf_em_todas_as_secoes_sem_reescrever_html() {
    use crate::formulario_web::CodificacaoDaTela;

    let secoes = [
        Secao {
            nome: "txaEditor_4".into(),
            html: "<table><tr><td>Antes<br>\r\r\nDepois\r</td></tr></table>".into(),
        },
        Secao {
            nome: "txaEditor_12".into(),
            html: "<p>Original</p>".into(),
        },
    ];
    let editado = "<p class='x'>Novo\r\ntexto\r";
    let htmls = htmls_para_post(&secoes, "txaEditor_12", editado);
    assert_eq!(
        htmls[0],
        "<table><tr><td>Antes<br>\nDepois\n</td></tr></table>"
    );
    assert_eq!(htmls[1], "<p class='x'>Novo\ntexto\n");
    assert_eq!(
        secoes[0].html,
        "<table><tr><td>Antes<br>\r\r\nDepois\r</td></tr></table>"
    );

    // O mesmo builder que enviar_classico utiliza monta o POST efetivo.
    let pares: Vec<_> = secoes
        .iter()
        .zip(&htmls)
        .map(|(s, html)| (s.nome.as_str(), html.as_str()))
        .collect();
    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let hash = "a".repeat(64);
    let html = HtmlServido::de_teste(&format!(
        "<form id='frmEditor' method='post' action='controlador.php?acao=editor_salvar&amp;infra_hash={hash}'>\
         <input type='hidden' name='hdnVersao' value='1'>\
         <input type='hidden' name='hdnIgnorarNovaVersao' value='N'>\
         <textarea name='txaEditor_4'>Antigo</textarea>\
         <textarea name='txaEditor_12'>Antigo</textarea></form>"
    ));
    let url = format!(
        "https://sei.exemplo.gov.br/sei/controlador.php?acao=editor_montar&infra_hash={hash}"
    );
    let submissao = derivar_submissao(
        &EDITAR,
        &instalacao,
        &TelaServida {
            url: &url,
            html: &html,
            codificacao: CodificacaoDaTela::Utf8,
        },
        None,
        &pares,
    )
    .unwrap();
    let corpo = String::from_utf8(submissao.corpo().to_vec()).unwrap();
    assert!(!corpo.contains("%0D"), "POST não deve conter CR escapado");
    assert!(corpo.contains("Antes%3Cbr%3E%0ADepois%0A"));
    assert!(corpo.contains("Novo%0Atexto%0A"));
    assert!(corpo.contains("txaEditor_4="));
    assert!(corpo.contains("txaEditor_12="));
}

#[test]
fn post_aplicado_com_resposta_perdida_e_incerto_sem_novo_envio() {
    use crate::formulario_web::ErroSubmissaoFormulario;

    let post_aplicado = Cell::new(false);
    let tentativas = Cell::new(0usize);
    let releituras = Cell::new(0usize);
    let resposta = {
        tentativas.set(tentativas.get() + 1);
        post_aplicado.set(true); // Simula o servidor gravando antes da perda da resposta.
        Err(ErroEditorNativo::EnvioSemAceite(
            ErroSubmissaoFormulario::EfeitoDaOperacaoNaoAvaliado,
        ))
    };
    let classificada = classificar_resposta_do_post(resposta).unwrap();
    assert!(matches!(classificada, RespostaDoPost::Incerta));
    assert!(post_aplicado.get());
    assert_eq!(tentativas.get(), 1);
    assert_eq!(releituras.get(), 0);

    // JSON malformado recebido após o POST também não permite afirmar recusa.
    let erro_json = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
    assert!(matches!(
        classificar_resposta_do_post(Err(ErroEditorNativo::JsonDoEditor(erro_json))).unwrap(),
        RespostaDoPost::Incerta
    ));
    // A classificação não chama uma closure de leitura ou repetição.
    assert_eq!(tentativas.get(), 1);
}

#[test]
fn erro_ou_aceite_inconclusivo_nao_faz_releitura() {
    let leituras = Cell::new(0usize);
    let reler = || {
        leituras.set(leituras.get() + 1);
        42
    };
    assert!(matches!(
        reabrir_apos_aceite(Err(ErroEditorNativo::EditorForaDaForma), 2, reler),
        Err(ErroEditorNativo::EditorForaDaForma)
    ));
    assert_eq!(leituras.get(), 0);

    let sem_versao = reabrir_apos_aceite(Ok(None), 2, reler).unwrap();
    assert_eq!(sem_versao, None);
    let errada =
        reabrir_apos_aceite(Ok(Some(VersaoDocumento::nova(3).unwrap())), 2, reler).unwrap();
    assert_eq!(errada, None);
    assert_eq!(leituras.get(), 0);

    let correta =
        reabrir_apos_aceite(Ok(Some(VersaoDocumento::nova(2).unwrap())), 2, reler).unwrap();
    assert_eq!(correta, Some(42));
    assert_eq!(leituras.get(), 1);
}

#[test]
fn oferta_de_edicao_vem_da_barra_do_documento_e_nao_de_script_generico() {
    use crate::arvore_processo::tests::{NUP, arvore, no_documento, no_processo};
    use crate::arvore_processo::tests_barra_documento::{barra, com_barra};

    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let arvore = arvore(&[
        no_processo(0, "10", NUP),
        no_documento(1, "21", "10", "0000101"),
        no_documento(2, "22", "10", "0000102"),
    ]);
    let html = com_barra(&arvore, 1, &barra("10", "21", Some('N')));
    assert!(conferir_oferta(&instalacao, &html, "21", "10").is_ok());
    let html = com_barra(&arvore, 1, &barra("10", "21", Some('S')));
    assert!(matches!(
        conferir_oferta(&instalacao, &html, "21", "10"),
        Err(ErroEditorNativo::DocumentoAssinado)
    ));

    for html in [
        arvore.clone(),
        com_barra(&arvore, 1, &barra("10", "21", None)),
        com_barra(&arvore, 1, &barra("10", "21", Some('X'))),
        com_barra(&arvore, 2, &barra("10", "22", Some('N'))),
    ] {
        assert!(matches!(
            conferir_oferta(&instalacao, &html, "21", "10"),
            Err(ErroEditorNativo::LinkDoEditorNaoConferido)
        ));
    }
}

struct Roteiro {
    tela: String,
    consulta: String,
    editor: String,
    sinal: char,
    pedidos: Vec<String>,
    posts: Vec<SubmissaoDerivada>,
}

impl EnvioDoControle for Roteiro {
    type Corpo = Cursor<Vec<u8>>;

    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<crate::formulario_web::RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        assert_eq!(cookie, "PHPSESSID=SINTETICO");
        self.posts.push(submissao);
        Ok(crate::formulario_web::RespostaDoPost {
            resposta: resposta(200, "text/html; charset=utf-8", b"OK 2".to_vec(), None),
            localizacao: None,
        })
    }

    fn get(&mut self, url: &str, _: &str) -> Result<RespostaWeb<Self::Corpo>, FalhaTransporte> {
        self.pedidos.push(url.to_owned());
        let (mime, corpo) = if url == self.tela {
            (
                "text/html; charset=utf-8",
                format!(
                    "<script>\nobjAjaxVerificacaoAssinatura = new infraAjaxComplementar(null,'{}');\nvar linkEditarConteudo = '{}';\n</script>",
                    self.consulta.split_once("/sei/").unwrap().1,
                    self.editor.split_once("/sei/").unwrap().1
                ),
            )
        } else if url == self.consulta {
            (
                "text/xml; charset=utf-8",
                format!(
                    "<resposta><complemento nome='SinAssinado'>{}</complemento></resposta>",
                    self.sinal
                ),
            )
        } else if url == self.editor {
            ("text/html; charset=utf-8", "<html></html>".into())
        } else {
            panic!("GET fora do roteiro")
        };
        Ok(resposta(200, mime, corpo.into_bytes(), None))
    }
}

// Hipótese sintética: divergência entre a barra e a resposta AJAX.
// Exercita a abertura real do editor, sem rede ou cassete fabricada.
#[test]
fn abrir_editor_exige_barra_e_consulta_sem_assinatura() {
    use crate::arvore_processo::ProcessoNaArvore;
    use crate::arvore_processo::tests::{NUP, arvore, interno, no_documento, no_processo};
    use crate::arvore_processo::tests_assinatura::{arvore_assinavel, link_da_tela_do_documento};
    use crate::arvore_processo::tests_barra_documento::{barra, com_barra};
    use crate::processo_obter::ResolvidoPor;
    use crate::transporte::sessao::CookieLido;

    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let origem = instalacao.url_base();
    let hash = "a".repeat(64);
    for (sinal_barra, sinal_ajax, outro_documento, quantidade) in [
        (Some('N'), 'N', false, 3),
        (Some('N'), 'S', false, 2),
        (Some('N'), 'X', false, 2),
        (Some('S'), 'N', false, 0),
        (Some('S'), 'S', false, 0),
        (None, 'N', false, 0),
        (Some('X'), 'N', false, 0),
        (None, 'N', true, 0),
    ] {
        let html = if outro_documento {
            let base = arvore(&[
                no_processo(0, "10", NUP),
                no_documento(1, "21", "10", "0000101"),
                no_documento(2, "22", "10", "0000102"),
            ])
            .replacen(
                "\"about:blank\"",
                &format!("\"{}\"", link_da_tela_do_documento("10", "21")),
                1,
            );
            com_barra(&base, 2, &barra("10", "22", Some('N')))
        } else {
            arvore_assinavel("10", "21", "0000101", &barra("10", "21", sinal_barra), &[])
        };
        let alvo = DocumentoNaArvoreAberto {
            numero: "0000101".into(),
            id_documento: "21".into(),
            rotulo: "Despacho 0000101".into(),
            natureza: interno(),
            processo: ProcessoNaArvore {
                id: "10".into(),
                nup: NUP.into(),
                tipo: "Tipo Sintetico".into(),
            },
            unidade: None,
            resolvido_por: ResolvidoPor::PesquisaRapida,
            html: HtmlServido::de_teste(&html),
            requisicoes: 0,
            cookie: CookieLido::sintetico("PHPSESSID=SINTETICO"),
        };
        let mut envio = Roteiro {
            tela: format!("{origem}/{}", link_da_tela_do_documento("10", "21")),
            consulta: format!(
                "{origem}/controlador_ajax.php?acao_ajax=documento_verificar_assinatura&id_documento=21&infra_hash={hash}"
            ),
            editor: format!(
                "{origem}/controlador.php?acao=editor_montar&id_procedimento=10&id_documento=21&infra_hash={hash}"
            ),
            sinal: sinal_ajax,
            pedidos: Vec::new(),
            posts: Vec::new(),
        };
        let resultado = abrir_editor(&mut envio, &instalacao, &alvo);
        match (sinal_barra, sinal_ajax) {
            (Some('N'), 'N') => assert_eq!(resultado.unwrap().url, envio.editor),
            (Some('N'), 'X') => assert!(matches!(
                resultado,
                Err(ErroEditorNativo::AssinaturaNaoVerificada)
            )),
            (Some('N'), 'S') | (Some('S'), _) => {
                assert!(matches!(
                    resultado,
                    Err(ErroEditorNativo::DocumentoAssinado)
                ));
            }
            _ => assert!(matches!(
                resultado,
                Err(ErroEditorNativo::LinkDoEditorNaoConferido)
            )),
        }
        let esperados = [&envio.tela, &envio.consulta, &envio.editor];
        assert_eq!(
            envio.pedidos.iter().collect::<Vec<_>>(),
            esperados[..quantidade],
            "{sinal_barra:?}/{sinal_ajax}"
        );
        assert!(envio.posts.is_empty());
    }
}

#[path = "nativo_post_testes.rs"]
mod post;
