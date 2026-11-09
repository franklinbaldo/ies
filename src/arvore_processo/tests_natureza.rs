//! RED/GREEN da natureza dos documentos: ícone (a7 a a9) e cancelamento
//! (ícone `documento_cancelado` e dica `Documento Cancelado: <motivo>`),
//! na forma observada em GO 5.0.3 e na `UnB` 4.0.9. Valores sintéticos; o
//! e-mail da dica nunca aparece no que é guardado.
use super::tests::{NUP, arvore, interno, no_documento, no_processo};
use super::*;

/// `DOCUMENTO` com rótulo, dica e os três ícones dados.
fn documento(rotulo: &str, dica: &str, icones: [&str; 3]) -> String {
    let [a7, a8, a9] = icones;
    format!(
        "Nos[1] = new infraArvoreNo(\"DOCUMENTO\",\"21\",\"10\",\"about:blank\",\"ifrConteudoVisualizacao\",\"{rotulo}\",\"{dica}\",\"{a7}\",\"{a8}\",\"{a9}\",true,false,null,null,null,\"0000101\");"
    )
}

fn natureza(no: String) -> NaturezaDoDocumento {
    let lida = ler_arvore(&arvore(&[no_processo(0, "10", NUP), no]), NUP, "10").unwrap();
    lida.documentos[0].natureza.clone()
}

fn esperada(
    icone: Option<&str>,
    cancelado: Option<bool>,
    motivo: Option<&str>,
) -> NaturezaDoDocumento {
    NaturezaDoDocumento {
        icone: icone.map(str::to_owned),
        cancelado,
        motivo_do_cancelamento: motivo.map(str::to_owned),
        dica: None,
    }
}

fn com_dica(natureza: NaturezaDoDocumento, dica: &str) -> NaturezaDoDocumento {
    NaturezaDoDocumento {
        dica: Some(dica.to_owned()),
        ..natureza
    }
}

const PDF: &str = "svg/documento_pdf.svg?11";
const CANCELADO: &str = "svg/documento_cancelado.svg?25";

#[test]
fn icone_e_o_radical_opaco_com_ou_sem_versao() {
    assert_eq!(natureza(no_documento(1, "21", "10", "0000101")), interno());
    assert_eq!(
        natureza(documento("Anexo 0000101", "Anexo 0000101", [PDF; 3])),
        esperada(Some("documento_pdf"), Some(false), None)
    );
}

#[test]
fn cancelado_com_as_duas_marcas_publica_o_motivo() {
    assert_eq!(
        natureza(documento(
            "Consulta Jurídica 0000101",
            "Documento Cancelado: erro",
            [CANCELADO; 3]
        )),
        com_dica(
            esperada(Some("documento_cancelado"), Some(true), Some("erro")),
            "Documento Cancelado: erro"
        )
    );
    assert_eq!(
        natureza(documento(
            "Ofício 0000101",
            "Documento Cancelado:  enviado em duplicidade ",
            [CANCELADO; 3]
        )),
        com_dica(
            esperada(
                Some("documento_cancelado"),
                Some(true),
                Some(" enviado em duplicidade ")
            ),
            "Documento Cancelado:  enviado em duplicidade "
        )
    );
}

#[test]
fn meia_marca_de_cancelamento_e_null() {
    assert_eq!(
        natureza(documento(
            "Ofício 0000101",
            "Ofício 0000101",
            [CANCELADO; 3]
        )),
        esperada(Some("documento_cancelado"), None, None)
    );
    assert_eq!(
        natureza(documento(
            "Ofício 0000101",
            "Documento Cancelado: erro",
            [PDF; 3]
        )),
        com_dica(
            esperada(Some("documento_pdf"), None, None),
            "Documento Cancelado: erro"
        )
    );
}

#[test]
fn icone_fora_da_forma_e_null_sem_recusar_a_arvore() {
    let casos = [
        [PDF, PDF, CANCELADO],
        ["/infra_css/imagens/documento.gif"; 3],
        ["svg/Documento_PDF.svg"; 3],
        ["svg/.svg"; 3],
        ["svg/documento_pdf.svg?x"; 3],
        ["svg/documento_pdf.png"; 3],
        ["svg/a/b.svg"; 3],
    ];
    for icones in casos {
        assert_eq!(
            natureza(documento("Anexo 0000101", "Anexo 0000101", icones)),
            esperada(None, None, None),
            "{icones:?}"
        );
    }
}

#[test]
fn email_publica_o_icone_e_a_dica_decodificada() {
    let email = documento(
        "E-mail 0000101",
        "Data de Envio: 01/01/2026 10:00:00\\nDe: pessoa@exemplo.gov.br\\nPara: outra@exemplo.gov.br\\nAssunto: x\\n",
        ["svg/documento_email.svg?25"; 3],
    );
    let lida = ler_arvore(&arvore(&[no_processo(0, "10", NUP), email]), NUP, "10").unwrap();
    assert_eq!(
        lida.documentos[0].natureza,
        NaturezaDoDocumento {
            dica: Some(
                "Data de Envio: 01/01/2026 10:00:00\nDe: pessoa@exemplo.gov.br\nPara: outra@exemplo.gov.br\nAssunto: x\n"
                    .into()
            ),
            ..esperada(Some("documento_email"), Some(false), None)
        }
    );
}

#[test]
fn documentos_de_pasta_tambem_tem_natureza() {
    let lida = ler_arvore(
        &super::tests_pastas::arvore_aberta(),
        NUP,
        super::tests_pastas::ID,
    )
    .unwrap();
    assert_eq!(lida.documentos.len(), 4);
    for documento in &lida.documentos {
        assert!(documento.pasta.is_some());
        assert_eq!(documento.natureza, interno());
    }
}
