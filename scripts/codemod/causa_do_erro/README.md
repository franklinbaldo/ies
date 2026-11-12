---
type: Guide
title: Codemod da causa descartada em map_err
description: Regras ast-grep que reescrevem map_err(|_| E::V) para map_err(E::V), de modo que o erro original vire a fonte (#[source]) da variante (#719)
---

# Codemod `map_err(|_| E::V)` → `map_err(E::V)` (#719)

Uma closure de `map_err` que ignora o erro recebido descarta a causa: a
variante diz que algo falhou, mas `source()` volta vazio. As regras de
`regras/` reescrevem a chamada para passar o construtor da variante
direto, e o erro original vira o campo `#[source]` dela:

```sh
scripts/codemod/causa_do_erro/aplicar.sh producao.rs ...            # aplica
scripts/codemod/causa_do_erro/aplicar.sh --dry-run producao.rs ...  # só lista
```

As regras ignoram módulos `#[cfg(test)]` e `#[cfg(all(test, …))]`. Cada
regra de reescrita repete até não mudar mais nada; a regra 90 só lista.

| Regra | Forma antiga | Forma nova |
| --- | --- | --- |
| `10-causa-descartada` | `x.map_err(\|_\| E::V)` | `x.map_err(E::V)` |
| `10b-causa-descartada-em-bloco` | `x.map_err(\|_\| { E::V })` | `x.map_err(E::V)` |
| `11-causa-nomeada-sem-uso` | `x.map_err(\|_e\| E::V)`, `\|_: T\|`, `\|StructUnitaria\|` | `x.map_err(E::V)` |
| `11b-causa-nomeada-sem-uso-em-bloco` | as mesmas, com o corpo em bloco | `x.map_err(E::V)` |
| `90-causa-descartada-a-mao` | closure que ignora o erro e não devolve só uma variante unitária (variante com campos, chamada) | não muda: ajuste à mão |

`E::V` só casa com caminho (`scoped_identifier`): uma variante com campos
ou uma chamada fica para a regra 90.

Depois do `aplicar.sh`: `cargo fmt` e o compilador. Cada `E::V` que ainda
é unitária dá erro de tipo no sítio reescrito, e a variante ganha o campo
à mão, `V(#[source] T)`, com `T` o erro real do sítio. O que o
compilador e a revisão pedem a mais:

- variante construída também em outro ponto, sem causa ou com causa de
  outro tipo: uma variante nova para o sítio, com o nome do fato, e a
  antiga fica para os outros pontos;
- causa sem `std::error::Error` (struct unitária de recusa): ela ganha
  `thiserror::Error` e `#[error("{self:?}")]`;
- enum que deriva `Copy` e causa que não é `Copy`: o enum perde `Copy`
  e quem o copiava passa a clonar;
- causa que não é erro nem pode ser guardada (o `Vec` de valores servidos
  que `try_into` devolve): a variante guarda só o que se conta dela;
- erro impossível (`VersaoDocumento::nova` de um `checked_add(1)`): sai a
  conversão falível, não entra variante nova.

Fica de fora, de propósito: `src/indice_local/unidades.rs`, que mapeia
para `rusqlite::Error` (o tipo do chamador); `src/visao_do_controle/definir.rs`,
que desmonta `ColunaIndisponivelNestaInstalacao { coluna }` e repassa o
campo à variante achatada de mesmo nome (embrulhar `X(X { coluna })` só
repetiria o fato); e os testes e auxiliares de cassette (`src/cassette*`),
que devolvem texto para o relatório.

O ast-grep roda pelo `uvx` (`ast-grep-cli==0.51.2`); o `sg` do sistema
pode ser outro programa. No padrão do ast-grep, `|$P|` não casa com
`|_|`: por isso as regras 10 e 11 são separadas.
