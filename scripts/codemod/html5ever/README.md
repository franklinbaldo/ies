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

As regras que apagam um comando ou um parâmetro (02a, 04a, 11f, 18a, 42, 43, 99) deixam a marca
`/* ies-codemod: remover linha */`; o `aplicar.sh` apaga no fim a linha que
só tem a marca, ou só a marca quando ela divide a linha com outro código.
As regras 30 a 31b limpam o que a migração deixaria: alias, `into_iter()`
sobre `Option` e cópia de `&str` usada só por `as_deref()`.

As regras 43, 44 e 44b saem de `modelos/`: o `aplicar.sh` lista primeiro as
funções dos arquivos dados que têm parâmetro `&tl::Parser` (43) ou
`&tl::HTMLTag`/`&tl::Node` (44) e só mexe nas chamadas delas. Uma regra que
não converge em 20 passadas interrompe o `aplicar.sh`.

Depois do `aplicar.sh`: `cargo fmt`, `cargo fix --allow-dirty --bin ies` (só
tira os `use` que ficaram sem uso) e o compilador. Função que ficou sem falha
perde o `Result`, e variante de erro sem construção sai, guiadas pelo
`clippy::unnecessary_wraps` e pelo `dead_code`.

O ast-grep roda pelo `uvx` (`ast-grep-cli==0.51.2`); o `sg` do sistema
pode ser outro programa. Depois: `cargo fmt` e o compilador. O que sobra
(imports, tipos de parâmetro, seletor em variável) é ajuste à mão.

