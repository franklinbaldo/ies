//! A resposta da consulta do autocomplete e a escolha do destino
//! (contracts/cli/processo-enviar.md). Observado nas sete instalações:
//! `application/xml; charset=iso-8859-1`, `<?xml …?>` e
//! `<itens><item id="…" descricao="SIGLA - Nome"></item>…</itens>` (vazio
//! quando nada casa). A sigla é o texto de `descricao` antes de ` - `.
use super::Destino;
use super::erro::{ErroEnviar, FalhaDaConsulta as F};
use crate::formulario_web::{RespostaDoPost, decodificar_atributo};
use crate::html_atributos::valor_do_atributo;
use crate::transporte::{FalhaTransporte, ler_limitado, recusar_status};

/// Mais itens que isso não é a resposta observada (o GO corta em 50).
const MAXIMO_DE_ITENS: usize = 500;

/// `(id, descricao)` de cada `<item>` da resposta, sem rede.
pub(crate) fn ler_resposta_da_consulta<R: std::io::Read>(
    resposta: RespostaDoPost<R>,
) -> Result<Vec<(String, String)>, F> {
    let resposta = resposta.resposta;
    if resposta.status_code != 200 {
        return Err(F::Transporte(recusar_status(resposta)));
    }
    let principal = resposta
        .mime
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if !matches!(principal.as_str(), "application/xml" | "text/xml") {
        return Err(F::RespostaDoAutocompletarNaoXml);
    }
    if resposta
        .content_length
        .is_some_and(|tamanho| tamanho > crate::MAX_HTML_BYTES)
    {
        return Err(F::Transporte(FalhaTransporte::RespostaExcessiva));
    }
    let bytes = ler_limitado(resposta.corpo, crate::MAX_HTML_BYTES).map_err(F::Transporte)?;
    let texto = crate::cassette::decode_http_body(&bytes, &resposta.mime)
        .map_err(|_| F::Transporte(FalhaTransporte::CorpoNaoDecodificavelNoCharsetDeclarado))?;
    itens_do_autocompletar(&texto)
}

/// Os itens de um `<itens>` único; cada `<item>` com `id` e `descricao`.
pub(crate) fn itens_do_autocompletar(xml: &str) -> Result<Vec<(String, String)>, F> {
    let dom = tl::parse(xml, tl::ParserOptions::default())
        .map_err(|_| F::RespostaDoAutocompletarForaDaForma)?;
    let parser = dom.parser();
    let contar = |seletor: &str| dom.query_selector(seletor).map_or(0, Iterator::count);
    if contar("itens") != 1 || contar("erro") != 0 {
        return Err(F::RespostaDoAutocompletarForaDaForma);
    }
    let mut itens = Vec::new();
    for no in dom.query_selector("item").into_iter().flatten() {
        let tag = no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(F::RespostaDoAutocompletarForaDaForma)?;
        let lido = |nome: &'static str| {
            valor_do_atributo(tag, nome)
                .ok()
                .flatten()
                .and_then(|bruto| decodificar_atributo(&bruto))
                .filter(|valor| !valor.trim().is_empty())
                .ok_or(F::RespostaDoAutocompletarForaDaForma)
        };
        itens.push((lido("id")?, lido("descricao")?));
        if itens.len() > MAXIMO_DE_ITENS {
            return Err(F::RespostaDoAutocompletarForaDaForma);
        }
    }
    Ok(itens)
}

/// O único item cuja sigla é `para` (sem distinguir maiúsculas ASCII).
pub(crate) fn escolher_destino(
    para: &str,
    itens: &[(String, String)],
) -> Result<Destino, ErroEnviar> {
    let para = para.trim();
    let achados: Vec<(&String, &String, &str)> = itens
        .iter()
        .filter_map(|(id, descricao)| {
            let (sigla, _) = descricao.split_once(" - ")?;
            let sigla = sigla.trim();
            sigla
                .eq_ignore_ascii_case(para)
                .then_some((id, descricao, sigla))
        })
        .collect();
    match achados.as_slice() {
        [(id, descricao, sigla)] => Ok(Destino {
            sigla: (*sigla).to_owned(),
            descricao: (*descricao).clone(),
            id_unidade: (*id).clone(),
        }),
        [] => Err(ErroEnviar::DestinoNaoOferecidoPeloSei {
            oferecidos: itens.iter().map(|(_, d)| d.clone()).collect(),
        }),
        varios => Err(ErroEnviar::VariosDestinosComASigla {
            candidatos: varios.iter().map(|(_, d, _)| (*d).clone()).collect(),
        }),
    }
}
