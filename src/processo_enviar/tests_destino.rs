//! RED/GREEN da resposta do autocomplete (XML `<itens>`, observado nas sete
//! instalações) e da escolha do destino pela sigla.
use super::Destino;
use super::destino::{escolher_destino, itens_do_autocompletar, ler_resposta_da_consulta};
use super::erro::{ErroEnviar, FalhaDaConsulta};
use super::tests_apoio::*;
use crate::formulario_web::RespostaDoPost;
use crate::tela_html::HtmlServido;
use crate::transporte::{FalhaTransporte, RespostaWeb};
use std::io::Cursor;

fn itens(lista: &[(&str, &str)]) -> Vec<(String, String)> {
    lista
        .iter()
        .map(|(i, d)| ((*i).to_owned(), (*d).to_owned()))
        .collect()
}

fn resposta(status: i32, mime: &str, corpo: &str) -> RespostaDoPost<Cursor<Vec<u8>>> {
    let bytes: Vec<u8> = corpo
        .chars()
        .map(|c| u8::try_from(c).unwrap_or(b'?'))
        .collect();
    RespostaDoPost {
        resposta: RespostaWeb {
            status_code: status,
            mime: mime.to_owned(),
            content_length: Some(bytes.len()),
            localizacao: None,
            corpo: Cursor::new(bytes),
        },
        localizacao: None,
    }
}

const XML: &str = "application/xml; charset=iso-8859-1";

#[test]
fn itens_na_forma_observada() {
    let xml = resposta_xml(&[
        ("110000933", "ADCENTRAL - Administração Central"),
        ("110000934", "ADCENTRAL/GR - Gabinete &amp; Reitoria"),
    ]);
    assert_eq!(
        itens_do_autocompletar(&HtmlServido::de_teste(&xml)).unwrap(),
        itens(&[
            ("110000933", "ADCENTRAL - Administração Central"),
            ("110000934", "ADCENTRAL/GR - Gabinete & Reitoria"),
        ])
    );
    assert_eq!(
        itens_do_autocompletar(&HtmlServido::de_teste(&resposta_xml(&[]))).unwrap(),
        []
    );
    for fora in [
        "<html><body>login</body></html>",
        "<?xml version=\"1.0\"?><erro descricao=\"x\"/>",
        "<?xml version=\"1.0\"?><itens></itens><itens></itens>",
        "<?xml version=\"1.0\"?><itens><item descricao=\"X - Y\"></item></itens>",
    ] {
        assert_eq!(
            itens_do_autocompletar(&HtmlServido::de_teste(fora)),
            Err(FalhaDaConsulta::RespostaDoAutocompletarForaDaFormaObservada),
            "{fora}"
        );
    }
}

#[test]
fn resposta_precisa_ser_xml_200() {
    assert_eq!(
        ler_resposta_da_consulta(resposta(200, XML, &um_destino())).unwrap(),
        itens(&[(ID_DESTINO, &descricao_do_destino())])
    );
    assert_eq!(
        ler_resposta_da_consulta(resposta(200, "text/xml; charset=iso-8859-1", &um_destino()))
            .map(|i| i.len()),
        Ok(1)
    );
    assert_eq!(
        ler_resposta_da_consulta(resposta(200, "text/html; charset=iso-8859-1", "<html/>")),
        Err(FalhaDaConsulta::RespostaDoAutocompletarNaoXml)
    );
    assert!(matches!(
        ler_resposta_da_consulta(resposta(500, XML, "")),
        Err(FalhaDaConsulta::Transporte(FalhaTransporte::Status(_)))
    ));
}

#[test]
fn destino_e_a_sigla_exata_sem_caixa() {
    let servidos = itens(&[
        ("110000933", "ADCENTRAL - Administração Central"),
        ("110000934", "ADCENTRAL/GR - Gabinete da Reitoria"),
        (
            "110000936",
            "ADCENTRAL/GR/ARQUIVO /PROCE - Protocolo Central",
        ),
    ]);
    for pedido in ["ADCENTRAL", "adcentral", " ADCENTRAL "] {
        assert_eq!(
            escolher_destino(pedido, &servidos).unwrap(),
            Destino {
                sigla: "ADCENTRAL".into(),
                descricao: "ADCENTRAL - Administração Central".into(),
                id_unidade: "110000933".into(),
            },
            "{pedido}"
        );
    }
    assert_eq!(
        escolher_destino("ADCENTRAL/GR/ARQUIVO /PROCE", &servidos).map(|d| d.id_unidade),
        Ok("110000936".into())
    );
    assert_eq!(
        escolher_destino("ADCENT", &servidos),
        Err(ErroEnviar::DestinoNaoOferecidoPeloSei {
            oferecidos: servidos.iter().map(|(_, d)| d.clone()).collect()
        })
    );
    let repetidos = itens(&[
        ("1", "TESTE - Um"),
        ("2", "teste - Dois"),
        ("3", "Sem sigla"),
    ]);
    assert_eq!(
        escolher_destino("TESTE", &repetidos),
        Err(ErroEnviar::VariosDestinosComASigla {
            candidatos: vec!["TESTE - Um".into(), "teste - Dois".into()]
        })
    );
    assert_eq!(
        escolher_destino("Sem sigla", &repetidos),
        Err(ErroEnviar::DestinoNaoOferecidoPeloSei {
            oferecidos: repetidos.iter().map(|(_, d)| d.clone()).collect()
        })
    );
}
