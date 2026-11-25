use super::*;
use std::cell::Cell;

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
    for sinal in [Some('N'), Some('S')] {
        let html = com_barra(&arvore, 1, &barra("10", "21", sinal));
        // S também é oferta, não autorização para perder assinaturas:
        // a consulta fresca de assinatura continua obrigatória.
        assert!(conferir_oferta(&instalacao, &html, "21", "10").is_ok());
    }

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
