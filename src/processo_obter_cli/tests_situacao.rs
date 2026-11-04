//! RED/GREEN de `aberto_em` e da natureza dos documentos no JSON e no
//! texto humano: lista, `[]` ou `null` com motivo (primeiro em
//! `nao_lidos`); `icone`, `cancelado` e motivo por documento.
use super::tests::aberto;
use super::*;
use crate::arvore_processo::{
    AbertoEm, Atribuicao, MotivoAbertoEm, NaturezaDoDocumento, UnidadeAberta,
};

fn nup() -> NumeroProcesso {
    NumeroProcesso::novo("00001.000001/2026-01").unwrap()
}

fn com_aberto_em(aberto_em: AbertoEm) -> ProcessoAberto {
    let mut processo = aberto(None);
    processo.arvore.aberto_em = aberto_em;
    processo
}

fn duas_unidades() -> AbertoEm {
    AbertoEm::Unidades(vec![
        UnidadeAberta {
            sigla: "Unidade1".into(),
            nome: "Unidade Um".into(),
            atribuido_a: Some(Atribuicao {
                sigla: "aluno2".into(),
                nome: "Aluno Dois".into(),
            }),
        },
        UnidadeAberta {
            sigla: "ACE".into(),
            nome: "Arquivo Central".into(),
            atribuido_a: None,
        },
    ])
}

#[test]
fn aberto_em_sai_como_lista_mesmo_sem_documentos() {
    let valor = json_do_processo(&nup(), &com_aberto_em(duas_unidades()), true);
    assert_eq!(
        valor["aberto_em"],
        json!([
            {"sigla": "Unidade1", "nome": "Unidade Um",
             "atribuido_a": {"sigla": "aluno2", "nome": "Aluno Dois"}},
            {"sigla": "ACE", "nome": "Arquivo Central", "atribuido_a": null}
        ])
    );
    assert!(valor.get("aberto_em_nao_lido_motivo").is_none());
    assert_eq!(
        valor["nao_lidos"],
        json!(["andamentos", "marcadores", "propriedades"])
    );
    let texto = humano(&nup(), &com_aberto_em(duas_unidades()), true);
    assert!(
        texto.contains("\nAberto em: Unidade1 (atribuído a aluno2), ACE."),
        "{texto}"
    );
}

#[test]
fn sem_andamentos_abertos_e_lista_vazia() {
    let processo = com_aberto_em(AbertoEm::Unidades(Vec::new()));
    let valor = json_do_processo(&nup(), &processo, true);
    assert_eq!(valor["aberto_em"], json!([]));
    assert!(humano(&nup(), &processo, true).contains("\nSem andamentos abertos."));
}

#[test]
fn nao_lido_e_null_com_motivo_e_primeiro_em_nao_lidos() {
    let processo = com_aberto_em(AbertoEm::NaoLido(
        MotivoAbertoEm::RaizPublicaOVinculoDeAnexacao,
    ));
    let valor = json_do_processo(&nup(), &processo, false);
    assert!(valor["aberto_em"].is_null());
    assert_eq!(
        valor["aberto_em_nao_lido_motivo"],
        "raiz_publica_o_vinculo_de_anexacao"
    );
    assert_eq!(
        valor["nao_lidos"],
        json!(["aberto_em", "andamentos", "marcadores", "propriedades"])
    );
    assert!(
        humano(&nup(), &processo, true)
            .contains("\nAberto em: não lido (raiz_publica_o_vinculo_de_anexacao).")
    );
}

#[test]
fn documento_cancelado_publica_motivo_e_meia_marca_e_null() {
    let mut processo = aberto(None);
    processo.arvore.documentos[0].natureza = NaturezaDoDocumento {
        icone: Some("documento_cancelado".into()),
        cancelado: Some(true),
        motivo_do_cancelamento: Some("erro".into()),
    };
    processo.arvore.documentos[1].natureza = NaturezaDoDocumento {
        icone: Some("documento_cancelado".into()),
        cancelado: None,
        motivo_do_cancelamento: None,
    };
    let valor = json_do_processo(&nup(), &processo, false);
    assert_eq!(valor["documentos"][0]["icone"], "documento_cancelado");
    assert_eq!(valor["documentos"][0]["cancelado"], true);
    assert_eq!(valor["documentos"][0]["motivo_do_cancelamento"], "erro");
    assert!(valor["documentos"][1]["cancelado"].is_null());
    let texto = humano(&nup(), &processo, false);
    assert!(
        texto.contains("- Despacho 0000101 (id 21) [cancelado: erro]"),
        "{texto}"
    );
    assert!(
        texto.contains("- E-mail (id 22) [cancelamento não confirmado]"),
        "{texto}"
    );
}
