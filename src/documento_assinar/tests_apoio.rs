//! Apoio dos testes de `documento assinar`: o transporte roteirizado e a
//! sessão de `processo concluir`, a árvore com o documento assinável, as
//! telas do documento (4.0 e 5.0) e a tela "Assinatura de Documento" na
//! forma das sete instalações. Hosts, hashes, ids, nomes e senha inventados.
use super::{
    AcaoDaAssinatura, Confirmacao, DocumentoAssinado, ErroAssinar, PedidoDaAssinatura,
    assinar_documento,
};
pub(crate) use crate::arvore_processo::tests_assinatura::{
    arvore_assinavel, barra_assinavel, caneta, link_da_tela_do_documento, link_de_assinar,
    tela_do_documento_40, tela_do_documento_50,
};
use crate::documento::NumeroDocumento;
pub(crate) use crate::processo_concluir::tests_apoio::*;
use crate::sessao_login::Credenciais;
use std::fmt::Write as _;

pub(crate) const DOC: &str = "0000101";
pub(crate) const ID_DOC: &str = "21";
pub(crate) const ASSINANTE: &str = "Usuário Teste";
pub(crate) const CARGO: &str = "Assessor(a)";
pub(crate) const SENHA: &str = "s3nh@ x";
pub(crate) const HASH_DOCUMENTOS: &str = "e31ea40e5c5cdcd45c0cb824f35255e9";
pub(crate) const HASH_DO_ENVIO: &str =
    "8888888888888888888888888888888888888888888888888888888888888888";
const CONTEXTO: &str = "infra_sistema=100000100&infra_unidade_atual=110000001";

pub(crate) fn action_de_assinar() -> String {
    format!(
        "controlador.php?acao=documento_assinar&acao_origem=documento_assinar&acao_retorno=principal&hash_documentos={HASH_DOCUMENTOS}&arvore=1&id_procedimento={ID}&id_documento={ID_DOC}&{CONTEXTO}&infra_hash={HASH_DO_ENVIO}"
    )
}

/// O `inicializar()` da tela servida pelo GET (sem assinatura) ou da
/// resposta aceita (`finalizar();`).
fn inicializar(assinado: bool) -> &'static str {
    if assinado {
        "function inicializar(){\n  //se realizou assinatura\n           finalizar();\n    return;\n  }\n"
    } else {
        "function inicializar(){\n  //se realizou assinatura\n    if (document.getElementById('selCargoFuncao').options.length==2){\n      document.getElementById('selCargoFuncao').options[1].selected = true;\n    }\n  }\n"
    }
}

/// A tela "Assinatura de Documento" do documento `id`, com os `cargos` e o
/// `selecionado`; `assinado`: a resposta do POST aceito.
pub(crate) fn tela_de_assinatura(
    id: &str,
    cargos: &[&str],
    selecionado: Option<&str>,
    assinado: bool,
) -> String {
    let mut opcoes = String::from("<option value=\"null\">&nbsp;</option>\n");
    for cargo in cargos {
        let marca = if Some(*cargo) == selecionado {
            " selected=\"selected\""
        } else {
            ""
        };
        let _ = writeln!(opcoes, "<option value=\"{cargo}\"{marca}>{cargo}</option>");
    }
    format!(
        "<html><head><title>SEI - Assinatura de Documento</title><script type=\"text/javascript\">\n{ini}\
         function assinarSenha(){{\n  document.getElementById('hdnFormaAutenticacao').value = 'S';\n  document.getElementById('frmAssinaturas').submit();\n}}\n\
         </script></head><body onload=\"inicializar();\">\n\
         <form id=\"frmAssinaturas\" method=\"post\" onsubmit=\"return OnSubmitForm();\" action=\"{action}\">\n\
         <input type=\"hidden\" id=\"hdnInfraTipoPagina\" name=\"hdnInfraTipoPagina\" value=\"2\" />\n\
         <button tabindex=\"451\" type=\"button\" accesskey=\"A\" onclick=\"assinarSenha();\" id=\"btnAssinar\" name=\"btnAssinar\" value=\"Assinar\" class=\"infraButton\">Assinar</button>\n\
         <select id=\"selOrgao\" name=\"selOrgao\" onchange=\"trocarOrgaoUsuario();\" class=\"infraSelect\">\n\
         <option value=\"null\">&nbsp;</option>\n<option value=\"0\" selected=\"selected\">ORGAO1</option>\n<option value=\"1\">TESTE</option>\n</select>\n\
         <input type=\"text\" id=\"txtUsuario\" name=\"txtUsuario\" class=\"infraText\" value=\"{ASSINANTE}\" />\n\
         <input type=\"hidden\" id=\"hdnIdUsuario\" name=\"hdnIdUsuario\" value=\"100000006\" />\n\
         <select id=\"selCargoFuncao\" name=\"selCargoFuncao\" class=\"infraSelect\">\n{opcoes}</select>\n\
         <input type=\"password\" id=\"pwdSenha\" name=\"pwdSenha\" class=\"infraText\" autocomplete=\"off\" value=\"\" maxlength=\"100\" />\n\
         <button type=\"button\" id=\"btnCopiarCodigo\" name=\"btnCopiarCodigo\" value=\"Copiar\" class=\"infraButton clipboard\">Disponibilizar</button>\n\
         <input type=\"hidden\" id=\"hdnFormaAutenticacao\" name=\"hdnFormaAutenticacao\" value=\"\" />\n\
         <input type=\"hidden\" id=\"hdnLinkRetorno\" name=\"hdnLinkRetorno\" value=\"\" />\n\
         <input type=\"hidden\" id=\"hdnFlagAssinatura\" name=\"hdnFlagAssinatura\" value=\"1\" />\n\
         <input type=\"hidden\" id=\"hdnIdDocumentos\" name=\"hdnIdDocumentos\" value=\"{id}\" />\n\
         <input type=\"hidden\" id=\"hdnIdBlocos\" name=\"hdnIdBlocos\" value=\"\" />\n\
         </form>\n<input type=\"hidden\" id=\"hdnInfraPrefixoCookie\" name=\"hdnInfraPrefixoCookie\" value=\"SEI_X\" />\n</body></html>",
        ini = inicializar(assinado),
        action = action_de_assinar(),
    )
}

