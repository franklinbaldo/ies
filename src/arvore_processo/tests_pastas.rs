//! RED/GREEN das pastas: HTML SINTÉTICO com a forma observada na `UnB`
//! 4.0.9 (cassettes `processo-em-pastas-unb` e `processo-pasta-mista-unb`):
//! `PASTA` e `AGUARDE` com 15 argumentos, a primeira `PASTA` na linha do
//! `.html` do processo, `Pastas[k]['protocolos']` antes dos nós, a barra de
//! cada documento com `id_rel_protocolo_protocolo` e a ação `ABRIR_PASTAS`.
//! Ids, números, hashes e rótulos inventados.
use super::tests::{NUP, interno, no_documento, no_processo};
use super::*;
use std::fmt::Write as _;

pub(crate) const ID: &str = "10";

/// Um nó da árvore sintética e, para documento de pasta, o protocolo da barra.
pub(crate) enum No {
    Processo,
    /// `k`, aberta (`a10`).
    Pasta(usize, bool),
    Aguarde(usize),
    /// id do documento, pai, protocolo na barra (`None`: barra vazia).
    Documento(&'static str, String, Option<&'static str>),
}

const ROMANOS: [&str; 4] = ["I", "II", "III", "IV"];

pub(crate) fn no_pasta(i: usize, k: usize, pai: &str, aberta: bool) -> String {
    format!(
        "Nos[{i}] = new infraArvoreNo(\"PASTA\",\"PASTA{k}\",\"{pai}\",\"javascript:abrirFecharPasta('PASTA{k}');\",null,\"{}\",\"Pasta {k}\",null,\"svg/processo_aberto.svg\",\"svg/processo_fechado.svg\",{aberta},true,null,null,null);",
        ROMANOS[k - 1]
    )
}

pub(crate) fn no_aguarde(i: usize, k: usize) -> String {
    format!(
        "Nos[{i}] = new infraArvoreNo(\"AGUARDE\",\"AGUARDE{k}\",\"PASTA{k}\",null,null,\"Aguarde...\",\"Aguarde...\",\"/infra_css/svg/aguarde.svg\",null,null,false,true,null,null,null);"
    )
}

/// Barra observada: links com `&` cru dentro de um literal com aspas simples;
/// o `comentario_listar` leva o `id_rel_protocolo_protocolo`.
pub(crate) fn barra(i: usize, documento: &str, protocolo: Option<&str>) -> String {
    let Some(protocolo) = protocolo else {
        return format!("Nos[{i}].acoes = '';");
    };
    // `"-"`: barra com links, mas sem `id_rel_protocolo_protocolo`.
    if protocolo == "-" {
        return format!(
            "Nos[{i}].acoes = '<a href=\"controlador.php?acao=documento_alterar&id_documento={documento}\"></a>';"
        );
    }
    format!(
        "Nos[{i}].acoes = '<a href=\"controlador.php?acao=documento_alterar&id_procedimento={ID}&id_documento={documento}&arvore=1&infra_hash={h}\" tabindex=\"452\" ><img src=\"svg/documento_alterar.svg\" alt=\"Consultar\" title=\"Consultar\"/></a><a href=\"#\" onclick=\"infraAbrirJanela(\\'x\\');\"></a><a href=\"controlador.php?acao=comentario_listar&acao_origem=arvore_visualizar&id_procedimento={ID}&id_rel_protocolo_protocolo={protocolo}&arvore=1&infra_hash={h}\" tabindex=\"452\" ><img src=\"svg/comentario.svg\"/></a>';",
        h = "e".repeat(64)
    )
}

/// `Pastas[k]` declaradas antes dos nós, como servidas (indentadas).
pub(crate) fn declaracoes(pastas: &[(usize, &[&str])]) -> String {
    let mut s = String::from("      var Pastas = [];\n");
    for (k, protocolos) in pastas {
        let _ = write!(
            s,
            "  Pastas[{k}] = [];\n  Pastas[{k}]['link'] = 'controlador.php?acao=procedimento_paginar&id_procedimento={ID}&no_pai=PASTA{k}&pagina_hash={}&infra_hash={}';\n  Pastas[{k}]['protocolos'] = '{}';\n\n",
            "5".repeat(32),
            "6".repeat(64),
            protocolos.join(",")
        );
    }
    s
}

