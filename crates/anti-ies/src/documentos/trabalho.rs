//! Tela de trabalho derivada do estado documental e da abertura da unidade.
use super::{erros_de_modelo, modelo};
use crate::{Estado, Resposta, escapar};
use iesei::documentos_simulados::Falha;
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::fmt::Write as _;

pub(super) fn pagina_do_processo(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
) -> Resposta {
    let Ok(Some(processo)) = modelo::ler_processo(banco, origem, cenario, id) else {
        return Resposta::nova(Estado::NaoEncontrado);
    };
    let Ok(documentos) = modelo::documentos(banco, origem, cenario, id) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    // A tela de trabalho e a árvore devem anunciar a mesma capacidade.
    // Depois de enviar sem manter aberto, o cadastro já recusa a escrita:
    // não se pode continuar oferecendo um link que não funcionará.
    let incluir = match modelo::conferir_abertura(banco, origem, cenario, id) {
        Ok(()) => format!(
            "<a id='incluirDocumento' href='controlador.php?acao=documento_escolher_tipo&amp;id_procedimento={id}&amp;infra_hash={}'>Incluir Documento</a>",
            "e".repeat(64)
        ),
        Err(Falha::ProcessoNaoAbertoNaUnidade) => String::new(),
        Err(falha) => return erros_de_modelo(&falha),
    };
    let mut html = format!(
        "<html><body><a id='lnkInfraUnidade' href='#'>{}</a>\
         <iframe id='ifrArvore' name='ifrArvore' src='controlador.php?acao=procedimento_visualizar&amp;id_procedimento={id}&amp;infra_hash={}'></iframe>\
         <span id='numeroProcesso'>{}</span><span id='unidade'>{}</span>\
         {incluir}<ul id='documentos'>",
        escapar(&processo.unidade),
        "e".repeat(64),
        escapar(&processo.nup),
        escapar(&processo.unidade),
    );
    for doc in documentos {
        write!(
            html,
            "<li data-id='{}' data-numero-sei='{}'>{} {}</li>",
            doc.id,
            doc.numero_sei,
            escapar(&doc.tipo),
            doc.numero_sei
        )
        .expect("formatação String");
    }
    html.push_str("</ul></body></html>");
    Resposta::html(html)
}
