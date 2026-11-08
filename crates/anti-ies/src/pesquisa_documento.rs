//! Busca sintética de número SEI: rota de pesquisa rápida e estado SQLite.
//! Somente loopback; sem consultas externas ou credenciais verdadeiras.
use crate::{Estado, Resposta, despachar, eh_controlador_sei, parametro_unico};
use iesei::documento_versoes::ids_por_numero;
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
    let mut numeros = corpo
        .split('&')
        .filter_map(|campo| campo.split_once('='))
        .filter(|(nome, _)| *nome == "txtPesquisaRapida")
        .map(|(_, valor)| valor);
    let Some(numero) = numeros.next() else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    if numeros.next().is_some()
        || numero.is_empty()
        || !numero.bytes().all(|b| b.is_ascii_digit())
    {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    match ids_por_numero(banco, instalacao, cenario, numero) {
        Ok(ids) => match ids.as_slice() {
            [] => Resposta::nova(Estado::NaoEncontrado),
            [id] => Resposta::redirecionar(format!(
                "controlador.php?acao=procedimento_trabalhar&acao_origem=protocolo_pesquisa_rapida&id_protocolo={id}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={}",
                "c".repeat(64)
            )),
            _ => Resposta::nova(Estado::RequisicaoInvalida),
        },
        Err(_) => Resposta::nova(Estado::DadosInsuficientes),
    }
}
