//! Fixtures SINTÉTICAS com a estrutura observada na `UnB` 4.0.9
//! (04/11/2026): a barra do nó `PROCESSO` com os links assinados e a tela
//! "Alterar Processo" (`form#frmProcedimentoCadastro`). Hosts, hashes, ids,
//! nomes, assuntos e datas inventados; nenhum valor servido.
use crate::acao_do_sei::AcaoDoSei;
use std::fmt::Write as _;

pub(crate) const HASH_PROPRIEDADES: &str =
    "3333333333333333333333333333333333333333333333333333333333333333";
const HASH_OUTRO: &str = "4444444444444444444444444444444444444444444444444444444444444444";
const CONTEXTO: &str = "infra_sistema=100000100&infra_unidade_atual=110000001";

/// O link relativo da tela, como a barra o publica (`&` cru).
pub(crate) fn link_das_propriedades(acao: AcaoDoSei, id: &str) -> String {
    format!(
        "controlador.php?acao={acao}&acao_origem=arvore_visualizar&acao_retorno=arvore_visualizar&id_procedimento={id}&arvore=1&{CONTEXTO}&infra_hash={HASH_PROPRIEDADES}",
        acao = acao.como_na_url()
    )
}

/// Um `<a>` da barra com o ícone, na forma servida.
pub(crate) fn acao_da_barra(href: &str) -> String {
    format!(
        "<a href=\"{href}\" tabindex=\"451\" ><img  src=\"svg/icone.svg?11\" alt=\"Acao\" title=\"Acao\"/></a>"
    )
}

/// Conteúdo da barra do processo: ações vizinhas e as `acoes` pedidas.
pub(crate) fn barra(acoes: &[String]) -> String {
    let mut texto = acao_da_barra(&format!(
        "controlador.php?acao=documento_escolher_tipo&acao_origem=arvore_visualizar&id_procedimento=10&arvore=1&{CONTEXTO}&infra_hash={HASH_OUTRO}"
    ));
    for acao in acoes {
        texto.push_str(acao);
    }
    texto.push_str("<a href=\"#\" onclick=\"concluirProcesso();\" tabindex=\"451\" ><img  src=\"svg/x.svg\" alt=\"Concluir\" title=\"Concluir\"/></a>");
    texto.push_str(&acao_da_barra(&format!(
        "controlador.php?acao=procedimento_consultar_historico&acao_origem=arvore_visualizar&id_procedimento=10&arvore=1&{CONTEXTO}&infra_hash={HASH_OUTRO}"
    )));
    texto
}

/// Troca a barra vazia do nó `i` de uma árvore sintética por `conteudo`,
/// escapado como literal JS de aspas simples.
pub(crate) fn com_barra(arvore: &str, i: usize, conteudo: &str) -> String {
    let literal = conteudo.replace('\\', "\\\\").replace('\'', "\\'");
    arvore.replacen(
        &format!("Nos[{i}].acoes = '';"),
        &format!("Nos[{i}].acoes = '{literal}';"),
        1,
    )
}

