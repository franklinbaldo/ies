//! RED/GREEN da borda de `ies processo buscar`: JSON das três respostas e
//! gramática Clap. Sem rede.
use super::*;
use crate::dominio::IdProcedimento;
use crate::pesquisa_web::ResultadoPesquisa;
use crate::pesquisa_web::resposta::{FaixaDaPesquisa, ListaDaPesquisa, RespostaDaPesquisa};

fn busca(resposta: RespostaDaPesquisa, requisicoes: u8) -> BuscaRealizada {
    BuscaRealizada {
        unidade: "UNIDADE TESTE".to_owned(),
        resposta,
        requisicoes,
        trabalho: None,
    }
}

fn termo() -> NumeroProcesso {
    NumeroProcesso::novo("  00001  ").unwrap()
}

#[test]
fn processo_unico_publica_so_o_id_e_o_termo() {
    let valor = json_da_busca(
        &termo(),
        &busca(
            RespostaDaPesquisa::ProcessoUnico {
                id: IdProcedimento::novo("4321").unwrap(),
            },
            3,
        ),
    );
    assert_eq!(
        valor,
        json!({
            "resultado": "processo_unico",
            "termo": "00001",
            "unidade": "UNIDADE TESTE",
            "origem": "web",
            "requisicoes": 3,
            "processo": {"id": "4321"}
        })
    );
}

#[test]
fn lista_publica_total_faixa_e_numeros() {
    let lista = ListaDaPesquisa {
        processos: vec![ResultadoPesquisa {
            numero: NumeroProcesso::novo("00001.000011/2026-00").unwrap(),
        }],
        total_declarado: 1234,
        faixa: FaixaDaPesquisa {
            inicio: 11,
            fim: 12,
        },
        linhas_ausentes_da_faixa: 1,
    };
    let realizada = busca(RespostaDaPesquisa::Lista(lista), 4);
    // Goiás: a faixa da barra é publicada com as posições sem linha.
    assert!(humano(&termo(), &realizada).contains("faixa 11 a 12, 1 sem linha na tabela"));
    let valor = json_da_busca(&termo(), &realizada);
    assert_eq!(
        valor,
        json!({
            "resultado": "lista",
            "termo": "00001",
            "unidade": "UNIDADE TESTE",
            "origem": "web",
            "requisicoes": 4,
            "total_declarado": 1234,
            "faixa_observada": {"inicio": 11, "fim": 12},
            "linhas_ausentes_da_faixa": 1,
            "processos": [{"numero": "00001.000011/2026-00"}]
        })
    );
}

#[test]
fn sem_resultado_publica_total_zero_sem_faixa() {
    let valor = json_da_busca(&termo(), &busca(RespostaDaPesquisa::SemResultado, 3));
    assert_eq!(
        valor,
        json!({
            "resultado": "sem_resultado",
            "termo": "00001",
            "unidade": "UNIDADE TESTE",
            "origem": "web",
            "requisicoes": 3,
            "total_declarado": 0,
            "processos": []
        })
    );
    assert!(valor.get("completa").is_none());
}

fn aceita(args: &[&str]) -> bool {
    use clap::Parser;
    let mut todos = vec!["ies", "processo"];
    todos.extend_from_slice(args);
    crate::Cli::try_parse_from(todos).is_ok()
}

#[test]
fn buscar_exige_termo_e_pagina_positiva() {
    assert!(aceita(&["buscar", "00001"]));
    assert!(aceita(&["buscar", "00001.000001/2026-00", "--pagina", "2"]));
    assert!(!aceita(&["buscar"]));
    assert!(!aceita(&["buscar", "1", "--pagina", "0"]));
    assert!(!aceita(&["buscar", "1", "--pagina", "x"]));
    assert!(!aceita(&["buscar", "--assunto", "x"]));
    assert!(!aceita(&["buscar", "1", "2"]));
}
