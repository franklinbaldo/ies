//! RED/GREEN do leitor de literais: escapes, formas recusadas e linhas
//! que não declaram nó. Linhas sintéticas na forma observada.
use super::literais::{Literal, ler_declaracao};
use crate::erro_html::ErroHtml;

fn texto(s: &str) -> Literal {
    Literal::Texto(s.to_owned())
}

#[test]
fn declaracao_observada_com_os_quatro_tipos_de_literal() {
    let linha = r#"Nos[0] = new infraArvoreNo("PROCESSO","10",null,'x',true,false,-3,7);"#;
    let (i, argumentos) = ler_declaracao(linha).unwrap().unwrap();
    assert_eq!(i, 0);
    assert_eq!(
        argumentos,
        [
            texto("PROCESSO"),
            texto("10"),
            Literal::Nulo,
            texto("x"),
            Literal::Booleano(true),
            Literal::Booleano(false),
            Literal::Inteiro("-3".to_owned()),
            Literal::Inteiro("7".to_owned()),
        ]
    );
}

#[test]
fn escapes_reconhecidos() {
    let linha = r#"Nos[12] = new infraArvoreNo("a\"b\\c\/d\ne\rf\tg",'h\'i');"#;
    let (i, argumentos) = ler_declaracao(linha).unwrap().unwrap();
    assert_eq!(i, 12);
    assert_eq!(argumentos, [texto("a\"b\\c/d\ne\rf\tg"), texto("h'i")]);
}

#[test]
fn texto_com_virgula_e_parenteses_nao_separa_argumentos() {
    let linha = r#"Nos[1] = new infraArvoreNo("a, b);","c");"#;
    let (_, argumentos) = ler_declaracao(linha).unwrap().unwrap();
    assert_eq!(argumentos, [texto("a, b);"), texto("c")]);
}

#[test]
fn linhas_que_nao_declaram_no_sao_ignoradas() {
    for linha in [
        "Nos[0].src = '';",
        "Nos[2].acoes = Nos[2].acoes.concat('');",
        "      Nos[0].processar = function (){",
        "var objArvore = null;",
        "",
    ] {
        assert_eq!(ler_declaracao(linha), Ok(None), "{linha}");
    }
}

#[test]
fn formas_recusadas() {
    let casos = [
        // Indentada ou com texto antes.
        (
            r#"  Nos[0] = new infraArvoreNo("A");"#,
            super::ErroArvore::Html(ErroHtml::valor("declaracao_de_no_fora_da_forma_observada")),
        ),
        (
            r#"x=1;Nos[0] = new infraArvoreNo("A");"#,
            super::ErroArvore::Html(ErroHtml::valor("declaracao_de_no_fora_da_forma_observada")),
        ),
        // Sem o fechamento exato, ou com algo depois.
        (
            r#"Nos[0] = new infraArvoreNo("A")"#,
            super::ErroArvore::Html(ErroHtml::valor("declaracao_de_no_fora_da_forma_observada")),
        ),
        (
            r#"Nos[0] = new infraArvoreNo("A"); x();"#,
            super::ErroArvore::Html(ErroHtml::valor("declaracao_de_no_fora_da_forma_observada")),
        ),
        // Índice que não é decimal.
        (
            r#"Nos[i] = new infraArvoreNo("A");"#,
            super::ErroArvore::Html(ErroHtml::valor("declaracao_de_no_fora_da_forma_observada")),
        ),
        (
            r#"Nos[] = new infraArvoreNo("A");"#,
            super::ErroArvore::Html(ErroHtml::valor("declaracao_de_no_fora_da_forma_observada")),
        ),
        // Expressões, identificadores e concatenação não são literais.
        (
            r#"Nos[0] = new infraArvoreNo("A"+"B");"#,
            super::ErroArvore::ArgumentoDeNoNaoLiteral,
        ),
        (
            r"Nos[0] = new infraArvoreNo(x);",
            super::ErroArvore::ArgumentoDeNoNaoLiteral,
        ),
        (
            r#"Nos[0] = new infraArvoreNo("A",,"B");"#,
            super::ErroArvore::ArgumentoDeNoNaoLiteral,
        ),
        (
            r#"Nos[0] = new infraArvoreNo("A", "B");"#,
            super::ErroArvore::ArgumentoDeNoNaoLiteral,
        ),
        (
            r"Nos[0] = new infraArvoreNo(nulo);",
            super::ErroArvore::ArgumentoDeNoNaoLiteral,
        ),
        (
            r"Nos[0] = new infraArvoreNo(1.5);",
            super::ErroArvore::ArgumentoDeNoNaoLiteral,
        ),
        // Texto aberto e escapes fora da lista.
        (
            r#"Nos[0] = new infraArvoreNo("A);"#,
            super::ErroArvore::TextoDeNoSemFechamento,
        ),
        (
            r#"Nos[0] = new infraArvoreNo("A\x41");"#,
            super::ErroArvore::Html(ErroHtml::valor("escape_de_no_nao_reconhecido")),
        ),
        (
            r#"Nos[0] = new infraArvoreNo("A\u{41}");"#,
            super::ErroArvore::Html(ErroHtml::valor("escape_de_no_nao_reconhecido")),
        ),
        (
            r#"Nos[0] = new infraArvoreNo("A\");"#,
            super::ErroArvore::TextoDeNoSemFechamento,
        ),
    ];
    for (linha, esperado) in casos {
        assert_eq!(ler_declaracao(linha), Err(esperado), "{linha}");
    }
}