/// Tela "Alterar Processo" na forma observada. Tipo marcado (id
/// `100000002`), especificação, um assunto, dois interessados, observação,
/// `optPublico` marcado, hipótese legal sem opções, listas ocultas vazias.
pub(crate) fn tela_das_propriedades(nup: &str, id: &str) -> String {
    let mut html = String::from(
        "<!DOCTYPE html><html lang=\"pt-br\"><head><meta charset=\"iso-8859-1\" />\
         <title>SEI - Alterar Processo</title>\n<style>\n#selAssuntos {position:absolute;}\n</style>\n\
         <script type=\"text/javascript\">\nfunction alterarNivelAcesso(){\n  infraSelectSelecionarItem('selHipoteseLegal','null');\n}\n\
         objLupaAssuntos = new infraLupaSelect('selAssuntos','hdnAssuntos','controlador.php?acao=assunto_selecionar');\n\
         if (document.getElementById('hdnAssuntos').value=='') {}\n</script></head><body>",
    );
    let _ = write!(
        html,
        "<form id=\"frmProcedimentoCadastro\" method=\"post\" onsubmit=\"return OnSubmitForm();\" action=\"controlador.php?acao=procedimento_alterar&acao_origem=procedimento_alterar&arvore=1&id_procedimento={id}&{CONTEXTO}&infra_hash={HASH_OUTRO}\" style=\"display:inline;\">\n\
         <input type=\"hidden\" id=\"hdnInfraTipoPagina\" name=\"hdnInfraTipoPagina\" value=\"1\" />\n\
         <button tabindex=\"451\" type=\"button\" onclick=\"confirmarDados()\" accesskey=\"S\" name=\"btnSalvar\" id=\"btnSalvar\" value=\"Salvar\" class=\"infraButton\">Salvar</button>\n\
         <div id=\"divTipoProcedimento\" class=\"infraAreaDados\"><label id=\"lblTipoProcedimento\" for=\"selTipoProcedimento\" class=\"infraLabelObrigatorio\">Tipo do Processo:</label>\n\
         <select id=\"selTipoProcedimento\" name=\"selTipoProcedimento\" onchange=\"trocarTipoProcedimento();\" class=\"infraSelect\" tabindex=\"501\">\n\
         <option value=\"null\">&nbsp;</option>\n<option value=\"100000002\" selected=\"selected\">Tipo Sintetico: Teste</option>\n<option value=\"100000003\">Outro Tipo Sintetico</option>\n</select></div>\n\
         <div id=\"divDescricao\" class=\"infraAreaDados\"><label id=\"lblDescricao\" for=\"txtDescricao\" class=\"infraLabelOpcional\">Especifica\u{e7}\u{e3}o:</label>\n\
         <input type=\"text\" id=\"txtDescricao\" name=\"txtDescricao\" onkeypress=\"return infraMascaraTexto(this,event,100);\" maxlength=\"100\" class=\"infraText\" value=\"Especifica\u{e7}\u{e3}o sint\u{e9}tica &amp; teste\" tabindex=\"502\" /></div>\n\
         <div id=\"divAssuntos\" class=\"infraAreaDados\"><input type=\"text\" id=\"txtAssunto\" name=\"txtAssunto\" class=\"infraText\" tabindex=\"503\" />\n\
         <input type=\"hidden\" id=\"hdnIdAssunto\" name=\"hdnIdAssunto\" class=\"infraText\" value=\"\" />\n\
         <select id=\"selAssuntos\" name=\"selAssuntos\" class=\"infraSelect\" multiple=\"multiple\" tabindex=\"504\" >\n<option value=\"2001\">000.01 - Assunto sint\u{e9}tico um</option>\n</select></div>\n\
         <div id=\"divInteressados\" class=\"infraAreaDados\"><input type=\"text\" id=\"txtInteressadoProcedimento\" name=\"txtInteressadoProcedimento\" class=\"infraText\" tabindex=\"507\" />\n\
         <input type=\"hidden\" id=\"hdnIdInteressadoProcedimento\" name=\"hdnIdInteressadoProcedimento\" class=\"infraText\" value=\"\" />\n\
         <select id=\"selInteressadosProcedimento\" name=\"selInteressadosProcedimento\" class=\"infraSelect\" multiple=\"multiple\" tabindex=\"510\"> \n\
         <option value=\"301\">Pessoa Sintetica A</option>\n<option value=\"302\">Entidade Sintetica B</option>\n</select></div>\n\
         <div id=\"divInteressadoUsuario\" class=\"infraAreaDados\"><input type=\"text\" id=\"txtInteressadoUsuario\" name=\"txtInteressadoUsuario\" class=\"infraText\" tabindex=\"514\" />\n\
         <input type=\"hidden\" id=\"hdnIdInteressadoUsuario\" name=\"hdnIdInteressadoUsuario\" class=\"infraText\" value=\"\" /></div>\n\
         <div id=\"divObservacoes\" class=\"infraAreaDados\"><textarea id=\"txaObservacoes\" name=\"txaObservacoes\" class=\"infraTextarea\" rows=\"2\" onkeypress=\"return infraLimitarTexto(this,event,1000);\" tabindex=\"517\" >\nObserva\u{e7}\u{e3}o sint\u{e9}tica</textarea></div>\n\
         <div id=\"divNivelAcesso\" class=\"infraAreaDados\"><fieldset id=\"fldNivelAcesso\" class=\"infraFieldset\">\n\
         <div id=\"divOptSigiloso\" class=\"infraDivRadio\">\n  <input disabled=\"disabled\" type=\"radio\" name=\"rdoNivelAcesso\" id=\"optSigiloso\" onchange=\"alterarNivelAcesso()\" value=\"2\"  class=\"infraRadio\" tabindex=\"1000\" />\n\
         <span disabled=\"disabled\" id=\"spnSigiloso\"><label id=\"lblSigiloso\" for=\"optSigiloso\" class=\"infraLabelRadio\">Sigiloso</label></span>\n\
         <select id=\"selGrauSigilo\" name=\"selGrauSigilo\" class=\"infraSelect\">\n<option value=\"null\" selected=\"selected\">&nbsp;</option>\n<option value=\"S\">Secreto</option>\n</select></div>\n\
         <div id=\"divOptRestrito\" class=\"infraDivRadio\">\n  <input  type=\"radio\" name=\"rdoNivelAcesso\" id=\"optRestrito\" onchange=\"alterarNivelAcesso()\" value=\"1\"  class=\"infraRadio\" tabindex=\"1000\" />\n\
         <span  id=\"spnRestrito\"><label id=\"lblRestrito\" for=\"optRestrito\" class=\"infraLabelRadio\">Restrito</label></span></div>\n\
         <div id=\"divOptPublico\" class=\"infraDivRadio\">\n  <input  type=\"radio\" name=\"rdoNivelAcesso\" id=\"optPublico\" onchange=\"alterarNivelAcesso()\" value=\"0\" checked=\"checked\" class=\"infraRadio\" tabindex=\"1000\" />\n\
         <span  id=\"spnPublico\"><label id=\"lblPublico\" for=\"optPublico\" class=\"infraLabelRadio\">P\u{fa}blico</label></span></div>\n\
         <label id=\"lblHipoteseLegal\" for=\"selHipoteseLegal\" class=\"infraLabelObrigatorio\">Hip\u{f3}tese Legal:</label>\n\
         <select id=\"selHipoteseLegal\" name=\"selHipoteseLegal\" class=\"infraSelect\">\n</select>\n</fieldset></div>\n\
         <input type=\"hidden\" id=\"hdnFlagProcedimentoCadastro\" name=\"hdnFlagProcedimentoCadastro\" value=\"1\"/>\n\
         <input type=\"hidden\" id=\"hdnIdTipoProcedimento\" name=\"hdnIdTipoProcedimento\" value=\"100000002\" />\n\
         <input type=\"hidden\" id=\"hdnNomeTipoProcedimento\" name=\"hdnNomeTipoProcedimento\" value=\"Tipo Sintetico: Teste\" />\n\
         <input type=\"hidden\" id=\"hdnAssuntos\" name=\"hdnAssuntos\" value=\"\" />\n\
         <input type=\"hidden\" id=\"hdnInteressadosProcedimento\" name=\"hdnInteressadosProcedimento\" value=\"\" />\n\
         <input type=\"hidden\" id=\"hdnIdProcedimento\" name=\"hdnIdProcedimento\" value=\"{id}\" />\n\
         <input type=\"hidden\" id=\"hdnProtocoloFormatado\" name=\"hdnProtocoloFormatado\" value=\"{nup}\" />\n\
         <input type=\"hidden\" id=\"hdnStaNivelAcessoGlobal\" name=\"hdnStaNivelAcessoGlobal\" value=\"0\" />\n\
         <input type=\"hidden\" id=\"hdnSinIndividual\" name=\"hdnSinIndividual\" value=\"N\" />\n\
         <input type=\"hidden\" id=\"hdnIdHipoteseLegalSugestao\" name=\"hdnIdHipoteseLegalSugestao\" value=\"\" />\n\
         <input type=\"hidden\" id=\"hdnDtaGeracao\" name=\"hdnDtaGeracao\" value=\"01/02/2026\" />\n\
         <input type=\"hidden\" id=\"hdnContatoObject\" name=\"hdnContatoObject\" value=\"\" />\n\
         <input type=\"hidden\" id=\"hdnContatoIdentificador\" name=\"hdnContatoIdentificador\" value=\"\" />\n\
         <input type=\"hidden\" id=\"hdnAssuntoIdentificador\" name=\"hdnAssuntoIdentificador\" value=\"\" />\n\
         <button tabindex=\"10000\" type=\"button\" onclick=\"confirmarDados()\" accesskey=\"S\" name=\"btnSalvar\" id=\"btnSalvar\" value=\"Salvar\" class=\"infraButton\">Salvar</button>\n\
         </form>\n<input type=\"hidden\" id=\"hdnInfraPrefixoCookie\" name=\"hdnInfraPrefixoCookie\" value=\"Sei_Prefixo\" />\n</body></html>"
    );
    html
}

/// A tela com o nível restrito e uma hipótese legal marcada.
pub(crate) fn tela_restrita(nup: &str, id: &str) -> String {
    tela_das_propriedades(nup, id)
        .replace(
            "value=\"0\" checked=\"checked\" class",
            "value=\"0\" class",
        )
        .replace(
            "value=\"1\"  class=\"infraRadio\"",
            "value=\"1\" checked=\"checked\" class=\"infraRadio\"",
        )
        .replace(
            "<select id=\"selHipoteseLegal\" name=\"selHipoteseLegal\" class=\"infraSelect\">\n</select>",
            "<select id=\"selHipoteseLegal\" name=\"selHipoteseLegal\" class=\"infraSelect\">\n<option value=\"null\">&nbsp;</option>\n<option value=\"7\" selected=\"selected\">Hipotese Sintetica (Art. 1)</option>\n</select>",
        )
}
