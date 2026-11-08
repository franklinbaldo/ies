//! Histórico e visualização de versões gerados pelo mesmo estado SQLite.
use crate::{Estado, Resposta, despachar};
use iesei::documento_versoes::{DocumentoSimulado, Revisao, ler};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::fmt::Write;

fn parametro<'a>(alvo: &'a str, nome: &str) -> Result<Option<&'a str>, Estado> {
    let Some((_, query)) = alvo.split_once('?') else {
        return Ok(None);
    };
    let mut valor = None;
    for parte in query.split('&') {
        let Some((chave, v)) = parte.split_once('=') else {
            return Err(Estado::RequisicaoInvalida);
        };
        if chave == nome {
            if v.is_empty() || valor.is_some_and(|anterior| anterior != v) {
                return Err(Estado::RequisicaoInvalida);
            }
            valor = Some(v);
        }
    }
    Ok(valor)
}

fn escapar(texto: &str) -> String {
    texto.replace('&', "&amp;").replace('<', "&lt;")
        .replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;")
}

fn resposta_html(corpo: String) -> Resposta {
    Resposta { estado: Estado::Ok, tipo_conteudo: "text/html; charset=utf-8", corpo }
}

fn historico(documento: &DocumentoSimulado) -> String {
    let mut corpo = format!(
        "<html><body><table class='infraTable'><caption>Lista de Versões ({} registros):</caption>         <tr><th></th><th>Versão</th><th>Usuário</th><th>Unidade</th>         <th>Última Modificação</th><th>Ações</th></tr>",
        documento.revisoes.len()
    );
    // Token fictício produzido somente em resposta de simulador local:
    // não há cookie, credencial ou URL assinada guardada no SQLite.
    let token_teste = "0".repeat(64);
    for r in &documento.revisoes {
        let id = escapar(&documento.id_documento);
        write!(
            corpo,
            "<tr><td></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td>             <td><a href='controlador.php?acao=documento_visualizar&amp;versao={}&amp;id_documento={id}&amp;infra_hash={token_teste}'>Visualizar</a></td></tr>",
            r.versao, escapar(&r.usuario), escapar(&r.unidade),
            escapar(&r.modificacao), r.versao
        ).expect("formatação String");
    }
    corpo.push_str("</table></body></html>");
    corpo
}

fn visualizar(documento: &DocumentoSimulado, revisao: &Revisao) -> String {
    format!(
        "<html><head><title>SEI/anti-IES - {} - {}</title></head>         <body>{}<div unselectable='on'>Criado por <a>{}</a>, versão {} por <a>{}</a> em {}.</div></body></html>",
        escapar(&documento.numero), escapar(&documento.tipo), revisao.html_corpo,
        escapar(&revisao.usuario), revisao.versao, escapar(&revisao.usuario),
        escapar(&revisao.modificacao)
    )
}

/// Rotas de leitura sem sessão, com versão N e atual coerentes com o mesmo estado.
///
/// # Cautela
/// O HTML é uma hipótese funcional para o anti-IES; não comprova que uma
/// instalação real do SEI usa precisamente essa forma.
#[must_use]
pub fn responder(
    metodo: &str,
    alvo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if !super::eh_controlador_sei(alvo) {
        return despachar(metodo, alvo);
    }
    let acao = match parametro(alvo, "acao") {
        Ok(Some(acao @ ("documento_versao_listar" | "documento_visualizar"))) => acao,
        Ok(_) => return despachar(metodo, alvo),
        Err(estado) => return Resposta::nova(estado),
    };
    if metodo != "GET" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let id = match parametro(alvo, "id_documento") {
        Ok(Some(id)) if id.bytes().all(|b| b.is_ascii_digit()) => id,
        _ => return Resposta::nova(Estado::RequisicaoInvalida),
    };
    let Ok(Some(documento)) = ler(banco, instalacao, cenario, id) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    if acao == "documento_versao_listar" {
        return resposta_html(historico(&documento));
    }
    let versao = match parametro(alvo, "versao") {
        Ok(None) => documento.revisoes.iter().map(|r| r.versao.get()).max(),
        Ok(Some(n)) => n.parse::<u32>().ok().filter(|n| *n > 0),
        Err(_) => None,
    };
    let Some(versao) = versao else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let Some(revisao) = documento.revisoes.iter().find(|r| r.versao.get() == versao) else {
        return Resposta::nova(Estado::NaoEncontrado);
    };
    resposta_html(visualizar(&documento, revisao))
}
