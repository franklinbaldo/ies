//! Apoio dos testes de `processo enviar`: telas SINTÉTICAS na estrutura
//! observada (tela "Enviar Processo" do SEI 4.0 — `UnB` 4.0.9, PEN1 4.0.12 —
//! e do 4.1/5.0 — UFRJ, PEN2, GO, PEN3, PEN4 —, resposta XML do autocomplete,
//! barra com o link de enviar) sobre o roteiro de `processo concluir`.
//! Hosts, hashes, ids e siglas inventados.
use super::{
    DesfechoDoEnvio, Destino, ErroEnviar, PedidoDoEnvio, ProcessoEnviado, enviar_processo,
};
use crate::arvore_processo::tests_concluir::absoluta;
use crate::processo_concluir::tests_apoio::*;
use crate::propriedades_processo::tests_apoio::acao_da_barra;
use std::fmt::Write as _;

const CONTEXTO: &str = "infra_sistema=100000100&infra_unidade_atual=110000001";
pub(crate) const HASH_ENVIAR: &str =
    "7777777777777777777777777777777777777777777777777777777777777777";
pub(crate) const HASH_ACTION_ENVIO: &str =
    "3333333333333333333333333333333333333333333333333333333333333333";
pub(crate) const HASH_AJAX: &str =
    "2222222222222222222222222222222222222222222222222222222222222222";
pub(crate) const DESTINO: &str = "TESTE_1_1";
pub(crate) const ID_DESTINO: &str = "110000002";

/// O link relativo da tela "Enviar Processo", como a barra o publica.
pub(crate) fn link_de_enviar(id: &str) -> String {
    format!(
        "controlador.php?acao=procedimento_enviar&acao_origem=arvore_visualizar&acao_retorno=arvore_visualizar&id_procedimento={id}&arvore=1&{CONTEXTO}&infra_hash={HASH_ENVIAR}"
    )
}

/// O `<a>` da barra com o link de enviar.
pub(crate) fn pelo_envio(id: &str) -> String {
    acao_da_barra(&link_de_enviar(id))
}

pub(crate) fn enviar() -> Vec<String> {
    vec![pelo_envio(ID)]
}

pub(crate) fn action_de_enviar() -> String {
    format!(
        "controlador.php?acao=procedimento_enviar&acao_origem=procedimento_enviar&arvore=1&id_procedimento={ID}&{CONTEXTO}&infra_hash={HASH_ACTION_ENVIO}"
    )
}

pub(crate) fn autocompletar() -> String {
    format!(
        "controlador_ajax.php?acao_ajax=unidade_auto_completar_envio_processo&{CONTEXTO}&infra_hash={HASH_AJAX}"
    )
}

/// Como a tela é servida em cada família de versões.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Versao {
    /// 4.0: `rdoPrazo`, `txtPrazo`, `txtDias`; sem `unidade_atual`.
    V40,
    /// 4.1 e 5.0: retorno e reabertura programados; `&unidade_atual=0`.
    V41,
}

fn preparar(versao: Versao) -> &'static str {
    match versao {
        Versao::V40 => {
            "    return 'palavras_pesquisa='+document.getElementById('txtUnidade').value+'&id_orgao='+document.getElementById('selOrgao').value;"
        }
        Versao::V41 => {
            "    return 'palavras_pesquisa='+document.getElementById('txtUnidade').value+'&id_orgao='+document.getElementById('selOrgao').value+'&unidade_atual=0';"
        }
    }
}

