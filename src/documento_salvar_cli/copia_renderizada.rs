//! Localiza a seção na foto conferida. Só a foto é analisada. A edição é
//! recortada entre o prefixo e o sufixo originais, sem passar pelo parser.
use super::super::ErroSalvarNoSei;
use crate::tela_html::{Elemento, HtmlServido, arvore};
use std::fmt::Write as _;

const ATRIBUTO: &str = "data-ies-localizacao";

pub(super) struct Secao {
    pub(super) nome: String,
    pub(super) html: String,
}

fn assinatura(e: Elemento<'_>) -> (String, Option<String>) {
    (e.nome().to_owned(), e.atributo("class").map(str::to_owned))
}

/// Marca as aberturas na cópia usada para localizar offsets. O DOM
/// descarta marcas em comentários e reconhece filhos de primeiro nível.
fn marcar(html: &str) -> Option<HtmlServido> {
    if html.contains(ATRIBUTO) {
        return None;
    }
    let mut marcado = String::new();
    let mut anterior = 0;
    for (inicio, _) in html.match_indices('<') {
        let depois = html.get(inicio + 1..)?;
        if !depois.starts_with(|c: char| c.is_ascii_alphabetic()) {
            continue;
        }
        let nome = depois.find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))?;
        let fim = inicio + 1 + nome;
        marcado.push_str(html.get(anterior..fim)?);
        let _ = write!(marcado, " {ATRIBUTO}=\"{inicio}\"");
        anterior = fim;
    }
    marcado.push_str(html.get(anterior..)?);
    HtmlServido::novo(marcado).ok()
}

fn limites_do_corpo(html: &str) -> Option<(usize, usize)> {
    let corpo = html.find("<body")?;
    let inicio = corpo + html.get(corpo..)?.find('>')? + 1;
    let rodape = html.rfind("<div unselectable=\"on\"")?;
    (inicio <= rodape).then_some((inicio, rodape))
}

fn assinaturas(html: &str) -> Option<Vec<(String, Option<String>)>> {
    let fonte = HtmlServido::novo(html.to_owned()).ok()?;
    let dom = arvore(&fonte);
    let corpo = dom.elementos("body").next()?;
    if corpo
        .nos()
        .any(|no| no.como_texto().is_some_and(|s| !s.trim().is_empty()))
    {
        return None;
    }
    Some(corpo.filhos().map(assinatura).collect())
}

fn moldura(html: &str, secoes: &[Secao], principal: &str) -> Option<(usize, usize)> {
    // A visualização junta as seções em ordem, depois acrescenta o rodapé
    // de versão. Os valores de variáveis podem mudar o texto da visualização.
    let (inicio, rodape) = limites_do_corpo(html)?;
    let corpo = marcar(html.get(inicio..rodape)?)?;
    let dom = arvore(&corpo);
    let corpo = dom.elementos("body").next()?;
    if corpo
        .nos()
        .any(|no| no.como_texto().is_some_and(|s| !s.trim().is_empty()))
    {
        return None;
    }
    let elementos: Vec<_> = corpo.filhos().collect();
    let mut atual = 0;
    let mut intervalo = None;
    for secao in secoes {
        let filhos = assinaturas(&secao.html)?;
        let seguinte = atual + filhos.len();
        let renderizados = elementos.get(atual..seguinte)?;
        if !renderizados.iter().copied().map(assinatura).eq(filhos) {
            return None;
        }
        if secao.nome == principal {
            let abertura = renderizados
                .first()?
                .atributo(ATRIBUTO)?
                .parse::<usize>()
                .ok()?
                + inicio;
            let fechamento = match elementos.get(seguinte) {
                Some(e) => e.atributo(ATRIBUTO)?.parse::<usize>().ok()? + inicio,
                None => rodape,
            };
            intervalo = Some((abertura, fechamento));
        }
        atual = seguinte;
    }
    let (abertura, fechamento) = intervalo?;
    if atual != elementos.len() || abertura >= fechamento {
        return None;
    }
    // As quebras entre seções vêm da visualização, não fazem parte do
    // cabeçalho ou do rodapé fixos. A edição pode substituí-las.
    let antes = html.get(..abertura)?.trim_end().len();
    Some((antes, fechamento))
}

