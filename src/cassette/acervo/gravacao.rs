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
    pub(crate) item: &'a Value,
}

pub(crate) fn respostas(json: &Value) -> Vec<Resposta<'_>> {
    let Some(lista) = json["interactions"].as_array() else {
        return Vec::new();
    };
    lista
        .iter()
        .enumerate()
        .map(|(passo, item)| Resposta {
            passo,
            operacao: item["request"]["operation"].as_str().unwrap_or(""),
            metodo: item["request"]["method"].as_str().unwrap_or(""),
            status: item["response"]["status_code"].as_u64().unwrap_or(0),
            item,
        })
        .collect()
}

/// Valores do parâmetro `nome` na consulta da URL servida.
pub(crate) fn parametro<'a>(item: &'a Value, nome: &str) -> Vec<&'a str> {
    let url = item["response"]["url"].as_str().unwrap_or("");
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
pub(crate) fn id_procedimento(item: &Value) -> Option<String> {
    decimal_unico(&parametro(item, "id_procedimento"))
}

/// O id da tela aberta pela pesquisa rápida: `id_protocolo`, a mesma
/// chave que `conferir_trabalho` lê do 302 (`OrigemDoTrabalho::PesquisaRapida`).
pub(crate) fn id_da_pesquisa_rapida(item: &Value) -> Option<String> {
    (parametro(item, "acao_origem") == ["protocolo_pesquisa_rapida"])
        .then(|| decimal_unico(&parametro(item, "id_protocolo")))
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

/// O `Location` de um 302 gravado.
pub(crate) fn localizacao(item: &Value) -> Option<&str> {
    let cabecalhos = item["response"]["headers"].as_object()?;
    cabecalhos
        .iter()
        .find(|(nome, _)| nome.eq_ignore_ascii_case("location"))
        .and_then(|(_, valor)| valor.as_str())
}