fn prazos(versao: Versao) -> &'static str {
    match versao {
        Versao::V40 => {
            "<input type=\"radio\" name=\"rdoPrazo\" id=\"optDataCerta\" onclick=\"configurarPrazo();\"  value=\"1\" class=\"infraRadio\"/>\n\
             <input type=\"text\" id=\"txtPrazo\" name=\"txtPrazo\" class=\"infraText\" value=\"\" tabindex=\"512\" />\n\
             <input type=\"radio\" name=\"rdoPrazo\" id=\"optDias\" onclick=\"configurarPrazo();\"  value=\"2\" class=\"infraRadio\"/>\n\
             <input type=\"text\" id=\"txtDias\" name=\"txtDias\" class=\"infraText\" value=\"\" maxlength=\"3\" tabindex=\"515\" />\n\
             <input type=\"checkbox\" id=\"chkSinDiasUteis\" name=\"chkSinDiasUteis\" class=\"infraCheckbox\"  tabindex=\"516\" />\n"
        }
        Versao::V41 => {
            "<input type=\"radio\" name=\"rdoPrazoRetornoProgramado\" id=\"optDataCertaRetornoProgramado\" value=\"1\" class=\"infraRadio\"/>\n\
             <input type=\"text\" id=\"txtPrazoRetornoProgramado\" name=\"txtPrazoRetornoProgramado\" class=\"infraText\" value=\"\" tabindex=\"513\" />\n\
             <input type=\"radio\" name=\"rdoPrazoRetornoProgramado\" id=\"optDiasRetornoProgramado\" value=\"2\" class=\"infraRadio\"/>\n\
             <input type=\"text\" id=\"txtDiasRetornoProgramado\" name=\"txtDiasRetornoProgramado\" class=\"infraText\" value=\"\" maxlength=\"3\" tabindex=\"516\" />\n\
             <input type=\"checkbox\" id=\"chkSinDiasUteisRetornoProgramado\" name=\"chkSinDiasUteisRetornoProgramado\" class=\"infraCheckbox\"  tabindex=\"517\" />\n\
             <input type=\"radio\" name=\"rdoPrazoReaberturaProgramada\" id=\"optDataCertaReaberturaProgramada\" value=\"1\" class=\"infraRadio\"/>\n\
             <input type=\"text\" id=\"txtPrazoReaberturaProgramada\" name=\"txtPrazoReaberturaProgramada\" class=\"infraText\" value=\"\" tabindex=\"519\" />\n\
             <input type=\"radio\" name=\"rdoPrazoReaberturaProgramada\" id=\"optDiasReaberturaProgramada\" value=\"2\" class=\"infraRadio\"/>\n\
             <input type=\"text\" id=\"txtDiasReaberturaProgramada\" name=\"txtDiasReaberturaProgramada\" class=\"infraText\" value=\"\" maxlength=\"3\" tabindex=\"522\" />\n\
             <input type=\"checkbox\" id=\"chkSinDiasUteisReaberturaProgramada\" name=\"chkSinDiasUteisReaberturaProgramada\" class=\"infraCheckbox\"  tabindex=\"523\" />\n"
        }
    }
}

