//! RED/GREEN da leitura da árvore: HTML sintético com a forma observada na
//! `UnB` e em GO (16 argumentos, `PROCESSO` e `DOCUMENTO`, linhas `.acoes`,
//! `.src` e `.html` entre as declarações). Nenhum valor real.
use super::*;
use std::fmt::Write as _;

pub(crate) const NUP: &str = "00001.000001/2026-01";

/// Declaração do nó `PROCESSO` na forma observada.
pub(crate) fn no_processo(i: usize, id: &str, nup: &str) -> String {
    format!(
        "Nos[{i}] = new infraArvoreNo(\"PROCESSO\",\"{id}\",null,\"controlador.php?acao=arvore_visualizar&id_procedimento={id}\",\"ifrVisualizacao\",\"{nup}\",\"Tipo Sintetico\",\"svg/processo.svg\",\"svg/processo.svg\",\"svg/processo.svg\",true,true,\"noVisitado\",null,\"noVisitado\",\"{nup}\");"
    )
}

/// Declaração de um `DOCUMENTO`; a dica (a6) traz um "e-mail" com `\n`.
pub(crate) fn no_documento(i: usize, id: &str, pai: &str, numero: &str) -> String {
    format!(
        "Nos[{i}] = new infraArvoreNo(\"DOCUMENTO\",\"{id}\",\"{pai}\",\"about:blank\",\"ifrVisualizacao\",\"Despacho {numero}\",\"De: pessoa@exemplo.gov.br\\nPara: outra@exemplo.gov.br\",\"svg/documento_interno.svg\",\"svg/documento_interno.svg\",\"svg/documento_interno.svg\",true,false,null,null,\"noVisitado\",\"{numero}\");"
    )
}

/// A natureza do `no_documento` sintético: ícone interno, não cancelado.
pub(crate) fn interno() -> NaturezaDoDocumento {
    NaturezaDoDocumento {
        icone: "documento_interno".into(),
        cancelamento: Cancelamento::Ativo,
        dica: Some(DICA_SINTETICA.into()),
    }
}

/// A dica decodificada de `no_documento`, diferente do rótulo.
pub(crate) const DICA_SINTETICA: &str = "De: pessoa@exemplo.gov.br\nPara: outra@exemplo.gov.br";

/// Árvore com as linhas que acompanham cada nó e o código depois deles.
pub(crate) fn arvore(nos: &[String]) -> String {
    let mut html = String::from(
        "<html><head><script>\nvar Nos = new Array();\nfunction x(){ if (this.nodes[i].tipo == 'PASTA') {} }\n",
    );
    for (i, no) in nos.iter().enumerate() {
        html.push_str(no);
        html.push('\n');
        let _ = write!(
            html,
            "Nos[{i}].acoes = '';\nNos[{i}].src = '';\nNos[{i}].html = '';\n"
        );
    }
    html.push_str(
        "  if (Nos.length){\n      Nos[0].processar = function (){};\n  }\n</script></head>",
    );
    html.push_str("<body><div id=\"divArvore\"></div><form id=\"frmArvore\"></form></body></html>");
    html
}

fn padrao() -> String {
    arvore(&[
        no_processo(0, "10", NUP),
        no_documento(1, "21", "10", "0000101"),
        no_documento(2, "22", "10", "0000102"),
    ])
}

#[test]
fn le_processo_e_documentos_do_primeiro_nivel_com_a_dica() {
    let lida = ler_arvore(&padrao(), NUP, "10").unwrap();
    assert_eq!(
        lida.processo,
        ProcessoNaArvore {
            id: "10".into(),
            nup: NUP.into(),
            tipo: "Tipo Sintetico".into(),
        }
    );
    assert_eq!(
        lida.documentos,
        [
            DocumentoNaArvore {
                id: "21".into(),
                rotulo: "Despacho 0000101".into(),
                numero_sei: Some("0000101".into()),
                pasta: None,
                natureza: interno(),
            },
            DocumentoNaArvore {
                id: "22".into(),
                rotulo: "Despacho 0000102".into(),
                numero_sei: Some("0000102".into()),
                pasta: None,
                natureza: interno(),
            },
        ]
    );
}

#[test]
fn processo_sem_documentos_e_numero_sei_nulo() {
    let lida = ler_arvore(&arvore(&[no_processo(0, "10", NUP)]), NUP, "10").unwrap();
    assert_eq!(lida.documentos, []);
    let nulo = no_documento(1, "21", "10", "X").replace(",\"X\");", ",null);");
    let lida = ler_arvore(&arvore(&[no_processo(0, "10", NUP), nulo]), NUP, "10").unwrap();
    assert_eq!(lida.documentos[0].numero_sei, None);
}

