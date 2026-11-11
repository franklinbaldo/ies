//! A resposta da consulta do autocomplete e a escolha do destino
//! (contracts/cli/processo-enviar.md). Observado nas sete instalações:
//! `application/xml; charset=iso-8859-1`, `<?xml …?>` e
//! `<itens><item id="…" descricao="SIGLA - Nome"></item>…</itens>` (vazio
//! quando nada casa). A sigla é o texto de `descricao` antes de ` - `.
use super::Destino;
use super::erro::{ErroEnviar, FalhaDaConsulta};
use crate::formulario_web::{RespostaDoPost, decodificar_atributo};
use crate::html_atributos::valor_do_atributo;
use crate::transporte::{FalhaTransporte, ler_limitado, recusar_status};

/// Mais itens que isso não é a resposta observada (o GO corta em 50).
const MAXIMO_DE_ITENS: usize = 500;

/// `(id, descricao)` de cada `<item>` da resposta, sem rede.
pub(crate) fn ler_resposta_da_consulta<R: std::io::Read>(
    resposta: RespostaDoPost<R>,
) -> Result<Vec<(String, String)>, FalhaDaConsulta> {
    let resposta = resposta.resposta;
    if resposta.status_code != 200 {
        return Err(FalhaDaConsulta::Transporte(recusar_status(resposta)));
    }
    let principal = resposta
        .mime
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if !matches!(principal.as_str(), "application/xml" | "text/xml") {
        return Err(FalhaDaConsulta::RespostaDoAutocompletarNaoXml);
    }
    if resposta
        .content_length
        .is_some_and(|tamanho| tamanho > crate::MAX_HTML_BYTES)
    {
        return Err(FalhaDaConsulta::Transporte(
            FalhaTransporte::RespostaExcessiva,
        ));
    }
    let bytes = ler_limitado(resposta.corpo, crate::MAX_HTML_BYTES)?;
    let texto =
        crate::tela_html::HtmlServido::do_corpo(&bytes, &resposta.mime).map_err(|erro| {
            FalhaDaConsulta::Transporte(FalhaTransporte::CorpoNaoDecodificavelNoCharsetDeclarado(
                erro,
            ))
        })?;
    itens_do_autocompletar(&texto)
}

/// Os itens de um `<itens>` único; cada `<item>` com `id` e `descricao`.
pub(crate) fn itens_do_autocompletar(
    xml: &crate::tela_html::HtmlServido,
) -> Result<Vec<(String, String)>, FalhaDaConsulta> {
    let dom = crate::tela_html::interpretar(xml);
    let parser = dom.parser();
    let contar = |seletor: &str| dom.query_selector(seletor).map_or(0, Iterator::count);
    if contar("itens") != 1 || contar("erro") != 0 {
        return Err(FalhaDaConsulta::RespostaDoAutocompletarForaDaFormaObservada);
    }
    let mut itens = Vec::new();
    for no in dom.query_selector("item").into_iter().flatten() {
        let tag = no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(FalhaDaConsulta::RespostaDoAutocompletarForaDaFormaObservada)?;
        let lido = |nome: &'static str| {
            valor_do_atributo(tag, nome)?
                .and_then(|bruto| decodificar_atributo(&bruto))
                .filter(|valor| !valor.trim().is_empty())
                .ok_or(FalhaDaConsulta::RespostaDoAutocompletarForaDaFormaObservada)
        };
        itens.push((lido("id")?, lido("descricao")?));
        if itens.len() > MAXIMO_DE_ITENS {
            return Err(FalhaDaConsulta::RespostaDoAutocompletarForaDaFormaObservada);
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

/// O `value` da única opção do `selOrgao` (fora `""` e `null`) cujo rótulo
/// é o prefixo da sigla antes da primeira `/` (`ABC` de
/// `ABC/GECONT-18141`), comparados com espaços colapsados e sem caixa.
pub(crate) fn orgao_da_sigla<'a>(para: &str, orgaos: &'a [(String, String)]) -> Option<&'a str> {
    let (prefixo, _) = para.split_once('/')?;
    let prefixo = chave(prefixo);
    if prefixo.is_empty() {
        return None;
    }
    let achados: Vec<&str> = orgaos
        .iter()
        .filter(|(valor, rotulo)| !valor.is_empty() && valor != "null" && chave(rotulo) == prefixo)
        .map(|(valor, _)| valor.as_str())
        .collect();
    match achados.as_slice() {
        [valor] => Some(valor),
        _ => None,
    }
}

/// Espaços colapsados, sem caixa.
fn chave(texto: &str) -> String {
    texto
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// O termo alargado de uma sigla com `/`: o último segmento sem o sufixo
/// `-<dígitos>` (`GECONT` de `ABC/GECONT-18141`); nenhum se vazio ou igual
/// à sigla.
pub(crate) fn termo_alargado(para: &str) -> Option<String> {
    let para = para.trim();
    let (_, ultimo) = para.rsplit_once('/')?;
    let sem_numero = match ultimo.rsplit_once('-') {
        Some((antes, numero))
            if !numero.is_empty() && numero.bytes().all(|b| b.is_ascii_digit()) =>
        {
            antes
        }
        _ => ultimo,
    };
    let termo = sem_numero.trim();
    (!termo.is_empty() && termo != para).then(|| termo.to_owned())
}

/// As consultas planejadas e a escolha (decisão em
/// docs/decisoes/processo-enviar.md), a ordem do Todos: a sigla e o termo
/// alargado no órgão do prefixo (quando é outro que o da tela), depois a
/// sigla e o termo alargado no órgão da tela. No máximo quatro, cada uma
/// uma vez; a seguinte só sai se a anterior não trouxe a sigla exata (vazia
/// ou sem ela). Um item exato decide, vários recusam; nenhum em todas:
/// não oferecido, com a última resposta. `consultar(termo, orgao)` faz um
/// POST; uma falha encerra a busca, sem repetir nada.
pub(crate) fn resolver_destino<FalhaDaConsulta>(
    para: &str,
    orgao_da_tela: &str,
    orgaos: &[(String, String)],
    mut consultar: FalhaDaConsulta,
) -> Result<Destino, ErroEnviar>
where
    FalhaDaConsulta: FnMut(&str, &str) -> Result<Vec<(String, String)>, ErroEnviar>,
{
    let para = para.trim();
    let alargado = termo_alargado(para);
    let filtro = orgao_da_sigla(para, orgaos).filter(|f| *f != orgao_da_tela);
    let mut passos: Vec<(&str, &str)> = Vec::with_capacity(4);
    for orgao in filtro.into_iter().chain([orgao_da_tela]) {
        passos.push((para, orgao));
        if let Some(termo) = alargado.as_deref() {
            passos.push((termo, orgao));
        }
    }
    let mut ultima = Vec::new();
    for (termo, orgao) in passos {
        ultima = consultar(termo, orgao)?;
        match escolher_destino(para, &ultima) {
            Err(ErroEnviar::DestinoNaoOferecidoPeloSei { .. }) => {}
            decidido => return decidido,
        }
    }
    escolher_destino(para, &ultima)
}
