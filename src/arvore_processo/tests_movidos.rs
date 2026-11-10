//! RED/GREEN dos documentos movidos: HTML SINTÉTICO com a forma observada
//! na PEN1 5.0.4 (04/11/2026): `DOCUMENTO_MOVIDO` com 16 argumentos, a1
//! composto `<id_documento>-<id_rel_protocolo_protocolo>`, a2 o processo,
//! a3 o `arvore_visualizar` do próprio processo, a15 o número SEI, `.acoes`
//! só com Comentários e `.html` com um único `<a>`: o
//! `procedimento_trabalhar` assinado do destino (`id_procedimento` do
//! destino, `id_documento` igual à primeira parte de a1), texto = NUP do
//! destino. Ids, números, hashes e textos inventados.
use super::tests::{NUP, arvore, no_documento, no_processo};
use super::*;
use crate::instalacao::Instalacao;

pub(crate) const ORIGEM: &str = "https://sei.exemplo.gov.br/sei";
pub(crate) const NUP_DESTINO: &str = "00008.000008/2026-08";
pub(crate) const ID_DESTINO: &str = "80";
const CONTEXTO: &str = "infra_sistema=100000100&infra_unidade_atual=110000001";

fn instalacao() -> Instalacao {
    Instalacao::interpretar(ORIGEM).unwrap()
}

/// Declaração do `DOCUMENTO_MOVIDO` `<n>-<m>` com pai `pai`.
pub(crate) fn no_movido(i: usize, composto: &str, pai: &str, numero: &str) -> String {
    format!(
        "Nos[{i}] = new infraArvoreNo(\"DOCUMENTO_MOVIDO\",\"{composto}\",\"{pai}\",\"controlador.php?acao=arvore_visualizar&acao_origem=procedimento_visualizar&id_procedimento={pai}&{CONTEXTO}&infra_hash={}\",\"ifrConteudoVisualizacao\",\"Oficio Sintetico ({numero})\",\"Documento movido para o processo {NUP_DESTINO}: motivo sintetico\",\"svg/documento_movido.svg\",\"svg/documento_movido.svg\",\"svg/documento_movido.svg\",true,true,null,null,\"noVisitado\",\"{numero}\");",
        "5".repeat(64)
    )
}

/// `href` do destino, como servido (`&` cru).
pub(crate) fn href_do_destino(destino: &str, id_documento: &str) -> String {
    format!(
        "controlador.php?acao=procedimento_trabalhar&acao_origem=arvore_visualizar&id_procedimento={destino}&id_documento={id_documento}&{CONTEXTO}&infra_hash={}",
        "6".repeat(64)
    )
}

/// O `.html` do nó movido na forma observada.
pub(crate) fn html_do_movido(href: &str, texto: &str) -> String {
    format!(
        "Documento movido para o processo <a href=\"{href}\" target=\"_blank\" class=\"ancoraVisualizacaoArvore\">{texto}</a>."
    )
}

/// Processo `id` com dois documentos e o movido `31-41` (nó 3), cujo
/// `.html` é `html` e cuja barra só tem Comentários.
pub(crate) fn arvore_com_movido(id: &str, nup: &str, html: &str) -> String {
    let barra = format!(
        "<a href=\"controlador.php?acao=comentario_listar&acao_origem=arvore_visualizar&id_procedimento={id}&id_rel_protocolo_protocolo=41&arvore=1&{CONTEXTO}&infra_hash={}\" tabindex=\"451\" ><img  src=\"svg/comentario.svg\" alt=\"Comentarios\" title=\"Comentarios\"/></a>",
        "7".repeat(64)
    );
    arvore(&[
        no_processo(0, id, nup),
        no_documento(1, "21", id, "0000101"),
        no_documento(2, "22", id, "0000102"),
        no_movido(3, "31-41", id, "0000103"),
    ])
    .replace("Nos[3].acoes = '';", &format!("Nos[3].acoes = '{barra}';"))
    .replace("Nos[3].html = '';", &format!("Nos[3].html = '{html}';"))
}

fn padrao() -> String {
    arvore_com_movido(
        "10",
        NUP,
        &html_do_movido(&href_do_destino(ID_DESTINO, "31"), NUP_DESTINO),
    )
}

