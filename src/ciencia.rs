//! Ciência Web de processo/documento: identifica a atividade do aceite e
//! prova o registro na lista de ciências **novamente lida**.
//!
//! Observado em Todos 323eb01, `tests/test_ciclo_ciencia_010.py` e
//! `tests/test_documento_ciente_010.py`; os arquivos da cassette do IES
//! referenciam capturas `UnB` (4.0), UFRJ (4.1) e PEN3 (5.0).
//! O GET de ciência é ESCRITA e não pode ser feito durante a prévia.
//! Nenhum JavaScript é executado, e nenhuma URL assinada é publicada.

mod documento;
mod fluxo;
mod lista;
mod lista_autoria;
pub(crate) use documento::{CienciaDoDocumento, PedidoDaCienciaDocumento, ciencia_no_documento};
#[cfg(test)]
pub(crate) use fluxo::ErroCienciaProcesso;
pub(crate) use fluxo::{CienciaDoProcesso, PedidoDaCiencia, ciencia_no_processo};
pub(crate) use lista::atividade_na_lista;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroCiencia {
    /// O tokenizador de atributos recusou a tag.
    Atributo(#[source] crate::html_atributos::ErroAtributoHtml),
    AceiteSemAtividade,
    AtividadesAmbiguas,
    ListaDeCienciasForaDaFormaObservada,
    AtividadesRepetidasNaLista,
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
    !valor.is_empty() && valor.bytes().all(|b| b.is_ascii_digit())
}

/// A resposta de ciência observada recarrega a árvore diretamente no
/// script ou no corpo de `function inicializar()`. A profundidade do lexer
/// sozinha não basta: um `if(false)` também tem profundidade 1.
/// Só um comando independente dessas duas formas fornece prova.
fn separador_vazio(byte: u8) -> bool {
    byte.is_ascii_whitespace() || byte == b';'
}

fn chamada_na_forma_do_aceite(script: &str, niveis: &[Option<usize>], posicao: usize) -> bool {
    let bytes = script.as_bytes();
    let Some(nivel) = niveis.get(posicao).copied().flatten() else {
        return false;
    };
    if nivel > 1 {
        return false;
    }
    // O texto imediatamente anterior não pode ser uma condição, atribuição,
    // retorno ou outra expressão que contenha a chamada. Comentários e
    // literais são ignorados pelo lexer; não se interpreta JavaScript.
    let inicio_comando = (0..posicao)
        .rev()
        .find(|&i| niveis[i].is_some() && matches!(bytes[i], b';' | b'{' | b'}'))
        .map_or(0, |i| i + 1);
    if (inicio_comando..posicao).any(|i| niveis[i].is_some() && !bytes[i].is_ascii_whitespace()) {
        return false;
    }
    if nivel == 0 {
        // Chamada direta só prova recarga quando é a primeira instrução
        // executável do script: um throw anterior pode torná-la inalcançável.
        return (0..posicao).all(|i| niveis[i] != Some(0) || separador_vazio(bytes[i]));
    }

    // Profundidade 1 só é aceita no corpo direto da função de inicialização
    // publicada pelo SEI, e nunca em um if, callback ou função arbitrária.
    let Some(abertura) = (0..posicao)
        .rev()
        .find(|&i| bytes[i] == b'{' && niveis[i] == Some(0))
    else {
        return false;
    };
    let inicio_declaracao = (0..abertura)
        .rev()
        .find(|&i| niveis[i].is_some() && matches!(bytes[i], b';' | b'{' | b'}'))
        .map_or(0, |i| i + 1);
    let assinatura: Vec<u8> = (inicio_declaracao..abertura)
        .filter(|&i| niveis[i].is_some() && !bytes[i].is_ascii_whitespace())
        .map(|i| bytes[i])
        .collect();
    // O corpo observado começa com a recarga. Não interpretar fluxo JS:
    // return, throw, condicionais ou qualquer outra instrução anterior
    // tornam o alcance da chamada incerto, mesmo após um ponto e vírgula.
    assinatura == b"functioninicializar()" && so_chamadas_simples(script, abertura + 1, posicao)
}

/// Antes da recarga, o corpo de `inicializar()` só pode ter instruções
/// sem desvio de fluxo (`UnB` 4.0.9: `exibirLinkVoltarArvoreVisualizar();`;
/// SEI 5: `HTML = "";` e `exibirVoltarAcoes(true);`). Retorno, exceção,
/// condição, laço ou bloco tornam o alcance da recarga incerto.
fn so_chamadas_simples(script: &str, inicio: usize, fim: usize) -> bool {
    const DESVIOS: [&str; 12] = [
        "return", "throw", "if", "else", "for", "while", "do", "switch", "try", "catch", "break",
        "continue",
    ];
    // Sem comentários; literais ficam, mas não decidem o fluxo.
    let mut ativo = String::new();
    let mut resto = &script[inicio..fim];
    while !resto.is_empty() {
        if let Some(depois) = resto.strip_prefix("//") {
            resto = depois.split_once('\n').map_or("", |(_, r)| r);
        } else if let Some(depois) = resto.strip_prefix("/*") {
            let Some((_, r)) = depois.split_once("*/") else {
                return false;
            };
            resto = r;
        } else {
            let mut letras = resto.chars();
            ativo.extend(letras.next());
            resto = letras.as_str();
        }
    }
    !ativo.contains(['{', '}'])
        && ativo.split(';').map(str::trim).all(|instrucao| {
            let palavra = instrucao
                .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .next()
                .unwrap_or_default();
            !DESVIOS.contains(&palavra)
        })
}

/// Literais como primeiro argumento de `atualizarArvore('...')`. Chamadas com
/// variável (PEN3) são ignoradas: não identificam uma atividade. Nenhum JS
/// é interpretado; a forma literal não comporta escapes.
fn literais_de_atualizar_arvore(script: &str) -> Vec<&str> {
    const CHAMADA: &str = "atualizarArvore(";
    // O mesmo lexer estático usado para autorizar o link de documento:
    // nenhuma chamada dentro de comentário, string ou template é ativa.
    let Some(niveis) = crate::arvore_processo::niveis_js(script) else {
        return Vec::new();
    };
    let mut achados = Vec::new();
    let mut cursor = 0;
    while let Some(indice) = script[cursor..].find(CHAMADA) {
        let posicao = cursor + indice;
        cursor = posicao + CHAMADA.len();
        if !chamada_na_forma_do_aceite(script, &niveis, posicao) {
            continue;
        }
        let argumento = script[cursor..].trim_start();
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
        // Só a chamada literal com um argumento foi observada no aceite.
        // Um segundo argumento pode lançar erro ANTES de executar a função.
        if !literal.contains('\\') && depois_do_literal.starts_with(')') {
            achados.push(literal);
        }
    }
    achados
}

fn atividade_do_endereco(url: &str, alvo: AlvoDaCiencia<'_>) -> Option<String> {
    let (caminho, restante) = url.split_once('?')?;
    if caminho != "controlador.php" {
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
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroCiencia::AceiteSemAtividade)?;
    let parser = dom.parser();
    // Com JavaScript ativo, scripts aninhados em <template> ou <noscript>
    // não rodam. Ignore somente esses nós, em vez de recusar a resposta
    // inteira quando há também um script ativo: marcação dentro de um literal
    // JavaScript nem sequer cria um contêiner HTML.
    let mut scripts_inertes = Vec::new();
    for seletor in ["template", "noscript"] {
        for no in dom.query_selector(seletor).into_iter().flatten() {
            if let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) {
                scripts_inertes.extend(tag.query_selector(parser, "script").into_iter().flatten());
            }
        }
    }
    let mut atividades = Vec::new();
    for script in dom.query_selector("script").into_iter().flatten() {
        if scripts_inertes.contains(&script) {
            continue;
        }
        let Some(tag) = script.get(parser).and_then(tl::Node::as_tag) else {
            continue;
        };
        // Scripts de dados (application/json, text/plain) e scripts com src
        // não executam seu conteúdo inline. Jamais publicar um id_atividade
        // apenas porque o HTML contém texto JavaScript inerte.
        let Ok(atributos) = crate::html_atributos::atributos(tag) else {
            continue;
        };
        let Ok(tipo) = atributos.valor("type") else {
            continue;
        };
        // O tipo de script clássico segue a semântica HTML: ausente,
        // vazio ou MIME JavaScript (sem parâmetros, independente de caixa).
        // Um tipo de dados ou um script externo não prova a atividade.
        let script_classico = tipo.is_none_or(|t| {
            let t = t.trim_matches(|c: char| c.is_ascii_whitespace());
            t.is_empty()
                || t.eq_ignore_ascii_case("text/javascript")
                || t.eq_ignore_ascii_case("application/javascript")
        });
        if atributos.presente("src") || !script_classico {
            continue;
        }
        // Texto e fonte bruta: a `UnB` 4.0.9 envolve o script em
        // `<!--//--><![CDATA[//><!--`, que inner_text omite (live 10/11/2026);
        // um `<script>` dentro de literal JS, ao contrário, trunca a fonte bruta.
        let texto = tag.inner_text(parser);
        let fonte = tag.raw().as_utf8_str();
        let bruto = fonte.split_once('>').map_or("", |(_, resto)| {
            resto.rsplit_once("</").map_or(resto, |(c, _)| c)
        });
        for url in [texto.as_ref(), bruto]
            .into_iter()
            .flat_map(literais_de_atualizar_arvore)
        {
            let Some(atividade) = atividade_do_endereco(url, alvo) else {
                continue;
            };
            if !atividades.contains(&atividade) {
                atividades.push(atividade);
            }
        }
    }
    match atividades.as_slice() {
        [] => Err(ErroCiencia::AceiteSemAtividade),
        [atividade] => Ok(atividade.clone()),
        _ => Err(ErroCiencia::AtividadesAmbiguas),
    }
}

#[cfg(test)]
mod acervo;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_aceite_unb;
#[cfg(test)]
mod tests_documento;
#[cfg(test)]
mod tests_fluxo;
#[cfg(test)]
mod tests_preparo;