| Regra | Forma antiga | Forma nova |
| --- | --- | --- |
| `01-interpretar` | `crate::tela_html::interpretar(h)` | `crate::tela_html::arvore(h)` |
| `01b-tag-de-fatia-no-match` | `[h] => h.get(p).and_then(tl::Node::as_tag).ok_or(e)?` | `[h] => *h` |
| `01c-…-let-else`, `01d-…-let` | `let Some(x) = h.get(..)… else {…};` / `let x = h.get(..)…ok_or(e)?;` com `h` de `let [h] = …` | `let x = *h;` |
| `02-tag-ok-or-let-usos`, `02a-…` | `let x = h.get(p).and_then(tl::Node::as_tag).ok_or(e)?;` (`h` identificador) | o `let` sai; os usos de `x` depois dele viram `h` |
| `03-tag-ok-or` | `h.get(p).and_then(tl::Node::as_tag).ok_or(e)?` | `h` |
| `04-tag-let-else-usos`, `04a-…` | `let Some(x) = h.get(p).and_then(tl::Node::as_tag) else {…};` (`h` identificador) | o `let` sai; os usos de `x` viram `h` |
| `04b-tag-let-else-expressao` | o mesmo, com `h` expressão | `let x = h;` |
| `05b-filhos` | `t.children().top().iter().filter_map(\|h\| h.get(p)).filter_map(tl::Node::as_tag)` | `t.filhos()` |
| `05-tag-filter-map` | `.filter_map(\|n\| n.get(p).and_then(tl::Node::as_tag))` | (sai) |
| `06-no-filter-map` | `.filter_map(\|n\| n.get(p))` | (sai) |
| `06c-no-and-then` | `x.and_then(\|n\| n.get(p))` | `x` |
| `07-tag-opcao` | `h.get(p).and_then(tl::Node::as_tag)` (resto) | `Some(h)` |
| `07c-tag-map-ok-or` | `h.get(p).map(\|n\| corpo).ok_or(e)` | `Ok(corpo)` com `h` no lugar de `n` |
| `07d-no-opcao` | `h.get(parser)` que sobrou | `Some(h)` |
| `08-seletor-no-elemento` | `t.query_selector(p, "nome")` com `into_iter().flatten()`, `ok_or(e)?` ou `?` | `t.elementos("nome")` |
| `08b-seletor-is-some-and` | `t.query_selector(p, "nome").is_some_and(\|mut i\| i.m(…))` | `t.elementos("nome").m(…)` |
| `08c-seletor-algum` | `t.query_selector(p, "nome").is_some_and(\|mut i\| i.next().is_some())` | `t.elementos("nome").next().is_some()` |
| `09-seletor-no-documento` | `dom.query_selector("nome")` como na 08 | `dom.elementos("nome")` |
| `09b-seletor-let-else` | `let Some(f) = dom.query_selector("nome") else {…};` | `let f = dom.elementos("nome");` |
| `09c-texto-decodificado` | `html_escape::decode_html_entities(&n.inner_text(p))` | `n.texto()` (o html5ever já decodifica) |
| `10-texto` | `n.inner_text(p)` e `.into_owned()` | `n.texto()` |
| `11-valor-do-atributo` | `valor_do_atributo(t, k)` / `valor_decodificado(t, k)` com `?` | `t.atributo(k).map(str::to_owned)` |
| `11c-valor-sem-interrogacao` | `valor_do_atributo(t, k).map_err(e)` sem `?` | `Ok(t.atributo(k).map(str::to_owned))` |
| `11d-sem-segunda-decodificacao` | `x.map(\|s\| decodificar_atributo(&s))` | `x` |
| `11e-decodificado-let-usos`, `11f-…` | `let y = decodificar_atributo(&b);` com `b` lido por `atributo()` | o `let` sai; os usos de `y` viram `b` |
| `11g-sem-segunda-decodificacao-let` | `decodificar_atributo(&b)` que sobrou, com `b` lido por `atributo()` | `b` |
| `12-atributo-presente` | `atributo_presente(t, k)` com `?` | `t.presente(k)` |
| `13-com-id`, `13b-…` | `nos_com_id(&dom, id)` / `nos_com_id(dom, id)` com `?` | `dom.com_id(id).collect::<Vec<_>>()` |
| `14-com-id-em` | `nos_com_id_em(t, p, id)` com `?` | `t.com_id(id).collect::<Vec<_>>()` |
| `15-nome`, `15b-…` | `t.name().as_utf8_str()`, `t.name() == s` | `t.nome()`, `t.nome() == s` |
| `15c-nome-diferente` | `t.name() != s` | `t.nome() != s` |
| `16-filhos-elementos` | `t.children().all(p).iter().filter_map(tl::Node::as_tag)` | `t.descendentes()` |
| `16c-as-tag-resto` | `.filter_map(tl::Node::as_tag)` / `.and_then(tl::Node::as_tag)` que sobrou | (sai) |
| `16d-for-filhos-por-nome` | `for f in t.children().all(p) { let Some(o) = f.as_tag().filter(\|x\| x.nome() == s) else { continue; }; … }` | `for o in t.elementos(s) { … }` |
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
| `30c-ok-and-then-ok-or` | `Ok(x).and_then(\|y\| y.ok_or(e))` | `x.ok_or(e)` |
| `31c-to-owned-as-deref` | `t.atributo(k).map(str::to_owned).as_deref()` | `t.atributo(k)` |
| `32-for-sem-collect` | `for x in d.com_id(i).collect::<Vec<_>>() {…}` | `for x in d.com_id(i) {…}` |
| `40-tipo-tag`, `40b-…` | `&tl::HTMLTag<'_>` / `&tl::Node<'_>` / `&'a tl::HTMLTag<'a>` | `crate::tela_html::Elemento<'_>` / `<'a>`, por valor |
| `41-tipo-documento` | `tl::VDom<'_>` | `crate::tela_html::Documento` |
| `42-parametro-parser` | parâmetro `parser: &tl::Parser<'_>` | (sai) |
| `43-argumento-parser` (de `modelos/`) | `parser` / `dom.parser()` / `self.parser` como argumento de uma função da regra 42 | (sai) |
| `44-copia-para-funcao`, `44b-…` (de `modelos/`) | `v.iter().map(\|t\| f(t))` / `x.map(\|t\| f(t))` com `f` da regra 40 | `v.iter().copied().map(f)` / `x.map(f)` |
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
`Elemento::com_id`; `filhos` entrou na F1) entram em `documento.rs` com o primeiro leitor que os
usar: método sem uso é código morto.