/// O HTML malformado pode mudar a árvore da visualização inteira. Uma
/// ocorrência literal basta quando as seções vizinhas também conferem.
fn moldura_literal(html: &str, secoes: &[Secao], principal: &str) -> Option<(usize, usize)> {
    let (inicio, rodape) = limites_do_corpo(html)?;
    let corpo = html.get(inicio..rodape)?;
    let posicao = secoes.iter().position(|s| s.nome == principal)?;
    let (anteriores, seguintes) = secoes.split_at_checked(posicao)?;
    let (secao, seguintes) = seguintes.split_first()?;
    if secao.html.is_empty() {
        return None;
    }
    let mut encontrados = corpo.match_indices(&secao.html);
    let (abertura, _) = encontrados.next()?;
    if encontrados.next().is_some() {
        return None;
    }
    let fechamento = abertura + secao.html.len();
    let iguais = |vizinhanca: &[Secao], trecho: &str| {
        let partes: Option<Vec<_>> = vizinhanca.iter().map(|s| assinaturas(&s.html)).collect();
        Some(
            assinaturas(trecho)?
                .into_iter()
                .eq(partes?.into_iter().flatten()),
        )
    };
    if !iguais(anteriores, corpo.get(..abertura)?)? || !iguais(seguintes, corpo.get(fechamento..)?)?
    {
        return None;
    }
    Some((inicio + abertura, inicio + fechamento))
}

pub(super) fn corpo_editado<'a>(
    base: &str,
    edicao: &'a str,
    secoes: &[Secao],
    principal: &str,
) -> Result<&'a str, ErroSalvarNoSei> {
    let (inicio, fim) = moldura(base, secoes, principal)
        .or_else(|| moldura_literal(base, secoes, principal))
        .ok_or(ErroSalvarNoSei::FotoLocalForaDaForma)?;
    let prefixo = base
        .get(..inicio)
        .ok_or(ErroSalvarNoSei::FotoLocalForaDaForma)?;
    let sufixo = base
        .get(fim..)
        .ok_or(ErroSalvarNoSei::FotoLocalForaDaForma)?;
    edicao
        .strip_prefix(prefixo)
        .and_then(|html| html.strip_suffix(sufixo))
        .ok_or(ErroSalvarNoSei::RegiaoNaoEditavelAlterada)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn usa_a_ordem_das_secoes_e_conserva_a_edicao_opaca() {
        let secoes = [
            Secao {
                nome: "7".into(),
                html: "<p>@unidade@</p>".into(),
            },
            Secao {
                nome: "28".into(),
                html: "<p>Antes</p>".into(),
            },
            Secao {
                nome: "3".into(),
                html: "<hr><table></table>".into(),
            },
        ];
        let inicio = "<body class='infra-doc'><p>Unidade</p>";
        let fim = "<hr><table></table><div unselectable=\"on\"></div></body>";
        let base = format!("{inicio}\r\n<p>Antes</p>{fim}");
        let editado = "<p title=\"aberto\r\r\n<strong>Exemplo";
        let edicao = format!("{inicio}{editado}{fim}");
        assert_eq!(
            corpo_editado(&base, &edicao, &secoes, "28").unwrap(),
            editado
        );
        assert!(matches!(
            corpo_editado(&base, &edicao.replace("Unidade", "Outra"), &secoes, "28"),
            Err(ErroSalvarNoSei::RegiaoNaoEditavelAlterada)
        ));
    }

    #[test]
    fn localiza_html_malformado_anterior_pelo_trecho_literal() {
        let antigo = "<p style=\"color:red\">Antigo<strong> sem fechamento";
        let antes = "<body><div>Unidade</div>\n";
        let depois = "<hr><p>Rodapé</p><div unselectable=\"on\"></div></body>";
        let base = format!("{antes}{antigo}{depois}");
        let secoes = [
            Secao {
                nome: "x".into(),
                html: "<div>@unidade@</div>".into(),
            },
            Secao {
                nome: "y".into(),
                html: antigo.into(),
            },
            Secao {
                nome: "z".into(),
                html: "<hr><p>Rodapé</p>".into(),
            },
        ];
        let editada = format!("{antes}<p>Novo</p>{depois}");
        assert_eq!(
            corpo_editado(&base, &editada, &secoes, "y").unwrap(),
            "<p>Novo</p>"
        );
    }
}