pub(crate) fn url_abrir(id: &str) -> String {
    format!(
        "controlador.php?acao=procedimento_visualizar&acao_origem=procedimento_visualizar&id_procedimento={id}&abrir_pastas=1&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={}",
        "f".repeat(64)
    )
}

/// `NosAcoes` da barra do processo: linha direta, abrir e fechar pastas.
pub(crate) fn acoes_das_pastas(url: &str) -> String {
    format!(
        "    NosAcoes[0] = new infraArvoreAcao(\"LINHA_DIRETA\",\"LD{ID}\",\"{ID}\",\"x\",null,\"Linha\",null,true);\n\
         NosAcoes[1] = new infraArvoreAcao(\"ABRIR_PASTAS\",\"AP{ID}\",\"{ID}\",\"{url}\",null,\"Abrir todas as Pastas\",\"/infra_css/svg/mais.svg\",true);\n\
         NosAcoes[2] = new infraArvoreAcao(\"FECHAR_PASTAS\",\"FP{ID}\",\"{ID}\",\"{}\",null,\"Fechar todas as Pastas\",\"/infra_css/svg/menos.svg\",true);\n",
        url.replace("abrir_pastas=1", "fechar_pastas=1")
    )
}

/// Árvore com `declaradas`, os `nos` e o rodapé (`acoes`, em geral as de
/// [`acoes_das_pastas`]).
pub(crate) fn arvore_em_pastas(declaradas: &str, nos: &[No], acoes: &str) -> String {
    let mut html = String::from(
        "<html><head><script>\nvar Nos = Array();\nvar NosAcoes = Array();\nfunction x(){ if (this.nodes[i].tipo == 'PASTA') {} }\n",
    );
    html.push_str(declaradas);
    for (i, no) in nos.iter().enumerate() {
        match no {
            No::Processo => {
                let _ = write!(
                    html,
                    "//CA=300\n{}\nNos[{i}].acoes = '';\nNos[{i}].src = '';\nNos[{i}].html = 'Processo aberto somente na unidade <a class=\"ancoraSigla\">U</a>.<br />';",
                    no_processo(i, ID, NUP)
                );
                // A declaração seguinte (a primeira pasta) vem na mesma linha.
                if !matches!(nos.get(i + 1), Some(No::Pasta(..))) {
                    html.push('\n');
                }
            }
            No::Pasta(k, aberta) => {
                let _ = writeln!(
                    html,
                    "{}\nNos[{i}].carregado = {aberta};",
                    no_pasta(i, *k, ID, *aberta)
                );
            }
            No::Aguarde(k) => {
                let _ = writeln!(html, "{}", no_aguarde(i, *k));
            }
            No::Documento(id, pai, protocolo) => {
                let _ = write!(
                    html,
                    "\n\n//CA=320\n{}\n{}\nNos[{i}].src = 'controlador.php?acao=documento_visualizar&id_documento={id}';\nNos[{i}].html = '';\n",
                    no_documento(i, id, pai, &format!("00{id}")),
                    barra(i, id, *protocolo)
                );
            }
        }
    }
    html.push_str(acoes);
    html.push_str("  if (Nos.length){\n      Nos[0].processar = function (){};\n  }\n</script></head><body><div id=\"divArvore\"></div><form id=\"frmArvore\"><input type=\"hidden\" id=\"hdnPastaAtual\" name=\"hdnPastaAtual\" value=\"\" /><input type=\"hidden\" id=\"hdnProtocolos\" name=\"hdnProtocolos\" value=\"\" /></form><iframe id=\"ifrPasta\" name=\"ifrPasta\"></iframe></body></html>");
    html
}

fn p(k: usize) -> String {
    format!("PASTA{k}")
}

