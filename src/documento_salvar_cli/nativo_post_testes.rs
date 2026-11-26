use super::*;

// Hipótese sintética, não replay: percorre o leitor de editor clássico,
// o localizador da foto e o mesmo builder do POST de produção.
#[test]
fn editor_classico_conserva_vizinhas_vazias_no_post() {
    use crate::formulario_web::CodificacaoDaTela;
    use crate::transporte::sessao::CookieDaSessao;

    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let hash = "a".repeat(64);
    let html = formulario_classico(&hash);
    let protocolo = Protocolo::ler(&html).unwrap();
    assert_eq!(protocolo.versao().unwrap().numero(), 1);
    let principal = protocolo.principal().unwrap();
    assert_eq!(principal, "txaEditor_12");
    let secoes = protocolo.secoes();
    assert_eq!(secoes.len(), 5);
    assert_eq!(secoes[1].html, "");
    assert_eq!(secoes[3].html, "");
    let antes = "<body><h1>Fixo</h1>";
    let depois = "<hr><div unselectable=\"on\"></div></body>";
    let base = format!("{antes}<p>Unidade</p>{depois}");
    let edicao = format!("{antes}<p>Novo\r\r\ntexto{depois}");
    let editado = copia_renderizada::corpo_editado(&base, &edicao, secoes, principal).unwrap();
    let htmls = htmls_para_post(secoes, principal, editado);
    assert_eq!(htmls, ["<h1>Fixo</h1>", "", "<p>Novo\ntexto", "", "<hr>"]);
    let origem = instalacao.url_base();
    let url = format!(
        "{origem}/controlador.php?acao=editor_montar&id_procedimento=10&id_documento=21&infra_hash={hash}"
    );
    let cookie = CookieDaSessao::sintetico("PHPSESSID=SINTETICO");
    let sessao = SessaoDoProcesso {
        url_do_controle: &url,
        origem: &origem,
        cookie: &cookie,
    };
    let alvo = alvo_classico();
    let alvo_edicao = AlvoDaEdicao {
        instalacao: &instalacao,
        sessao: &sessao,
        unidade: None,
        documento: &alvo,
    };
    let mut envio = Roteiro {
        tela: String::new(),
        consulta: String::new(),
        editor: String::new(),
        sinal: 'N',
        pedidos: Vec::new(),
        posts: Vec::new(),
    };
    for (campo, outro) in [
        ("id_documento=21", "id_documento=22"),
        ("id_procedimento=10", "id_procedimento=11"),
    ] {
        let editor = Editor {
            url: url.clone(),
            html: HtmlServido::de_teste(&html.replace(campo, outro)),
            codificacao: CodificacaoDaTela::Utf8,
        };
        assert!(matches!(
            enviar_classico(
                &mut envio,
                &alvo_edicao,
                &editor,
                secoes,
                principal,
                editado
            ),
            Err(ErroEditorNativo::FormularioDeOutroDocumento)
        ));
        assert_eq!(envio.posts.len(), 0);
    }
    let editor = Editor {
        url: url.clone(),
        html,
        codificacao: CodificacaoDaTela::Utf8,
    };
    let aceite = enviar_classico(
        &mut envio,
        &alvo_edicao,
        &editor,
        secoes,
        principal,
        editado,
    )
    .unwrap();
    assert_eq!(aceite.map(VersaoDocumento::numero), Some(2));
    assert_eq!(envio.pedidos.len(), 0);
    assert_eq!(envio.posts.len(), 1);
    let corpo = String::from_utf8(envio.posts[0].corpo().to_vec()).unwrap();
    conferir_campos_classicos(&corpo);
}

fn conferir_campos_classicos(corpo: &str) {
    let campos: Vec<_> = corpo.split('&').collect();
    for campo in [
        "txaEditor_4=%3Ch1%3EFixo%3C%2Fh1%3E",
        "txaEditor_5=",
        "txaEditor_12=%3Cp%3ENovo%0Atexto",
        "txaEditor_6=",
        "txaEditor_7=%3Chr%3E",
        "hdnVersao=1",
        "hdnIgnorarNovaVersao=N",
    ] {
        assert_eq!(campos.iter().filter(|c| **c == campo).count(), 1, "{campo}");
    }
}