fn ler_tudo(html: &str) -> Result<ArvoreLida, super::ErroArvore> {
    let mut lida = ler_arvore(html, NUP, "10")?;
    ler_destinos_dos_movidos(&instalacao(), html, &mut lida)?;
    Ok(lida)
}

#[test]
fn movido_nao_e_documento_do_processo() {
    let lida = ler_arvore(&padrao(), NUP, "10").unwrap();
    let ids: Vec<&str> = lida.documentos.iter().map(|d| d.id.as_str()).collect();
    assert_eq!(ids, ["21", "22"]);
    assert_eq!(
        lida.documentos_movidos,
        [DocumentoMovido {
            id: "31-41".into(),
            rotulo: "Oficio Sintetico (0000103)".into(),
            numero_sei: Some("0000103".into()),
            destino: None,
            presente_entre_os_documentos: false,
        }]
    );
    assert!(!format!("{lida:?}").contains("motivo sintetico"));
}

#[test]
fn destino_pelo_link_assinado_do_no() {
    let lida = ler_tudo(&padrao()).unwrap();
    let destino = lida.documentos_movidos[0].destino.as_ref().unwrap();
    assert_eq!(destino.numero, NUP_DESTINO);
    assert_eq!(destino.id_procedimento, ID_DESTINO);
    assert_eq!(
        destino.trabalhar.as_str(),
        format!("{ORIGEM}/{}", href_do_destino(ID_DESTINO, "31"))
    );
    let debug = format!("{lida:?}");
    assert!(!debug.contains(&"6".repeat(64)), "{debug}");
    // O link do movido não é vínculo de anexação da raiz.
    assert_eq!(
        ler_vinculo_de_anexacao(&instalacao(), &padrao(), "10"),
        Ok(None)
    );
}

#[test]
fn sem_link_o_destino_nao_foi_observado() {
    let sem_html = padrao().replace(
        &format!(
            "Nos[3].html = '{}';\n",
            html_do_movido(&href_do_destino(ID_DESTINO, "31"), NUP_DESTINO)
        ),
        "",
    );
    for html in [
        arvore_com_movido("10", NUP, ""),
        arvore_com_movido(
            "10",
            NUP,
            "Documento movido para o processo 00008.000008/2026-08.",
        ),
        sem_html,
    ] {
        let lida = ler_tudo(&html).unwrap();
        assert_eq!(lida.documentos_movidos.len(), 1, "{html}");
        assert_eq!(lida.documentos_movidos[0].destino, None, "{html}");
    }
    // Sem movido: lista vazia.
    let lida = ler_tudo(&arvore(&[no_processo(0, "10", NUP)])).unwrap();
    assert_eq!(lida.documentos_movidos, []);
}

#[test]
fn numero_sei_nulo() {
    let html = padrao().replace(",\"noVisitado\",\"0000103\");", ",\"noVisitado\",null);");
    assert_eq!(
        ler_tudo(&html).unwrap().documentos_movidos[0].numero_sei,
        None
    );
}

