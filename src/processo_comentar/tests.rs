use super::*;
use crate::processo_comentarios::{ComentarioLido, VisaoDosComentarios};

fn lista(texto: &str) -> ComentariosLidos {
    // Caso: Todos tests/test_comentarios_010.py @323eb01, conferência do POST.
    ComentariosLidos {
        visao: VisaoDosComentarios::Processo,
        total_declarado: 1,
        itens: vec![ComentarioLido {
            id: "42".to_owned(),
            data_hora: "16/10/2026 09:14:44".to_owned(),
            unidade: "ADC".to_owned(),
            usuario: "u".to_owned(),
            texto: texto.to_owned(),
            protocolo: None,
            tipo: None,
        }],
    }
}

#[test]
fn aceita_somente_id_do_aceite_e_texto_exato() {
    assert_eq!(conferir_efeito(&lista("texto novo"), "42", "texto novo"), Ok(()));
    assert_eq!(
        conferir_efeito(&lista("texto antigo"), "42", "texto novo"),
        Err(FalhaDaConferencia::TextoDiferenteDoPedido)
    );
}

#[test]
fn id_ausente_nao_prova_escrita() {
    assert_eq!(
        conferir_efeito(&lista("x"), "", "x"),
        Err(FalhaDaConferencia::IdDoAceiteAusente)
    );
    assert_eq!(
        conferir_efeito(&lista("x"), "43", "x"),
        Err(FalhaDaConferencia::ComentarioNaoVisivelNaReleitura)
    );
}

#[test]
fn duplicata_na_releitura_e_recusada() {
    let mut lidos = lista("x");
    lidos.itens.push(lidos.itens[0].clone());
    lidos.total_declarado = 2;
    assert_eq!(
        conferir_efeito(&lidos, "42", "x"),
        Err(FalhaDaConferencia::ComentarioNovoRepetido)
    );
}
