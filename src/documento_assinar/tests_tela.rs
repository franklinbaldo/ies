//! RED/GREEN da tela "Assinatura de Documento" e da resposta do POST, na
//! forma das sete instalações (`documento-escritas-*`). HTML sintético.
use super::tela::{TelaDeAssinatura, assinatura_realizada, conferir_tela_de_assinatura};
use super::tests_apoio::*;

#[test]
fn tela_conferida_publica_assinante_orgao_e_cargos() {
    assert_eq!(
        conferir_tela_de_assinatura(&tela_padrao(), ID_DOC),
        Ok(TelaDeAssinatura {
            assinante: ASSINANTE.into(),
            orgao: "0".into(),
            cargos: vec![CARGO.into(), "Presidente".into()],
            cargo_da_tela: Some(CARGO.into()),
        })
    );
}

#[test]
fn cargo_unico_e_o_da_tela_mesmo_sem_selected() {
    let tela = tela_de_assinatura(ID_DOC, &["Coordenador"], None, false);
    let lida = conferir_tela_de_assinatura(&tela, ID_DOC).unwrap();
    assert_eq!(lida.cargo_da_tela.as_deref(), Some("Coordenador"));
    let tela = tela_de_assinatura(ID_DOC, &["A", "B"], None, false);
    assert_eq!(
        conferir_tela_de_assinatura(&tela, ID_DOC)
            .unwrap()
            .cargo_da_tela,
        None
    );
}

#[test]
fn recusas_da_tela() {
    let padrao = tela_padrao();
    let casos = [
        (padrao.replace("frmAssinaturas\" method", "frmOutro\" method"), super::erro::ErroTelaDeAssinatura::FormularioDeAssinaturaAusente(
            crate::formulario_web::ErroSubmissaoFormulario::FormularioAusenteNaTela
        )),
        (padrao.clone(), super::erro::ErroTelaDeAssinatura::TelaDeAssinaturaDeOutroDocumento),
        (
            padrao.replace("<input type=\"hidden\" id=\"hdnIdBlocos\"", "<input type=\"text\" name=\"txtExtra\" value=\"\" /><input type=\"hidden\" id=\"hdnIdBlocos\""),
            super::erro::ErroTelaDeAssinatura::CampoNaoPrevistoNaTelaDeAssinatura,
        ),
        (
            padrao.replace(&format!("value=\"{ASSINANTE}\""), "value=\" \""),
            super::erro::ErroTelaDeAssinatura::AssinanteNaoPublicadoNaTela,
        ),
        (
            padrao.replace("<option value=\"0\" selected=\"selected\">", "<option value=\"0\">"),
            super::erro::ErroTelaDeAssinatura::OrgaoDoAssinanteNaoSelecionado,
        ),
        (
            padrao.replace("type=\"password\" id=\"pwdSenha\"", "type=\"hidden\" id=\"pwdSenha\""),
            super::erro::ErroTelaDeAssinatura::CampoNaoPrevistoNaTelaDeAssinatura,
        ),
        (
            padrao.replace("<input type=\"password\" id=\"pwdSenha\" name=\"pwdSenha\"", "<input type=\"password\" id=\"pwdSenha\""),
            super::erro::ErroTelaDeAssinatura::CampoDeSenhaAusenteNaTela,
        ),
        (
            padrao.replace("type=\"button\" accesskey=\"A\"", "type=\"submit\" accesskey=\"A\""),
            super::erro::ErroTelaDeAssinatura::CampoNaoPrevistoNaTelaDeAssinatura,
        ),
    ];
    for (i, (html, erro)) in casos.iter().enumerate() {
        let id = if i == 1 { "22" } else { ID_DOC };
        assert_eq!(
            conferir_tela_de_assinatura(html, id),
            Err(erro.clone()),
            "caso {i}"
        );
    }
}

#[test]
fn so_a_resposta_com_finalizar_e_aceita() {
    assert!(assinatura_realizada(&resposta_aceita()));
    assert!(!assinatura_realizada(&tela_padrao()));
    assert!(!assinatura_realizada("<html><body>Erro</body></html>"));
}
