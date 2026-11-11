//! RED/GREEN dos leitores da troca: link do `lnkInfraUnidade`, tela Trocar
//! Unidade, escolha da sigla, POST derivado e `Location` dos 302. Telas
//! sintéticas na forma observada (ver `tests_apoio`).
use super::retorno::{hash_do_salto, url_do_salto};
use super::tela::{TROCAR_UNIDADE, campos_do_envio, escolher, ler_tela_de_troca, link_da_troca};
use super::tests_apoio::*;
use crate::formulario_web::{CodificacaoDaTela, TelaServida, derivar_submissao};
use crate::instalacao::Instalacao;
use crate::tela_html::HtmlServido;

fn instalacao() -> Instalacao {
    Instalacao::interpretar(ORIGEM).unwrap()
}

#[test]
fn link_da_troca_vem_do_par_de_lnk_infra_unidade() {
    let html = HtmlServido::de_teste(&controle("PR-A", ID_A));
    assert_eq!(
        link_da_troca(&instalacao(), &url_controle(ID_A), &html),
        Ok(url_da_troca(ID_A))
    );
}

#[test]
fn link_da_troca_ausente_divergente_ou_fora_da_forma_e_recusado() {
    let sei = instalacao();
    let url = url_controle(ID_A);
    let sem = HtmlServido::de_teste("<a id=\"lnkInfraUnidade\" href=\"#\">PR-A</a>");
    assert_eq!(
        link_da_troca(&sei, &url, &sem),
        Err(super::ErroTroca::LinkDeTrocaDeUnidadeAusente)
    );
    assert_eq!(
        link_da_troca(&sei, &url, &HtmlServido::de_teste("<p>x</p>")),
        Err(super::ErroTroca::LinkDeTrocaDeUnidadeAusente)
    );
    let divergente = HtmlServido::de_teste(&barra("PR-A", ID_A).replacen(ID_A, ID_B, 1));
    assert_eq!(
        link_da_troca(&sei, &url, &divergente),
        Err(super::ErroTroca::LinksDeTrocaDivergentes)
    );
    for (de, para) in [
        ("acao=infra_trocar_unidade", "acao=procedimento_controlar"),
        (
            "acao=infra_trocar_unidade",
            "acao=infra_trocar_unidade&acao=x",
        ),
        (
            "acao=infra_trocar_unidade",
            "acao=infra_trocar_unidade&acao_ajax=1",
        ),
        ("&infra_hash=", "&infra_hash=1"),
        ("window.location.href='", "location.href='"),
        (
            "controlador.php?",
            "https://outro.exemplo/sei/controlador.php?",
        ),
    ] {
        let html = HtmlServido::de_teste(&barra("PR-A", ID_A).replace(de, para));
        assert_eq!(
            link_da_troca(&sei, &url, &html),
            Err(super::ErroTroca::LinkDeTrocaDeUnidadeForaDaFormaObservada),
            "{para}"
        );
    }
}

#[test]
fn tela_de_troca_lida_com_unidades_marcada_e_filtros() {
    let tela = ler_tela_de_troca(&HtmlServido::de_teste(&tela())).unwrap();
    let siglas: Vec<_> = tela.unidades.iter().map(|u| u.sigla.as_str()).collect();
    assert_eq!(siglas, ["PR-A", "PR-B"]);
    assert_eq!(tela.unidades[1].id_unidade, ID_B);
    assert_eq!(tela.unidades[1].descricao, "Unidade PR-B & Cia");
    assert_eq!(tela.marcada.sigla, "PR-A");
    assert_eq!(
        tela.filtros,
        [
            ("txtInfraSiglaUnidade", String::new()),
            ("txtInfraDescricaoUnidade", String::new()),
            // Dois `selected`: o navegador fica com o último.
            ("selInfraOrgaoUnidade", String::new()),
        ]
    );
}

