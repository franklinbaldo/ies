//! Casos sintéticos derivados de `marcadores_processo/tests_apoio.rs` e da
//! identidade id#nome observada no relatório live 2026-11-04 (Refs #255).
use super::*;
use crate::marcadores_processo::MarcadorDoProcesso;
fn marcadores(ids: &[(&str, &str)], total: usize) -> MarcadoresLidos {
    MarcadoresLidos {
        total_declarado: total,
        itens: ids
            .iter()
            .map(|(id, nome)| MarcadorDoProcesso {
                id: (*id).to_owned(),
                nome: (*nome).to_owned(),
                cor: "azul".into(),
                texto: None,
                usuario: None,
                data_hora: "04/11/2026 09:00".into(),
            })
            .collect(),
    }
}
#[test]
fn rotulo_id_nome_pode_conter_outras_cerquilhas() {
    assert_eq!(
        interpretar_rotulo("501#A#B"),
        Ok(Rotulo {
            id: "501".into(),
            nome: "A#B".into()
        })
    );
}
#[test]
fn rotulo_invalido_falha_antes_da_sessao() {
    for (texto, esperado) in [
        ("501", ErroMarcar::RotuloSemSeparador),
        ("abc#A", ErroMarcar::IdDoMarcadorNaoDecimal),
        ("#A", ErroMarcar::IdDoMarcadorNaoDecimal),
        ("1# ", ErroMarcar::NomeDoMarcadorVazio),
        ("1#A\nB", ErroMarcar::NomeDoMarcadorComControle),
    ] {
        assert_eq!(interpretar_rotulo(texto), Err(esperado), "{texto:?}");
    }
}
#[test]
fn ja_presente_na_atribuicao_nao_escreve() {
    assert_eq!(
        decidir(
            &marcadores(&[("501", "A")], 1),
            &interpretar_rotulo("501#A").unwrap(),
            Operacao::Atribuir
        ),
        Ok(Decisao::NadaAFazer)
    );
}
#[test]
fn retirada_presente_exibe_previa_sem_post() {
    assert_eq!(
        decidir(
            &marcadores(&[("501", "A")], 1),
            &interpretar_rotulo("501#A").unwrap(),
            Operacao::Retirar
        ),
        Ok(Decisao::PreviaDeRetirada)
    );
}
#[test]
fn retirada_ausente_sem_escrita() {
    assert_eq!(
        decidir(
            &marcadores(&[], 0),
            &interpretar_rotulo("501#A").unwrap(),
            Operacao::Retirar
        ),
        Ok(Decisao::NadaAFazer)
    );
}
#[test]
fn atribuir_ausente_segue_para_o_catalogo_sem_escrita() {
    assert_eq!(
        decidir(
            &marcadores(&[], 0),
            &interpretar_rotulo("501#A").unwrap(),
            Operacao::Atribuir
        ),
        Ok(Decisao::PreviaDeAtribuicao)
    );
}
#[test]
fn total_declarado_maior_que_observado_nao_autoriza_ausencia() {
    assert_eq!(
        decidir(
            &marcadores(&[], 1),
            &interpretar_rotulo("501#A").unwrap(),
            Operacao::Retirar
        ),
        Err(ErroMarcar::ListaDeMarcadoresParcial)
    );
}
#[test]
fn mesmo_id_com_outro_nome_nao_e_identidade_igual() {
    assert_eq!(
        decidir(
            &marcadores(&[("501", "B")], 1),
            &interpretar_rotulo("501#A").unwrap(),
            Operacao::Retirar
        ),
        Err(ErroMarcar::RotuloDiferenteDoMarcadorLido)
    );
}

#[test]
fn texto_do_marcador_respeita_o_limite_observado_de_250() {
    assert_eq!(validar_texto(&"Á".repeat(250)), Ok(()));
    assert_eq!(
        validar_texto(&"a".repeat(251)),
        Err(ErroMarcar::TextoDoMarcadorLongo)
    );
}
#[test]
fn texto_de_textarea_admite_multiplas_linhas_e_nao_presume_charset() {
    for texto in [
        "Primeira linha\nSegunda linha",
        "Emoji 🚀",
        "  Texto preservado  ",
    ] {
        assert_eq!(validar_texto(texto), Ok(()));
    }
}
