//! Usuário indicado na mesma linha da atividade comprovada pela Lista de Ciências.
//! Ausência de cabeçalho explícito ou de nome continua desconhecida, não é
//! preenchida a partir da revisão do documento ou de outra atividade.
use super::{ErroCiencia, atividade_na_lista};

pub(crate) fn usuario_da_atividade_na_lista(
    html: &crate::tela_html::HtmlServido,
    atividade: &str,
) -> Result<Option<String>, ErroCiencia> {
    // O comprovante já valida tabela única, legenda, atividade única e
    // ausência de contêineres inertes. A autoria nunca substitui essa prova.
    if !atividade_na_lista(html, atividade)? {
        return Ok(None);
    }
    let dom = crate::tela_html::arvore(html);
    let tabelas = dom.com_id("tblAndamentos").collect::<Vec<_>>();
    let [unica] = tabelas.as_slice() else {
        return Err(ErroCiencia::ListaDeCienciasForaDaFormaObservada);
    };
    let tabela = *unica;

    // Não presumir que a segunda coluna é o nome: nas instalações
    // reais a disposição pode variar ou sequer estar identificada.
    let coluna_usuario = tabela.elementos("tr").find_map(|linha| {
        linha
            .elementos("th")
            .enumerate()
            .find_map(|(indice, cabecalho)| {
                let titulo = cabecalho.texto();
                titulo
                    .trim()
                    .eq_ignore_ascii_case("usuário")
                    .then_some(indice)
            })
    });
    let Some(coluna_usuario) = coluna_usuario else {
        return Ok(None);
    };

    for no in tabela.elementos("tr") {
        let contem_atividade = no.elementos("input").any(|input| {
            let tipo = input.atributo("type");
            let nome = input.atributo("name").map(str::to_owned);
            let valor = input.atributo("value");
            tipo == Some("checkbox")
                && nome.as_deref().is_some_and(|nome| {
                    nome.strip_prefix("chkInfraItem")
                        .is_some_and(|sufixo| sufixo.bytes().all(|b| b.is_ascii_digit()))
                })
                && valor == Some(atividade)
        });
        if !contem_atividade {
            continue;
        }
        let nome = no
            .elementos("td")
            .nth(coluna_usuario)
            .map(|celula| celula.texto().trim().to_owned());
        return Ok(nome.filter(|nome| !nome.is_empty()));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nome_vem_da_mesma_linha_e_da_coluna_identificada() {
        let html = crate::tela_html::HtmlServido::de_teste("<table id='tblAndamentos'><caption>Lista de Ciências:</caption>
            <tr><th>Atividade</th><th>Unidade</th><th>Usuário</th></tr>
            <tr><td><input type='checkbox' name='chkInfraItem' value='88'></td><td>OUTRA</td><td>Outro</td></tr>
            <tr><td><input type='checkbox' name='chkInfraItem' value='42'></td><td>TESTE</td><td>Agente &amp; Cia</td></tr>
            </table>");
        assert_eq!(
            usuario_da_atividade_na_lista(&html, "42")
                .unwrap()
                .as_deref(),
            Some("Agente & Cia")
        );
    }

    #[test]
    fn sem_cabecalho_nao_inventa_usuario_de_outra_coluna() {
        let html = crate::tela_html::HtmlServido::de_teste(
            "<table id='tblAndamentos'><caption>Lista de Ciências:</caption>
            <tr><td><input type='checkbox' name='chkInfraItem' value='42'></td>
            <td>Unidade</td><td>Revisor</td></tr></table>",
        );
        assert_eq!(usuario_da_atividade_na_lista(&html, "42").unwrap(), None);
    }
}
