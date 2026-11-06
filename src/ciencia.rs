//! Ciência Web de processo/documento: identifica a atividade do aceite e
//! prova o registro na lista de ciências **novamente lida**.
//!
//! Observado em Todos 323eb01, `tests/test_ciclo_ciencia_010.py` e
//! `tests/test_documento_ciente_010.py`; os arquivos da cassette do IES
//! referenciam capturas `UnB` (4.0), UFRJ (4.1) e PEN3 (5.0).
//! O GET de ciência é ESCRITA e não pode ser feito durante a prévia.
//! Nenhum JavaScript é executado, e nenhuma URL assinada é publicada.

use crate::html_atributos::{nos_com_id, valor_do_atributo};
mod fluxo;
mod documento;
pub(crate) use documento::{CienciaDoDocumento, ErroCienciaDocumento, PedidoDaCienciaDocumento, ciencia_no_documento};
pub(crate) use fluxo::{
    CienciaDoProcesso, ErroCienciaProcesso, PedidoDaCiencia, ciencia_no_processo,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AlvoDaCiencia<'a> {
    Processo {
        id_procedimento: &'a str,
    },
    Documento {
        id_procedimento: &'a str,
        id_documento: &'a str,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroCiencia {
    AceiteSemAtividade,
    AtividadesAmbiguas,
    ListaDeCienciasForaDaFormaObservada,
    AtividadesRepetidasNaLista,
}

impl ErroCiencia {
    pub(crate) const fn subcodigo(self) -> &'static str {
        match self {
            Self::AceiteSemAtividade => "aceite_da_ciencia_sem_atividade",
            Self::AtividadesAmbiguas => "atividades_da_ciencia_ambiguas",
            Self::ListaDeCienciasForaDaFormaObservada => {
                "lista_de_ciencias_fora_da_forma_observada"
            }
            Self::AtividadesRepetidasNaLista => "atividades_repetidas_na_lista_de_ciencias",
        }
    }
}

/// Só admite o valor da chave com uma única ocorrência. Uma chave duplicada
/// não permite escolher o primeiro nem o último valor.
fn parametro_unico<'a>(consulta: &'a str, chave: &str) -> Option<&'a str> {
    let mut valores = consulta
        .split('&')
        .filter_map(|parte| parte.split_once('='))
        .filter(|(nome, _)| *nome == chave)
        .map(|(_, valor)| valor);
    match (valores.next(), valores.next()) {
        (Some(valor), None) => Some(valor),
        _ => None,
    }
}

fn identificador(valor: &str) -> bool {
    !valor.is_empty() && valor.len() <= 20 && valor.bytes().all(|b| b.is_ascii_digit())
}

/// Literais como primeiro argumento de `atualizarArvore('...')`. Chamadas com
/// variável (PEN3) são ignoradas: não identificam uma atividade. Nenhum JS
/// é interpretado; a forma literal não comporta escapes.
fn literais_de_atualizar_arvore(script: &str) -> Vec<&str> {
    let mut achados = Vec::new();
    let mut restante = script;
    while let Some((_, depois)) = restante.split_once("atualizarArvore(") {
        restante = depois;
        let argumento = depois.trim_start();
        let aspas = match argumento.as_bytes().first() {
            Some(b'\'') => '\'',
            Some(b'"') => '"',
            _ => continue,
        };
        let resto = &argumento[1..];
        let Some(fim) = resto.find(aspas) else {
            continue;
        };
        let literal = &resto[..fim];
        let depois_do_literal = resto[fim + 1..].trim_start();
        if !literal.contains('\\')
            && (depois_do_literal.starts_with(')') || depois_do_literal.starts_with(','))
        {
            achados.push(literal);
        }
    }
    achados
}