/// Tela "Enviar Processo" na forma observada, do processo `id`.
pub(crate) fn tela_de_envio(versao: Versao, id: &str) -> String {
    format!(
        "<html><head><title>SEI - Enviar Processo</title>\n<script type=\"text/javascript\">\n\
         function inicializar(){{\n\
         \x20 objLupaUnidades = new infraLupaSelect('selUnidades','hdnUnidades','controlador.php?acao=unidade_selecionar_envio_processo&tipo_selecao=2&id_object=objLupaUnidades&{CONTEXTO}&infra_hash={HASH_AJAX}');\n\
         \x20 objAutoCompletarUnidade = new infraAjaxAutoCompletar('hdnIdUnidade','txtUnidade','{ajax}');\n\
         \x20 //objAutoCompletarUnidade.tamanhoMinimo = 3;\n\
         \x20 objAutoCompletarUnidade.limparCampo = true;\n\
         \x20 objAutoCompletarUnidade.prepararExecucao = function(){{\n\
         {preparar}\n\
         \x20 }};\n\
         }}\n</script></head><body onload=\"inicializar();\">\n\
         <div id=\"divInfraBarraLocalizacao\" class=\"infraBarraLocalizacao\">Enviar Processo</div>\n\
         <form id=\"frmAtividadeListar\" method=\"post\" onsubmit=\"return OnSubmitForm();\" action=\"{action}\">\n\
         <input type=\"hidden\" id=\"hdnInfraTipoPagina\" name=\"hdnInfraTipoPagina\" value=\"2\" />\n\
         <button tabindex=\"451\" type=\"submit\" accesskey=\"E\" name=\"sbmEnviar\" id=\"sbmEnviar\" value=\"Enviar\" class=\"infraButton\"><span class=\"infraTeclaAtalho\">E</span>nviar</button>\n\
         <select id=\"selProcedimentos\" name=\"selProcedimentos\" multiple=\"multiple\" class=\"infraSelect\" tabindex=\"501\">\n\
         <option value=\"{id}\">{NUP} - Tipo Sintetico</option>\n</select>\n\
         <select id=\"selOrgao\" name=\"selOrgao\" class=\"infraSelect\" tabindex=\"502\">\n\
         <option value=\"null\" selected=\"selected\">&nbsp;</option>\n\
         <option value=\"\" selected=\"selected\">Todos</option>\n\
         <option value=\"0\">ORGAO</option>\n</select>\n\
         <input type=\"text\" id=\"txtUnidade\" name=\"txtUnidade\" class=\"infraText\" tabindex=\"503\" />\n\
         <a id=\"ancUnidadesTramitacao\" href=\"javascript:void(0);\" onclick=\"selecionarTramitacao();\" class=\"ancoraPadraoPreta\">Mostrar unidades por onde tramitou</a>\
         <input type=\"hidden\" id=\"hdnIdUnidade\" name=\"hdnIdUnidade\" class=\"infraText\" value=\"\" />\n\
         <select id=\"selUnidades\" name=\"selUnidades\" multiple=\"multiple\" class=\"infraSelect\" tabindex=\"504\">\n</select>\n\
         <input type=\"checkbox\" id=\"chkSinManterAberto\" name=\"chkSinManterAberto\" class=\"infraCheckbox\"  tabindex=\"508\" />\n\
         <label id=\"lblSinManterAberto\" for=\"chkSinManterAberto\" class=\"infraLabelCheckbox\">Manter processo aberto na unidade atual</label>\n\
         <input type=\"checkbox\" id=\"chkSinRemoverAnotacoes\" name=\"chkSinRemoverAnotacoes\" class=\"infraCheckbox\"  tabindex=\"509\" />\n\
         <input type=\"checkbox\" id=\"chkSinEnviarEmailNotificacao\" name=\"chkSinEnviarEmailNotificacao\" class=\"infraCheckbox\"  tabindex=\"510\" />\n\
         {prazos}\
         <input type=\"hidden\" id=\"hdnIdProtocolos\" name=\"hdnIdProtocolos\" value=\"{id}\" />\n\
         <input type=\"hidden\" id=\"hdnIdAtividades\" name=\"hdnIdAtividades\" value=\"42468\" />\n\
         <input type=\"hidden\" id=\"hdnUnidades\" name=\"hdnUnidades\" value=\"\" />\n\
         </form>\n<input type=\"hidden\" id=\"hdnInfraPrefixoCookie\" name=\"hdnInfraPrefixoCookie\" value=\"SEI_X\" />\n</body></html>",
        ajax = autocompletar(),
        preparar = preparar(versao),
        action = action_de_enviar(),
        prazos = prazos(versao),
    )
}

/// Resposta do autocomplete com `itens` (`(id, descricao)`).
pub(crate) fn resposta_xml(itens: &[(&str, &str)]) -> String {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"iso-8859-1\"?>\n<itens>");
    for (id, descricao) in itens {
        let _ = write!(xml, "<item id=\"{id}\" descricao=\"{descricao}\"></item>");
    }
    xml.push_str("</itens>");
    xml
}

pub(crate) fn descricao_do_destino() -> String {
    format!("{DESTINO} - Unidade de Testes 1.1")
}

/// A resposta observada no PEN: um item, o destino.
pub(crate) fn um_destino() -> String {
    resposta_xml(&[(ID_DESTINO, &descricao_do_destino())])
}

pub(crate) fn destino() -> Destino {
    Destino {
        sigla: DESTINO.into(),
        descricao: descricao_do_destino(),
        id_unidade: ID_DESTINO.into(),
    }
}

/// O corpo que o navegador envia para a consulta.
pub(crate) fn corpo_da_consulta(versao: Versao) -> String {
    match versao {
        Versao::V40 => format!("palavras_pesquisa={DESTINO}&id_orgao="),
        Versao::V41 => format!("palavras_pesquisa={DESTINO}&id_orgao=&unidade_atual=0"),
    }
}

/// `hdnUnidades` codificado: `ID±DESCRIÇÃO` em ISO-8859-1.
pub(crate) fn unidades_codificadas() -> String {
    format!("{ID_DESTINO}%B1{DESTINO}+-+Unidade+de+Testes+1.1")
}

/// O corpo do POST de enviar (ordem do documento).
pub(crate) fn corpo_do_envio(versao: Versao, manter_aberto: bool) -> String {
    let manter = if manter_aberto {
        "chkSinManterAberto=on&"
    } else {
        ""
    };
    let textos = match versao {
        Versao::V40 => "txtPrazo=&txtDias=",
        Versao::V41 => {
            "txtPrazoRetornoProgramado=&txtDiasRetornoProgramado=&txtPrazoReaberturaProgramada=&txtDiasReaberturaProgramada="
        }
    };
    format!(
        "hdnInfraTipoPagina=2&sbmEnviar=Enviar&selOrgao=&txtUnidade=&hdnIdUnidade=&{manter}{textos}&hdnIdProtocolos={ID}&hdnIdAtividades=42468&hdnUnidades={}",
        unidades_codificadas()
    )
}

