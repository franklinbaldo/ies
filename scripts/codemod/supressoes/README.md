---
type: Guide
title: "Codemod de supressões de lint: allow com comentário vira expect com reason"
description: Regras ast-grep que reescrevem #[allow(clippy::…)] com o motivo em comentário em #[expect(clippy::…, reason = "…")], para que o compilador acuse a supressão morta (#721)
---

# Codemod `#[allow(clippy::…)]` → `#[expect(clippy::…, reason = "…")]` (#721)

Um `#[allow]` continua calado quando o lint deixa de disparar; um
`#[expect]` que não se cumpre vira o aviso `unfulfilled_lint_expectations`,
e o `-D warnings` do gate o torna erro. Cada regra em `regras/` leva o
motivo que já estava num comentário `//` vizinho para o `reason` e apaga o
comentário. As regras rodam em ordem de nome, cada uma até não mudar mais
nada:

```sh
scripts/codemod/supressoes/aplicar.sh arquivo.rs ...            # aplica
scripts/codemod/supressoes/aplicar.sh --dry-run arquivo.rs ...  # só lista
```

| Regra | Forma antiga | Forma nova |
| --- | --- | --- |
| `10-allow-com-comentario-ao-lado` | `#[allow(clippy::L)] // Motivo.` | `#[expect(clippy::L, reason = "Motivo")]` |
| `20-juntar-comentario-de-duas-linhas` | `// Motivo em` / `// duas linhas.` logo acima do `allow` | `// Motivo em duas linhas.` (uma linha, para a 21) |
| `21-allow-com-comentario-acima` | `// Motivo.` / `#[allow(clippy::L)]` | `#[expect(clippy::L, reason = "Motivo")]` |

Comentário de documentação (`///`, `//!`) não é motivo: fica onde está. O
ponto final do comentário sai do `reason`. A 20 existe porque o
`expandStart` do ast-grep 0.51 só alcança um nó vizinho.

Depois do `aplicar.sh`: `cargo fmt` e `cargo clippy --workspace
--all-targets -- -D warnings -W clippy::pedantic`. Um `expect` não cumprido
é supressão morta: apague o atributo (e o motivo com ele).

O que fica à mão:

- `#[allow]` sem comentário: o motivo sai do contexto (o vizinho com o
  mesmo lint, o doc comment da função).
- Os arquivos que outra frente está migrando ficam fora da lista de
  arquivos.

## `iter().any(|x| x == y)` → `contains`

Não tem regra aqui. Uma regra que reescrevia `xs.iter().any(|x| x == y)`
em `xs.contains(y)` foi tentada: todos os sítios do pacote `ies` comparam
`String` com `&str` (`Vec<String>` contra `unidade: &str`), em que
`contains` pede `&String` e não compila sem alocar. Quando os tipos
coincidem, o `clippy::manual_contains` já acusa, e o gate está limpo.
