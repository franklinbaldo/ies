//! Ações de filtro e seleção de tipo: prova de script inline ativo (#246).
//! O teste do corpo é literal; comentários, textos e blocos JS não provam ação.

#[derive(Clone, Copy, PartialEq, Eq)]
enum Contexto {
    Codigo,
    ComentarioDeLinha,
    ComentarioDeBloco,
    Cadeia(char),
    Regex { classe: bool },
}

/// Uma declaração só é prova quando começa em código JavaScript de
/// nível superior. O conteúdo esperado é integral e literal; não é
/// interpretado nem executado. Formas desconhecidas falham fechadas.
fn definicoes_ativas(texto: &str, assinatura: &str, corpo_esperado: &str) -> Option<usize> {
    let mut contexto = Contexto::Codigo;
    let mut profundidade = 0_u32;
    let mut ultimo = ' ';
    let mut encontradas = 0;
    let mut caracteres = texto.char_indices().peekable();
    while let Some((indice, atual)) = caracteres.next() {
        match contexto {
            Contexto::Codigo => match atual {
                '/' if caracteres.peek().is_some_and(|&(_, c)| c == '/') => {
                    caracteres.next();
                    contexto = Contexto::ComentarioDeLinha;
                }
                '/' if caracteres.peek().is_some_and(|&(_, c)| c == '*') => {
                    caracteres.next();
                    contexto = Contexto::ComentarioDeBloco;
                }
                '\'' | '"' | '\u{60}' => contexto = Contexto::Cadeia(atual),
                '/' if matches!(ultimo, '=' | ':' | '(' | ',' | '[' | '!') => {
                    contexto = Contexto::Regex { classe: false };
                }
                '{' => {
                    profundidade = profundidade.checked_add(1)?;
                    ultimo = atual;
                }
                '}' => {
                    profundidade = profundidade.checked_sub(1)?;
                    ultimo = atual;
                }
                'f' if texto.split_at(indice).1.starts_with("function") => {
                    let inicio: String = texto
                        .split_at(indice)
                        .1
                        .chars()
                        .filter(|c| !c.is_ascii_whitespace())
                        .take(assinatura.len())
                        .collect();
                    if inicio == assinatura {
                        // Uma expressão nomeada não publica uma função global
                        // para o onclick. Aceitar só o início do script ou
                        // uma declaração após instrução/bloco terminado.
                        if profundidade != 0 || !matches!(ultimo, ' ' | ';' | '}') {
                            return None;
                        }
                        encontradas += 1;
                        if encontradas > 1 {
                            return None;
                        }
                        let literal: String = texto
                            .split_at(indice)
                            .1
                            .chars()
                            .filter(|c| !c.is_ascii_whitespace())
                            .take(corpo_esperado.len())
                            .collect();
                        if literal != corpo_esperado {
                            return None;
                        }
                    }
                    ultimo = atual;
                }
                _ => {
                    if !atual.is_ascii_whitespace() {
                        ultimo = atual;
                    }
                }
            },
            Contexto::ComentarioDeLinha => {
                if atual == '\n' {
                    contexto = Contexto::Codigo;
                }
            }
            Contexto::ComentarioDeBloco => {
                if atual == '*' && caracteres.peek().is_some_and(|&(_, c)| c == '/') {
                    caracteres.next();
                    contexto = Contexto::Codigo;
                }
            }
            Contexto::Cadeia(delimitador) => {
                if atual == '\\' {
                    caracteres.next()?;
                } else if atual == delimitador {
                    contexto = Contexto::Codigo;
                }
            }
            Contexto::Regex { classe } => match atual {
                '\\' => {
                    caracteres.next()?;
                }
                '[' if !classe => contexto = Contexto::Regex { classe: true },
                ']' if classe => contexto = Contexto::Regex { classe: false },
                '/' if !classe => contexto = Contexto::Codigo,
                _ => {}
            },
        }
    }
    if !matches!(contexto, Contexto::Codigo | Contexto::ComentarioDeLinha) || profundidade != 0 {
        return None;
    }
    Some(encontradas)
}

/// Por que a função de navegação não foi comprovada. Falha fechada: em
/// qualquer caso, nenhum POST de navegação sai.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroGuardaDeNavegacao {
    /// Declaração fora do nível superior, corpo diferente do literal
    /// observado, ou script que termina dentro de cadeia/comentário/bloco.
    ScriptForaDaFormaObservada,
    FuncaoNaoDeclaradaEmScriptAtivo,
    FuncaoDeclaradaMaisDeUmaVez,
}

/// Comprova uma função de navegação apenas em script inline clássico ATIVO.
/// A antiga busca em todo o HTML aceitava textos de `template`, JSON
/// e comentários HTML como se fossem código executável da página.
/// Mantém a exigência literal de corpo e unicidade sem executar JavaScript.
pub(super) fn script_publica_funcao_de_navegacao(
    html: &crate::tela_html::HtmlServido,
    assinatura: &str,
    corpo_esperado: &str,
) -> Result<(), ErroGuardaDeNavegacao> {
    use ErroGuardaDeNavegacao::{
        FuncaoDeclaradaMaisDeUmaVez, FuncaoNaoDeclaradaEmScriptAtivo, ScriptForaDaFormaObservada,
    };
    let dom = crate::tela_html::arvore(html);
    let mut encontradas = 0_usize;
    for script in dom.elementos("script") {
        if script
            .atributo("type")
            .is_some_and(|tipo| !tipo.trim().eq_ignore_ascii_case("text/javascript"))
            || script.presente("src")
        {
            continue;
        }
        let ocorrencias = definicoes_ativas(&script.texto(), assinatura, corpo_esperado)
            .ok_or(ScriptForaDaFormaObservada)?;
        if ocorrencias == 0 {
            continue;
        }
        encontradas += ocorrencias;
        if encontradas > 1 {
            return Err(FuncaoDeclaradaMaisDeUmaVez);
        }
    }
    if encontradas == 1 {
        Ok(())
    } else {
        Err(FuncaoNaoDeclaradaEmScriptAtivo)
    }
}