fn alvo_classico() -> DocumentoNaArvoreAberto {
    use crate::arvore_processo::ProcessoNaArvore;
    use crate::arvore_processo::tests::{NUP, interno};
    use crate::processo_obter::ResolvidoPor;
    use crate::transporte::sessao::CookieLido;

    DocumentoNaArvoreAberto {
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
        html: HtmlServido::de_teste(""),
        requisicoes: 0,
        cookie: CookieLido::sintetico("PHPSESSID=SINTETICO"),
    }
}

fn formulario_classico(hash: &str) -> HtmlServido {
    HtmlServido::de_teste(&format!(
        "<form id='frmEditor' method='post' action='controlador.php?acao=editor_salvar&amp;id_procedimento=10&amp;id_documento=21&amp;infra_hash={hash}'>\
         <input type='hidden' name='hdnVersao' value='1'>\
         <input type='hidden' name='hdnIgnorarNovaVersao' value='N'>\
         <textarea name='txaEditor_4' readonly>&lt;h1&gt;Fixo&lt;/h1&gt;</textarea>\
         <textarea name='txaEditor_5' readonly></textarea>\
         <textarea name='txaEditor_12'>&lt;p&gt;@unidade@&lt;/p&gt;</textarea>\
         <textarea name='txaEditor_6' readonly></textarea>\
         <textarea name='txaEditor_7' readonly>&lt;hr&gt;</textarea></form>\
         <script>CKEDITOR.replace('txaEditor_12', {{\"readOnly\":false}});</script>"
    ))
}

// Transporte exclusivamente em processo: a resposta vem do anti-IES/SQLite,
// inclusive a recusa de versão, em vez de um aceite roteirizado.
struct EnvioAntiIes<'a> {
    banco: &'a rusqlite::Connection,
    instalacao: &'a Instalacao,
    posts: usize,
}

impl EnvioAntiIes<'_> {
    fn responder(&self, metodo: &str, url: &str, corpo: &str) -> anti_ies::Resposta {
        let host = format!("https://{}", self.instalacao.host());
        anti_ies::editor_documento::responder(
            metodo,
            url.strip_prefix(&host).unwrap(),
            corpo,
            self.banco,
            self.instalacao,
            "ensaio",
        )
    }
}

impl EnvioDoControle for EnvioAntiIes<'_> {
    type Corpo = Cursor<Vec<u8>>;

    fn get(&mut self, url: &str, _: &str) -> Result<RespostaWeb<Self::Corpo>, FalhaTransporte> {
        let r = self.responder("GET", url, "");
        Ok(resposta(
            i32::from(r.estado.codigo_http()),
            r.tipo_conteudo,
            r.corpo.into_bytes(),
            None,
        ))
    }

    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        _: &str,
    ) -> Result<crate::formulario_web::RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        self.posts += 1;
        let r = self.responder(
            "POST",
            submissao.url(),
            std::str::from_utf8(submissao.corpo()).unwrap(),
        );
        Ok(crate::formulario_web::RespostaDoPost {
            resposta: resposta(
                i32::from(r.estado.codigo_http()),
                r.tipo_conteudo,
                r.corpo.into_bytes(),
                None,
            ),
            localizacao: r.localizacao,
        })
    }
}