pub(crate) const PASTA1: [&str; 2] = ["900", "901"];
pub(crate) const PASTA2: [&str; 2] = ["902", "903"];

/// Como a interação 8 de `processo-em-pastas-unb`: a primeira pasta
/// fechada (`AGUARDE`), a última aberta com os documentos dela.
pub(crate) fn arvore_fechada() -> String {
    arvore_em_pastas(
        &declaracoes(&[(1, &PASTA1), (2, &PASTA2)]),
        &[
            No::Processo,
            No::Pasta(1, false),
            No::Aguarde(1),
            No::Pasta(2, true),
            No::Documento("31", p(2), Some("902")),
            No::Documento("32", p(2), Some("903")),
        ],
        &acoes_das_pastas(&url_abrir(ID)),
    )
}

/// Como a interação 9 (resposta de `ABRIR_PASTAS`): todas as pastas
/// primeiro, depois os documentos de cada uma, sem `AGUARDE`.
pub(crate) fn arvore_aberta() -> String {
    arvore_aberta_com(&[
        No::Documento("21", p(1), Some("900")),
        No::Documento("22", p(1), Some("901")),
        No::Documento("31", p(2), Some("902")),
        No::Documento("32", p(2), Some("903")),
    ])
}

pub(crate) fn arvore_aberta_com(documentos: &[No]) -> String {
    let mut nos = vec![No::Processo, No::Pasta(1, true), No::Pasta(2, true)];
    for d in documentos {
        if let No::Documento(id, pai, protocolo) = d {
            nos.push(No::Documento(id, pai.clone(), *protocolo));
        }
    }
    arvore_em_pastas(
        &declaracoes(&[(1, &PASTA1), (2, &PASTA2)]),
        &nos,
        &acoes_das_pastas(&url_abrir(ID)),
    )
}

fn documento(id: &str, pasta: Option<&str>) -> DocumentoNaArvore {
    DocumentoNaArvore {
        id: id.into(),
        rotulo: format!("Despacho 00{id}"),
        numero_sei: Some(format!("00{id}")),
        pasta: pasta.map(str::to_owned),
        natureza: interno(),
    }
}

/// Verificação e (identidade, contagem) de uma pasta esperada.
pub(crate) type Conferida = (Option<VerificacaoDaPasta>, usize, usize);

pub(crate) fn pastas_declaradas(primeira: Conferida, segunda: Conferida) -> Vec<PastaNaArvore> {
    [("PASTA1", "I", primeira), ("PASTA2", "II", segunda)]
        .into_iter()
        .map(
            |(id, rotulo, (verificacao, identidade, contagem))| PastaNaArvore {
                id: id.into(),
                rotulo: rotulo.into(),
                total_declarado: 2,
                verificacao,
                conferidos_por_identidade: identidade,
                conferidos_por_contagem: contagem,
            },
        )
        .collect()
}

const POR_IDENTIDADE: Conferida = (Some(VerificacaoDaPasta::PorIdentidade), 2, 0);

#[test]
fn arvore_aberta_le_os_documentos_de_cada_pasta_na_ordem() {
    let lida = ler_arvore(&arvore_aberta(), NUP, ID).unwrap();
    assert!(!lida.pastas_nao_carregadas);
    assert_eq!(
        lida.pastas,
        pastas_declaradas(POR_IDENTIDADE, POR_IDENTIDADE)
    );
    assert_eq!(
        lida.documentos,
        [
            documento("21", Some("PASTA1")),
            documento("22", Some("PASTA1")),
            documento("31", Some("PASTA2")),
            documento("32", Some("PASTA2")),
        ]
    );
}

#[test]
fn arvore_fechada_marca_as_pastas_nao_carregadas() {
    let lida = ler_arvore(&arvore_fechada(), NUP, ID).unwrap();
    assert!(lida.pastas_nao_carregadas);
    // A fechada não foi conferida: `None`, nunca uma verificação alegada.
    assert_eq!(lida.pastas, pastas_declaradas((None, 0, 0), POR_IDENTIDADE));
    // Só os da pasta aberta: a fechada não é conferida nem publicada aqui.
    assert_eq!(
        lida.documentos,
        [
            documento("31", Some("PASTA2")),
            documento("32", Some("PASTA2"))
        ]
    );
}

