//! RED/GREEN do JSON dos documentos movidos: id composto opaco, rótulo,
//! número SEI e o destino (`{numero, id_procedimento}` ou ausente), sem URL
//! nem a dica; `[]` sem movido; some com `--sem-documentos`. O movido
//! cujo documento está entre os documentos sai com
//! `presente_entre_os_documentos: true` e marcado no texto humano.
use super::tests::aberto;
use super::*;
use crate::arvore_processo::tests_movido_presente::arvore_de_ida_e_volta;
use crate::arvore_processo::tests_movidos::{
    ID_DESTINO, NUP_DESTINO, ORIGEM, arvore_com_movido, href_do_destino, html_do_movido,
};
use crate::arvore_processo::{ler_arvore, ler_destinos_dos_movidos};

const NUP: &str = "00001.000001/2026-01";

fn com_movidos(html: &str) -> ProcessoAberto {
    let instalacao = crate::instalacao::Instalacao::interpretar(ORIGEM).unwrap();
    let mut lida = ler_arvore(html, NUP, "10").unwrap();
    ler_destinos_dos_movidos(&instalacao, html, &mut lida).unwrap();
    let mut aberto = aberto(None);
    aberto.arvore.documentos_movidos = lida.documentos_movidos;
    aberto
}

fn com_destino() -> ProcessoAberto {
    com_movidos(&arvore_com_movido(
        "10",
        NUP,
        &html_do_movido(&href_do_destino(ID_DESTINO, "31"), NUP_DESTINO),
    ))
}

#[test]
fn movido_publica_o_destino_sem_url_nem_dica() {
    let nup = NumeroProcesso::novo(NUP).unwrap();
    let valor = json_do_processo(&nup, &com_destino(), false);
    assert_eq!(
        valor["documentos_movidos"],
        json!([{
            "id": "31-41",
            "rotulo": "Oficio Sintetico (0000103)",
            "numero_sei": "0000103",
            "destino": {"numero": NUP_DESTINO, "id_procedimento": ID_DESTINO},
            "presente_entre_os_documentos": false
        }])
    );
    let texto = valor.to_string();
    assert!(!texto.contains("controlador.php") && !texto.contains(&"6".repeat(64)));
    assert!(!texto.contains("motivo sintetico"));
    let sem_link = json_do_processo(&nup, &com_movidos(&arvore_com_movido("10", NUP, "")), false);
    assert!(sem_link["documentos_movidos"][0].get("destino").is_none());
    assert_eq!(
        json_do_processo(&nup, &aberto(None), false)["documentos_movidos"],
        json!([])
    );
    assert!(
        json_do_processo(&nup, &com_destino(), true)
            .get("documentos_movidos")
            .is_none()
    );
}

#[test]
fn texto_humano_lista_o_movido_com_a_dica() {
    let nup = NumeroProcesso::novo(NUP).unwrap();
    let texto = humano(&nup, &com_destino(), false);
    assert!(
        texto.contains(&format!(
            "1 documento(s) movido(s) para outro processo:\n- Oficio Sintetico (0000103) (id 31-41): movido para {NUP_DESTINO}, ies processo {NUP_DESTINO}"
        )),
        "{texto}"
    );
    let sem_link = humano(&nup, &com_movidos(&arvore_com_movido("10", NUP, "")), false);
    assert!(
        sem_link.contains("(id 31-41): destino não publicado"),
        "{sem_link}"
    );
    assert!(!humano(&nup, &aberto(None), false).contains("movido"));
}

#[test]
fn movido_presente_sai_marcado_no_json_e_no_texto() {
    let nup = NumeroProcesso::novo(NUP).unwrap();
    let aberto = com_movidos(&arvore_de_ida_e_volta());
    let valor = json_do_processo(&nup, &aberto, false);
    assert_eq!(valor["documentos_movidos"][0]["id"], "21-41");
    assert_eq!(
        valor["documentos_movidos"][0]["presente_entre_os_documentos"],
        true
    );
    let texto = humano(&nup, &aberto, false);
    assert!(
        texto.contains(&format!(
            "(id 21-41): movido para {NUP_DESTINO}, ies processo {NUP_DESTINO} (o documento está de novo neste processo)"
        )),
        "{texto}"
    );
    let ausente = humano(&nup, &com_destino(), false);
    assert!(!ausente.contains("de novo"), "{ausente}");
}
