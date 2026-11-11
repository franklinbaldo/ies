//! Usuário indicado na mesma linha da atividade comprovada pela Lista de Ciências.
//! Ausência de cabeçalho explícito ou de nome continua desconhecida, não é
//! preenchida a partir da revisão do documento ou de outra atividade.
use super::{ErroCiencia, atividade_na_lista};
use crate::html_atributos::{nos_com_id, valor_do_atributo};

pub(crate) fn usuario_da_atividade_na_lista(
    html: &str,
    atividade: &str,
) -> Result<Option<String>, ErroCiencia> {
    // O comprovante já valida tabela única, legenda, atividade única e
    // ausência de contêineres inertes. A autoria nunca substitui essa prova.
    if !atividade_na_lista(html, atividade)? {
        return Ok(None);
    }
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(ErroCiencia::ListaDeCienciasNaoInterpretada)?;
    let parser = dom.parser();
    let tabelas = nos_com_id(&dom, "tblAndamentos").map_err(ErroCiencia::Atributo)?;
    let [unica] = tabelas.as_slice() else {
        return Err(ErroCiencia::ListaDeCienciasForaDaFormaObservada);
    };
    let tabela = unica
        .get(parser)
        .and_then(tl::Node::as_tag)
        .ok_or(ErroCiencia::ListaDeCienciasForaDaFormaObservada)?;

    // Não presumir que a segunda coluna é o nome: nas instalações
    // reais a disposição pode variar ou sequer estar identificada.
    let coluna_usuario = tabela
        .query_selector(parser, "tr")
        .into_iter()
        .flatten()
        .filter_map(|no| no.get(parser).and_then(tl::Node::as_tag))
        .find_map(|linha| {
            linha
                .query_selector(parser, "th")
                .into_iter()
                .flatten()
                .filter_map(|no| no.get(parser).and_then(tl::Node::as_tag))
                .enumerate()
                .find_map(|(indice, cabecalho)| {
                    let titulo = html_escape::decode_html_entities(&cabecalho.inner_text(parser))
                        .into_owned();
                    titulo
                        .trim()
                        .eq_ignore_ascii_case("usuário")
                        .then_some(indice)
                })
        });
    let Some(coluna_usuario) = coluna_usuario else {
        return Ok(None);
    };

    for no in tabela.query_selector(parser, "tr").into_iter().flatten() {
        let Some(linha) = no.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        let contem_atividade = linha
            .query_selector(parser, "input")
            .into_iter()
            .flatten()
            .filter_map(|no| no.get(parser).and_then(tl::Node::as_tag))
            .any(|input| {
                let tipo = valor_do_atributo(input, "type").ok().flatten();
                let nome = valor_do_atributo(input, "name").ok().flatten();
                let valor = valor_do_atributo(input, "value").ok().flatten();
                tipo.as_deref() == Some("checkbox")
                    && nome.as_deref().is_some_and(|nome| {
                        nome.strip_prefix("chkInfraItem")
                            .is_some_and(|sufixo| sufixo.bytes().all(|b| b.is_ascii_digit()))
                    })
                    && valor.as_deref() == Some(atividade)
            });
        if !contem_atividade {
            continue;
        }
        let nome = linha
            .query_selector(parser, "td")
            .into_iter()
            .flatten()
            .filter_map(|no| no.get(parser).and_then(tl::Node::as_tag))
            .nth(coluna_usuario)
            .map(|celula| {
                html_escape::decode_html_entities(&celula.inner_text(parser))
                    .into_owned()
                    .trim()
                    .to_owned()
            });
        return Ok(nome.filter(|nome| !nome.is_empty()));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nome_vem_da_mesma_linha_e_da_coluna_identificada() {
        let html = "<table id='tblAndamentos'><caption>Lista de Ciências:</caption>
            <tr><th>Atividade</th><th>Unidade</th><th>Usuário</th></tr>
            <tr><td><input type='checkbox' name='chkInfraItem' value='88'></td><td>OUTRA</td><td>Outro</td></tr>
            <tr><td><input type='checkbox' name='chkInfraItem' value='42'></td><td>TESTE</td><td>Agente &amp; Cia</td></tr>
            </table>";
        assert_eq!(
            usuario_da_atividade_na_lista(html, "42")
                .unwrap()
                .as_deref(),
            Some("Agente & Cia")
        );
    }

    #[test]
    fn sem_cabecalho_nao_inventa_usuario_de_outra_coluna() {
        let html = "<table id='tblAndamentos'><caption>Lista de Ciências:</caption>
            <tr><td><input type='checkbox' name='chkInfraItem' value='42'></td>
            <td>Unidade</td><td>Revisor</td></tr></table>";
        assert_eq!(usuario_da_atividade_na_lista(html, "42").unwrap(), None);
    }
}
