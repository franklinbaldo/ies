use super::*;
use crate::banco_cassettes::iniciar;
use crate::documento_versoes::{DocumentoSimulado, Revisao, guardar, ler};

fn secoes_iniciais() -> Vec<Secao> {
    vec![
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
            html: "<p>Antes</p>".into(),
        },
    ]
}

#[test]
fn publica_secoes_e_corpo_atomicamente_preservando_versao_anterior() {
    let mut banco = Connection::open_in_memory().unwrap();
    iniciar(&banco).unwrap();
    let origem = Instalacao::interpretar("https://simulado.invalid/sei").unwrap();
    let v2 = NonZeroU32::new(2).unwrap();
    let v3 = NonZeroU32::new(3).unwrap();
    let documento = DocumentoSimulado {
        instalacao: origem.clone(),
        id_documento: "42".into(),
        id_procedimento: "7".into(),
        numero_processo: "00001.000001/2026-01".into(),
        numero: "0011519".into(),
        tipo: "Despacho".into(),
        revisoes: vec![Revisao {
            versao: v2,
            usuario: "TESTE".into(),
            unidade: "TESTE".into(),
            modificacao: "26/11/2026".into(),
            html_corpo: "<p>Antes</p>".into(),
        }],
    };
    guardar(&mut banco, "ensaio", &documento).unwrap();
    registrar_oferta(
        &banco,
        &origem,
        "ensaio",
        "42",
        Oferta {
            editavel: true,
            assinaturas: 2,
        },
    )
    .unwrap();
    let secoes = secoes_iniciais();
    registrar_secoes(&banco, &origem, "ensaio", "42", v2, &secoes).unwrap();
    guardar(&mut banco, "ensaio", &documento).unwrap();
    assert_eq!(
        ler_secoes(&banco, &origem, "ensaio", "42", v2).unwrap(),
        secoes
    );
    let edicao = Edicao {
        versao_base: v2,
        html_base: "<p>Antes</p>",
        html_editado: "<p>Depois</p>",
        confirmar_assinaturas: true,
    };
    let mut novas = secoes.clone();
    novas[0].html = "<h1>Fixo\n</h1>".into();
    novas[2].html = "<p>Depois</p>".into();
    assert!(publicar(&banco, &origem, "ensaio", "42", edicao).is_err());
    assert!(publicar_secoes(&banco, &origem, "ensaio", "42", edicao, &novas[1..]).is_err());
    banco
        .execute_batch(
            "CREATE TRIGGER falha_secao BEFORE INSERT ON documento_revisao_secao
             WHEN NEW.versao=3 AND NEW.nome='txaEditor_5'
             BEGIN SELECT RAISE(ABORT, 'falha sintetica'); END;",
        )
        .unwrap();
    assert!(publicar_secoes(&banco, &origem, "ensaio", "42", edicao, &novas).is_err());
    assert_eq!(
        ler(&banco, &origem, "ensaio", "42").unwrap(),
        Some(documento)
    );
    assert_eq!(
        ler_secoes(&banco, &origem, "ensaio", "42", v3).unwrap(),
        Vec::<Secao>::new()
    );
    assert_eq!(
        consultar_oferta(&banco, &origem, "ensaio", "42")
            .unwrap()
            .unwrap()
            .assinaturas,
        2
    );
    banco.execute_batch("DROP TRIGGER falha_secao").unwrap();
    let publicada = publicar_secoes(&banco, &origem, "ensaio", "42", edicao, &novas).unwrap();
    assert_eq!(publicada.versao_nova, v3);
    assert_eq!(
        ler_secoes(&banco, &origem, "ensaio", "42", v2).unwrap(),
        secoes
    );
    assert_eq!(
        ler_secoes(&banco, &origem, "ensaio", "42", v3).unwrap(),
        novas
    );
    assert!(publicar_secoes(&banco, &origem, "ensaio", "42", edicao, &novas).is_err());
    // Reimportar uma revisão divergente não reutiliza seções antigas.
    // Exercita também o modo SQLite sem enforcement de FK.
    banco.pragma_update(None, "foreign_keys", false).unwrap();
    let mut reimportado = ler(&banco, &origem, "ensaio", "42").unwrap().unwrap();
    reimportado.revisoes[0].html_corpo = "<p>Substituido</p>".into();
    guardar(&mut banco, "ensaio", &reimportado).unwrap();
    assert_eq!(
        ler_secoes(&banco, &origem, "ensaio", "42", v2).unwrap(),
        Vec::<Secao>::new()
    );
    assert_eq!(
        ler_secoes(&banco, &origem, "ensaio", "42", v3).unwrap(),
        novas
    );
}
