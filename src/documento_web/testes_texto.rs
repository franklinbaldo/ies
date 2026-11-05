use super::*;

/// O corpo servido pelo `documento_visualizar` traz entidades nomeadas
/// do Latin-1 (`&aacute;`, `&nbsp;`, `&ordm;`), observadas ao vivo na
/// `UnB` (Refs #60): o texto publicado é o decodificado.
#[test]
fn entidades_do_corpo_sao_decodificadas_no_texto() {
    let html = "<body><p>Coordena&ccedil;&atilde;o&nbsp;&nbsp;de Gest&atilde;o</p>\
                    <p>Processo n&ordm; 1 &amp; SEI n&#186; 2 &eacute; &desconhecida;</p></body>";
    assert_eq!(
        texto_do_corpo(html),
        Ok("Coordenação de Gestão Processo nº 1 & SEI nº 2 é &desconhecida;".to_owned())
    );
}

#[test]
fn elementos_executaveis_visiveis_continuam_recusados_por_tipo() {
    for tag in ["script", "style", "iframe", "noscript"] {
        let html = format!("<body><{tag}>oculto</{tag}><p>Visível</p></body>");
        assert_eq!(
            texto_do_corpo(&html),
            Err(ErroCorpoDocumento::CorpoComConteudoExecutavel),
            "{tag}"
        );
    }
}

#[test]
fn script_em_template_inerte_nao_impede_texto_do_documento() {
    let html = "<body><template><script>nao_executado()</script></template>\
                    <p>Texto visível legítimo.</p></body>";
    assert_eq!(
        texto_do_corpo(html),
        Ok("Texto visível legítimo.".to_owned())
    );
}

#[test]
fn iframe_e_style_em_template_nao_substituem_texto_visivel() {
    let html = "<body><template><style>.segredo{}</style>\
                    <iframe src='about:blank'>oculto</iframe></template>\
                    <p>Corpo exibido.</p></body>";
    assert_eq!(texto_do_corpo(html), Ok("Corpo exibido.".to_owned()));
}

#[test]
fn template_com_script_sem_paragrafo_visivel_eh_corpo_vazio() {
    assert_eq!(
        texto_do_corpo("<body><template><script>inativo()</script></template></body>"),
        Err(ErroCorpoDocumento::CorpoVazio)
    );
}

#[test]
fn svg_de_icone_nao_entra_no_texto_do_documento() {
    let html = "<html><head><title>SEI/UnB - 0007 - Ofício</title></head>\
                    <body><svg><title>Abrir menu</title><text>Controle</text></svg>\
                    <p>Texto do documento.</p></body></html>";
    assert_eq!(texto_do_corpo(html), Ok("Texto do documento.".to_owned()));
}

#[test]
fn template_nao_vaza_rascunho_nem_substitui_texto_real() {
    let html = "<body><template><p>Rascunho não exibido</p></template>\
                    <p>Texto público de teste.</p></body>";
    assert_eq!(
        texto_do_corpo(html),
        Ok("Texto público de teste.".to_owned())
    );
}

#[test]
fn corpo_apenas_com_decoracao_recusa_com_erro_tipado() {
    let html = "<body><svg><title>Ícone</title></svg>\
                    <template><p>Não renderizado</p></template></body>";
    assert_eq!(texto_do_corpo(html), Err(ErroCorpoDocumento::CorpoVazio));
}

#[test]
fn extrai_somente_texto_visivel_de_paragrafos() {
    let html = "<html><head><title>SEI/UnB - 0007 - Ofício</title><style>SEGREDO_CSS</style></head><body><p>Olá,</p><p>mundo!</p></body></html>";
    assert_eq!(texto_do_corpo(html), Ok("Olá, mundo!".to_owned()));
}
#[test]
fn corpo_sem_texto_tem_erro_tipado() {
    assert_eq!(
        texto_do_corpo("<html><body></body></html>"),
        Err(ErroCorpoDocumento::CorpoVazio)
    );
}
#[test]
fn corpo_ausente_tem_erro_distinto() {
    assert_eq!(
        texto_do_corpo("<html><head></head></html>"),
        Err(ErroCorpoDocumento::Html(
            crate::erro_html::ErroHtml::ausente(
                "corpo_documento_ausente",
                "a tela não tem o elemento observado"
            )
        ))
    );
}
#[test]
fn duas_areas_de_corpo_nao_omitem_texto_silenciosamente() {
    let html =
        "<html><body><p>Primeiro documento</p></body><body><p>Segundo documento</p></body></html>";
    assert_eq!(
        texto_do_corpo(html),
        Err(ErroCorpoDocumento::Html(
            crate::erro_html::ErroHtml::repetido(
                "corpos_de_documento_repetidos",
                "o elemento aparece mais de uma vez"
            )
        ))
    );
}
#[test]
fn recusa_scripts_no_corpo_para_nao_expor_tokens() {
    let html = "<body><script>infra_hash=SEGREDO</script><p>Conteúdo</p></body>";
    assert_eq!(
        texto_do_corpo(html),
        Err(ErroCorpoDocumento::CorpoComConteudoExecutavel)
    );
}
