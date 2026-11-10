//! RED/GREEN da tela "Marcadores do Processo": identidade (título e
//! `form#frmGerenciarMarcador`), linhas da `tblMarcadores` como lista,
//! legenda contra linhas e recusas. Telas SINTÉTICAS (`tests_apoio`).
use super::tests_apoio::{Linha, dois, legenda, linha, tabela, tela_com, tela_dos_marcadores};
use super::{ErroMarcadores as E, MarcadorDoProcesso, TelaDosMarcadores, ler_marcadores};
use crate::arvore_processo::tests::NUP;
use crate::erro_html::ErroHtml;

const ID: &str = "10";

fn lidos(html: &str) -> Vec<MarcadorDoProcesso> {
    match ler_marcadores(html, NUP, ID) {
        Ok(TelaDosMarcadores::Lidos(lidos)) => {
            assert_eq!(lidos.total_declarado, lidos.itens.len());
            lidos.itens
        }
        outro => panic!("não lidos: {outro:?}"),
    }
}

fn erro(html: &str) -> E {
    ler_marcadores(html, NUP, ID).unwrap_err()
}

#[test]
fn processo_com_dois_marcadores_publica_os_dois_em_ordem() {
    let itens = lidos(&tela_dos_marcadores(NUP, ID));
    assert_eq!(
        itens,
        [
            MarcadorDoProcesso {
                id: "501".into(),
                nome: "Revis\u{e3}o".into(),
                cor: "azul".into(),
                texto: None,
                usuario: Some("pessoa1".into()),
                data_hora: "03/11/2026 10:15".into(),
            },
            MarcadorDoProcesso {
                id: "502".into(),
                nome: "Pendente".into(),
                cor: "amarelo".into(),
                texto: Some("Texto sint\u{e9}tico & anota\u{e7}\u{e3}o".into()),
                usuario: Some("pessoa2".into()),
                data_hora: "02/11/2026 09:00".into(),
            },
        ]
    );
}

#[test]
fn um_marcador_com_legenda_no_singular_e_usuario_vazio() {
    let mut um = dois();
    um[1].usuario = "";
    let html = tela_com(NUP, ID, &tabela(&legenda(1), &um[1..]));
    let itens = lidos(&html);
    assert_eq!(itens.len(), 1);
    assert_eq!(itens[0].usuario, None);
}

#[test]
fn legenda_maior_que_as_linhas_publica_o_total_sem_alegar_completude() {
    let html = tela_com(NUP, ID, &tabela(&legenda(5), &dois()));
    let Ok(TelaDosMarcadores::Lidos(lidos)) = ler_marcadores(&html, NUP, ID) else {
        panic!("não lidos");
    };
    assert_eq!(lidos.total_declarado, 5);
    assert_eq!(lidos.itens.len(), 2);
}

#[test]
fn linhas_alem_da_legenda_sao_recusa() {
    let html = tela_com(NUP, ID, &tabela(&legenda(1), &dois()));
    assert_eq!(erro(&html), E::MarcadoresAlemDoTotalDeclarado);
}

#[test]
fn tela_sem_tabela_e_nao_observada_e_nao_lista_vazia() {
    let html = tela_com(NUP, ID, "");
    assert_eq!(
        ler_marcadores(&html, NUP, ID),
        Ok(TelaDosMarcadores::SemTabela)
    );
}

#[test]
fn identidade_vem_antes_de_qualquer_linha() {
    let html = tela_dos_marcadores(NUP, ID);
    assert_eq!(
        erro(&tela_dos_marcadores("00009.000009/2026-09", ID)),
        E::NumeroDosMarcadoresDiferenteDoSolicitado
    );
    assert_eq!(
        erro(&tela_dos_marcadores(NUP, "99")),
        E::IdDosMarcadoresDiferenteDoAberto
    );
    assert_eq!(
        erro(&html.replace(
            "SEI - Marcadores do Processo",
            "SEI - Controle de Processos"
        )),
        E::TelaDosMarcadoresAusente
    );
    assert_eq!(
        erro("<html><body>Link sem assinatura</body></html>"),
        E::TelaDosMarcadoresAusente
    );
    assert_eq!(
        erro(&html.replace("id=\"frmGerenciarMarcador\"", "id=\"frmOutro\"")),
        E::TelaDosMarcadoresAusente
    );
    for erro in [
        E::TelaDosMarcadoresAusente,
        E::NumeroDosMarcadoresDiferenteDoSolicitado,
        E::IdDosMarcadoresDiferenteDoAberto,
    ] {
        assert!(erro.link_envelhecido(), "{erro:?}");
    }
}

#[test]
fn legenda_ou_cabecalho_fora_da_forma_observada_sao_recusa() {
    let html = tela_dos_marcadores(NUP, ID);
    for legenda in [
        "Marcadores:",
        "Marcadores (dois registros):",
        "Marcadores (2 registros - 1 a 2):",
        "Lista (2 registros):",
    ] {
        assert_eq!(
            erro(&html.replace("Marcadores (2 registros):", legenda)),
            E::Html(ErroHtml::valor(
                "legenda_dos_marcadores_fora_da_forma_observada"
            )),
            "{legenda}"
        );
    }
    assert_eq!(
        erro(&html.replace(
            "<th class=\"infraTh\">Texto</th>",
            "<th class=\"infraTh\">Nota</th>"
        )),
        E::Html(ErroHtml::valor("cabecalho_dos_marcadores_nao_reconhecido"))
    );
    let duas = html.replace(
        "</div>\n<input type=\"hidden\" id=\"hdnInfraNroItens\"",
        &format!(
            "{}</div>\n<input type=\"hidden\" id=\"hdnInfraNroItens\"",
            tabela(&legenda(0), &[])
        ),
    );
    assert_eq!(
        erro(&duas),
        E::Html(ErroHtml::repetido("tabelas_dos_marcadores_repetidas"))
    );
}

#[test]
fn linha_fora_da_forma_observada_e_recusa() {
    let base = linha(0, &dois()[1]);
    let ruins = [
        base.replace("value=\"502\"", "value=\"27x\""),
        base.replace("tdAndamentoMarcador502", "tdAndamentoMarcador280"),
        base.replace("lnkInfraID-502", "lnkInfraID-280"),
        base.replace("svg/marcador_amarelo.svg", "svg/outro.svg"),
        base.replace("title=\"Pendente\" alt", "title=\"Outro\" alt"),
        base.replace("&nbsp;Pendente</td>", "&nbsp;Outro</td>"),
        base.replace("02/11/2026 09:00", "ontem"),
        base.replace(
            "<td align=\"center\" valign=\"center\">02/11/2026 09:00</td>",
            "",
        ),
    ];
    for ruim in ruins {
        let tabela = tabela(&legenda(1), &[]).replace("</table>", &format!("{ruim}</table>"));
        let html = tela_com(NUP, ID, &tabela);
        assert_eq!(
            erro(&html),
            E::Html(ErroHtml::valor("linha_do_marcador_fora_da_forma_observada")),
            "{ruim}"
        );
    }
}

#[test]
fn marcador_repetido_na_tela_e_recusa() {
    let [a, _] = dois();
    let [b, _]: [Linha<'_>; 2] = dois();
    let html = tela_com(NUP, ID, &tabela(&legenda(2), &[a, b]));
    assert_eq!(
        erro(&html),
        E::Html(ErroHtml::repetido("marcador_repetido_na_tela"))
    );
}

#[test]
fn subcodigos_estaveis() {}
