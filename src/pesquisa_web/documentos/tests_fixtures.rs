//! Telas SINTÉTICAS da pesquisa em documentos, com a estrutura observada nas
//! cassettes públicas (`pesquisa-paginas-pen3`, `documento-numero-e-busca-*`):
//! `a.arvore` e `a.protocoloNormal` do processo com `id_documento`, o link
//! do documento entre parênteses e o número SEI à direita; metadados
//! "Unidade:", "Usuário:" e "Inclusão:". Nenhum valor real.
use crate::pesquisa_web::tests_fixtures::{
    CONTEXTO, conteudo_com_linhas, formulario, hash, pagina,
};

pub(crate) fn numero_sei(posicao: u64) -> String {
    format!("{:07}", 5000 + posicao)
}

pub(crate) fn nup(posicao: u64) -> String {
    format!("00002.{posicao:06}/2026-00")
}

/// `href` do processo da linha, com `id_documento`.
pub(crate) fn href_do_processo(posicao: u64) -> String {
    format!(
        "controlador.php?acao=procedimento_trabalhar&acao_origem=protocolo_pesquisar&id_procedimento={}&id_documento={}&{CONTEXTO}&infra_hash={}",
        2000 + posicao,
        3000 + posicao,
        hash('e')
    )
}

/// `href` do documento: `documento_visualizar` (mesmo `id_documento`) ou,
/// no anexo, `documento_download_anexo` com `id_anexo`.
pub(crate) fn href_do_documento(posicao: u64, anexo: bool) -> String {
    if anexo {
        format!(
            "controlador.php?acao=documento_download_anexo&acao_origem=protocolo_pesquisar&id_anexo={}&{CONTEXTO}&infra_hash={}",
            900 + posicao,
            hash('7')
        )
    } else {
        format!(
            "controlador.php?acao=documento_visualizar&acao_origem=protocolo_pesquisar&id_documento={}&{CONTEXTO}&infra_hash={}",
            3000 + posicao,
            hash('8')
        )
    }
}

/// Uma linha como na PEN3 5.0.4, com as três linhas do registro.
pub(crate) fn linha_de_documento(posicao: u64, anexo: bool) -> String {
    let processo = href_do_processo(posicao);
    let documento = href_do_documento(posicao, anexo);
    let (nup, numero) = (nup(posicao), numero_sei(posicao));
    format!(
        "<tr class=\"pesquisaTituloRegistro\">\n<td colspan=\"2\" class=\"pesquisaTituloEsquerda\">\
         <a onclick=\"x(this);\" href=\"{processo}\" target=\"_blank\" class=\"arvore\">\
         <img border=\"0\" src=\"svg/arvore.svg?25\" alt=\"Tipo N {nup} - Despacho ({numero})\" class=\"arvore\" /></a>\
         <span>Tipo: Assunto N&ordm; </span><a onclick=\"x(this);\" target=\"_blank\" class=\"protocoloNormal\" \
         href=\"{processo}\" title=\"Tipo\" class=\"protocoloNormal\" tabindex=\"1002\">{nup}</a> \
         (<a onclick=\"x(this);\" target=\"_blank\" class=\"protocoloNormal\" href=\"{documento}\" \
         title=\"Despacho\" tabindex=\"1003\">Despacho {posicao}</a>)</td>\
         <td class=\"pesquisaTituloDireita\"><a onclick=\"x(this);\" target=\"_blank\" class=\"protocoloNormal\" \
         href=\"{documento}\" title=\"Despacho\" tabindex=\"1004\">{numero}</a></td></tr>\
         <tr><td colspan=\"3\" class=\"pesquisaSnippet\">Despacho <b>Processo</b> n&ordm; {nup}</td></tr>\n\
         <tr><td width=\"33%\" class=\"pesquisaMetatag\"><b>Unidade:</b> <a alt=\"Unidade de Teste\" \
         title=\"Unidade de Teste\" class=\"ancoraSigla\">TESTE</a></td>\n\
         <td width=\"33%\" class=\"pesquisaMetatag\"><b>Usu&aacute;rio:</b> <a class=\"ancoraSigla\">USUARIO</a></td>\n\
         <td width=\"33%\" class=\"pesquisaMetatag\"><b>Inclus&atilde;o:</b> 12/10/2026</td>\n</tr>\n"
    )
}

/// Linhas `inicio..=fim`; a cada 3, uma de anexo.
pub(crate) fn linhas(inicio: u64, fim: u64) -> String {
    (inicio..=fim)
        .map(|p| linha_de_documento(p, p % 3 == 0))
        .collect()
}

/// Resposta 200 de uma lista em documentos, com `D` marcado.
pub(crate) fn resultado_de_documentos(termo: &str, inicio: u64, fim: u64, total: &str) -> String {
    let conteudo = conteudo_com_linhas(inicio, fim, total, 10, &linhas(inicio, fim));
    pagina(&formulario(termo, false, &conteudo))
}

/// A mesma página com o `conteudo` dado no lugar da tabela.
pub(crate) fn resultado_com(termo: &str, conteudo: &str) -> String {
    pagina(&formulario(termo, false, conteudo))
}

/// Barra "Exibindo" e tabela com as `linhas` dadas.
pub(crate) fn tabela(inicio: u64, fim: u64, total: &str, linhas: &str) -> String {
    conteudo_com_linhas(inicio, fim, total, 10, linhas)
}

/// `Location` do 302 do número SEI exato: `id_documento` decimal.
pub(crate) fn localizacao_de_documento(id_procedimento: &str, id_documento: &str) -> String {
    format!(
        "controlador.php?acao=procedimento_trabalhar&acao_origem=protocolo_pesquisar&id_procedimento={id_procedimento}&id_documento={id_documento}&{CONTEXTO}&infra_hash={}",
        hash('d')
    )
}
