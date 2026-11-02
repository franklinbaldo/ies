//! RED/GREEN do Nível de detalhe: tela sintética modelada na estrutura da
//! tela `procedimento_configurar_detalhe` de PEN1 e GO (02/11/2026).
use super::*;

/// Tela com as 12 opções; `marcadas` são os campos com `checked`.
pub(crate) fn tela(marcadas: &[&str]) -> String {
    use std::fmt::Write as _;
    let mut opcoes = String::new();
    for coluna in ColunaDoDetalhe::TODAS {
        let campo = coluna.campo();
        let checked = if marcadas.contains(&campo) {
            " checked=\"checked\""
        } else {
            ""
        };
        let sufixo = campo.strip_prefix("chk").unwrap();
        write!(
            opcoes,
            "<div><input type=\"checkbox\" id=\"{campo}\" name=\"{campo}\" class=\"infraCheckbox\" \
             onchange=\"validarNumeroCriterios(this)\"{checked} tabindex=\"501\" />\
             <label id=\"lbl{sufixo}\" for=\"{campo}\" class=\"infraLabelCheckbox\">{}</label></div>",
            coluna.rotulo()
        )
        .unwrap();
    }
    format!(
        "<html><head><title>SEI - Configurar Detalhe</title></head><body>\
         <form id=\"frmAcessoExternoCancelar\" method=\"post\" onsubmit=\"return OnSubmitForm();\" \
         action=\"controlador.php?acao=procedimento_configurar_detalhe&amp;acao_origem=procedimento_configurar_detalhe&amp;infra_hash=abc\">\
         <input type=\"hidden\" id=\"hdnInfraTipoPagina\" name=\"hdnInfraTipoPagina\" value=\"2\"/>\
         <button type=\"submit\" accesskey=\"S\" name=\"sbmSalvar\" value=\"Salvar\" class=\"infraButton\">Salvar</button>\
         {opcoes}</form></body></html>"
    )
}

#[test]
fn modelo_tem_doze_opcoes_com_campos_rotulos_e_chaves_unicos() {
    use std::collections::HashSet;
    let todas = ColunaDoDetalhe::TODAS;
    for f in [
        ColunaDoDetalhe::campo,
        ColunaDoDetalhe::rotulo,
        ColunaDoDetalhe::chave,
    ] {
        assert_eq!(
            todas.iter().map(|c| f(*c)).collect::<HashSet<_>>().len(),
            12
        );
    }
    assert_eq!(
        todas.iter().filter(|c| c.tem_cabecalho_de_texto()).count(),
        9
    );
    for coluna in todas {
        assert_eq!(ColunaDoDetalhe::pelo_campo(coluna.campo()), Some(coluna));
        let (CabecalhoDaColuna::Texto(rotulo) | CabecalhoDaColuna::Icone(rotulo)) =
            coluna.cabecalho();
        assert_eq!(ColunaDoDetalhe::pelo_cabecalho(rotulo), Some(coluna));
    }
    assert_eq!(
        ColunaDoDetalhe::pelo_cabecalho("Última Mov."),
        Some(ColunaDoDetalhe::UltimaMovimentacao)
    );
    assert_eq!(ColunaDoDetalhe::pelo_cabecalho("Processo"), None);
}

#[test]
fn tela_com_tres_marcadas_como_a_pen1() {
    let html = tela(&[
        "chkSinAtribuicao",
        "chkSinTipoProcesso",
        "chkSinInteressados",
    ]);
    let nivel = ler_nivel_de_detalhe(&html).unwrap();
    assert_eq!(nivel.opcoes.len(), 12);
    let marcadas: Vec<_> = nivel
        .opcoes
        .iter()
        .filter(|o| o.marcada)
        .map(|o| o.coluna.chave())
        .collect();
    assert_eq!(marcadas, ["atribuicao", "tipo", "interessados"]);
    let json = nivel.json();
    assert_eq!(json[0]["campo"], "chkSinAtribuicao");
    assert_eq!(json[0]["marcada"], true);
    assert_eq!(json[1]["marcada"], false);
    assert_eq!(json[7]["cabecalho_tipo"], "icone");
}

#[test]
fn tela_com_todas_marcadas_como_go_e_booleano_antes_dos_atributos() {
    let todas: Vec<_> = ColunaDoDetalhe::TODAS.iter().map(|c| c.campo()).collect();
    let html = crate::html_atributos::com_booleano_antes_dos_atributos(&tela(&todas));
    let nivel = ler_nivel_de_detalhe(&html).unwrap();
    assert!(nivel.opcoes.iter().all(|o| o.marcada));
}

#[test]
fn opcoes_faltando_repetidas_extras_ou_com_rotulo_trocado_sao_recusadas() {
    use ErroNivelDeDetalhe as E;
    let base = tela(&[]);
    let sem = base.replacen("name=\"chkSinMarcadores\"", "name=\"outro\"", 1);
    assert_eq!(
        ler_nivel_de_detalhe(&sem),
        Err(E::OpcaoDoDetalheAusente {
            coluna: ColunaDoDetalhe::Marcadores
        })
    );
    let repetida = base.replacen("name=\"chkSinAnotacao\"", "name=\"chkSinAtribuicao\"", 1);
    assert!(matches!(
        ler_nivel_de_detalhe(&repetida),
        Err(E::OpcaoDoDetalheRepetida { .. } | E::OpcaoDoDetalheAusente { .. })
    ));
    let extra = base.replacen(
        "</form>",
        "<input type=\"checkbox\" name=\"chkSinNovaColuna\"></form>",
        1,
    );
    assert_eq!(
        ler_nivel_de_detalhe(&extra),
        Err(E::OpcaoDoDetalheNaoPrevista)
    );
    let rotulo = base.replacen(">Prioridade</label>", ">Urgência</label>", 1);
    assert_eq!(
        ler_nivel_de_detalhe(&rotulo),
        Err(E::RotuloDaOpcaoDoDetalheDiferente {
            coluna: ColunaDoDetalhe::Prioridade
        })
    );
    let acao = base.replacen(
        "acao=procedimento_configurar_detalhe&amp;",
        "acao=outra&amp;",
        1,
    );
    assert_eq!(
        ler_nivel_de_detalhe(&acao),
        Err(E::AcaoDoFormularioDoNivelDeDetalheDiferente)
    );
    assert_eq!(
        ler_nivel_de_detalhe("<html><form id='x'></form></html>"),
        Err(E::FormularioDoNivelDeDetalheAusente)
    );
    let dois = format!("{base}{base}");
    assert_eq!(
        ler_nivel_de_detalhe(&dois),
        Err(E::FormulariosDoNivelDeDetalheRepetidos)
    );
}
