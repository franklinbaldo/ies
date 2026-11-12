---
type: Guide
title: Codemod do tl para a árvore html5ever
description: Regras ast-grep, uma por forma de uso do tl e do html_atributos, que reescrevem um leitor para a árvore de src/tela_html (#676)
---

# Codemod `tl` → árvore html5ever (#676)

Cada arquivo em `regras/` reescreve uma forma de uso do `tl` ou de
`crate::html_atributos` na forma equivalente da árvore de
`src/tela_html/documento.rs` (`Documento`, `Elemento`). As regras ignoram
módulos `#[cfg(test)]`. Rodam em ordem de nome, cada uma até não mudar mais
nada:

```sh
scripts/codemod/html5ever/aplicar.sh src/leitor.rs ...           # aplica
scripts/codemod/html5ever/aplicar.sh --dry-run src/leitor.rs ... # só lista
```

As regras que apagam um comando (02a, 04a, 18a, 99) deixam a marca
`/* ies-codemod: remover linha */`; o `aplicar.sh` apaga essas linhas no fim.
As regras 30 a 31b limpam o que a migração deixaria: alias, `into_iter()`
sobre `Option` e cópia de `&str` usada só por `as_deref()`.

O ast-grep roda pelo `uvx` (`ast-grep-cli==0.51.2`); o `sg` do sistema
pode ser outro programa. Depois: `cargo fmt` e o compilador. O que sobra
(imports, tipos de parâmetro, seletor em variável) é ajuste à mão.

| Regra | Forma antiga | Forma nova |
| --- | --- | --- |
| `01-interpretar` | `crate::tela_html::interpretar(h)` | `crate::tela_html::arvore(h)` |
| `02-tag-ok-or-let-usos`, `02a-…` | `let x = h.get(p).and_then(tl::Node::as_tag).ok_or(e)?;` (`h` identificador) | o `let` sai; os usos de `x` depois dele viram `h` |
| `03-tag-ok-or` | `h.get(p).and_then(tl::Node::as_tag).ok_or(e)?` | `h` |
| `04-tag-let-else-usos`, `04a-…` | `let Some(x) = h.get(p).and_then(tl::Node::as_tag) else {…};` (`h` identificador) | o `let` sai; os usos de `x` viram `h` |
| `04b-tag-let-else-expressao` | o mesmo, com `h` expressão | `let x = h;` |
| `05-tag-filter-map` | `.filter_map(\|n\| n.get(p).and_then(tl::Node::as_tag))` | (sai) |
| `06-no-filter-map` | `.filter_map(\|n\| n.get(p))` | (sai) |
| `07-tag-opcao` | `h.get(p).and_then(tl::Node::as_tag)` (resto) | `Some(h)` |
| `08-seletor-no-elemento` | `t.query_selector(p, "nome")` com `into_iter().flatten()`, `ok_or(e)?` ou `?` | `t.elementos("nome")` |
| `08b-seletor-is-some-and` | `t.query_selector(p, "nome").is_some_and(\|mut i\| i.m(…))` | `t.elementos("nome").m(…)` |
| `09-seletor-no-documento` | `dom.query_selector("nome")` como na 08 | `dom.elementos("nome")` |
| `09b-seletor-let-else` | `let Some(f) = dom.query_selector("nome") else {…};` | `let f = dom.elementos("nome");` |
| `10-texto` | `n.inner_text(p)` e `.into_owned()` | `n.texto()` |
| `11-valor-do-atributo` | `valor_do_atributo(t, k)` / `valor_decodificado(t, k)` com `?` | `t.atributo(k).map(str::to_owned)` |
| `12-atributo-presente` | `atributo_presente(t, k)` com `?` | `t.presente(k)` |
| `13-com-id`, `13b-…` | `nos_com_id(&dom, id)` / `nos_com_id(dom, id)` com `?` | `dom.com_id(id).collect::<Vec<_>>()` |
| `14-com-id-em` | `nos_com_id_em(t, p, id)` com `?` | `t.com_id(id).collect::<Vec<_>>()` |
| `15-nome`, `15b-…` | `t.name().as_utf8_str()`, `t.name() == s` | `t.nome()`, `t.nome() == s` |
| `16-filhos-elementos` | `t.children().all(p).iter().filter_map(tl::Node::as_tag)` | `t.descendentes()` |
| `17-classe` | `atributos(t)?.tem_classe(c)` | `t.tem_classe(c)` |
| `17b-valores` | `a.valores(k)`, só em função que chama `atributos(…)` | `a.atributo(k).into_iter()` |
| `18-atributos-let-else-usos`, `18a-…` | `let Ok(a) = atributos(t) else {…};` (`t` identificador) | o `let` sai; os usos de `a` viram `t` |
| `18a2-atributos-let-else-expressao` | o mesmo, com `t` expressão | `let a = t;` |
| `18b-atributos-is-ok-and` | `atributos(t).is_ok_and(\|a\| …)` | `Some(t).is_some_and(\|a\| …)` |
| `20-elemento-direto` | `Some(x).is_some_and(\|y\| corpo)` | `corpo` com `x` no lugar de `y` |
| `21-elemento-e-classe`, `21b-…` | `nos_por_elemento_e_classe(dom, None \| Some(r), n, c)` com `?` | `….elementos(n).filter(\|e\| e.tem_classe(c)).collect::<Vec<_>>()` |
| `30-opcao-any`, `30b-opcao-all` | `x.atributo(k).into_iter().any(f)` / `.all(f)` | `x.atributo(k).is_some_and(f)` / `.is_none_or(f)` |
| `31-atributo-emprestado` | `let v = t.atributo(k).map(str::to_owned);` com `v` usado só em `v.as_deref()` | `let v = t.atributo(k);` |
| `31b-as-deref-de-emprestado` | `v.as_deref()` depois de `let v = t.atributo(k);` | `v` |
| `99-parser-local` | `let parser = dom.parser();` | (a linha sai) |

O que muda no comportamento, de propósito:

- Ramos impossíveis somem (03, 04, 05, 18): um nó achado na árvore sempre é
  o elemento, e a tag sempre é legível. A variante de erro que ficar sem
  construção sai em seguida, guiada pelo `dead_code`.
- O html5ever entrega o valor do atributo já decodificado, e o primeiro
  vale em repetição (11, 17b). `valor_do_atributo` devolvia o valor bruto.
- Seletores só por nome de elemento literal (`"[a-z][a-z0-9]*"`) são
  reescritos; seletor em variável ou composto fica para ajuste à mão.

Alvos ainda sem método na árvore (`presente`, `tem_classe`, `descendentes`,
`Elemento::com_id`) entram em `documento.rs` com o primeiro leitor que os
usar: método sem uso é código morto.
