//! RED/GREEN da borda de `ies documento buscar`: JSON das três respostas,
//! termo e gramática Clap. Sem rede.
use super::*;
use crate::dominio::IdProcedimento;
use crate::pesquisa_web::documentos::{DocumentoEncontrado, ListaDeDocumentos};
use crate::pesquisa_web::resposta::FaixaDaPesquisa;
use crate::processo::NumeroProcesso;

fn busca(resposta: RespostaDeDocumentos, requisicoes: u8) -> BuscaRealizada<RespostaDeDocumentos> {
    BuscaRealizada {
        unidade: "UNIDADE TESTE".to_owned(),
        resposta,
        requisicoes,
        trabalho: None,
    }
}

fn base(resultado: &str, requisicoes: u8) -> Value {
    json!({
        "resultado": resultado,
        "termo": "0001",
        "unidade": "UNIDADE TESTE",
        "origem": "web",
        "requisicoes": requisicoes,
    })
}

fn com(mut valor: Value, extra: &Value) -> Value {
    valor
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    valor
}

#[test]
fn documento_unico_publica_so_os_ids_internos() {
    let resposta = RespostaDeDocumentos::DocumentoUnico {
        documento: "8765".to_owned(),
        processo: IdProcedimento::novo("4321").unwrap(),
    };
    assert_eq!(
        json_da_busca("0001", &busca(resposta, 3)),
        com(
            base("documento_unico", 3),
            &json!({"documento": {"id": "8765"}, "processo": {"id": "4321"}})
        )
    );
}

#[test]
fn lista_publica_total_faixa_e_documentos_observados() {
    let lista = ListaDeDocumentos {
        documentos: vec![DocumentoEncontrado {
            numero_sei: "0001018".to_owned(),
            rotulo: "Despacho".to_owned(),
            processo: NumeroProcesso::novo("00001.000011/2026-00").unwrap(),
            unidade: "TESTE".to_owned(),
            inclusao: "12/10/2026".to_owned(),
            anexo: true,
        }],
        total_declarado: 16,
        faixa: FaixaDaPesquisa {
            inicio: 11,
            fim: 12,
        },
        linhas_ausentes_da_faixa: 1,
    };
    let realizada = busca(RespostaDeDocumentos::Lista(lista), 4);
    // Goiás: a faixa da barra é publicada com as posições sem linha.
    assert!(humano("0001", &realizada).contains("faixa 11 a 12, 1 sem linha na tabela"));
    assert_eq!(
        json_da_busca("0001", &realizada),
        com(
            base("lista", 4),
            &json!({
                "total_declarado": 16,
                "faixa_observada": {"inicio": 11, "fim": 12},
                "linhas_ausentes_da_faixa": 1,
                "documentos": [{
                    "numero_sei": "0001018",
                    "rotulo": "Despacho",
                    "processo": "00001.000011/2026-00",
                    "unidade": "TESTE",
                    "inclusao": "12/10/2026",
                    "anexo": true
                }]
            })
        )
    );
}

#[test]
fn sem_resultado_publica_total_zero_sem_faixa() {
    assert_eq!(
        json_da_busca("0001", &busca(RespostaDeDocumentos::SemResultado, 3)),
        com(
            base("sem_resultado", 3),
            &json!({"total_declarado": 0, "documentos": []})
        )
    );
}

#[test]
fn termo_recusa_controle_antes_de_aparar_e_vazio() {
    assert_eq!(validar_termo("  0001 "), Ok("0001"));
    assert_eq!(
        validar_termo("0001\n"),
        Err(TermoDeDocumentoInvalido::Controle)
    );
    assert_eq!(validar_termo("   "), Err(TermoDeDocumentoInvalido::Vazio));
    assert_eq!(validar_termo("00.1/2"), Ok("00.1/2"));
}

fn aceita(args: &[&str]) -> bool {
    use clap::Parser;
    let mut todos = vec!["ies", "documento"];
    todos.extend_from_slice(args);
    crate::Cli::try_parse_from(todos).is_ok()
}

#[test]
fn buscar_exige_termo_e_pagina_positiva() {
    assert!(aceita(&["buscar", "0001018"]));
    assert!(aceita(&["buscar", "00010", "--pagina", "2"]));
    assert!(!aceita(&["buscar"]));
    assert!(!aceita(&["buscar", "1", "--pagina", "0"]));
    assert!(!aceita(&["buscar", "--assunto", "x"]));
    assert!(!aceita(&["buscar", "1", "2"]));
}