#[test]
fn no_movido_fora_da_forma_e_recusa() {
    let p = || no_processo(0, "10", NUP);
    let d = || no_documento(1, "21", "10", "0000101");
    let m = |composto: &str, pai: &str| no_movido(2, composto, pai, "0000103");
    let com_a3 = |de: &str, para: &str| m("31-41", "10").replace(de, para);
    let casos: Vec<(Vec<String>, super::ErroArvore)> = vec![
        (
            vec![p(), d(), m("31", "10")],
            super::ErroArvore::DocumentoMovidoForaDaFormaObservada,
        ),
        (
            vec![p(), d(), m("31-", "10")],
            super::ErroArvore::DocumentoMovidoForaDaFormaObservada,
        ),
        (
            vec![p(), d(), m("-41", "10")],
            super::ErroArvore::DocumentoMovidoForaDaFormaObservada,
        ),
        (
            vec![p(), d(), m("3a-41", "10")],
            super::ErroArvore::DocumentoMovidoForaDaFormaObservada,
        ),
        (
            vec![p(), d(), m("31-41-5", "10")],
            super::ErroArvore::DocumentoMovidoForaDaFormaObservada,
        ),
        (
            vec![
                p(),
                d(),
                com_a3("id_procedimento=10&", "id_procedimento=10&id_documento=31&"),
            ],
            super::ErroArvore::DocumentoMovidoForaDaFormaObservada,
        ),
        (
            vec![
                p(),
                d(),
                com_a3("acao=arvore_visualizar", "acao=procedimento_trabalhar"),
            ],
            super::ErroArvore::DocumentoMovidoForaDaFormaObservada,
        ),
        (
            vec![p(), d(), com_a3("\"controlador.php?", "\"about:blank?")],
            super::ErroArvore::DocumentoMovidoForaDaFormaObservada,
        ),
        (
            vec![
                p(),
                d(),
                com_a3(",\"Oficio Sintetico (0000103)\",", ",\"\","),
            ],
            super::ErroArvore::DocumentoMovidoForaDaFormaObservada,
        ),
        // a3 do outro processo acompanha o pai: o pai é que diverge.
        (
            vec![p(), d(), m("31-41", "99")],
            super::ErroArvore::DocumentoMovidoComPaiDiferenteDoProcesso,
        ),
        (
            vec![p(), d(), m("31-41", "PASTA1")],
            super::ErroArvore::DocumentoMovidoComPaiDiferenteDoProcesso,
        ),
        (
            vec![
                p(),
                d(),
                m("31-41", "10"),
                no_movido(3, "31-41", "10", "0000104"),
            ],
            super::ErroArvore::DocumentoMovidoRepetidoNaArvore,
        ),
        (
            vec![
                p(),
                d(),
                m("31-41", "10"),
                no_movido(3, "31-42", "10", "0000104"),
            ],
            super::ErroArvore::DocumentoMovidoRepetidoNaArvore,
        ),
        (
            vec![
                p(),
                d(),
                com_a3(",\"noVisitado\",\"0000103\");", ",\"0000103\");"),
            ],
            super::ErroArvore::QuantidadeDeArgumentosDoNoDiferenteDaObservada,
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
fn destino_fora_da_forma_e_recusa() {
    let href = href_do_destino(ID_DESTINO, "31");
    let com_href = |h: &str| arvore_com_movido("10", NUP, &html_do_movido(h, NUP_DESTINO));
    let com_texto = |t: &str| arvore_com_movido("10", NUP, &html_do_movido(&href, t));
    let fora = [
        com_href(&href_do_destino(ID_DESTINO, "32")),
        com_href(&href_do_destino("10", "31")),
        com_href(&href_do_destino("8a", "31")),
        com_href(&href.replace("&id_documento=31", "")),
        com_href(&href.replace("&id_documento=31", "&id_documento=31&id_documento=31")),
        com_href(&href.replace(
            &format!("id_procedimento={ID_DESTINO}"),
            &format!("id_procedimento={ID_DESTINO}&id_procedimento={ID_DESTINO}"),
        )),
        com_href(&href.replace(
            "acao=procedimento_trabalhar",
            "acao=procedimento_visualizar",
        )),
        com_href(&href.replace(&"6".repeat(64), "6")),
        com_href(&format!("{href}&id_procedimento_anexado=10")),
        com_href(&format!("https://outro.exemplo/{href}")),
        com_texto(""),
        com_texto(" 00008.000008/2026-08"),
        com_texto(NUP),
        com_texto("<b>00008.000008/2026-08</b>"),
    ];
    for html in fora {
        assert_eq!(
            ler_tudo(&html),
            Err(super::ErroArvore::DestinoDoDocumentoMovidoForaDaFormaObservada),
            "{html}"
        );
    }
    let link = html_do_movido(&href, NUP_DESTINO);
    let dois_links = arvore_com_movido("10", NUP, &format!("{link}{link}"));
    let dois_html = padrao().replace(
        "Nos[3].src = '';",
        &format!("Nos[3].src = '';\nNos[3].html = '{link}';"),
    );
    for html in [dois_links, dois_html] {
        assert_eq!(
            ler_tudo(&html),
            Err(super::ErroArvore::DestinosDoDocumentoMovidoRepetidos),
            "{html}"
        );
    }
}
