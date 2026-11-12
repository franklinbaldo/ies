//! Prova das reescritas das regras: para cada padrão e cada texto, a forma
//! antiga (`split(P).next()` tratado como `Option`) e a nova dão o mesmo
//! valor. Roda fora do crate, só com a biblioteca padrão:
//!
//! rustc --edition 2024 --test equivalencia.rs -o /tmp/equivalencia && /tmp/equivalencia

const TEXTOS: [&str; 17] = [
    "",
    ";",
    "?",
    "/",
    "#",
    "a",
    "a;b",
    ";b",
    "a;",
    "a;b;c",
    "a?b?c",
    "x/y/z",
    "/x",
    "x/",
    "a\\b/c",
    "ção;ü?é/#",
    "</template<script</template",
];

/// Cada padrão das regras (`char`, lista de `char`, fecho e `&str`) sobre
/// cada texto. O `Pattern` da biblioteca padrão não é estável: a macro
/// repete o corpo com o padrão literal.
macro_rules! para_cada_padrao {
    (|$x:ident, $p:ident| $corpo:block) => {
        for $x in TEXTOS {
            {
                let $p = ';';
                $corpo
            }
            {
                let $p = '?';
                $corpo
            }
            {
                let $p = ['/', '?'];
                $corpo
            }
            {
                let $p = ['/', '\\'];
                $corpo
            }
            {
                let $p = |c: char| !(c.is_ascii_alphanumeric() || c == '_');
                $corpo
            }
            {
                let $p = "</template";
                $corpo
            }
        }
    };
}

#[test]
fn split_next_nunca_e_none() {
    para_cada_padrao!(|x, p| {
        assert!(x.split(p).next().is_some(), "{x:?}");
        assert!(x.rsplit(p).next().is_some(), "{x:?}");
    });
}

/// Regras 20, 22 e 29: o texto antes do primeiro separador, ou o inteiro.
#[test]
fn split_com_padrao_ou_erro_e_split_once() {
    para_cada_padrao!(|x, p| {
        let nova = x.split_once(p).map_or(x, |(antes, _)| antes);
        assert_eq!(x.split(p).next().unwrap_or_default(), nova, "{x:?}");
        assert_eq!(x.split(p).next().unwrap_or("sentinela"), nova, "{x:?}");
        assert_eq!(x.split(p).next().ok_or(()), Ok(nova), "{x:?}");
        assert_eq!(x.split(p).next(), Some(nova), "{x:?}");
    });
}

/// Regras 21 e 23: o texto depois do último separador, ou o inteiro.
#[test]
fn rsplit_com_padrao_ou_comparado_e_rsplit_once() {
    para_cada_padrao!(|x, p| {
        let nova = x.rsplit_once(p).map_or(x, |(_, depois)| depois);
        assert_eq!(x.rsplit(p).next().unwrap_or_default(), nova, "{x:?}");
        assert_eq!(x.rsplit(p).next().unwrap_or("sentinela"), nova, "{x:?}");
        for alvo in TEXTOS {
            assert_eq!(x.rsplit(p).next() != Some(alvo), nova != alvo, "{x:?}");
            assert_eq!(x.rsplit(p).next() == Some(alvo), nova == alvo, "{x:?}");
        }
    });
}

/// Regra 24: `and_then` de um `split(P).next()` é um `map`.
#[test]
fn and_then_split_e_map() {
    for prefixo in [None, Some("")] {
        para_cada_padrao!(|x, p| {
            let entrada = prefixo.map(|_| x);
            assert_eq!(
                entrada.and_then(|r| r.split(p).next()),
                entrada.map(|r| r.split_once(p).map_or(r, |(antes, _)| antes)),
                "{x:?}"
            );
        });
    }
}

/// Regra 10: o tipo principal do MIME (`mime_web::principal`).
#[test]
fn tipo_principal_e_split_once_aparado() {
    for x in TEXTOS.into_iter().chain([" text/html ; charset=x", "  ", " a "]) {
        assert_eq!(
            x.split(';').next().unwrap_or_default().trim(),
            x.split_once(';').map_or(x, |(tipo, _)| tipo).trim(),
            "{x:?}"
        );
    }
}

/// Regra 30: a consulta da URL, vazia sem `?` (`ler("")`) e ausente
/// (`da_url(None)`), é o mesmo texto quando há `?`.
#[test]
fn consulta_vazia_e_consulta_ausente() {
    for x in TEXTOS {
        let antiga = x.split_once('?').map_or("", |(_, c)| c);
        match x.split_once('?').map(|(_, c)| c) {
            Some(nova) => assert_eq!(antiga, nova, "{x:?}"),
            None => assert_eq!(antiga, "", "{x:?}"),
        }
    }
}
