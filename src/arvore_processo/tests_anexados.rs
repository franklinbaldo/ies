//! RED/GREEN dos processos anexados: HTML SINTÉTICO com a forma observada
//! na `UnB` 4.0.9 (03/11/2026): `PROCESSO_ANEXADO` com 16 argumentos, pai
//! (a2) igual ao id do processo, a3 `arvore_visualizar` com
//! `id_procedimento` do processo e `id_procedimento_anexado` do anexado,
//! a12 a a14 `null`, a15 igual a a5 e uma barra própria. Os documentos do
//! anexado não aparecem na árvore. Ids, números e rótulos inventados.
use super::tests::{NUP, arvore, no_documento, no_processo};
use super::tests_pastas::no_pasta;
use super::*;

pub(crate) const NUP_ANEXADO: &str = "00002.000002/2026-02";

/// Declaração do `PROCESSO_ANEXADO` na forma observada.
pub(crate) fn no_anexado(i: usize, id: &str, pai: &str, nup: &str) -> String {
    format!(
        "Nos[{i}] = new infraArvoreNo(\"PROCESSO_ANEXADO\",\"{id}\",\"{pai}\",\"controlador.php?acao=arvore_visualizar&acao_origem=procedimento_visualizar&id_procedimento={pai}&id_procedimento_anexado={id}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={h}\",\"ifrVisualizacao\",\"{nup}\",\"Tipo Anexado Sintetico\",\"svg/processo_anexado.svg?11\",\"svg/processo_anexado.svg?11\",\"svg/processo_anexado.svg?11\",true,true,null,null,null,\"{nup}\");",
        h = "e".repeat(64)
    )
}

/// A barra do anexado observada começa por `procedimento_alterar`.
fn com_barra_do_anexado(html: &str, i: usize) -> String {
    html.replace(
        &format!("Nos[{i}].acoes = '';"),
        &format!(
            "Nos[{i}].acoes = '<a href=\"controlador.php?acao=procedimento_alterar&id_procedimento=20&infra_hash={}\"></a>';",
            "d".repeat(64)
        ),
    )
}

fn anexado(id: &str) -> ProcessoNaArvore {
    ProcessoNaArvore {
        id: id.into(),
        nup: NUP_ANEXADO.into(),
        tipo: "Tipo Anexado Sintetico".into(),
    }
}

#[test]
fn le_o_anexado_filho_do_processo_e_os_documentos_do_processo() {
    let html = com_barra_do_anexado(
        &arvore(&[
            no_processo(0, "10", NUP),
            no_anexado(1, "20", "10", NUP_ANEXADO),
            no_documento(2, "21", "10", "0000101"),
        ]),
        1,
    );
    let lida = ler_arvore(&html, NUP, "10").unwrap();
    assert_eq!(lida.processo.id, "10");
    assert_eq!(lida.anexados, [anexado("20")]);
    assert_eq!(lida.documentos.len(), 1);
    assert_eq!(lida.documentos[0].id, "21");
    let debug = format!("{lida:?}");
    assert!(!debug.contains(&"e".repeat(64)), "{debug}");
}

#[test]
fn sem_anexado_a_lista_e_vazia_e_varios_saem_na_ordem() {
    let lida = ler_arvore(&arvore(&[no_processo(0, "10", NUP)]), NUP, "10").unwrap();
    assert_eq!(lida.anexados, []);
    let outro = "00003.000003/2026-03";
    let html = arvore(&[
        no_processo(0, "10", NUP),
        no_anexado(1, "20", "10", NUP_ANEXADO),
        no_anexado(2, "30", "10", outro),
    ]);
    let lida = ler_arvore(&html, NUP, "10").unwrap();
    let ids: Vec<&str> = lida.anexados.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(ids, ["20", "30"]);
    assert_eq!(lida.anexados[1].nup, outro);
}

#[test]
fn identidade_do_processo_continua_conferida_com_anexado() {
    let html = arvore(&[
        no_processo(0, "10", NUP),
        no_anexado(1, "20", "10", NUP_ANEXADO),
    ]);
    // O NUP do anexado não é o processo pedido.
    assert_eq!(
        ler_arvore(&html, NUP_ANEXADO, "20"),
        Err(super::ErroArvore::IdentidadeDoProcessoDiferenteDaPedida)
    );
}

