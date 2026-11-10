//! O que a gravação estabelece sozinha: passos, operação, método, status,
//! parâmetros da URL servida (só nomes e valores decimais saem daqui), a
//! instalação (`sei_root` + `pathname`) e o `Location` de um 302.
use crate::instalacao::Instalacao;
use serde_json::Value;

pub(crate) struct Resposta<'a> {
    pub(crate) passo: usize,
    pub(crate) operacao: &'a str,
    pub(crate) metodo: &'a str,
    pub(crate) status: u64,
    pub(crate) interacao: &'a Value,
}

pub(crate) fn respostas(json: &Value) -> Vec<Resposta<'_>> {
    let Some(lista) = json["interactions"].as_array() else {
        return Vec::new();
    };
    lista
        .iter()
        .enumerate()
        .map(|(passo, interacao)| Resposta {
            passo,
            operacao: interacao["request"]["operation"].as_str().unwrap_or(""),
            metodo: interacao["request"]["method"].as_str().unwrap_or(""),
            status: interacao["response"]["status_code"].as_u64().unwrap_or(0),
            interacao,
        })
        .collect()
}

/// Valores do parâmetro `nome` na consulta da URL servida.
pub(crate) fn parametro<'a>(interacao: &'a Value, nome: &str) -> Vec<&'a str> {
    let url = interacao["response"]["url"].as_str().unwrap_or("");
    let consulta = url.split_once('?').map_or("", |(_, c)| c);
    consulta
        .split('&')
        .filter_map(|par| par.split_once('='))
        .filter(|(chave, _)| *chave == nome)
        .map(|(_, valor)| valor)
        .collect()
}

/// O `id_procedimento` da URL servida: um único valor decimal (repetido
/// igual é aceito, como em `arvore_visualizar`).
pub(crate) fn id_procedimento(interacao: &Value) -> Option<String> {
    decimal_unico(&parametro(interacao, "id_procedimento"))
}

/// O id da tela aberta pela pesquisa rápida: `id_protocolo`, a mesma
/// chave que `conferir_trabalho` lê do 302 (`OrigemDoTrabalho::PesquisaRapida`).
pub(crate) fn id_da_pesquisa_rapida(interacao: &Value) -> Option<String> {
    (parametro(interacao, "acao_origem") == ["protocolo_pesquisa_rapida"])
        .then(|| decimal_unico(&parametro(interacao, "id_protocolo")))
        .flatten()
}

fn decimal_unico(valores: &[&str]) -> Option<String> {
    let primeiro = *valores.first()?;
    let decimal = !primeiro.is_empty() && primeiro.bytes().all(|b| b.is_ascii_digit());
    (decimal && valores.iter().all(|v| *v == primeiro)).then(|| primeiro.to_owned())
}

/// A instalação da gravação: `sei_root` e o caminho de `controlador.php`.
pub(crate) fn instalacao(json: &Value) -> Option<Instalacao> {
    let raiz = json["sei_root"].as_str()?;
    let caminho = json["interactions"]
        .as_array()?
        .iter()
        .filter_map(|it| it["request"]["pathname"].as_str())
        .find(|c| c.ends_with("/controlador.php"))?;
    let base = caminho.strip_suffix("/controlador.php")?;
    Instalacao::interpretar(&format!("{raiz}{base}"))
}

/// A URL (sem consulta, que não é gravada) da requisição de `interacao`.
pub(crate) fn url_sem_consulta(json: &Value, interacao: &Value) -> String {
    let raiz = json["sei_root"].as_str().unwrap_or_default();
    let caminho = interacao["request"]["pathname"]
        .as_str()
        .unwrap_or_default();
    format!("{raiz}{caminho}")
}

/// O `Location` de um 302 gravado.
pub(crate) fn localizacao(interacao: &Value) -> Option<&str> {
    let cabecalhos = interacao["response"]["headers"].as_object()?;
    cabecalhos
        .iter()
        .find(|(nome, _)| nome.eq_ignore_ascii_case("location"))
        .and_then(|(_, valor)| valor.as_str())
}

/// O corpo gravado (base64), vazio se ausente ou ilegível.
pub(crate) fn corpo(interacao: &Value) -> Vec<u8> {
    use base64::Engine as _;
    interacao["response"]["body_base64"]
        .as_str()
        .and_then(|b| base64::engine::general_purpose::STANDARD.decode(b).ok())
        .unwrap_or_default()
}

/// O `Content-Type` gravado, vazio se ausente.
pub(crate) fn mime(interacao: &Value) -> String {
    interacao["response"]["headers"]
        .as_object()
        .and_then(|h| {
            h.iter()
                .find(|(nome, _)| nome.eq_ignore_ascii_case("content-type"))
                .and_then(|(_, v)| v.as_str())
        })
        .unwrap_or("")
        .to_owned()
}