#[test]
fn tela_de_troca_fora_da_forma_e_recusada() {
    let duas = format!(
        "{}{}",
        linha("PR-A", ID_A, true),
        linha("PR-B", ID_B, false)
    );
    let ativa = ("PR-A", ID_A);
    let casos = [
        (
            tela_com(ativa, 3, &duas, SCRIPT),
            super::ErroTroca::ListaDeUnidadesIncompleta,
        ),
        (
            tela_com(ativa, 2, &duas, "<script>function x(){}</script>"),
            super::ErroTroca::SelecaoDeUnidadePorScriptNaoPublicada,
        ),
        (
            tela_com(
                ativa,
                2,
                &duas.replace("selecionarUnidade(110000002)", "selecionarUnidade(9)"),
                SCRIPT,
            ),
            super::ErroTroca::LinhaDeUnidadeForaDaFormaObservada,
        ),
        (
            tela_com(
                ativa,
                2,
                &duas.replace("title=\"PR-B\"", "title=\"PR-C\""),
                SCRIPT,
            ),
            super::ErroTroca::LinhaDeUnidadeForaDaFormaObservada,
        ),
        (
            tela_com(
                ativa,
                2,
                &format!(
                    "{}{}",
                    linha("PR-A", ID_A, true),
                    linha("PR-B", ID_A, false)
                ),
                SCRIPT,
            ),
            super::ErroTroca::LinhaDeUnidadeForaDaFormaObservada,
        ),
        (
            tela_com(ativa, 2, &duas.replace("checked=\"checked\" ", ""), SCRIPT),
            super::ErroTroca::UnidadeMarcadaDiferenteDaAtiva,
        ),
        (
            tela_com(
                ativa,
                2,
                &format!("{}{}", linha("PR-A", ID_A, true), linha("PR-B", ID_B, true)),
                SCRIPT,
            ),
            super::ErroTroca::UnidadeMarcadaDiferenteDaAtiva,
        ),
        (
            controle("PR-A", ID_A),
            super::ErroTroca::TelaDeTrocaDeUnidadeNaoReconhecida,
        ),
    ];
    for (html, esperado) in casos {
        assert_eq!(
            ler_tela_de_troca(&HtmlServido::de_teste(&html)).map(|_| ()),
            Err(esperado.clone()),
            "{esperado:?}"
        );
    }
}

#[test]
fn atributos_repetidos_preservam_a_causa_da_recusa_na_troca() {
    use crate::html_atributos::ErroAtributoHtml;

    // Contraprovas sintéticas: as cassettes da troca não registram
    // um rádio ou filtro com o atributo value repetido.
    let linha_repetida = HtmlServido::de_teste(&tela().replacen(
        &format!("value=\"{ID_B}\" onclick="),
        &format!("value=\"{ID_B}\" value=\"{ID_C}\" onclick="),
        1,
    ));
    assert!(matches!(
        ler_tela_de_troca(&linha_repetida),
        Err(super::ErroTroca::AtributoDaLinhaDeUnidadeIlegivel(
            ErroAtributoHtml::AtributoRepetidoNaTag { atributo: "value" }
        ))
    ));

    let filtro_repetido = HtmlServido::de_teste(&tela().replacen(
        "id=\"txtInfraSiglaUnidade\" name=\"txtInfraSiglaUnidade\" class=\"infraText\" value=\"\"",
        "id=\"txtInfraSiglaUnidade\" name=\"txtInfraSiglaUnidade\" class=\"infraText\" value=\"\" value=\"outra\"",
        1,
    ));
    assert!(matches!(
        ler_tela_de_troca(&filtro_repetido),
        Err(super::ErroTroca::AtributoDaTelaDeTrocaIlegivel(
            ErroAtributoHtml::AtributoRepetidoNaTag { atributo: "value" }
        ))
    ));
}

#[test]
fn escolha_e_exata_e_recusa_ausente_e_ambigua() {
    let tela = ler_tela_de_troca(&HtmlServido::de_teste(&tela())).unwrap();
    assert_eq!(escolher(&tela, "PR-B").unwrap().id_unidade, ID_B);
    let ausente = escolher(&tela, "pr-b");
    assert_eq!(
        ausente,
        Err(super::ErroTroca::UnidadeNaoOferecida {
            oferecidas: tela.unidades.clone()
        })
    );
    let linhas = format!(
        "{}{}{}",
        linha("PR-A", ID_A, true),
        linha("PR-B", ID_B, false),
        linha("PR-B", ID_C, false)
    );
    let repetida = ler_tela_de_troca(&HtmlServido::de_teste(&tela_com(
        ("PR-A", ID_A),
        3,
        &linhas,
        SCRIPT,
    )))
    .unwrap();
    match escolher(&repetida, "PR-B") {
        Err(super::ErroTroca::SiglaDeUnidadeAmbigua { candidatas }) => {
            let ids: Vec<_> = candidatas.iter().map(|u| u.id_unidade.as_str()).collect();
            assert_eq!(ids, [ID_B, ID_C]);
        }
        outro => panic!("{outro:?}"),
    }
}

