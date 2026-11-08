//! Busca sintética de número SEI: rota de pesquisa rápida e estado SQLite.
//! Somente loopback; sem consultas externas ou credenciais verdadeiras.
use crate::{Estado, Resposta, despachar, eh_controlador_sei, parametro_unico};
use iesei::documento_versoes::ids_por_numero;
use iesei::documentos_simulados::ids_por_nup;
use iesei::instalacao::Instalacao;
use rusqlite::Connection;

/// Consulta o numero solicitado pela CLI. Nao cria documento nem
/// escolhe um ID arbitrario se a pesquisa encontra mais de um.
#[must_use]
pub fn responder(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if !eh_controlador_sei(alvo)
        || parametro_unico(alvo, "acao") != Some("protocolo_pesquisa_rapida")
    {
        return despachar(metodo, alvo);
    }
    if metodo != "POST" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Some(numero) = super::documentos::campo_formulario_unico(corpo, "txtPesquisaRapida")
        .filter(|numero| !numero.is_empty())
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    // A pesquisa rápida da CLI recebe NUP pontuado ou número SEI.
    // Primeiro os processos registrados; documentos históricos continuam
    // sendo consultáveis pelo número decimal sem colisão de escopos.
    let (ids, chave) = match ids_por_nup(banco, instalacao, cenario, &numero) {
        Ok(ids) if !ids.is_empty() => (ids, "id_protocolo"),
        Ok(_) if numero.bytes().all(|b| b.is_ascii_digit()) => {
            match ids_por_numero(banco, instalacao, cenario, &numero) {
                Ok(ids) => (ids, "id_documento"),
                Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
            }
        }
        Ok(_) => (Vec::new(), "id_protocolo"),
        Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
    };
    match ids.as_slice() {
        // SQLite parcial não prova a inexistência do NUP/documento.
        [] => Resposta::nova(Estado::DadosInsuficientes),
        [id] => Resposta::redirecionar(format!(
            "controlador.php?acao=procedimento_trabalhar&acao_origem=protocolo_pesquisa_rapida&{chave}={id}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={}",
            "c".repeat(64)
        )),
        _ => Resposta::nova(Estado::RequisicaoInvalida),
    }
}