#[test]
fn identidade_e_comparada_byte_a_byte() {
    // Pontuação e espaços não são normalizados.
    assert_eq!(
        ler_arvore(&padrao(), "000010000012026-01", "10"),
        Err(super::ErroArvore::IdentidadeDoProcessoDiferenteDaPedida)
    );
    assert_eq!(
        ler_arvore(&padrao(), &format!("{NUP} "), "10"),
        Err(super::ErroArvore::IdentidadeDoProcessoDiferenteDaPedida)
    );
    assert_eq!(
        ler_arvore(&padrao(), NUP, "11"),
        Err(super::ErroArvore::IdDoProcessoNaArvoreDiferenteDoAberto)
    );
    assert!(super::ErroArvore::IdentidadeDoProcessoDiferenteDaPedida.diverge_do_pedido());
    assert!(!super::ErroArvore::TipoDeNoDaArvoreNaoSuportado.diverge_do_pedido());
}

#[test]
fn recusas_estruturais() {
    let p = || no_processo(0, "10", NUP);
    let pasta = no_documento(1, "30", "10", "1").replace("\"DOCUMENTO\"", "\"PASTA\"");
    let outro_tipo = no_documento(1, "30", "10", "1").replace("\"DOCUMENTO\"", "\"ANEXADO\"");
    let casos: Vec<(Vec<String>, super::ErroArvore)> = vec![
        (vec![], super::ErroArvore::NoDoProcessoAusenteNaArvore),
        (
            vec![no_documento(0, "21", "10", "1")],
            super::ErroArvore::NoDoProcessoAusenteNaArvore,
        ),
        (
            vec![p(), no_processo(1, "11", NUP)],
            super::ErroArvore::NosDoProcessoRepetidosNaArvore,
        ),
        // `PASTA` tem 15 argumentos; outro tipo é recusa.
        (
            vec![p(), pasta],
            super::ErroArvore::QuantidadeDeArgumentosDoNoDiferenteDaObservada {
                esperada: 15,
                observada: 16,
            },
        ),
        (
            vec![p(), outro_tipo],
            super::ErroArvore::TipoDeNoDaArvoreNaoSuportado,
        ),
        (
            vec![p(), "Nos[1] = new infraArvoreNo();".to_owned()],
            super::ErroArvore::NoSemArgumentos,
        ),
        (
            vec![p(), no_documento(1, "21", "99", "1")],
            super::ErroArvore::DocumentoComPaiDiferenteDoProcesso,
        ),
        (
            vec![
                p(),
                no_documento(1, "21", "10", "1"),
                no_documento(2, "21", "10", "2"),
            ],
            super::ErroArvore::DocumentoRepetidoNaArvore,
        ),
        (
            vec![p(), no_documento(1, "2a", "10", "1")],
            super::ErroArvore::IdDeNoNaoDecimal,
        ),
        (
            vec![p().replace(",null,\"controlador", ",\"9\",\"controlador")],
            super::ErroArvore::NoDoProcessoComPai,
        ),
        (
            vec![p().replace(&format!(",\"{NUP}\");"), ",\"outro\");")],
            super::ErroArvore::NupDoNoDoProcessoInconsistente,
        ),
        (
            vec![p().replace("\"PROCESSO\",\"10\"", "\"PROCESSO\",10")],
            super::ErroArvore::ArgumentoDeNoComTipoDiferenteDoObservado,
        ),
        (
            vec![p().replace(",\"noVisitado\",\"", ",\"")],
            super::ErroArvore::QuantidadeDeArgumentosDoNoDiferenteDaObservada {
                esperada: 16,
                observada: 15,
            },
        ),
        (
            vec![p().replace("Nos[0]", "Nos[1]")],
            super::ErroArvore::IndiceDeNoForaDaSequencia,
        ),
        (
            vec![p(), no_documento(2, "21", "10", "1")],
            super::ErroArvore::IndiceDeNoForaDaSequencia,
        ),
    ];
    for (nos, esperado) in casos {
        assert_eq!(
            ler_arvore(&arvore(&nos), NUP, "10"),
            Err(esperado),
            "{nos:?}"
        );
    }
}

#[test]
fn declaracao_fora_da_forma_em_qualquer_linha_e_recusa() {
    let mut html = padrao();
    html.push_str("\n  var y = new infraArvoreNo(\"DOCUMENTO\");\n");
    assert_eq!(
        ler_arvore(&html, NUP, "10"),
        Err(super::ErroArvore::DeclaracaoDeNoForaDaFormaObservada)
    );
    // CRLF no fim da linha é tolerado.
    let crlf = padrao().replace('\n', "\r\n");
    assert!(ler_arvore(&crlf, NUP, "10").is_ok());
}
