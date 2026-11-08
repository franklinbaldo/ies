//! Histórico e visualização de versões gerados pelo mesmo estado SQLite.
use crate::{Estado, Resposta, despachar};
use iesei::acesso_versao::{self, AcessoVersao};
use iesei::documento_versoes::{DocumentoSimulado, Revisao, ler};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::fmt::Write;

pub(crate) fn parametro<'a>(alvo: &'a str, nome: &str) -> Result<Option<&'a str>, Estado> {
    let Some((_, query)) = alvo.split_once('?') else {
        return Ok(None);
    };
    let mut valor = None;
    for parte in query.split('&') {
        let Some((chave, v)) = parte.split_once('=') else {
            return Err(Estado::RequisicaoInvalida);
        };
        if chave == nome {
            // O SEI publica duas vezes id_documento no link da versão.
            // Aceitar somente a repetição idêntica; qualquer ambiguidade recusa.
            if v.is_empty() || (valor.is_some() && (nome != "id_documento" || valor != Some(v))) {
                return Err(Estado::RequisicaoInvalida);
            }
            valor = Some(v);
        }
    }
    Ok(valor)
}

fn escapar(texto: &str) -> String {
    texto
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn resposta_html(corpo: String) -> Resposta {
    Resposta::html(corpo)
}

fn historico(documento: &DocumentoSimulado) -> String {
    let mut corpo = format!(
        "<html><body><table class='infraTable'><caption>Lista de Versões ({} registros):</caption>         <tr><th></th><th>Versão</th><th>Usuário</th><th>Unidade</th>         <th>Última Modificação</th><th>Ações</th></tr>",
        documento.revisoes.len()
    );
    // Token fictício produzido somente em resposta de simulador local:
    // não há cookie, credencial ou URL assinada guardada no SQLite.
    let token_teste = crate::protocolo_sintetico::INFRA_HASH;
    for r in &documento.revisoes {
        let id = escapar(&documento.id_documento);
        let processo = escapar(&documento.id_procedimento);
        write!(
            corpo,
            "<tr><td></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td>             <td><a href='controlador.php?acao=documento_visualizar&versao={}&id_documento={id}&arvore=1&id_procedimento={processo}&id_documento={id}&infra_hash={token_teste}'>Visualizar</a></td></tr>",
            r.versao, escapar(&r.usuario), escapar(&r.unidade),
            escapar(&r.modificacao), r.versao
        ).expect("formatação String");
    }
    corpo.push_str("</table></body></html>");
    corpo
}

fn visualizar(documento: &DocumentoSimulado, revisao: &Revisao) -> String {
    format!(
        "<html><head><title>SEI/anti-IES - {} - {}</title></head>         <body><main id='corpoEditavel'>{}</main><div unselectable='on'>Criado por <a>{}</a>, versão {} por <a>{}</a> em {}.</div></body></html>",
        escapar(&documento.numero),
        escapar(&documento.tipo),
        revisao.html_corpo,
        escapar(&revisao.usuario),
        revisao.versao,
        escapar(&revisao.usuario),
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
    let versao_pedida = if acao == "documento_visualizar" {
        match parametro(alvo, "versao") {
            Ok(None) => None,
            Ok(Some(n)) => match n.parse::<u32>().ok().filter(|n| *n > 0) {
                Some(n) => Some(n),
                None => return Resposta::nova(Estado::RequisicaoInvalida),
            },
            Err(estado) => return Resposta::nova(estado),
        }
    } else {
        None
    };
    let Ok(Some(documento)) = ler(banco, instalacao, cenario, id) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    // A URL publicada pode declarar o processo: nunca entregar conteúdo
    // de outro processo quando a identidade composta não corresponde.
    match parametro(alvo, "id_procedimento") {
        Ok(Some(processo)) if processo != documento.id_procedimento.as_str() => {
            return Resposta::nova(Estado::NaoEncontrado);
        }
        Err(estado) => return Resposta::nova(estado),
        _ => {}
    }
    if acao == "documento_versao_listar" {
        return resposta_html(historico(&documento));
    }
    let versao = versao_pedida.or_else(|| documento.revisoes.iter().map(|r| r.versao.get()).max());
    let Some(versao) = versao else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let Some(revisao) = documento.revisoes.iter().find(|r| r.versao.get() == versao) else {
        return Resposta::nova(Estado::NaoEncontrado);
    };
    match acesso_versao::consultar(banco, instalacao, cenario, id, revisao.versao) {
        Ok(AcessoVersao::Recusado) => return Resposta::nova(Estado::AcessoNegado),
        Ok(AcessoVersao::Acessivel) => {}
        Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
    }
    resposta_html(visualizar(&documento, revisao))
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn id_documento_repetido_igual_na_lista_e_aceito_sem_abrir_ambiguidades() {
        let alvo = "/sei/controlador.php?acao=documento_visualizar&versao=1&id_documento=42&id_procedimento=7&id_documento=42";
        assert_eq!(parametro(alvo, "id_documento"), Ok(Some("42")));
        assert_eq!(
            parametro(
                "/sei/controlador.php?id_documento=42&id_documento=43",
                "id_documento"
            ),
            Err(Estado::RequisicaoInvalida)
        );
        assert_eq!(
            parametro("/sei/controlador.php?versao=1&versao=1", "versao"),
            Err(Estado::RequisicaoInvalida)
        );
    }
}