fn preparar_anti_ies(instalacao: &Instalacao) -> rusqlite::Connection {
    use iesei::documento_versoes::{DocumentoSimulado, Revisao, guardar};
    use iesei::editor_documento::{Oferta, registrar_oferta};
    use rusqlite::Connection;
    use std::num::NonZeroU32;

    let mut banco = Connection::open_in_memory().unwrap();
    iesei::banco_cassettes::iniciar(&banco).unwrap();
    guardar(
        &mut banco,
        "ensaio",
        &DocumentoSimulado {
            instalacao: instalacao.clone(),
            id_documento: "21".into(),
            id_procedimento: "10".into(),
            numero_processo: "00001.000001/2026-01".into(),
            numero: "0000101".into(),
            tipo: "Despacho".into(),
            revisoes: vec![Revisao {
                versao: NonZeroU32::new(1).unwrap(),
                usuario: "Teste".into(),
                unidade: "TESTE".into(),
                modificacao: "26/11/2026".into(),
                html_corpo: "<p>Antes</p>".into(),
            }],
        },
    )
    .unwrap();
    registrar_oferta(
        &banco,
        instalacao,
        "ensaio",
        "21",
        Oferta {
            editavel: true,
            assinaturas: 0,
        },
    )
    .unwrap();
    banco
}

fn ler_editor_anti_ies(envio: &mut EnvioAntiIes<'_>, url: &str) -> Editor {
    let r = envio.get(url, "").unwrap();
    let codificacao = codificacao_da_tela(&r.mime).unwrap();
    Editor {
        url: url.to_owned(),
        html: decodificar_resposta(r).unwrap(),
        codificacao,
    }
}

#[test]
fn formulario_classico_anti_ies_usa_leitor_e_post_nativos() {
    use crate::transporte::sessao::CookieDaSessao;
    use iesei::documento_versoes::ler;

    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let banco = preparar_anti_ies(&instalacao);
    let origem = instalacao.url_base();
    let url =
        format!("{origem}/controlador.php?acao=editor_montar&id_procedimento=10&id_documento=21");
    let cookie = CookieDaSessao::sintetico("PHPSESSID=SINTETICO");
    let sessao = SessaoDoProcesso {
        url_do_controle: &url,
        origem: &origem,
        cookie: &cookie,
    };
    let alvo = alvo_classico();
    let alvo_edicao = AlvoDaEdicao {
        instalacao: &instalacao,
        sessao: &sessao,
        unidade: None,
        documento: &alvo,
    };
    let mut envio = EnvioAntiIes {
        banco: &banco,
        instalacao: &instalacao,
        posts: 0,
    };
    let editor = ler_editor_anti_ies(&mut envio, &url);
    let protocolo = Protocolo::ler(&editor.html).unwrap();
    let principal = protocolo.principal().unwrap();
    assert_eq!(principal, "txaEditor_1");
    assert_eq!(protocolo.versao().unwrap().numero(), 1);
    assert_eq!(protocolo.secoes()[0].html, "<p>Antes</p>");

    let aceite = enviar_classico(
        &mut envio,
        &alvo_edicao,
        &editor,
        protocolo.secoes(),
        principal,
        "<p>Depois</p>",
    );
    let relida = reabrir_apos_aceite(aceite, 2, || ler_editor_anti_ies(&mut envio, &url))
        .unwrap()
        .unwrap();
    assert_eq!(envio.posts, 1);
    let fresco = Protocolo::ler(&relida.html).unwrap();
    assert_eq!(fresco.versao().unwrap().numero(), 2);
    assert_eq!(fresco.secoes()[0].html, "<p>Depois</p>");
    assert!(releitura::confere(&protocolo, &fresco, 2, "<p>Depois</p>"));

    // Segundo escritor independente, com a fotografia de versão 1,
    // e conteúdo diferente: não é retry da operação anterior.
    let recusa = enviar_classico(
        &mut envio,
        &alvo_edicao,
        &editor,
        protocolo.secoes(),
        principal,
        "<p>Concorrente</p>",
    );
    assert!(matches!(recusa, Err(ErroEditorNativo::EnvioSemAceite(_))));
    assert_eq!(envio.posts, 2);
    let documento = ler(&banco, &instalacao, "ensaio", "21").unwrap().unwrap();
    assert_eq!(documento.revisoes.len(), 2);
    assert_eq!(documento.revisoes[0].html_corpo, "<p>Antes</p>");
    assert_eq!(documento.revisoes[1].html_corpo, "<p>Depois</p>");
}
