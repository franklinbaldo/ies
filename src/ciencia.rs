//! Ciência Web de processo/documento: identifica a atividade do aceite e
//! prova o registro na lista de ciências **novamente lida**.
//!
//! Observado em Todos 323eb01, `tests/test_ciclo_ciencia_010.py` e
//! `tests/test_documento_ciente_010.py`; os arquivos da cassette do IES
//! referenciam capturas `UnB` (4.0), UFRJ (4.1) e PEN3 (5.0).
//! O GET de ciência é ESCRITA e não pode ser feito durante a prévia.
//! Nenhum JavaScript é executado, e nenhuma URL assinada é publicada.
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::ConsultaDoControlador;

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

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroCiencia {
    Legenda(#[from] crate::legenda::ErroLegenda),
    AceiteSemAtividade,
    AtividadesAmbiguas,
    ListaDeCienciasForaDaFormaObservada,
    AtividadesRepetidasNaLista,
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
    // O byte e a profundidade do lexer na posição `i`; fora do script, nada.
    let em = |i: usize| bytes.get(i).copied().zip(niveis.get(i).copied());
    let ativo_e = |i: usize, teste: fn(u8) -> bool| {
        em(i).is_some_and(|(byte, nivel)| nivel.is_some() && teste(byte))
    };
    let Some(nivel) = niveis.get(posicao).copied().flatten() else {
        return false;
    };
    if nivel > 1 {
        return false;
    }
    // O texto imediatamente anterior não pode ser uma condição, atribuição,
    // retorno ou outra expressão que contenha a chamada. Comentários e
    // literais são ignorados pelo lexer; não se interpreta JavaScript.
    let fim_de_comando = |byte: u8| matches!(byte, b';' | b'{' | b'}');
    let visivel = |byte: u8| !byte.is_ascii_whitespace();
    let inicio_comando = (0..posicao)
        .rev()
        .find(|&i| ativo_e(i, fim_de_comando))
        .map_or(0, |i| i + 1);
    if (inicio_comando..posicao).any(|i| ativo_e(i, visivel)) {
        return false;
    }
    if nivel == 0 {
        // Chamada direta só prova recarga quando é a primeira instrução
        // executável do script: um throw anterior pode torná-la inalcançável.
        return (0..posicao)
            .all(|i| em(i).is_some_and(|(byte, nivel)| nivel != Some(0) || separador_vazio(byte)));
    }

    // Profundidade 1 só é aceita no corpo direto da função de inicialização
    // publicada pelo SEI, e nunca em um if, callback ou função arbitrária.
    let Some(abertura) = (0..posicao).rev().find(|&i| em(i) == Some((b'{', Some(0)))) else {
        return false;
    };
    let inicio_declaracao = (0..abertura)
        .rev()
        .find(|&i| ativo_e(i, fim_de_comando))
        .map_or(0, |i| i + 1);
    let assinatura: Vec<u8> = (inicio_declaracao..abertura)
        .filter_map(em)
        .filter(|&(byte, nivel)| nivel.is_some() && visivel(byte))
        .map(|(byte, _)| byte)
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
    let Some(mut resto) = script.get(inicio..fim) else {
        return false;
    };
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
                .split_once(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .map_or(instrucao, |(antes, _)| antes);
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
    for (posicao, _) in script.match_indices(CHAMADA) {
        if !chamada_na_forma_do_aceite(script, &niveis, posicao) {
            continue;
        }
        let (_, depois) = script.split_at(posicao);
        let Some(argumento) = depois.strip_prefix(CHAMADA).map(str::trim_start) else {
            continue;
        };
        let Some((aspas, resto)) = ['\'', '"']
            .into_iter()
            .find_map(|aspas| argumento.strip_prefix(aspas).map(|resto| (aspas, resto)))
        else {
            continue;
        };
        let Some((literal, depois_do_literal)) = resto.split_once(aspas) else {
            continue;
        };
        let depois_do_literal = depois_do_literal.trim_start();
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
    let Ok(consulta) = ConsultaDoControlador::ler(
        restante
            .split_once('#')
            .map_or(restante, |(antes, _)| antes),
    ) else {
        return None;
    };
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
    if consulta.acao() != Ok(AcaoDoSei::ProcedimentoVisualizar.como_na_url())
        || consulta.unico("id_procedimento") != Ok(id_procedimento)
        || consulta.unico(marca) != Ok("1")
        || id_documento.is_some_and(|doc| consulta.unico("id_documento") != Ok(doc))
        || (id_documento.is_none() && consulta.valores("id_documento").next().is_some())
    {
        return None;
    }
    let Ok(atividade) = consulta.unico("id_atividade") else {
        return None;
    };
    identificador(atividade).then(|| atividade.to_owned())
}

/// Extrai a atividade exata que o SEI indicou na recarga da árvore.
///
/// Só literais de `atualizarArvore` dentro de scripts e com identidade do
/// processo/documento coerente contam. Duas atividades distintas recusam;
/// a ausência não é confundida com sucesso ou ciência inexistente.
pub(crate) fn atividade_do_aceite(
    html: &crate::tela_html::HtmlServido,
    alvo: AlvoDaCiencia<'_>,
) -> Result<String, ErroCiencia> {
    let dom = crate::tela_html::arvore(html);
    // Com JavaScript ativo, scripts em <template> ou <noscript> não rodam;
    // como no navegador, o conteúdo de <template> fica fora da árvore e o de
    // <noscript> é texto, então esses scripts nem aparecem aqui.
    let mut atividades = Vec::new();
    for script in dom.elementos("script") {
        // Scripts de dados (application/json, text/plain) e scripts com src
        // não executam seu conteúdo inline. Jamais publicar um id_atividade
        // apenas porque o HTML contém texto JavaScript inerte.
        let tipo = script.atributo("type");
        // O tipo de script clássico segue a semântica HTML: ausente,
        // vazio ou MIME JavaScript (sem parâmetros, independente de caixa).
        // Um tipo de dados ou um script externo não prova a atividade.
        let script_classico = tipo.is_none_or(|t| {
            let t = t.trim_matches(|c: char| c.is_ascii_whitespace());
            t.is_empty()
                || t.eq_ignore_ascii_case("text/javascript")
                || t.eq_ignore_ascii_case("application/javascript")
        });
        if script.presente("src") || !script_classico {
            continue;
        }
        // O conteúdo inteiro do script, como o navegador o guarda: inclusive o
        // invólucro `<!--//--><![CDATA[//><!--` que a `UnB` 4.0.9 publica
        // (live 10/11/2026).
        let texto = script.texto();
        for url in literais_de_atualizar_arvore(&texto) {
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