#[test]
fn documento_na_raiz_ao_lado_das_pastas_sai_sem_pasta() {
    let html = arvore_aberta_com(&[
        No::Documento("21", p(1), Some("900")),
        No::Documento("22", p(1), Some("901")),
        No::Documento("31", p(2), Some("902")),
        No::Documento("32", p(2), Some("903")),
        No::Documento("40", ID.to_owned(), None),
    ]);
    let lida = ler_arvore(&html, NUP, ID).unwrap();
    assert_eq!(lida.documentos[4], documento("40", None));
}

#[test]
fn arvore_sem_pastas_nao_le_a_barra_dos_documentos() {
    // A barra dos documentos da raiz não é lida: uma forma estranha nela
    // não derruba a leitura (fatia 1).
    let html = super::tests::arvore(&[no_processo(0, ID, NUP), no_documento(1, "21", ID, "1")])
        .replace("Nos[1].acoes = '';", "Nos[1].acoes = x + '';");
    let lida = ler_arvore(&html, NUP, ID).unwrap();
    assert!(lida.pastas.is_empty() && !lida.pastas_nao_carregadas);
    assert_eq!(lida.documentos[0].pasta, None);
}

#[test]
fn identidade_vem_do_id_rel_protocolo_protocolo_da_barra() {
    use pastas::identidade_da_barra as id;
    assert_eq!(
        id("<a href=\"c.php?acao=x&id_rel_protocolo_protocolo=77&arvore=1\">").unwrap(),
        Some("77".into())
    );
    assert_eq!(
        id("<a href=\"c.php?id_rel_protocolo_protocolo=77\"></a><a href=\"c.php?acao=y&id_rel_protocolo_protocolo=77&z=1\">").unwrap(),
        Some("77".into())
    );
    assert_eq!(id("").unwrap(), None);
    // Nome parecido não é o parâmetro.
    assert_eq!(
        id("<a href=\"c.php?xid_rel_protocolo_protocolo=77\">").unwrap(),
        None
    );
    assert_eq!(
        id(
            "<a href=\"c.php?a=1&id_rel_protocolo_protocolo=77\"></a><a href=\"c.php?a=1&id_rel_protocolo_protocolo=78\">"
        ),
        Err(ErroArvore::IdentidadesDivergentesNaBarraDoDocumento)
    );
    assert_eq!(
        id("<a href=\"c.php?a=1&id_rel_protocolo_protocolo=7x\">"),
        Err(ErroArvore::BarraDeNoForaDaFormaObservada)
    );
    assert_eq!(
        id("<a href=\"c.php?a=1&id_rel_protocolo_protocolo=\">"),
        Err(ErroArvore::BarraDeNoForaDaFormaObservada)
    );
}

#[test]
fn primeira_pasta_na_linha_do_html_e_lida_so_na_forma_observada() {
    use literais::ler_declaracao as ler;
    let (i, a) = ler(r#"Nos[0].html = 'x <a class="s">U</a>';Nos[1] = new infraArvoreNo("A");"#)
        .unwrap()
        .unwrap();
    assert_eq!((i, a), (1, vec![Literal::Texto("A".into())]));
    for linha in [
        // Índice que não é o seguinte, outra propriedade, algo no meio.
        r#"Nos[0].html = '';Nos[2] = new infraArvoreNo("A");"#,
        r#"Nos[0].src = '';Nos[1] = new infraArvoreNo("A");"#,
        r#"Nos[0].html = ''; Nos[1] = new infraArvoreNo("A");"#,
        r#"Nos[0].html = x;Nos[1] = new infraArvoreNo("A");"#,
        r#"Nos[0].html = '';Nos[1] = new infraArvoreNo("A");x();"#,
    ] {
        assert!(ler(linha).is_err(), "{linha}");
    }
}