/// `Location` do 302 observado depois do POST.
pub(crate) fn volta_a_arvore_do_envio() -> String {
    format!(
        "controlador.php?acao=arvore_visualizar&acao_origem=procedimento_enviar&atualizar_arvore=1&arvore=1&id_procedimento={ID}&{CONTEXTO}&infra_hash={}",
        "b".repeat(64)
    )
}

pub(crate) fn url_da_tela() -> String {
    absoluta(&link_de_enviar(ID))
}

/// Tela e consulta: o que a prévia pede depois da abertura.
pub(crate) fn pedidos_da_previa(versao: Versao) -> Vec<Pedido> {
    let mut todos = pedidos_da_abertura();
    todos.push(get(&url_da_tela()));
    todos.push(post(
        &absoluta(&autocompletar()),
        &corpo_da_consulta(versao),
    ));
    todos
}

/// Prévia, POST e verificação.
pub(crate) fn pedidos_da_escrita(versao: Versao, manter_aberto: bool) -> Vec<Pedido> {
    let mut todos = pedidos_da_previa(versao);
    todos.push(post(
        &absoluta(&action_de_enviar()),
        &corpo_do_envio(versao, manter_aberto),
    ));
    todos.push(get(&url_arvore()));
    todos
}

pub(crate) fn com(mut base: Vec<Resposta>, depois: Vec<Resposta>) -> Vec<Resposta> {
    base.extend(depois);
    base
}

pub(crate) fn siglas(lista: &[&str]) -> Vec<String> {
    lista.iter().map(|s| (*s).to_owned()).collect()
}

/// Roda `processo enviar` com `respostas`; confere que todas foram usadas.
pub(crate) fn rodar_envio(
    para: &str,
    manter_aberto: bool,
    confirmar: bool,
    respostas: Vec<Resposta>,
) -> (Result<ProcessoEnviado, ErroEnviar>, Vec<Pedido>) {
    let amb = ambiente();
    rodar_com(&amb, respostas, |sessao, nup, indice, roteiro| {
        let pedido = PedidoDoEnvio {
            nup,
            unidade: None,
            para,
            manter_aberto,
            remover_anotacoes: false,
            confirmar,
        };
        enviar_processo(sessao, &pedido, indice, roteiro)
    })
}

/// A prévia esperada com `aberto_em`.
pub(crate) fn previa(aberto_em: &[&str], manter_aberto: bool) -> ProcessoEnviado {
    ProcessoEnviado {
        nup: NUP.into(),
        id_procedimento: ID.into(),
        unidade: UNIDADE.into(),
        manter_aberto,
        remocao: None,
        destino: destino(),
        aberto_em: siglas(aberto_em),
        desfecho: DesfechoDoEnvio::Previa,
        requisicoes: 5,
    }
}

/// A tela do GO 5.0.3 (64 opções no `selOrgao`, a ABC é a `28`; rótulos
/// com espaço como `CASA CIVIL`), recortada: a estrutura da forma 4.1/5.0
/// com outros órgãos.
pub(crate) fn tela_de_envio_go(id: &str) -> String {
    tela_de_envio(Versao::V41, id).replace(
        "<option value=\"0\">ORGAO</option>\n",
        "<option value=\"28\">ABC</option>\n<option value=\"31\">AGEHAB</option>\n\
         <option value=\"30\">CASA CIVIL</option>\n<option value=\"5\">SEAD</option>\n",
    )
}

/// O corpo da consulta do 4.1/5.0 com `termo` (já codificado) e `orgao`.
pub(crate) fn corpo_da_consulta_com(termo: &str, orgao: &str) -> String {
    format!("palavras_pesquisa={termo}&id_orgao={orgao}&unidade_atual=0")
}

/// Abertura, tela e uma consulta por corpo, na ordem.
pub(crate) fn pedidos_da_busca(corpos: &[String]) -> Vec<Pedido> {
    let mut todos = pedidos_da_abertura();
    todos.push(get(&url_da_tela()));
    for corpo in corpos {
        todos.push(post(&absoluta(&autocompletar()), corpo));
    }
    todos
}
