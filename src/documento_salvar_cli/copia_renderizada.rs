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

// Uma assinatura de tag/classe não identifica a seção: <p>A</p> e
// <p>B</p> têm a mesma forma. Quando há um trecho literal único na foto,
// ele precisa pertencer à região correspondente. Placeholders de
// visualização podem não aparecer literalmente, por isso não são âncoras.
fn ancora_literal(
    html: &str,
    corpo: (usize, usize),
    secao: &str,
    regiao: (usize, usize),
) -> Option<bool> {
    if secao.is_empty() {
        return Some(false);
    }
    let trecho = html.get(corpo.0..corpo.1)?;
    let mut ocorrencias = trecho.match_indices(secao);
    let Some((posicao, _)) = ocorrencias.next() else {
        return Some(false);
    };
    if ocorrencias.next().is_some() {
        return Some(false);
    }
    let posicao = corpo.0 + posicao;
    (regiao.0 <= posicao && posicao + secao.len() <= regiao.1).then_some(true)
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
    let mut provas = Vec::new();
    for secao in secoes {
        // Uma seção vizinha literalmente vazia não contribui elementos
        // à foto. A principal ainda precisa de uma região identificável.
        if secao.nome != principal && secao.html.is_empty() {
            continue;
        }
        let filhos = assinaturas(&secao.html)?;
        let seguinte = atual + filhos.len();
        let renderizados = elementos.get(atual..seguinte)?;
        if !renderizados
            .iter()
            .copied()
            .map(assinatura)
            .eq(filhos.iter().cloned())
        {
            return None;
        }
        let abertura_secao = renderizados
            .first()?
            .atributo(ATRIBUTO)?
            .parse::<usize>()
            .ok()?
            + inicio;
        let fechamento_secao = match elementos.get(seguinte) {
            Some(e) => e.atributo(ATRIBUTO)?.parse::<usize>().ok()? + inicio,
            None => rodape,
        };
        let ancorada = ancora_literal(
            html,
            (inicio, rodape),
            &secao.html,
            (abertura_secao, fechamento_secao),
        )?;
        provas.push((filhos, ancorada));
        if secao.nome == principal {
            intervalo = Some((abertura_secao, fechamento_secao));
        }
        atual = seguinte;
    }
    // Grupos de seções indistinguíveis por tag/classe precisam de
    // âncoras literais únicas para todas, exceto no máximo uma. Sem
    // essas âncoras a posição da principal seria apenas uma suposição.
    for (forma, _) in &provas {
        let total = provas.iter().filter(|(outra, _)| outra == forma).count();
        let ancoradas = provas
            .iter()
            .filter(|(outra, ancorada)| outra == forma && *ancorada)
            .count();
        if total > 1 && ancoradas + 1 < total {
            return None;
        }
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
    fn localiza_principal_renderizada_entre_secoes_vizinhas_vazias() {
        let secoes = [
            Secao {
                nome: "cabecalho".into(),
                html: "<h1>Fixo</h1>".into(),
            },
            Secao {
                nome: "vazia_antes".into(),
                html: String::new(),
            },
            Secao {
                nome: "principal".into(),
                html: "<p>@unidade@</p>".into(),
            },
            Secao {
                nome: "vazia_depois".into(),
                html: String::new(),
            },
            Secao {
                nome: "rodape".into(),
                html: "<hr>".into(),
            },
        ];
        let antes = "<body><h1>Fixo</h1>";
        let depois = "<hr><div unselectable=\"on\"></div></body>";
        let base = format!("{antes}<p>Unidade</p>{depois}");
        let corpo = "<p><strong>Novo\r\r\nsem fechamento";
        let edicao = format!("{antes}{corpo}{depois}");
        assert_eq!(
            corpo_editado(&base, &edicao, &secoes, "principal").unwrap(),
            corpo
        );
        assert!(matches!(
            corpo_editado(
                &base,
                &edicao.replace("Fixo", "Alterado"),
                &secoes,
                "principal"
            ),
            Err(ErroSalvarNoSei::RegiaoNaoEditavelAlterada)
        ));
        // Sem uma região renderizada não há posição da principal vazia.
        assert!(matches!(
            corpo_editado(&base, &edicao, &secoes, "vazia_antes"),
            Err(ErroSalvarNoSei::FotoLocalForaDaForma)
        ));
    }

    #[test]
    fn recusa_secoes_de_mesma_forma_com_conteudo_trocado() {
        let secoes = [
            Secao {
                nome: "vizinha".into(),
                html: "<p>Vizinha</p>".into(),
            },
            Secao {
                nome: "principal".into(),
                html: "<p>Principal</p>".into(),
            },
        ];
        let base = "<body><p>Principal</p><p>Vizinha</p><div unselectable=\"on\"></div></body>";
        let edicao = "<body><p>Principal</p><p>Alterada</p><div unselectable=\"on\"></div></body>";
        assert!(matches!(
            corpo_editado(base, edicao, &secoes, "principal"),
            Err(ErroSalvarNoSei::FotoLocalForaDaForma)
        ));
    }

    #[test]
    fn recusa_duas_secoes_sem_ancora_literal_na_mesma_forma() {
        let secoes = [
            Secao {
                nome: "vizinha".into(),
                html: "<p>@unidade@</p>".into(),
            },
            Secao {
                nome: "principal".into(),
                html: "<p>@destinatario@</p>".into(),
            },
        ];
        let base = "<body><p>Unidade</p><p>Destinatário</p><div unselectable=\"on\"></div></body>";
        let edicao = "<body><p>Unidade</p><p>Outro</p><div unselectable=\"on\"></div></body>";
        assert!(matches!(
            corpo_editado(base, edicao, &secoes, "principal"),
            Err(ErroSalvarNoSei::FotoLocalForaDaForma)
        ));
    }

    #[test]
    fn localiza_html_malformado_anterior_pelo_trecho_literal() {
        let antigo = "<div style=\"color:red\">Antigo<strong> sem fechamento";
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
