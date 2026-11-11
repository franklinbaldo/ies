//! Casos sintéticos derivados de `marcadores_processo/tests_apoio.rs` e da
//! identidade id#nome observada no relatório live 2026-11-04 (Refs #255).
use super::escrita::action_de_remocao;
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

/// Atribuição de `action` como o SEI a publica no corpo de uma função (sintética).
fn atribuicao_de_acao(url: &str) -> String {
    format!("document.getElementById('frmGerenciarMarcador').action='{url}';")
}

/// Tela de remoção sintética: `acaoRemover` com `em_remover` no corpo e
/// `acaoRemocaoMultipla` logo depois, com `na_multipla`.
fn tela_de_remocao(em_remover: &str, na_multipla: &str) -> String {
    format!(
        r#"<a onclick="acaoRemover('501','A');">Remover</a>
<script>
function acaoRemover(id,desc){{
if (confirm("Confirma remoção do Marcador \"" + desc + "\" do processo?")){{
document.getElementById('hdnInfraItemId').value=id;
{em_remover}
document.getElementById('frmGerenciarMarcador').submit();
}}
}}
function acaoRemocaoMultipla(){{
{na_multipla}
}}
</script>"#
    )
}

const URL_REMOVER: &str =
    "controlador.php?acao=andamento_marcador_remover&id_procedimento=1&infra_hash=SINTETICO";
const URL_MULTIPLA: &str =
    "controlador.php?acao=andamento_marcador_remover&id_procedimento=1&infra_hash=OUTRO";

#[test]
fn acao_de_remocao_aceita_uma_atribuicao_no_corpo_de_acao_remover_mesmo_com_outra_na_multipla() {
    let html = tela_de_remocao(
        &atribuicao_de_acao(URL_REMOVER),
        &atribuicao_de_acao(URL_MULTIPLA),
    );
    assert_eq!(action_de_remocao(&html, "501"), Ok(URL_REMOVER.to_owned()));
}

#[test]
fn acao_de_remocao_recusa_duas_atribuicoes_dentro_de_acao_remover() {
    let html = tela_de_remocao(
        &format!(
            "{}\n{}",
            atribuicao_de_acao(URL_REMOVER),
            atribuicao_de_acao(URL_MULTIPLA)
        ),
        "",
    );
    assert_eq!(
        action_de_remocao(&html, "501"),
        Err(ErroMarcar::AcaoDeRemocaoAmbigua)
    );
}

#[test]
fn acao_de_remocao_recusa_atribuicao_so_fora_do_corpo_de_acao_remover() {
    let html = tela_de_remocao("", &atribuicao_de_acao(URL_MULTIPLA));
    assert_eq!(
        action_de_remocao(&html, "501"),
        Err(ErroMarcar::AcaoDeRemocaoAusente)
    );
}

#[test]
fn acao_de_remocao_recusa_id_sem_handler_na_tela() {
    let html = tela_de_remocao(&atribuicao_de_acao(URL_REMOVER), "");
    assert_eq!(
        action_de_remocao(&html, "999"),
        Err(ErroMarcar::AcaoDeRemocaoAusente)
    );
}
