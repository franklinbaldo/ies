use super::{ConsultaDoControlador, ErroConsultaDoControlador};
use crate::acao_do_sei::AcaoDoSei;
use crate::transporte::FalhaTransporte;

#[test]
fn le_acao_e_valor_unico() {
    let consulta =
        ConsultaDoControlador::ler("acao=arvore_visualizar&id_procedimento=42&infra_hash=ab")
            .unwrap();
    assert_eq!(consulta.acao(), Ok(AcaoDoSei::ArvoreVisualizar));
    assert_eq!(consulta.unico("id_procedimento"), Ok("42"));
}

#[test]
fn cada_recusa_tem_o_seu_erro() {
    let casos = [
        (
            "acao=arvore_visualizar&solto",
            ErroConsultaDoControlador::ParSemIgual,
        ),
        (
            "Acao=arvore_visualizar",
            ErroConsultaDoControlador::RecusadaPeloTransporte(
                FalhaTransporte::ChaveDeConsultaNaoCanonica,
            ),
        ),
        (
            "acao=arvore_visualizar;x",
            ErroConsultaDoControlador::RecusadaPeloTransporte(
                FalhaTransporte::SeparadorDeConsultaAmbiguo,
            ),
        ),
    ];
    for (texto, erro) in casos {
        assert_eq!(
            ConsultaDoControlador::ler(texto).err(),
            Some(erro),
            "{texto}"
        );
    }
}

#[test]
fn chave_ausente_repetida_e_acao_desconhecida() {
    let consulta = ConsultaDoControlador::ler("acao=nao_existe&id=1&id=2").unwrap();
    assert_eq!(
        consulta.unico("id_procedimento"),
        Err(ErroConsultaDoControlador::ChaveAusente {
            chave: "id_procedimento"
        })
    );
    assert_eq!(
        consulta.unico("id"),
        Err(ErroConsultaDoControlador::ChaveRepetida { chave: "id" })
    );
    assert_eq!(
        consulta.acao(),
        Err(ErroConsultaDoControlador::AcaoDesconhecida)
    );
    assert_eq!(consulta.valores("id").collect::<Vec<_>>(), ["1", "2"]);
}