/// A tela padrão: dois cargos, o primeiro sugerido.
pub(crate) fn tela_padrao() -> String {
    tela_de_assinatura(ID_DOC, &[CARGO, "Presidente"], Some(CARGO), false)
}

/// A resposta aceita do POST.
pub(crate) fn resposta_aceita() -> String {
    tela_de_assinatura(ID_DOC, &[CARGO, "Presidente"], Some(CARGO), true)
}

/// O corpo que o navegador envia (ordem do documento, ISO-8859-1).
pub(crate) const CORPO_DA_ASSINATURA: &str = "hdnInfraTipoPagina=2&selOrgao=0&txtUsuario=Usu%E1rio+Teste&hdnIdUsuario=100000006&selCargoFuncao=Assessor%28a%29&pwdSenha=s3nh%40+x&hdnFormaAutenticacao=S&hdnLinkRetorno=&hdnFlagAssinatura=1&hdnIdDocumentos=21&hdnIdBlocos=";

/// A árvore do processo `ID` com o documento, a barra assinável e as canetas.
pub(crate) fn arvore_do_documento(edicao: char, canetas: &[String]) -> String {
    arvore_assinavel(
        ID,
        ID_DOC,
        DOC,
        &barra_assinavel(ID, ID_DOC, Some(edicao)),
        canetas,
    )
}

pub(crate) fn caneta_da_sessao(cargo: &str) -> String {
    caneta(0, ID_DOC, ASSINANTE, cargo, UNIDADE, false)
}

pub(crate) fn credenciais() -> Credenciais {
    Credenciais::interpretar(&format!("usuario=u\nsenha={SENHA}\norgao=ORGAO1\n")).unwrap()
}

pub(crate) fn url_da_tela_do_documento() -> String {
    absoluta(&link_da_tela_do_documento(ID, ID_DOC))
}

pub(crate) fn url_de_assinar() -> String {
    absoluta(&link_de_assinar(ID, ID_DOC))
}

/// Roda `documento --processo NUP assinar DOC` com `respostas`.
pub(crate) fn rodar_assinar(
    cargo: Option<&str>,
    confirmar: bool,
    respostas: Vec<Resposta>,
) -> (Result<DocumentoAssinado, ErroAssinar>, Vec<Pedido>) {
    let amb = ambiente();
    let lidas = credenciais();
    rodar_com(&amb, respostas, |sessao, nup, indice, roteiro| {
        let numero = NumeroDocumento::novo(DOC).unwrap();
        // Como na CLI: confirmar sem cargo continua sendo prévia.
        let acao = match (confirmar, cargo) {
            (true, Some(cargo)) => AcaoDaAssinatura::Confirmar(Confirmacao {
                cargo,
                credenciais: &lidas,
            }),
            (_, cargo) => AcaoDaAssinatura::Previa { cargo },
        };
        let pedido = PedidoDaAssinatura {
            numero: &numero,
            unidade: None,
            processo: Some(nup),
            acao,
        };
        assinar_documento(sessao, &pedido, indice, roteiro)
    })
}
