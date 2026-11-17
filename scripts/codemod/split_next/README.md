---
type: Guide
title: Codemod de split(P).next() tratado como Option
description: Regras ast-grep que reescrevem split(P).next() e rsplit(P).next() com padrão, erro ou ? impossíveis em split_once e rsplit_once, com equivalência exata
---

# Codemod `split(P).next()` tratado como `Option`

`str::split(P).next()` e `str::rsplit(P).next()` nunca são `None`: sem o
separador, devolvem o texto inteiro. Por isso o `unwrap_or_default()`, o
`unwrap_or(X)`, o `ok_or(E)?`, o `?` e o `Some(…)` comparado em volta deles
são casos impossíveis: um padrão que nunca vale, um erro que nunca nasce
(#722, regra "não observado"). Cada arquivo em `regras/` reescreve uma
forma em que isso aparece na forma equivalente sem `Option`:

```sh
scripts/codemod/split_next/aplicar.sh producao.rs ...            # aplica
scripts/codemod/split_next/aplicar.sh --dry-run producao.rs ...  # só lista
```

As regras rodam em ordem de nome, cada uma até não mudar mais nada, e
pulam módulos `#[cfg(test)]`. Elas não sabem que um arquivo inteiro é de
teste (`#[path] mod tests;`, acervo, cassette): passe só os arquivos de
produção, que são os do dep-info do binário sem testes
(`target/debug/deps/ies-*.d`).

| Regra | Forma antiga | Forma nova |
| --- | --- | --- |
| `10-tipo-principal-do-mime`, `10b-…` | `x.split(';').next().unwrap_or_default().trim()` | `crate::mime_web::principal(x)` (`&x.campo` para campo) |
| `20-split-com-padrao`, `20b-…` | `x.split(P).next().unwrap_or_default()` / `.unwrap_or(D)` | `x.split_once(P).map_or(x, \|(antes, _)\| antes)` |
| `21-rsplit-com-padrao`, `21b-…` | `x.rsplit(P).next().unwrap_or_default()` / `.unwrap_or(D)` | `x.rsplit_once(P).map_or(x, \|(_, depois)\| depois)` |
| `22-split-com-erro`, `22b-…` | `x.split(P).next().ok_or(E)?` / `.ok_or_else(E)?` / `?` | `x.split_once(P).map_or(x, \|(antes, _)\| antes)` |
| `23-rsplit-comparado`, `23b-…` | `x.rsplit(P).next() != Some(v)` / `==` | `x.rsplit_once(P).map_or(x, \|(_, depois)\| depois) != v` / `==` |
| `24-and-then-split` | `o.and_then(\|r\| r.split(P).next())` | `o.map(\|r\| r.split_once(P).map_or(r, \|(antes, _)\| antes))` |
| `29-split-no-retorno` | `x.split(P).next()` como valor de bloco (`Option` por outro motivo) | `Some(x.split_once(P).map_or(x, \|(antes, _)\| antes))` |
| `30-consulta-da-url` | `ConsultaDoControlador::ler(u.split_once('?').map_or("", \|(_, c)\| c))` | `ConsultaDoControlador::da_url(u.split_once('?').map(\|(_, c)\| c))` |

`x` é um identificador ou, nas variantes `b`, um campo (`tela.url`). No
campo, o padrão do `map_or` é `x.as_str()`, porque os campos lidos são
`String`. Para identificador, a regra supõe `&str`: um identificador
`String` não compila, e o compilador aponta onde pôr `.as_str()`.

A regra 30 vale porque `ConsultaDoControlador::da_url(None)` é
`Ok` sem pares, o mesmo que `ler("")`; com `?`, `da_url(Some(c))` é
`ler(c)`.

## Prova

`equivalencia.rs` confere, só com a biblioteca padrão, que a forma antiga
e a nova dão o mesmo valor para cada padrão usado (`char`, lista de `char`,
fecho e `&str`) e para textos com e sem separador, vazio, separador nas
pontas e repetido:

```sh
rustc --edition 2024 --test scripts/codemod/split_next/equivalencia.rs -o /tmp/eq && /tmp/eq
```

No crate, `mime_web::tests` fixa `principal` e
`tests_consulta_do_controlador.rs` compara `ler` e `da_url` sobre URLs com
e sem `?`.

## O que fica à mão

- `x.split(P).next().is_some_and(|a| corpo)`: a regra teria de pôr o
  corpo do fecho em volta do texto. São dois sítios: `mime_web::e_html`
  (virou `principal(mime).eq_ignore_ascii_case(…)`) e o `</template` de
  `arvore_processo/ciencia.rs`.
- Identificador `String` no padrão do `map_or` (`.as_str()`), apontado
  pelo compilador.
