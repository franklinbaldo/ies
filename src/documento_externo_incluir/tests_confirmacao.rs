//! Cenários técnicos auxiliares do BDD-332-03/04; transporte exclusivamente sintético.
use super::*;
use crate::arvore_processo::NaturezaDoDocumento;
use crate::documento_externo_incluir::cadastro::observar_cadastro;
use crate::formulario_web::{CodificacaoDaTela, Redirecionamento302};

const URL: &str = "https://sei.exemplo.gov.br/sei/controlador.php?acao=documento_receber&id_procedimento=10&infra_hash=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn html() -> String {
    format!(
        r#"<form id="frmDocumentoCadastro" method="post" action="controlador.php?acao=documento_receber&amp;id_procedimento=10&amp;infra_hash={hash}">
<select id="selSerie" name="selSerie"><option value="293" selected>Anexo</option></select>
<input type="hidden" id="hdnIdSerie" name="hdnIdSerie" value="293">
<input type="hidden" id="hdnFlagDocumentoCadastro" name="hdnFlagDocumentoCadastro" value="1">
<input type="hidden" id="hdnAnexos" name="hdnAnexos" value="">
<input type="text" name="txtDataElaboracao" value="">
<input type="radio" name="rdoFormato" value="N" checked>
<input type="radio" name="rdoNivelAcesso" value="0" checked>
</form><form id="frmAnexos"><input type="file" name="filArquivo"></form>
<script>var arrExt = [];
arrExt[0] = {{nome : "pdf", tamanho : 200}};
objTabelaAnexos.adicionar([arr['nome_upload'],arr['nome'],arr['data_hora'],arr['tamanho'],infraFormatarTamanhoBytes(arr['tamanho'])]);
objUpload = new infraUpload('frmAnexos','controlador.php?acao=documento_upload_anexo&infra_hash=SINTETICO');</script>"#,
        hash = "a".repeat(64)
    )
}

fn foto(novo: bool) -> FotoProcesso {
    let documentos = if novo {
        vec![DocumentoNaArvore {
            id: "42".into(),
            rotulo: "Anexo (123)".into(),
            numero_sei: Some("123".into()),
            pasta: None,
            natureza: NaturezaDoDocumento {
                icone: Some("documento_pdf".into()),
                cancelado: Some(false),
                motivo_do_cancelamento: None,
                dica: None,
            },
        }]
    } else {
        vec![]
    };
    FotoProcesso {
        id: "10".into(),
        documentos,
        pastas_nao_carregadas: false,
    }
}

fn recibo() -> ReciboUpload {
    ReciboUpload {
        nome_upload: "temp-42".into(),
        nome: "prova.pdf".into(),
        tipo: "application/pdf".into(),
        tamanho_bytes: 68,
        data_hora: "11/10/2026 11:07:50".into(),
    }
}

fn contexto() -> (String, Instalacao) {
    (
        html(),
        Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap(),
    )
}

#[test]
fn confirmado_so_apos_cadastro_e_releitura_unica() {
    let (html, instalacao) = contexto();
    let cadastro = observar_cadastro(&html, "Anexo", 68).unwrap();
    let tela = TelaServida {
        url: URL,
        html: &html,
        codificacao: CodificacaoDaTela::Iso88591,
    };
    let mut passos = Vec::new();
    let resultado = confirmar_sequencia(
        &mut passos,
        &instalacao,
        &tela,
        "ABC",
        &cadastro,
        &foto(false),
        "Anexo",
        |p| {
            p.push("upload");
            Ok(recibo())
        },
        |p, s| {
            p.push("cadastro");
            assert!(
                std::str::from_utf8(s.corpo())
                    .unwrap()
                    .contains("hdnAnexos=temp-42")
            );
            Ok(RespostaDoEnvio::Redirecionamento(
                Redirecionamento302::sintetico(
                    Some("controlador.php?acao=arvore_visualizar&id_documento=42"),
                    URL,
                ),
            ))
        },
        |p| {
            p.push("reler");
            Ok(foto(true))
        },
    );
    assert_eq!(resultado.unwrap(), "123");
    assert_eq!(passos, ["upload", "cadastro", "reler"]);
}

#[test]
fn upload_recusado_nao_cadastra_nem_rele() {
    let (html, instalacao) = contexto();
    let cadastro = observar_cadastro(&html, "Anexo", 68).unwrap();
    let tela = TelaServida {
        url: URL,
        html: &html,
        codificacao: CodificacaoDaTela::Iso88591,
    };
    let mut passos = Vec::new();
    let erro = confirmar_sequencia(
        &mut passos,
        &instalacao,
        &tela,
        "ABC",
        &cadastro,
        &foto(false),
        "Anexo",
        |p| {
            p.push("upload");
            Err(FalhaUpload::Recusado {
                motivo: "recusado".into(),
            })
        },
        |_, _| panic!("sem cadastro"),
        |_| panic!("sem releitura"),
    )
    .unwrap_err();
    assert!(matches!(erro, FalhaConfirmacao::UploadRecusado(_)));
    assert_eq!(passos, ["upload"]);
}

#[test]
fn html_generico_e_arvore_nova_nao_provam_autoria() {
    let (html, instalacao) = contexto();
    let cadastro = observar_cadastro(&html, "Anexo", 68).unwrap();
    let tela = TelaServida {
        url: URL,
        html: &html,
        codificacao: CodificacaoDaTela::Iso88591,
    };
    let mut passos = Vec::new();
    let erro = confirmar_sequencia(
        &mut passos,
        &instalacao,
        &tela,
        "ABC",
        &cadastro,
        &foto(false),
        "Anexo",
        |p| {
            p.push("upload");
            Ok(recibo())
        },
        |p, _| {
            p.push("cadastro");
            Ok(RespostaDoEnvio::Html("<html>ok</html>".into()))
        },
        |p| {
            p.push("reler");
            Ok(foto(true))
        },
    )
    .unwrap_err();
    assert!(matches!(erro, FalhaConfirmacao::NaoConfirmado));
    assert!(erro.temporario_possivel());
    assert_eq!(passos, ["upload", "cadastro", "reler"]);
}

#[test]
fn bdd_332_03_nao_confirma_documento_interno_com_mesmo_rotulo() {
    // BDD-332-03 exige documento EXTERNO comprovado pela releitura.
    // Rótulo/Número novos, isoladamente, não comprovam sua natureza.
    for icone in [Some("documento_interno"), None] {
        let (html, instalacao) = contexto();
        let cadastro = observar_cadastro(&html, "Anexo", 68).unwrap();
        let tela = TelaServida {
            url: URL,
            html: &html,
            codificacao: CodificacaoDaTela::Iso88591,
        };
        let erro = confirmar_sequencia(
            &mut (),
            &instalacao,
            &tela,
            "ABC",
            &cadastro,
            &foto(false),
            "Anexo",
            |()| Ok(recibo()),
            |(), _| {
                Ok(RespostaDoEnvio::Redirecionamento(
                    Redirecionamento302::sintetico(
                        Some("controlador.php?acao=arvore_visualizar&id_documento=42"),
                        URL,
                    ),
                ))
            },
            |()| {
                let mut leitura = foto(true);
                leitura.documentos[0].natureza.icone = icone.map(str::to_owned);
                Ok(leitura)
            },
        )
        .unwrap_err();
        assert!(matches!(erro, FalhaConfirmacao::NaoConfirmado));
    }
}
