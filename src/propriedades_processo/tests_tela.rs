//! RED/GREEN do leitor da tela de propriedades: forma observada na `UnB`
//! 4.0.9, identidade, campos e recusas tipadas. Fixtures sintéticas.
use super::tests_apoio::{tela_das_propriedades, tela_restrita};
use super::*;
use crate::html_atributos::com_booleano_antes_dos_atributos;

const NUP: &str = "00001.000001/2026-01";
const ID: &str = "10";

fn ler(html: &str) -> Result<PropriedadesLidas, super::ErroPropriedades> {
    ler_propriedades(html, NUP, ID, FonteDasPropriedades::Alterar)
}

fn padrao() -> String {
    tela_das_propriedades(NUP, ID)
}

fn trocar(de: &str, para: &str) -> String {
    let html = padrao();
    assert!(html.contains(de), "fixture sem {de}");
    html.replacen(de, para, 1)
}

#[test]
fn le_os_campos_da_forma_observada() {
    let lidas = ler(&padrao()).unwrap();
    assert_eq!(
        lidas,
        PropriedadesLidas {
            tipo: Some("Tipo Sintetico: Teste".into()),
            especificacao: "Especificação sintética & teste".into(),
            interessados: vec!["Pessoa Sintetica A".into(), "Entidade Sintetica B".into()],
            assuntos: vec![Assunto {
                codigo: Some("000.01".into()),
                descricao: "Assunto sintético um".into(),
            }],
            observacoes: "Observação sintética".into(),
            nivel_de_acesso: NivelDeAcesso::Publico,
            hipotese_legal: None,
            data_de_geracao: Some("01/02/2026".into()),
            fonte: FonteDasPropriedades::Alterar,
        }
    );
}

#[test]
fn booleano_antes_dos_atributos_nao_muda_a_leitura() {
    let html = com_booleano_antes_dos_atributos(&padrao());
    assert_eq!(ler(&html), ler(&padrao()));
}

#[test]
fn fonte_consultar_e_publicada() {
    let lidas = ler_propriedades(&padrao(), NUP, ID, FonteDasPropriedades::Consultar).unwrap();
    assert_eq!(lidas.fonte, FonteDasPropriedades::Consultar);
}

#[test]
fn restrito_publica_a_hipotese_marcada() {
    let lidas = ler(&tela_restrita(NUP, ID)).unwrap();
    assert_eq!(lidas.nivel_de_acesso, NivelDeAcesso::Restrito);
    assert_eq!(
        lidas.hipotese_legal.as_deref(),
        Some("Hipotese Sintetica (Art. 1)")
    );
}

#[test]
fn hipotese_marcada_com_valor_null_e_nao_observada() {
    let html = tela_restrita(NUP, ID)
        .replace(
            "<option value=\"7\" selected=\"selected\">",
            "<option value=\"7\">",
        )
        .replace(
            "<option value=\"null\">&nbsp;</option>\n<option value=\"7\"",
            "<option value=\"null\" selected=\"selected\">&nbsp;</option>\n<option value=\"7\"",
        );
    assert_eq!(ler(&html).unwrap().hipotese_legal, None);
}

#[test]
fn sigiloso_pelo_valor_2() {
    let html = trocar("value=\"0\" checked=\"checked\" class", "value=\"0\" class").replacen(
        "value=\"2\"  class",
        "value=\"2\" checked=\"checked\" class",
        1,
    );
    assert_eq!(ler(&html).unwrap().nivel_de_acesso, NivelDeAcesso::Sigiloso);
}

#[test]
fn identidade_e_conferida() {
    assert_eq!(
        ler_propriedades(
            &padrao(),
            "00009.000009/2026-09",
            ID,
            FonteDasPropriedades::Alterar
        ),
        Err(super::ErroPropriedades::NumeroDasPropriedadesDiferenteDoSolicitado)
    );
    assert_eq!(
        ler_propriedades(&padrao(), NUP, "11", FonteDasPropriedades::Alterar),
        Err(super::ErroPropriedades::IdDasPropriedadesDiferenteDoAberto)
    );
    let sem = trocar(
        "id=\"hdnProtocoloFormatado\" name=\"hdnProtocoloFormatado\"",
        "",
    );
    assert_eq!(
        ler(&sem),
        Err(super::ErroPropriedades::CampoDasPropriedadesAusente)
    );
    let repetido = padrao().replace(
        "</form>",
        &format!("<input type=\"hidden\" id=\"hdnIdProcedimento\" name=\"hdnIdProcedimento\" value=\"{ID}\" /></form>"),
    );
    assert_eq!(
        ler(&repetido),
        Err(super::ErroPropriedades::CampoDasPropriedadesRepetido)
    );
}

#[test]
fn formulario_ausente_ou_repetido() {
    let sem = trocar("id=\"frmProcedimentoCadastro\"", "id=\"frmOutro\"");
    assert_eq!(
        ler(&sem),
        Err(super::ErroPropriedades::FormularioDasPropriedadesAusente)
    );
    assert_eq!(
        ler("<html><body>Link sem assinatura</body></html>"),
        Err(super::ErroPropriedades::FormularioDasPropriedadesAusente)
    );
    let dois = padrao().replace(
        "</body>",
        "<form id=\"frmProcedimentoCadastro\"></form></body>",
    );
    assert_eq!(
        ler(&dois),
        Err(super::ErroPropriedades::FormulariosDasPropriedadesRepetidos)
    );
}

