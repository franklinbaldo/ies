---
type: OperationEvidence
title: unidade.listar_retornos
operation: unidade.listar_retornos
source: todos-demonstration-matrix
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
writes: false
ies_implemented: true
upstream_steps: 14
unresolved_steps: 0
---

# unidade.listar_retornos

Origem: matriz de demonstrações do Todos. IES Rust: PARCIAL WEB na PR Draft #260.
Os estados abaixo pertencem ao Todos, não são vereditos do IES.

As referências #N são passos de roteiro; o link leva à revisão com
o mesmo cenário, mas ainda não certifica igualdade de revisão.
Confronte os hashes com o manifesto atual antes de alegar paridade.

## unb

Estado registrado no Todos: **demonstrada**.

- `unidade-listar-retornos-unb#4,5` → [unidade-listar-retornos-unb--61ae9a77b5df.md](../../cassettes/unb/unidade-listar-retornos-unb--61ae9a77b5df.md)

## pen1

Estado registrado no Todos: **demonstrada**.

- `unidade-listar-retornos-pen1#4,5` → [unidade-listar-retornos-pen1--8163e3fcb3db.md](../../cassettes/pen1/unidade-listar-retornos-pen1--8163e3fcb3db.md)

## pen2

Estado registrado no Todos: **demonstrada**.

- `unidade-listar-retornos-pen2#4,5` → [unidade-listar-retornos-pen2--8bbd8541acd6.md](../../cassettes/pen2/unidade-listar-retornos-pen2--8bbd8541acd6.md)

## ufrj

Estado registrado no Todos: **demonstrada**.

- `unidade-listar-retornos-ufrj#4,5` → [unidade-listar-retornos-ufrj--3a6152f7d1bc.md](../../cassettes/ufrj/unidade-listar-retornos-ufrj--3a6152f7d1bc.md)

## pen3

Estado registrado no Todos: **demonstrada**.

- `unidade-listar-retornos-pen3#4,5` → [unidade-listar-retornos-pen3--2d16437b3e19.md](../../cassettes/pen3/unidade-listar-retornos-pen3--2d16437b3e19.md)

## pen4

Estado registrado no Todos: **demonstrada**.

- `unidade-listar-retornos-pen4#4,5` → [unidade-listar-retornos-pen4--87e2e8e70f44.md](../../cassettes/pen4/unidade-listar-retornos-pen4--87e2e8e70f44.md)

## go

Estado registrado no Todos: **demonstrada**.

- `unidade-listar-retornos-go#4,5` → [unidade-listar-retornos-go--645ae924663b.md](../../cassettes/go/unidade-listar-retornos-go--645ae924663b.md)

## Limite da evidência

As respostas HTTP de ambientes públicos não comprovam, sozinhas,
corpos integrais de POST, paridade da CLI ou efeito de uma escrita.
Exija conferência pós-operação antes de declarar sucesso.

Consulte [cassettes](../../docs/cassettes.md),
[SEI empiricamente](../../docs/como-o-sei-funciona/README.md)
e [contrato provisório](../cli.md).

## Recorte Rust da PR Draft #260

A CLI `ies unidade retornos [--mes AAAA-MM]` lê pelo menu e pelo link
`Ver todo o mes`; confere contagens das duas tabelas, rejeita paginacao
nao observada e tem testes sinteticos. **RED no CI inicial: 3 testes
falhando** antes do parser; compilacao local e live NOT_RUN.
Replay original de **14 páginas mensais em sete instalações: PASS (14/14,
zero recusas)**. [Relatório verificado](../../docs/validacao-replay/2026-11-05-unidade-retornos.md).
A validação ao vivo GO/PEN3 é independente e deve registrar seu próprio resultado.
[Contrato](../cli/unidade-listar-retornos.md) e [decisao](../../docs/decisoes/unidade-listar-retornos.md).
