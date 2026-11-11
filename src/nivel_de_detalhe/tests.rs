//! RED/GREEN do Nível de detalhe: tela sintética modelada na estrutura da
//! tela `procedimento_configurar_detalhe` de PEN1 e GO (02/11/2026).
use super::*;
use crate::tela_html::HtmlServido;

/// Tela com as 12 opções; `marcadas` são os campos com `checked`.
pub(crate) fn tela(marcadas: &[&str]) -> String {
    tela_com(&ColunaDoDetalhe::TODAS, marcadas)
}

/// Tela que publica só `publicadas`, na ordem dada.
pub(crate) fn tela_com(publicadas: &[ColunaDoDetalhe], marcadas: &[&str]) -> String {
    use std::fmt::Write as _;
    let mut opcoes = String::new();
    for &coluna in publicadas {
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
    let html = HtmlServido::de_teste(&tela(&[
        "chkSinAtribuicao",
        "chkSinTipoProcesso",
        "chkSinInteressados",
    ]));
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
    let html = HtmlServido::de_teste(&crate::html_atributos::com_booleano_antes_dos_atributos(
        &tela(&todas),
    ));
    let nivel = ler_nivel_de_detalhe(&html).unwrap();
    assert!(nivel.opcoes.iter().all(|o| o.marcada));
}

#[test]
fn opcoes_faltando_repetidas_extras_ou_com_rotulo_trocado_sao_recusadas() {
    let base = tela(&[]);
    assert_eq!(
        ler_nivel_de_detalhe(&HtmlServido::de_teste(&tela_com(&[], &[]))),
        Err(ErroNivelDeDetalhe::NenhumaOpcaoDoDetalhePublicada)
    );
    let repetida = HtmlServido::de_teste(&base.replacen(
        "name=\"chkSinAnotacao\"",
        "name=\"chkSinAtribuicao\"",
        1,
    ));
    assert_eq!(
        ler_nivel_de_detalhe(&repetida),
        Err(ErroNivelDeDetalhe::OpcaoDoDetalheRepetida {
            coluna: ColunaDoDetalhe::Atribuicao
        })
    );
    let extra = HtmlServido::de_teste(&base.replacen(
        "</form>",
        "<input type=\"checkbox\" name=\"chkSinNovaColuna\"></form>",
        1,
    ));
    assert_eq!(
        ler_nivel_de_detalhe(&extra),
        Err(ErroNivelDeDetalhe::OpcaoDoNivelDeDetalheNaoPrevista)
    );
    let rotulo =
        HtmlServido::de_teste(&base.replacen(">Prioridade</label>", ">Urgência</label>", 1));
    assert_eq!(
        ler_nivel_de_detalhe(&rotulo),
        Err(ErroNivelDeDetalhe::RotuloDaOpcaoDoDetalheDiferente {
            coluna: ColunaDoDetalhe::Prioridade
        })
    );
    let acao = base.replacen(
        "acao=procedimento_configurar_detalhe&amp;",
        "acao=outra&amp;",
        1,
    );
    assert_eq!(
        ler_nivel_de_detalhe(&HtmlServido::de_teste(&acao)),
        Err(ErroNivelDeDetalhe::AcaoDoFormularioNaoLida(
            crate::consulta_do_controlador::ErroChaveDaConsulta::AcaoDesconhecida
        ))
    );
    let repetida = HtmlServido::de_teste(&base.replacen(
        "acao=procedimento_configurar_detalhe&amp;",
        "acao=procedimento_configurar_detalhe&amp;acao=procedimento_configurar_detalhe&amp;",
        1,
    ));
    assert_eq!(
        ler_nivel_de_detalhe(&repetida),
        Err(ErroNivelDeDetalhe::AcaoDoFormularioNaoLida(
            crate::consulta_do_controlador::ErroChaveDaConsulta::Repetida { chave: "acao" }
        ))
    );
    let solta = HtmlServido::de_teste(&base.replacen(
        "acao=procedimento_configurar_detalhe&amp;",
        "acao=procedimento_configurar_detalhe&amp;solta&amp;",
        1,
    ));
    assert_eq!(
        ler_nivel_de_detalhe(&solta),
        Err(ErroNivelDeDetalhe::ConsultaDoFormularioIlegivel(
            crate::consulta_do_controlador::ErroConsultaDoControlador::ParSemIgual
        ))
    );
    assert_eq!(
        ler_nivel_de_detalhe(&HtmlServido::de_teste("<html><form id='x'></form></html>")),
        Err(ErroNivelDeDetalhe::FormularioDoNivelDeDetalheAusente(
            crate::formulario_web::ErroSubmissaoFormulario::FormularioAusenteNaTela
        ))
    );
    let dois = HtmlServido::de_teste(&format!("{base}{base}"));
    assert_eq!(
        ler_nivel_de_detalhe(&dois),
        Err(ErroNivelDeDetalhe::FormulariosDoNivelDeDetalheRepetidos)
    );
}

/// Estrutura da tela da `UnB` 4.0.9 (ensaio live de 02/11/2026): 11 caixas,
/// sem `chkSinNivelPrioritarios`.
fn onze_da_unb() -> Vec<ColunaDoDetalhe> {
    ColunaDoDetalhe::TODAS
        .into_iter()
        .filter(|c| *c != ColunaDoDetalhe::Prioridade)
        .collect()
}

#[test]
fn tela_da_unb_sem_prioridade_publica_a_opcao_indisponivel() {
    let html = HtmlServido::de_teste(&tela_com(
        &onze_da_unb(),
        &["chkSinAtribuicao", "chkSinControlePrazo"],
    ));
    let nivel = ler_nivel_de_detalhe(&html).unwrap();
    assert_eq!(nivel.opcoes.len(), 12);
    let prioridade = nivel.opcoes[4];
    assert_eq!(prioridade.coluna, ColunaDoDetalhe::Prioridade);
    assert!(!prioridade.disponivel);
    assert!(!prioridade.marcada);
    assert_eq!(nivel.opcoes.iter().filter(|o| o.disponivel).count(), 11);
    assert_eq!(nivel.opcoes.iter().filter(|o| o.marcada).count(), 2);
    let json = nivel.json();
    assert_eq!(json[4]["disponivel"], false);
    assert_eq!(json[4]["marcada"], false);
    assert_eq!(json[0]["disponivel"], true);
    assert_eq!(json[0]["marcada"], true);
}

#[test]
fn subconjunto_ainda_recusa_caixa_desconhecida_repetida_ou_rotulo_trocado() {
    let base = tela_com(&onze_da_unb(), &[]);
    let desconhecida = HtmlServido::de_teste(&base.replacen(
        "</form>",
        "<input type=\"checkbox\" name=\"chkSinNivelUrgencia\"></form>",
        1,
    ));
    assert_eq!(
        ler_nivel_de_detalhe(&desconhecida),
        Err(ErroNivelDeDetalhe::OpcaoDoNivelDeDetalheNaoPrevista)
    );
    let repetida = HtmlServido::de_teste(&base.replacen(
        "name=\"chkSinAnotacao\"",
        "name=\"chkSinMarcadores\"",
        1,
    ));
    assert_eq!(
        ler_nivel_de_detalhe(&repetida),
        Err(ErroNivelDeDetalhe::OpcaoDoDetalheRepetida {
            coluna: ColunaDoDetalhe::Marcadores
        })
    );
    let rotulo =
        HtmlServido::de_teste(&base.replacen(">Marcadores</label>", ">Etiquetas</label>", 1));
    assert_eq!(
        ler_nivel_de_detalhe(&rotulo),
        Err(ErroNivelDeDetalhe::RotuloDaOpcaoDoDetalheDiferente {
            coluna: ColunaDoDetalhe::Marcadores
        })
    );
    // Uma só caixa publicada ainda é tela válida.
    let uma = HtmlServido::de_teste(&tela_com(&[ColunaDoDetalhe::Marcadores], &[]));
    let nivel = ler_nivel_de_detalhe(&uma).unwrap();
    assert_eq!(nivel.opcoes.iter().filter(|o| o.disponivel).count(), 1);
}