#[test]
fn tipo_sem_marcacao_e_nulo_e_id_oculto_diferente_e_recusa() {
    let sem = trocar(
        "<option value=\"100000002\" selected=\"selected\">",
        "<option value=\"100000002\">",
    );
    assert_eq!(ler(&sem).unwrap().tipo, None);
    let outro = trocar(
        "id=\"hdnIdTipoProcedimento\" name=\"hdnIdTipoProcedimento\" value=\"100000002\"",
        "id=\"hdnIdTipoProcedimento\" name=\"hdnIdTipoProcedimento\" value=\"100000003\"",
    );
    assert_eq!(
        ler(&outro),
        Err(super::ErroPropriedades::TipoDoProcessoInconsistente)
    );
    let dois = trocar(
        "<option value=\"100000003\">",
        "<option value=\"100000003\" selected=\"selected\">",
    );
    assert_eq!(
        ler(&dois),
        Err(super::ErroPropriedades::OpcoesMarcadasRepetidas)
    );
}

#[test]
fn assunto_sem_codigo_publica_so_a_descricao() {
    let html = trocar("000.01 - Assunto sint\u{e9}tico um", "Assunto sem codigo");
    assert_eq!(
        ler(&html).unwrap().assuntos,
        [Assunto {
            codigo: None,
            descricao: "Assunto sem codigo".into()
        }]
    );
}

#[test]
fn assunto_fora_da_forma_e_recusa() {
    for (de, para) in [
        ("<option value=\"2001\">", "<option value=\"x1\">"),
        ("<option value=\"2001\">", "<option>"),
        ("000.01 - Assunto sint\u{e9}tico um", "  "),
    ] {
        assert_eq!(
            ler(&trocar(de, para)),
            Err(super::ErroPropriedades::AssuntoForaDaFormaObservada),
            "{para}"
        );
    }
}

#[test]
fn interessado_sem_texto_e_recusa_e_sem_opcoes_e_lista_vazia() {
    let vazio = trocar("Pessoa Sintetica A", "");
    assert_eq!(
        ler(&vazio),
        Err(super::ErroPropriedades::InteressadoForaDaFormaObservada)
    );
    let sem = trocar(
        "<option value=\"301\">Pessoa Sintetica A</option>\n<option value=\"302\">Entidade Sintetica B</option>\n",
        "",
    );
    assert_eq!(ler(&sem).unwrap().interessados, Vec::<String>::new());
}

#[test]
fn lista_oculta_preenchida_nunca_observada_e_recusa() {
    for campo in ["hdnAssuntos", "hdnInteressadosProcedimento"] {
        let html = trocar(
            &format!("id=\"{campo}\" name=\"{campo}\" value=\"\""),
            &format!("id=\"{campo}\" name=\"{campo}\" value=\"1\""),
        );
        assert_eq!(
            ler(&html),
            Err(super::ErroPropriedades::ListaOcultaDasPropriedadesNaoObservada),
            "{campo}"
        );
    }
}

#[test]
fn nivel_de_acesso_nao_marcado_ou_fora_da_forma() {
    let nenhum = trocar("value=\"0\" checked=\"checked\" class", "value=\"0\" class");
    assert_eq!(
        ler(&nenhum),
        Err(super::ErroPropriedades::NivelDeAcessoNaoMarcado)
    );
    let dois = padrao().replacen(
        "value=\"1\"  class",
        "value=\"1\" checked=\"checked\" class",
        1,
    );
    assert_eq!(
        ler(&dois),
        Err(super::ErroPropriedades::NivelDeAcessoForaDaFormaObservada)
    );
    let valor = trocar(
        "value=\"0\" checked=\"checked\"",
        "value=\"9\" checked=\"checked\"",
    );
    assert_eq!(
        ler(&valor),
        Err(super::ErroPropriedades::NivelDeAcessoForaDaFormaObservada)
    );
    let id = trocar("id=\"optPublico\"", "id=\"optOutro\"");
    assert_eq!(
        ler(&id),
        Err(super::ErroPropriedades::NivelDeAcessoForaDaFormaObservada)
    );
}

#[test]
fn data_de_geracao_vazia_e_nula_e_fora_da_forma_e_recusa() {
    let vazia = trocar("value=\"01/02/2026\"", "value=\"\"");
    assert_eq!(ler(&vazia).unwrap().data_de_geracao, None);
    let fora = trocar("value=\"01/02/2026\"", "value=\"2026-02-01\"");
    assert_eq!(
        ler(&fora),
        Err(super::ErroPropriedades::DataDeGeracaoForaDaFormaObservada)
    );
}

#[test]
fn entidade_desconhecida_e_recusa() {
    let html = trocar("&amp; teste", "&desconhecida; teste");
    assert_eq!(
        ler(&html),
        Err(super::ErroPropriedades::TextoDasPropriedadesNaoDecodificado)
    );
}

#[test]
fn campos_obrigatorios_ausentes_sao_recusa() {
    for id in [
        "selTipoProcedimento",
        "txtDescricao",
        "selAssuntos",
        "selInteressadosProcedimento",
        "txaObservacoes",
        "selHipoteseLegal",
    ] {
        let html = trocar(&format!("id=\"{id}\""), "id=\"outro\"");
        assert_eq!(
            ler(&html),
            Err(super::ErroPropriedades::CampoDasPropriedadesAusente),
            "{id}"
        );
    }
}

#[test]
fn debug_do_endereco_e_redigido() {
    let endereco = EnderecoDasPropriedades::novo(
        FonteDasPropriedades::Alterar,
        "https://x/controlador.php?infra_hash=segredo".into(),
    );
    assert!(!format!("{endereco:?}").contains("segredo"));
}
