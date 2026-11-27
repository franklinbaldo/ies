//! Editor clássico: transporte, persistência e projeção de versões.
use super::*;

#[test]
fn editor_classico_rele_todas_as_secoes_transportadas() {
    use iesei::editor_documento::{Secao, ler_secoes, registrar_secoes};

    let banco = preparar(0);
    let v2 = NonZeroU32::new(2).unwrap();
    let v3 = NonZeroU32::new(3).unwrap();
    let secoes = vec![
        Secao {
            nome: "txaEditor_4".into(),
            principal: false,
            html: "<h1>Fixo\r\r\n</h1>".into(),
        },
        Secao {
            nome: "txaEditor_5".into(),
            principal: false,
            html: String::new(),
        },
        Secao {
            nome: "txaEditor_12".into(),
            principal: true,
            html: "<p>antes</p>".into(),
        },
    ];
    let foto = |alvo| {
        let resposta =
            anti_ies::documento_versoes::responder("GET", alvo, &banco, &origem(), "ensaio");
        assert_eq!(resposta.estado, Estado::Ok);
        resposta.corpo
    };
    let atual = "/sei/controlador.php?acao=documento_visualizar&id_documento=42";
    let antiga = "/sei/controlador.php?acao=documento_visualizar&id_documento=42&versao=2";
    let legado = foto(atual);
    assert!(legado.contains("<main id='corpoEditavel'><p>antes</p></main>"));
    registrar_secoes(&banco, &origem(), "ensaio", "42", v2, &secoes).unwrap();
    let foto_base = foto(atual);
    assert!(foto_base.contains("<body><h1>Fixo\r\r\n</h1><p>antes</p><div unselectable=\"on\">"));
    assert!(!foto_base.contains("corpoEditavel"));
    let montar = "/sei/controlador.php?acao=editor_montar&id_documento=42";
    let salvar =
        "/sei/editor/editor_processar.php?acao=editor_salvar&id_procedimento=7&id_documento=42";
    let previa = responder("GET", montar, "", &banco, &origem(), "ensaio");
    assert_eq!(previa.estado, Estado::Ok);
    assert!(previa.corpo.contains("name='txaEditor_5'></textarea>"));
    assert!(
        previa
            .corpo
            .contains("CKEDITOR.replace('txaEditor_12', {\"readOnly\":false})")
    );
    let post = "hdnVersao=2&hdnIgnorarNovaVersao=N&txaEditor_4=%3Ch1%3EFixo%0A%3C%2Fh1%3E&txaEditor_5=&txaEditor_12=%3Cp%3Edepois%3C%2Fp%3E";
    let ausente = post.replace("&txaEditor_5=", "");
    let duplicada = format!("{post}&txaEditor_5=");
    let extra = format!("{post}&txaEditor_99=");
    for invalido in [ausente, duplicada, extra] {
        assert_eq!(
            responder("POST", salvar, &invalido, &banco, &origem(), "ensaio").estado,
            Estado::RequisicaoInvalida
        );
    }
    assert_eq!(
        ler_secoes(&banco, &origem(), "ensaio", "42", v3).unwrap(),
        Vec::<Secao>::new()
    );
    assert_eq!(
        responder("POST", salvar, post, &banco, &origem(), "ensaio").estado,
        Estado::Ok
    );
    assert_eq!(
        ler_secoes(&banco, &origem(), "ensaio", "42", v2).unwrap(),
        secoes
    );
    let mut esperado = secoes;
    esperado[0].html = "<h1>Fixo\n</h1>".into();
    esperado[2].html = "<p>depois</p>".into();
    assert_eq!(
        ler_secoes(&banco, &origem(), "ensaio", "42", v3).unwrap(),
        esperado
    );
    let foto_nova = foto(atual);
    assert!(foto_nova.contains("<body><h1>Fixo\n</h1><p>depois</p><div unselectable=\"on\">"));
    assert!(foto_nova.contains("versão 3 por"));
    assert_eq!(foto(antiga), foto_base);
    let releitura = responder("GET", montar, "", &banco, &origem(), "ensaio");
    assert!(releitura.corpo.contains("name='txaEditor_5'></textarea>"));
    assert!(
        releitura
            .corpo
            .contains("name='txaEditor_12'>&lt;p&gt;depois&lt;/p&gt;</textarea>")
    );
    assert_eq!(
        responder("POST", salvar, post, &banco, &origem(), "ensaio").estado,
        Estado::RequisicaoInvalida
    );
}
