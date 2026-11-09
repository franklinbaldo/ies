//! Ciência do processo via HTTP local e estado SQLite.
use crate::{Estado, Resposta, eh_controlador_sei, parametro_unico};
use iesei::ciencia_processo::{self as modelo, Estado as Ciencia, Resultado};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;

/// Tela da função de ciência, separada do GET mutador.
pub(crate) fn tela(
    metodo: &str,
    alvo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Option<Resposta> {
    if !eh_controlador_sei(alvo)
        || parametro_unico(alvo, "acao") != Some("arvore_visualizar")
        || parametro_unico(alvo, "id_documento").is_some()
    {
        return None;
    }
    let id = parametro_unico(alvo, "id_procedimento")?;
    if !id.bytes().all(|b| b.is_ascii_digit()) {
        return Some(Resposta::nova(Estado::RequisicaoInvalida));
    }
    let estado = match modelo::consultar(banco, origem, cenario, id) {
        Ok(Some(estado)) => estado,
        Ok(None) => return None,
        Err(_) => return Some(Resposta::nova(Estado::DadosInsuficientes)),
    };
    if metodo != "GET" {
        return Some(Resposta::nova(Estado::MetodoNaoPermitido));
    }
    let funcao = if matches!(estado, Ciencia::Pendente) {
        format!(
            "function cienciaProcesso(){{\n//if (confirm('Registrar ciência?')){{\nlocation.href = 'controlador.php?acao=procedimento_ciencia&id_procedimento={id}&infra_hash={}';\n}}",
            crate::protocolo_sintetico::INFRA_HASH
        )
    } else {
        String::new()
    };
    Some(Resposta::html(format!(
        "<html><head><script type='text/javascript'>\n{funcao}\n</script></head><body></body></html>"
    )))
}

/// Consulta ou grava uma atividade no processo. A lista C nunca é a CD.
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
    if parametro_unico(alvo, "id_documento").is_some() {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    let Some(id) = parametro_unico(alvo, "id_procedimento")
        .filter(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    match parametro_unico(alvo, "acao") {
        Some("procedimento_ciencia") => match modelo::registrar(banco, origem, cenario, id) {
            Ok(Resultado::Registrada { id_atividade }) => Resposta::html(format!(
                "<html><head><script>function inicializar(){{ atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento={id}&procedimento_visualizar_ciencias=1&id_atividade={id_atividade}#ID-{id_atividade}'); }}</script></head></html>"
            )),
            Ok(Resultado::JaRegistrada) => Resposta::html(
                "<html><body><textarea id='txaInfraValidacao' style='display:none'>Usuário já deu ciência no processo hoje.</textarea></body></html>".to_owned()
            ),
            Ok(Resultado::ProcessoNaoEncontrado) => Resposta::nova(Estado::NaoEncontrado),
            Err(_) => Resposta::nova(Estado::DadosInsuficientes),
        },
        Some("protocolo_ciencia_listar") => match modelo::consultar(banco, origem, cenario, id) {
            Ok(Some(Ciencia::Registrada { id_atividade })) => {
                match modelo::lista_invisivel(banco, origem, cenario, id) {
                    Ok(true) => return Resposta::html(
                        "<html><table id='tblAndamentos'><caption>Lista de Ciências:</caption><tr><th>Atividade</th><th>Usuário</th></tr></table></html>".to_owned()
                    ),
                    Ok(false) => {}
                    Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
                }
                let usuario = match modelo::usuario_da_ciencia(banco, origem, cenario, id) {
                    Ok(usuario) => usuario,
                    Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
                };
                let nome = usuario.as_deref().map_or_else(
                    || String::from("<td></td>"),
                    |nome| format!("<td>{}</td>", crate::escapar(nome)),
                );
                Resposta::html(format!(
                    "<html><table id='tblAndamentos'><caption>Lista de Ciências:</caption><tr><th>Atividade</th><th>Usuário</th></tr><tr><td><input type='checkbox' name='chkInfraItem' value='{id_atividade}'/></td>{nome}</tr></table></html>"
                ))
            }
            Ok(Some(Ciencia::Pendente)) => Resposta::nova(Estado::DadosInsuficientes),
            Ok(None) => Resposta::nova(Estado::NaoEncontrado),
            Err(_) => Resposta::nova(Estado::DadosInsuficientes),
        },
        _ => Resposta::nova(Estado::NaoEncontrado),
    }
}