fn atividade_do_endereco(url: &str, alvo: AlvoDaCiencia<'_>) -> Option<String> {
    let (caminho, restante) = url.split_once('?')?;
    if caminho != "controlador.php" || url.len() > 8_192 {
        return None;
    }
    let consulta = restante.split('#').next()?;
    let (id_procedimento, marca, id_documento) = match alvo {
        AlvoDaCiencia::Processo { id_procedimento } => {
            (id_procedimento, "procedimento_visualizar_ciencias", None)
        }
        AlvoDaCiencia::Documento {
            id_procedimento,
            id_documento,
        } => (
            id_procedimento,
            "documento_visualizar_ciencias",
            Some(id_documento),
        ),
    };
    if parametro_unico(consulta, "acao") != Some("procedimento_visualizar")
        || parametro_unico(consulta, "id_procedimento") != Some(id_procedimento)
        || parametro_unico(consulta, marca) != Some("1")
        || id_documento.is_some_and(|doc| parametro_unico(consulta, "id_documento") != Some(doc))
        || (id_documento.is_none() && parametro_unico(consulta, "id_documento").is_some())
    {
        return None;
    }
    let atividade = parametro_unico(consulta, "id_atividade")?;
    identificador(atividade).then(|| atividade.to_owned())
}

/// Extrai a atividade exata que o SEI indicou na recarga da árvore.
///
/// Só literais de `atualizarArvore` dentro de scripts e com identidade do
/// processo/documento coerente contam. Duas atividades distintas recusam;
/// a ausência não é confundida com sucesso ou ciência inexistente.
pub(crate) fn atividade_do_aceite(
    html: &str,
    alvo: AlvoDaCiencia<'_>,
) -> Result<String, ErroCiencia> {
    use ErroCiencia as E;
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| E::AceiteSemAtividade)?;
    let parser = dom.parser();
    let mut atividades = Vec::new();
    for script in dom.query_selector("script").into_iter().flatten() {
        let Some(tag) = script.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        for url in literais_de_atualizar_arvore(&tag.inner_text(parser)) {
            let Some(atividade) = atividade_do_endereco(url, alvo) else {
                continue;
            };
            if !atividades.contains(&atividade) {
                atividades.push(atividade);
            }
        }
    }
    match atividades.as_slice() {
        [] => Err(E::AceiteSemAtividade),
        [atividade] => Ok(atividade.clone()),
        _ => Err(E::AtividadesAmbiguas),
    }
}

/// Verifica a atividade no único `table#tblAndamentos` da Lista de Ciências
/// que o processo/documento publicou na **releitura**. O chamador deve antes
/// conferir a URL da lista contra o alvo e a sessão: o HTML isolado não prova
/// qual processo serviu a tabela. Não deduz usuário nem data não observados.
pub(crate) fn atividade_na_lista(html: &str, atividade: &str) -> Result<bool, ErroCiencia> {
    use ErroCiencia as E;
    if !identificador(atividade) {
        return Err(E::ListaDeCienciasForaDaFormaObservada);
    }
    let fora = E::ListaDeCienciasForaDaFormaObservada;
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| fora)?;
    let parser = dom.parser();
    let tabelas = nos_com_id(&dom, "tblAndamentos").map_err(|_| fora)?;
    let [unica] = tabelas.as_slice() else {
        return Err(fora);
    };
    let tabela = unica.get(parser).and_then(tl::Node::as_tag).ok_or(fora)?;
    if tabela.name().as_utf8_str() != "table" {
        return Err(fora);
    }
    let legendas: Vec<String> = tabela
        .query_selector(parser, "caption")
        .into_iter()
        .flatten()
        .filter_map(|no| no.get(parser))
        .map(|no| no.inner_text(parser).trim().to_owned())
        .collect();
    if legendas.as_slice() != ["Lista de Ciências:"] {
        return Err(fora);
    }
    let mut ocorrencias = 0;
    for no in tabela.query_selector(parser, "input").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
            return Err(fora);
        };
        let tipo = valor_do_atributo(tag, "type").map_err(|_| fora)?;
        if tipo.as_deref() != Some("checkbox") {
            continue;
        }
        let nome = valor_do_atributo(tag, "name").map_err(|_| fora)?;
        let valor = valor_do_atributo(tag, "value").map_err(|_| fora)?;
        let Some(nome) = nome else {
            return Err(fora);
        };
        let Some(resto) = nome.strip_prefix("chkInfraItem") else {
            continue;
        };
        if !resto.bytes().all(|c| c.is_ascii_digit()) {
            return Err(fora);
        }
        if valor.as_deref() == Some(atividade) {
            ocorrencias += 1;
        }
    }
    match ocorrencias {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(E::AtividadesRepetidasNaLista),
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_fluxo;
#[cfg(test)]
mod tests_documento;
