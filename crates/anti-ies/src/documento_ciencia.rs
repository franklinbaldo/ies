//! Ciência documental sintética: tela de nó, GET mutador único e lista relida.
//! O estado é o mesmo SQLite descartável do IES; nenhum cookie ou link real.
use crate::{Estado, Resposta, eh_controlador_sei, parametro_unico};
use iesei::ciencia_documental::{self as modelo, Estado as Ciencia, Resultado};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;

/// Ação oferecida na barra do documento, nunca no processo vizinho.
pub(crate) fn barra(ciencia: &Ciencia) -> &'static str {
    match ciencia {
        Ciencia::Pendente => {
            r##"<a href="#" onclick="cienciaDocumento();" title="Ciência">Ciência</a>"##
        }
        Ciencia::Registrada { .. } => "",
    }
}

/// Uma lista só é renderizada depois de uma atividade realmente persistida.
pub(crate) fn acao_da_lista(
    indice: usize,
    documento: &str,
    processo: &str,
    ciencia: &Ciencia,
) -> String {
    let Ciencia::Registrada { .. } = ciencia else {
        return String::new();
    };
    let hash = crate::protocolo_sintetico::INFRA_HASH;
    format!(
        r#"NosAcoes[{indice}] = new infraArvoreAcao("CIENCIAS","CD{documento}","{documento}","controlador.php?acao=protocolo_ciencia_listar&id_procedimento={processo}&id_documento={documento}&infra_hash={hash}","ifrVisualizacao","Visualizar Ciências","svg/ciencia.svg",true);
"#
    )
}

pub(crate) fn estado(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    documento: &str,
    processo: &str,
) -> Result<Option<Ciencia>, Estado> {
    modelo::consultar(banco, origem, cenario, documento, processo)
        .map_err(|_| Estado::DadosInsuficientes)
}

/// Tela de nó, separada da árvore. Não reproduz HTML gravado de cassettes.
pub(crate) fn tela(
    metodo: &str,
    alvo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Option<Resposta> {
    if !eh_controlador_sei(alvo) || parametro_unico(alvo, "acao") != Some("arvore_visualizar") {
        return None;
    }
    let (Some(documento), Some(processo)) = (
        parametro_unico(alvo, "id_documento"),
        parametro_unico(alvo, "id_procedimento"),
    ) else {
        return None;
    };
    if !documento.bytes().all(|b| b.is_ascii_digit())
        || !processo.bytes().all(|b| b.is_ascii_digit())
    {
        return Some(Resposta::nova(Estado::RequisicaoInvalida));
    }
    let existente = match estado(banco, origem, cenario, documento, processo) {
        Ok(Some(ciencia)) => ciencia,
        Ok(None) => return None,
        Err(erro) => return Some(Resposta::nova(erro)),
    };
    if metodo != "GET" {
        return Some(Resposta::nova(Estado::MetodoNaoPermitido));
    }
    let hash = crate::protocolo_sintetico::INFRA_HASH;
    let link = if matches!(existente, Ciencia::Pendente) {
        format!(
            "var linkCienciaDocumento = 'controlador.php?acao=documento_ciencia&id_documento={documento}&infra_hash={hash}';"
        )
    } else {
        String::new()
    };
    Some(Resposta::html(format!(
        "<html><head><title>SEI - Visualizar Árvore</title><script type=\"text/javascript\">\n{link}\n</script></head><body></body></html>"
    )))
}

/// Rotas cuja consulta à lista não modifica o documento.
/// O GET `documento_ciencia` é uma única escrita, nunca um "GET" inocente.
pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Resposta {
    if !eh_controlador_sei(alvo) {
        return Resposta::nova(Estado::NaoEncontrado);
    }
    if metodo != "GET" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Some(documento) =
        parametro_unico(alvo, "id_documento").filter(|id| id.bytes().all(|b| b.is_ascii_digit()))
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    match parametro_unico(alvo, "acao") {
        Some("documento_ciencia") => {
            let processo = match modelo::processo_do_documento(banco, origem, cenario, documento) {
                Ok(Some(processo)) => processo,
                Ok(None) => return Resposta::nova(Estado::NaoEncontrado),
                Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
            };
            if parametro_unico(alvo, "id_procedimento").is_some_and(|id| id != processo) {
                return Resposta::nova(Estado::RequisicaoInvalida);
            }
            match modelo::registrar(banco, origem, cenario, documento, &processo) {
                Ok(Resultado::Registrada { id_atividade }) => Resposta::html(format!(
                    "<html><head><script>function inicializar(){{ atualizarArvore(         'controlador.php?acao=procedimento_visualizar&id_procedimento={processo}&id_documento={documento}&documento_visualizar_ciencias=1&id_atividade={id_atividade}#ID-{id_atividade}'); }}</script></head></html>"
                )),
                Ok(Resultado::JaRegistrada) => Resposta::html(
                    "<html><body><textarea id='txaInfraValidacao' style='display:none'>\
                     Usuário já deu ciência neste documento.</textarea></body></html>"
                        .to_owned(),
                ),
                Ok(Resultado::DocumentoNaoEncontrado) => Resposta::nova(Estado::NaoEncontrado),
                Err(_) => Resposta::nova(Estado::DadosInsuficientes),
            }
        }
        Some("protocolo_ciencia_listar") => {
            let Some(processo) = parametro_unico(alvo, "id_procedimento")
                .filter(|id| id.bytes().all(|b| b.is_ascii_digit()))
            else {
                return Resposta::nova(Estado::RequisicaoInvalida);
            };
            match estado(banco, origem, cenario, documento, processo) {
                Ok(Some(Ciencia::Registrada { id_atividade })) => {
                    let Ok(usuario) = modelo::usuario_da_ciencia(
                        banco, origem, cenario, documento, processo,
                    ) else {
                        return Resposta::nova(Estado::DadosInsuficientes);
                    };
                    // A lista só nomeia o autor quando a sessão sintética
                    // o identificou. Revisores do documento não são autores
                    // inferidos da ciência.
                    let autoria = usuario.as_deref().map_or_else(String::new, |nome| {
                        format!("<td>{}</td>", crate::escapar(nome))
                    });
                    Resposta::html(format!(
                        "<html><table id='tblAndamentos'><caption>Lista de Ciências:</caption><tr><td><input type='checkbox' name='chkInfraItem' value='{id_atividade}' /></td>{autoria}</tr></table></html>"
                    ))
                }
                Ok(Some(Ciencia::Pendente)) => Resposta::nova(Estado::DadosInsuficientes),
                Ok(None) => Resposta::nova(Estado::NaoEncontrado),
                Err(erro) => Resposta::nova(erro),
            }
        }
        _ => Resposta::nova(Estado::NaoEncontrado),
    }
}