#[test]
fn recusas_do_anexado() {
    let p = || no_processo(0, "10", NUP);
    let a = || no_anexado(1, "20", "10", NUP_ANEXADO);
    let casos: Vec<(Vec<String>, super::ErroArvore)> = vec![
        (
            vec![
                p(),
                a().replace(&format!(",\"{NUP_ANEXADO}\");"), ",\"outro\");"),
            ],
            super::ErroArvore::NupDoNoDoProcessoAnexadoInconsistente,
        ),
        (
            vec![
                p(),
                a().replace(&format!(",\"{NUP_ANEXADO}\");"), ",null);"),
            ],
            super::ErroArvore::ArgumentoDeNoComTipoDiferenteDoObservado,
        ),
        (
            vec![p(), no_anexado(1, "20", "99", NUP_ANEXADO)],
            super::ErroArvore::ProcessoAnexadoComPaiDiferenteDoProcesso,
        ),
        (
            vec![
                p(),
                a().replace(",\"10\",\"controlador", ",null,\"controlador"),
            ],
            super::ErroArvore::ArgumentoDeNoComTipoDiferenteDoObservado,
        ),
        (
            vec![p(), a(), no_anexado(2, "20", "10", "00003.000003/2026-03")],
            super::ErroArvore::ProcessoAnexadoRepetidoNaArvore,
        ),
        (
            vec![p(), a(), no_anexado(2, "30", "10", NUP_ANEXADO)],
            super::ErroArvore::ProcessoAnexadoRepetidoNaArvore,
        ),
        (
            vec![p(), no_anexado(1, "10", "10", NUP_ANEXADO)],
            super::ErroArvore::ProcessoAnexadoIgualAoProcesso,
        ),
        (
            vec![p(), no_anexado(1, "20", "10", NUP)],
            super::ErroArvore::ProcessoAnexadoIgualAoProcesso,
        ),
        // Filhos do anexado nunca observados: recusa, não palpite.
        (
            vec![p(), a(), no_documento(2, "21", "20", "1")],
            super::ErroArvore::NoComPaiNoProcessoAnexado,
        ),
        (
            vec![p(), a(), no_pasta(2, 1, "20", true)],
            super::ErroArvore::NoComPaiNoProcessoAnexado,
        ),
        (
            vec![p(), a(), no_anexado(2, "30", "20", "00003.000003/2026-03")],
            super::ErroArvore::NoComPaiNoProcessoAnexado,
        ),
        (
            vec![p(), no_anexado(1, "2a", "10", NUP_ANEXADO)],
            super::ErroArvore::IdDeNoNaoDecimal,
        ),
        (
            vec![p(), a().replace(",true,true,null,", ",true,true,")],
            super::ErroArvore::QuantidadeDeArgumentosDoNoDiferenteDaObservada {
                esperada: 16,
                observada: 15,
            },
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
fn endereco_e_campos_fixos_do_anexado_na_forma_observada() {
    let a = no_anexado(1, "20", "10", NUP_ANEXADO);
    let formas = [
        a.replace("id_procedimento_anexado=20", "id_procedimento_anexado=21"),
        a.replace("&id_procedimento_anexado=20", ""),
        a.replace("id_procedimento=10&", "id_procedimento=11&"),
        a.replace("acao=arvore_visualizar", "acao=procedimento_trabalhar"),
        a.replace(
            "&infra_sistema",
            "&id_procedimento_anexado=20&infra_sistema",
        ),
        a.replace(
            "\"controlador.php?",
            "\"https://outro.exemplo/controlador.php?",
        ),
        a.replace(",true,true,null,null,null,", ",true,true,\"x\",null,null,"),
        a.replace(",true,true,null,null,null,", ",true,true,null,null,\"x\","),
    ];
    for forma in formas {
        let html = arvore(&[no_processo(0, "10", NUP), forma.clone()]);
        assert_eq!(
            ler_arvore(&html, NUP, "10"),
            Err(super::ErroArvore::ProcessoAnexadoForaDaFormaObservada),
            "{forma}"
        );
    }
}

#[test]
fn anexado_fora_da_forma_nao_e_divergencia_do_pedido() {
    for erro in [
        super::ErroArvore::ProcessoAnexadoForaDaFormaObservada,
        super::ErroArvore::NupDoNoDoProcessoAnexadoInconsistente,
        super::ErroArvore::ProcessoAnexadoComPaiDiferenteDoProcesso,
        super::ErroArvore::ProcessoAnexadoRepetidoNaArvore,
        super::ErroArvore::ProcessoAnexadoIgualAoProcesso,
        super::ErroArvore::NoComPaiNoProcessoAnexado,
    ] {
        assert!(!erro.diverge_do_pedido());
    }
}

/// A árvore do pai aberta a partir do anexado (`ifrArvore` com
/// `id_procedimento_anexado`) marca o nó do anexado: a12 `"noVisitado"`
/// (`UnB` 4.0.9, 03/11/2026). Só esse valor de estado visual é aceito.
#[test]
fn anexado_marcado_como_visitado_na_arvore_aberta_por_ele_e_aceito() {
    let visitado = no_anexado(1, "20", "10", NUP_ANEXADO).replace(
        "true,true,null,null,null,",
        "true,true,\"noVisitado\",null,null,",
    );
    assert!(visitado.contains("\"noVisitado\",null,null,"));
    let html = com_barra_do_anexado(&arvore(&[no_processo(0, "10", NUP), visitado.clone()]), 1);
    assert_eq!(
        ler_arvore(&html, NUP, "10").unwrap().anexados,
        [anexado("20")]
    );
    for outro in [
        "\"selecionado\",null,null,",
        "null,\"noVisitado\",null,",
        "null,null,\"noVisitado\",",
    ] {
        let html = com_barra_do_anexado(
            &arvore(&[
                no_processo(0, "10", NUP),
                visitado.replace("\"noVisitado\",null,null,", outro),
            ]),
            1,
        );
        assert_eq!(
            ler_arvore(&html, NUP, "10").map(|_| ()),
            Err(super::ErroArvore::ProcessoAnexadoForaDaFormaObservada),
            "{outro}"
        );
    }
}