#[test]
fn post_derivado_e_o_do_navegador_com_sel_infra_unidades_no_fim() {
    let html = tela();
    let lida = ler_tela_de_troca(&HtmlServido::de_teste(&html)).unwrap();
    let escolhida = escolher(&lida, "PR-B").unwrap();
    let campos = campos_do_envio(&lida, escolhida);
    let url = url_da_troca(ID_A);
    let servida = TelaServida {
        url: &url,
        html: &HtmlServido::de_teste(&html),
        codificacao: CodificacaoDaTela::Iso88591,
    };
    let s = derivar_submissao(
        &TROCAR_UNIDADE,
        &instalacao(),
        &servida,
        Some("PR-A"),
        &campos,
    )
    .unwrap();
    assert_eq!(
        s.url(),
        format!(
            "{ORIGEM}/controlador.php?acao=infra_trocar_unidade&acao_origem=infra_trocar_unidade&infra_sistema=100000100&infra_unidade_atual={ID_A}&infra_hash={}",
            hash('c', 64)
        )
    );
    assert_eq!(
        String::from_utf8(s.corpo().to_vec()).unwrap(),
        format!(
            "hdnInfraTipoPagina=1&txtInfraSiglaUnidade=&txtInfraDescricaoUnidade=&selInfraOrgaoUnidade=\
             &chkInfraItem={ID_B}&hdnInfraNroItens=2&hdnInfraItemId=&hdnInfraItens={ID_A}%2C{ID_B}\
             &hdnInfraItensHash={}&hdnInfraItensSelecionados={ID_B}&hdnInfraSelecoes=Infra\
             &hdnInfraPaginaSelecao=Sim&selInfraUnidades={ID_B}",
            hash('e', 64)
        )
    );
}

#[test]
fn localizacao_do_salto_https_http_e_relativa_no_mesmo_host() {
    let post = url_da_troca(ID_A);
    assert_eq!(
        url_do_salto(&localizacao_com_aviso("https", ID_B), &post, ID_B),
        Ok(url_com_aviso(ID_B))
    );
    // GO: `http://` no mesmo host; o pedido é `https://`.
    assert_eq!(
        url_do_salto(&localizacao_com_aviso("http", ID_B), &post, ID_B),
        Ok(url_com_aviso(ID_B))
    );
    assert_eq!(
        url_do_salto(&localizacao_limpa("http", ID_B), &url_com_aviso(ID_B), ID_B),
        Ok(url_limpa(ID_B))
    );
    let relativa = localizacao_limpa("https", ID_B).replace(&format!("https://{HOST}/sei/"), "");
    assert_eq!(url_do_salto(&relativa, &post, ID_B), Ok(url_limpa(ID_B)));
}

#[test]
fn localizacao_do_salto_fora_da_forma_ou_de_outra_unidade_e_recusada() {
    let post = url_da_troca(ID_A);
    let limpa = localizacao_limpa("https", ID_B);
    assert_eq!(
        url_do_salto(&localizacao_limpa("https", ID_C), &post, ID_B),
        Err(super::FalhaDepoisDoEnvio::LocalizacaoDeOutraUnidade)
    );
    let forma = super::FalhaDepoisDoEnvio::LocalizacaoForaDaFormaObservada;
    let consulta = super::FalhaDepoisDoEnvio::ConsultaDaLocalizacaoForaDaForma(
        super::tela::ConsultaForaDaForma,
    );
    for (ruim, esperado) in [
        (limpa.replace(HOST, "outro.exemplo.gov.br"), forma.clone()),
        (limpa.replace("/sei/", "/outra/"), forma.clone()),
        (limpa.replace(HOST, &format!("{HOST}:8443")), forma.clone()),
        (
            limpa.replace("acao=procedimento_controlar", "acao=infra_trocar_unidade"),
            consulta.clone(),
        ),
        (
            limpa.replace(
                "acao=procedimento_controlar",
                "acao=procedimento_controlar&acao=x",
            ),
            consulta.clone(),
        ),
        (
            limpa.replace("&infra_hash=", "&acao_ajax=1&infra_hash="),
            consulta.clone(),
        ),
        (
            limpa.replace(&hash('9', 192), &hash('9', 100)),
            consulta.clone(),
        ),
        (
            limpa.replace("&infra_unidade_atual=110000002", ""),
            forma.clone(),
        ),
        (
            limpa.replace(
                "infra_sistema",
                "infra_sistema=1&infra_unidade_atual=110000002&x",
            ),
            consulta.clone(),
        ),
        (
            limpa.replace("infra_sistema=100000100", "infra_sistema=1;2"),
            consulta,
        ),
        (limpa.replace("acao=", "ação="), forma.clone()),
        (
            limpa.replace("procedimento_controlar", "procedimento_controlar#x"),
            forma.clone(),
        ),
        (
            "ftp://sei.exemplo.gov.br/sei/controlador.php?acao=procedimento_controlar".to_owned(),
            forma,
        ),
    ] {
        assert_eq!(
            url_do_salto(&ruim, &post, ID_B).map(|_| ()),
            Err(esperado),
            "{ruim}"
        );
    }
}

#[test]
fn hash_do_salto_tem_64_ou_192_hexadecimais() {
    assert!(hash_do_salto(&hash('a', 64)));
    assert!(hash_do_salto(&hash('F', 192)));
    for n in [0, 63, 65, 128, 191, 193] {
        assert!(!hash_do_salto(&hash('a', n)), "{n}");
    }
    assert!(!hash_do_salto(&hash('g', 64)));
}
