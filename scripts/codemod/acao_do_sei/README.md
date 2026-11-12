---
type: Guide
title: Codemod da ação do SEI em texto para AcaoDoSei
description: Regras ast-grep, uma por forma em que a ação do SEI ainda circula como texto, que a reescrevem para o enum AcaoDoSei (#731)
---

# Codemod ação em texto → `AcaoDoSei` (#731)

Cada arquivo em `regras/` e em `modelos/` reescreve uma forma em que a ação
do `controlador.php` (`acao=`) circula como texto na forma equivalente com
`crate::acao_do_sei::AcaoDoSei`. O texto da ação fica só na fronteira: a
leitura passa por `AcaoDoSei::ler` ou `ConsultaDoControlador::acao()`, e a
URL montada passa por `como_na_url()`. As regras rodam em ordem de nome, cada
uma até não mudar mais nada:

```sh
scripts/codemod/acao_do_sei/aplicar.sh producao.rs ...                    # aplica
scripts/codemod/acao_do_sei/aplicar.sh --dry-run producao.rs ...          # só lista
scripts/codemod/acao_do_sei/aplicar.sh --testes tests_x.rs acervo.rs ...  # só 30 a 33, 89 e 90
```

As regras 10 a 24 ignoram módulos `#[cfg(test)]` e `#[cfg(all(test, …))]`,
mas não sabem que um arquivo inteiro é de teste (`#[path] mod tests;`).
Por isso os arquivos de teste vão com `--testes`, que só tipa os
parâmetros `acao: &str` dos auxiliares e as chamadas deles. Nos testes, o
literal que é o dado gravado (`r.operacao == "…"` no acervo, URL de cassette)
fica como está.

A tabela valor → variante não é escrita à mão: o `aplicar.sh` a extrai do
enum `AcaoDoSei` de `src/acao_do_sei.rs` (só `Variante => "valor",` dentro de
`AcaoDoSei { … }`). Cada modelo com `VALOR` e `VARIANTE` vira um YAML com um
documento por linha da tabela. Um literal que não está na tabela não é
reescrito: a variante entra em `acao_do_sei.rs` antes, e só se a produção
usar aquele valor.

As regras 31 e 32 saem de `modelos/` com a lista `FUNCOES`: o `aplicar.sh`
lista primeiro as funções dos arquivos dados que têm parâmetro `acao: &str`
(regra 30) e só mexe nas chamadas delas. As variantes `b` dessas regras
cobrem a chamada dentro de macro (`assert_eq!`, `vec!`, `format!`), em que o
ast-grep só vê tokens.

Depois do `aplicar.sh`: `cargo fmt` e o compilador. O que sobra é ajuste à
mão: tipo de retorno de uma função cujo ramo virou variante (23), cópia
local de `como_na_url()` passada a um auxiliar da regra 30, `to_owned()` do
texto da ação.

O ast-grep roda pelo `uvx` (`ast-grep-cli==0.51.2`); o `sg` do sistema
pode ser outro programa.

| Regra | Forma antiga | Forma nova |
| --- | --- | --- |
| `10-acao-unica-comparada`, `10b-…` | `c.unico("acao") == Ok(v.como_na_url())` / `!=` | `c.acao() == Ok(v)` / `!=` |
| `11-acao-de-origem-comparada`, `11b-…` | `c.unico("acao_origem") == Ok(v.como_na_url())` / `!=` | `c.acao_de_origem() == Ok(v)` / `!=` |
| `12-texto-comparado-com-a-url`, `12b-…` | `texto == v.como_na_url()` / `!=` | `AcaoDoSei::ler(texto) == Some(v)` / `!=` |
| `20-literal-comparado`, `20b-…` (de `modelos/`) | `acao == "valor"` / `!=` (`acao…` identificador) | `AcaoDoSei::ler(acao) == Some(AcaoDoSei::Variante)` / `!=` |
| `21-unica-comparada-com-literal`, `21b-…` (de `modelos/`) | `c.unico("acao") == Ok("valor")` / `!=` | `c.acao() == Ok(AcaoDoSei::Variante)` / `!=` |
| `22-lista-comparada-com-literal`, `22b-…` (de `modelos/`) | `acoes == ["valor"]` / `!=` (lista de textos lidos) | `acoes.into_iter().map(AcaoDoSei::ler).eq([Some(AcaoDoSei::Variante)])` / `.ne(…)` |
| `23-literal-em-ramo` (de `modelos/`) | `"valor"` / `Some("valor")` como valor de ramo, em `fn …acao…` | `AcaoDoSei::Variante` / `Some(AcaoDoSei::Variante)` |
| `24-literal-na-url-formatada` (de `modelos/`) | `format!("…acao=valor&…", …)` | `format!("…acao={acao}&…", …, acao = AcaoDoSei::Variante.como_na_url())` |
| `30-parametro-acao` | parâmetro `acao: &str` | `acao: AcaoDoSei` |
| `31-argumento-como-na-url`, `31b-…` (de `modelos/`) | `f(v.como_na_url())` com `f` da regra 30 | `f(v)` |
| `32-argumento-literal`, `32b-…` (de `modelos/`) | `f("valor")` com `f` da regra 30 | `f(AcaoDoSei::Variante)` |
| `33-acao-no-format` | `format!("…{acao}…")` em função da regra 30 | `format!("…{acao}…", acao = acao.como_na_url())` |
| `89-importar` | arquivo que nomeia `crate::acao_do_sei::AcaoDoSei` fora de módulo interno, sem importá-lo | `use crate::acao_do_sei::AcaoDoSei;` antes do primeiro `use` |
| `90-caminho-curto`, `90b-…` | `crate::acao_do_sei::AcaoDoSei` em arquivo que o importa | `AcaoDoSei` |

O que fica como texto, de propósito:

- Busca de trecho em `href` ou HTML cru antes de qualquer leitura
  (`href.contains("acao=…")`, `starts_with("controlador.php?acao=…&")`,
  marcador em `const`): trocar por `menciona_acao` ou por
  `ConsultaDoControlador` muda o que é aceito (prefixo, posição na
  consulta, valor com sufixo).
- `acao_origem` e `acao_retorno` dentro de URL de teste sintética e as
  operações gravadas nas cassettes (`r.operacao == "…"`): são o dado.
