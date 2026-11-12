//! Lista de Ciências relida depois do GET mutador: a atividade do aceite
//! tem de aparecer uma única vez no `table#tblAndamentos` publicado.
use super::{ErroCiencia, identificador};
use crate::legenda::ler_legenda;

/// Verifica a atividade no único `table#tblAndamentos` da Lista de Ciências
/// que o processo/documento publicou na **releitura**. O chamador deve antes
/// conferir a URL da lista contra o alvo e a sessão: o HTML isolado não prova
/// qual processo serviu a tabela. Não deduz usuário nem data não observados.
pub(crate) fn atividade_na_lista(
    html: &crate::tela_html::HtmlServido,
    atividade: &str,
) -> Result<bool, ErroCiencia> {
    if !identificador(atividade) {
        return Err(ErroCiencia::ListaDeCienciasForaDaFormaObservada);
    }
    let dom = crate::tela_html::arvore(html);
    let tabelas = dom.com_id("tblAndamentos").collect::<Vec<_>>();
    let [tabela] = tabelas.as_slice() else {
        return Err(ErroCiencia::ListaDeCienciasForaDaFormaObservada);
    };
    if tabela.nome() != "table" {
        return Err(ErroCiencia::ListaDeCienciasForaDaFormaObservada);
    }
    // O SEI depende de JavaScript ativo: como no navegador, o conteúdo de
    // <template> fica fora da árvore e o de <noscript> é texto, então uma
    // tabela sob eles não é achada. Um deles dentro da tabela recusa a
    // lista: uma atividade antiga num nó inerte não prova o efeito do GET.
    for seletor in ["template", "noscript"] {
        if tabela.elementos(seletor).next().is_some() {
            return Err(ErroCiencia::ListaDeCienciasForaDaFormaObservada);
        }
    }
    let legendas: Vec<String> = tabela.elementos("caption").map(|no| no.texto()).collect();
    let [legenda] = legendas.as_slice() else {
        return Err(ErroCiencia::ListaDeCienciasForaDaFormaObservada);
    };
    // Com ou sem contagem: `UnB` 4.0.9 publica "Lista de Ciências (1 registro):".
    let contagem = legenda
        .strip_prefix("Lista de Ciências")
        .and_then(|resto| resto.strip_suffix(':'))
        .ok_or(ErroCiencia::ListaDeCienciasForaDaFormaObservada)?;
    if !contagem.is_empty() {
        ler_legenda(legenda)?;
    }
    let mut ocorrencias = 0;
    for no in tabela.elementos("input") {
        let tipo = no.atributo("type");
        if tipo != Some("checkbox") {
            continue;
        }
        let nome = no.atributo("name").map(str::to_owned);
        let valor = no.atributo("value");
        let Some(nome) = nome else {
            return Err(ErroCiencia::ListaDeCienciasForaDaFormaObservada);
        };
        let Some(resto) = nome.strip_prefix("chkInfraItem") else {
            continue;
        };
        if !resto.bytes().all(|c| c.is_ascii_digit()) {
            return Err(ErroCiencia::ListaDeCienciasForaDaFormaObservada);
        }
        if valor == Some(atividade) {
            ocorrencias += 1;
        }
    }
    match ocorrencias {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(ErroCiencia::AtividadesRepetidasNaLista),
    }
}
