use super::*;

// Hipótese sintética, não replay: percorre o leitor de editor clássico,
// o localizador da foto e o mesmo builder do POST de produção.
#[test]
fn editor_classico_conserva_vizinhas_vazias_no_post() {
    use crate::arvore_processo::ProcessoNaArvore;
    use crate::arvore_processo::tests::{NUP, interno};
    use crate::formulario_web::CodificacaoDaTela;
    use crate::processo_obter::ResolvidoPor;
    use crate::transporte::sessao::{CookieDaSessao, CookieLido};

    let instalacao = Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap();
    let hash = "a".repeat(64);
    let html = HtmlServido::de_teste(&format!(
        "<form id='frmEditor' method='post' action='controlador.php?acao=editor_salvar&amp;id_procedimento=10&amp;id_documento=21&amp;infra_hash={hash}'>\
         <input type='hidden' name='hdnVersao' value='1'>\
         <input type='hidden' name='hdnIgnorarNovaVersao' value='N'>\
         <textarea name='txaEditor_4' readonly>&lt;h1&gt;Fixo&lt;/h1&gt;</textarea>\
         <textarea name='txaEditor_5' readonly></textarea>\
         <textarea name='txaEditor_12'>&lt;p&gt;@unidade@&lt;/p&gt;</textarea>\
         <textarea name='txaEditor_6' readonly></textarea>\
         <textarea name='txaEditor_7' readonly>&lt;hr&gt;</textarea></form>\
         <script>CKEDITOR.replace('txaEditor_12', {{\"readOnly\":false}});</script>"
    ));
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
        html: HtmlServido::de_teste(""),
        requisicoes: 0,
        cookie: CookieLido::sintetico("PHPSESSID=SINTETICO"),
    };
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
        assert!(envio.posts.is_empty());
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
    assert!(envio.pedidos.is_empty());
    assert_eq!(envio.posts.len(), 1);
    let corpo = String::from_utf8(envio.posts[0].corpo().to_vec()).unwrap();
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
