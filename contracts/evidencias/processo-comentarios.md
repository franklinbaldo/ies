---
type: OperationEvidence
title: processo.comentarios
operation: processo.comentarios
source: todos-demonstration-matrix
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
writes: false
ies_implemented: false
upstream_steps: 21
unresolved_steps: 0
---

# processo.comentarios

Origem: matriz de demonstrações do Todos. **Leitura CLI Rust BDD-257-01,
prévia sem POST e confirmação sintética BDD-257-02 implementadas**.
A PR #659 entregou a prévia; a PR #661 habilitou `--confirmar`
exclusivamente nas origens públicas de treinamento (incluindo
`sei.test` na bancada). A confirmação deriva a requisição do formulário
publicado, faz POST único e relê a lista completa exigindo id e texto
exatos. Sem confirmação, saída 1 sem retry. Essa implementação **não**
equivale a confronto semântico de cassettes nem a validação live.
Os estados do Todos abaixo não são vereditos do IES.
Os estados do Todos abaixo não são vereditos do IES.

As referências #N são passos de roteiro; o link leva à revisão com
o mesmo cenário, mas ainda não certifica igualdade de revisão.
Confronte os hashes com o manifesto atual antes de alegar paridade.

## unb

Estado registrado no Todos: **demonstrada**.

- `comentarios-ciclo-unb#3,7,8` → [comentarios-ciclo-unb--c7d0c8a53b5e.md](../../cassettes/unb/comentarios-ciclo-unb--c7d0c8a53b5e.md)

## pen1

Estado registrado no Todos: **demonstrada**.

- `comentarios-ciclo-pen1#3,7,8` → [comentarios-ciclo-pen1--64e37f041d62.md](../../cassettes/pen1/comentarios-ciclo-pen1--64e37f041d62.md)

## pen2

Estado registrado no Todos: **demonstrada**.

- `comentarios-ciclo-pen2#3,7,8` → [comentarios-ciclo-pen2--23ae97b720e8.md](../../cassettes/pen2/comentarios-ciclo-pen2--23ae97b720e8.md)

## ufrj

Estado registrado no Todos: **demonstrada**.

- `comentarios-ciclo-ufrj#3,7,8` → [comentarios-ciclo-ufrj--6a443c99186b.md](../../cassettes/ufrj/comentarios-ciclo-ufrj--6a443c99186b.md)

## pen3

Estado registrado no Todos: **demonstrada**.

- `comentarios-ciclo-pen3#3,7,8` → [comentarios-ciclo-pen3--a3febc3da646.md](../../cassettes/pen3/comentarios-ciclo-pen3--a3febc3da646.md)

## pen4

Estado registrado no Todos: **demonstrada**.

- `comentarios-ciclo-pen4#3,7,8` → [comentarios-ciclo-pen4--a76f4c52f70e.md](../../cassettes/pen4/comentarios-ciclo-pen4--a76f4c52f70e.md)

## go

Estado registrado no Todos: **demonstrada**.

- `comentarios-ciclo-go#3,7,8` → [comentarios-ciclo-go--76c984b41a29.md](../../cassettes/go/comentarios-ciclo-go--76c984b41a29.md), [comentarios-ciclo-go--8bf61843326d.md](../../cassettes/go/comentarios-ciclo-go--8bf61843326d.md) (revisões não desambiguadas)

## Verificações da implementação

- O workflow `inspecionar-cassettes.yml` verificou duas capturas,
  [UnB/GO, 76 interações](https://github.com/franklinbaldo/ies/actions/runs/42786966461),
  SHA-1/SHA-256, esquema e estrutura dos métodos, incluindo GET
  `comentario_listar`. Isso **não é replay nem confronto semântico**.
- `tests/aies_comentarios_cli.rs`: leitura e prévia sem POST, confirmação
  com exatamente um POST, resposta 302 e consulta por uma **segunda CLI**
  sobre o SQLite persistido; o comentário anterior precisa permanecer.
- PR #661: RED funcional comprovado no
  [CI #42805414597](https://github.com/franklinbaldo/ies/actions/runs/42805414597):
  BDD-257-02 `confirmacao_post_unico_e_releitura_independente` falhou
  por bloqueio antecipado de `--confirmar` (quatro testes anteriores PASS).
  O GREEN requer CI oficial no head final. Testes sintéticos não substituem
  replay semântico independente.
- PR #659: prévia via CLI real com GET do formulário publicado e zero POST,
  sem evidência live.
- **Confronto semântico independente:** NOT_RUN. **Treinamento/live:** NOT_RUN.

## Limite da evidência

As respostas HTTP de ambientes públicos não comprovam, sozinhas,
corpos integrais de POST, paridade da CLI ou efeito de uma escrita.
Exija conferência pós-operação antes de declarar sucesso.

Consulte [cassettes](../../docs/cassettes.md),
[SEI empiricamente](../../docs/como-o-sei-funciona/README.md)
e [contrato provisório](../cli.md).
