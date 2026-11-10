//! RED/GREEN do movido cujo documento está entre os documentos: HTML
//! SINTÉTICO com a forma das cassettes públicas
//! `documento-mover-cancelar-{go,pen1,pen2,pen3}` (documento movido de A
//! para B e de B para A): a árvore de A traz `Nos[1]` o `DOCUMENTO_MOVIDO`
//! `<n>-<m>` com o destino B no `.html` e `Nos[2]` o `DOCUMENTO` `n`, de
//! mesmo rótulo e número SEI. Ids, números, hashes e textos inventados.
use super::tests::{NUP, arvore, no_documento, no_processo};
use super::tests_movidos::{
    ID_DESTINO, NUP_DESTINO, ORIGEM, href_do_destino, html_do_movido, no_movido,
};
use super::*;
use crate::erro_html::ErroHtml;
use crate::instalacao::Instalacao;

/// O `DOCUMENTO` `id` com o rótulo do movido sintético.
fn documento_como_o_movido(i: usize, id: &str, numero: &str) -> String {
    no_documento(i, id, "10", numero).replace(
        &format!("\"Despacho {numero}\""),
        &format!("\"Oficio Sintetico ({numero})\""),
    )
}

/// Árvore de A depois da ida e volta: movido `21-41` (nó 1, destino
/// publicado), o `DOCUMENTO` `21` (nó 2) e outro documento (nó 3).
pub(crate) fn arvore_de_ida_e_volta() -> String {
    let link = html_do_movido(&href_do_destino(ID_DESTINO, "21"), NUP_DESTINO);
    arvore(&[
        no_processo(0, "10", NUP),
        no_movido(1, "21-41", "10", "0000101"),
        documento_como_o_movido(2, "21", "0000101"),
        no_documento(3, "22", "10", "0000102"),
    ])
    .replace("Nos[1].html = '';", &format!("Nos[1].html = '{link}';"))
}

fn ler_tudo(html: &str) -> Result<ArvoreLida, super::ErroArvore> {
    let mut lida = ler_arvore(html, NUP, "10")?;
    let instalacao = Instalacao::interpretar(ORIGEM).unwrap();
    ler_destinos_dos_movidos(&instalacao, html, &mut lida)?;
    Ok(lida)
}

#[test]
fn ida_e_volta_le_o_documento_e_o_movido() {
    let lida = ler_tudo(&arvore_de_ida_e_volta()).unwrap();
    let ids: Vec<&str> = lida.documentos.iter().map(|d| d.id.as_str()).collect();
    assert_eq!(ids, ["21", "22"]);
    let [movido] = lida.documentos_movidos.as_slice() else {
        panic!("{:?}", lida.documentos_movidos);
    };
    assert_eq!(movido.id, "21-41");
    assert!(movido.presente_entre_os_documentos);
    assert_eq!(movido.destino.as_ref().unwrap().numero, NUP_DESTINO);
}

#[test]
fn so_o_id_classifica_nem_ordem_nem_rotulo() {
    // O `DOCUMENTO` antes do movido e com outro rótulo e outro número.
    let html = arvore(&[
        no_processo(0, "10", NUP),
        no_documento(1, "21", "10", "0000109"),
        no_movido(2, "21-41", "10", "0000101"),
    ]);
    let lida = ler_arvore(&html, NUP, "10").unwrap();
    assert!(lida.documentos_movidos[0].presente_entre_os_documentos);
    // Segunda parte igual a um id de documento não conta.
    let html = arvore(&[
        no_processo(0, "10", NUP),
        no_documento(1, "41", "10", "0000109"),
        no_movido(2, "21-41", "10", "0000101"),
    ]);
    let lida = ler_arvore(&html, NUP, "10").unwrap();
    assert!(!lida.documentos_movidos[0].presente_entre_os_documentos);
}

#[test]
fn primeira_parte_repetida_entre_movidos_continua_recusa() {
    for segundo in ["21-41", "21-42"] {
        let html = arvore(&[
            no_processo(0, "10", NUP),
            no_movido(1, "21-41", "10", "0000101"),
            documento_como_o_movido(2, "21", "0000101"),
            no_movido(3, segundo, "10", "0000101"),
        ]);
        assert_eq!(
            ler_arvore(&html, NUP, "10"),
            Err(super::ErroArvore::Html(ErroHtml::repetido(
                "documento_movido_repetido_na_arvore"
            ))),
            "{segundo}"
        );
    }
}
