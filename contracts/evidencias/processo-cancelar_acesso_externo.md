---
type: OperationEvidence
title: processo.cancelar_acesso_externo
operation: processo.cancelar_acesso_externo
source: todos-demonstration-matrix
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
writes: true
ies_implemented: false
upstream_steps: 5
unresolved_steps: 0
---

# processo.cancelar_acesso_externo

Origem: matriz de demonstrações do Todos. IES Rust: NÃO IMPLEMENTADO.
Os estados abaixo pertencem ao Todos, não são vereditos do IES.

As referências #N são passos de roteiro; o link leva à revisão com
o mesmo cenário, mas ainda não certifica igualdade de revisão.
Confronte os hashes com o manifesto atual antes de alegar paridade.

## unb

Estado registrado no Todos: **na**.

Motivo registrado: nenhuma unidade das contas de treinamento tem e-mail cadastrado: medido em 2026-10-20 na Unidade1 e na Unidade2 (aluno1) e na Unidade3 e na Unidade4 (aluno3), o select de remetente vem só com o "null" e a ação recusa com UnidadeSemRemetente; o remetente é cadastro de administrador

Nenhum passo de prova publicado nesta célula.


## pen1

Estado registrado no Todos: **demonstrada**.

- `processo-acesso-externo-pen1#7` → [processo-acesso-externo-pen1--a5fc9f019e6b.md](../../cassettes/pen1/processo-acesso-externo-pen1--a5fc9f019e6b.md)

## pen2

Estado registrado no Todos: **demonstrada**.

- `processo-acesso-externo-pen2#7` → [processo-acesso-externo-pen2--4cab0467d307.md](../../cassettes/pen2/processo-acesso-externo-pen2--4cab0467d307.md)

## ufrj

Estado registrado no Todos: **demonstrada**.

- `processo-acesso-externo-ufrj#7` → [processo-acesso-externo-ufrj--ba3a98eecfc8.md](../../cassettes/ufrj/processo-acesso-externo-ufrj--ba3a98eecfc8.md)

## pen3

Estado registrado no Todos: **na**.

Motivo registrado: o servidor de e-mails da instalação não conecta: medido em 2026-10-20 em quatro tentativas seguidas do processo-acesso-externo no PEN3 (processos 98390.000077/2026-85, 98390.000086/2026-76, 98390.000087/2026-11 e 98390.000088/2026-65, a última sem espera entre as anteriores), o SEI recusa liberar o acesso externo com "Falha na conexão com o servidor de e-mails", e sem liberação não há o que cancelar; se o servidor voltar, a célula deixa de ser N/A e volta a pendente

Nenhum passo de prova publicado nesta célula.


## pen4

Estado registrado no Todos: **demonstrada**.

- `processo-acesso-externo-pen4#7` → [processo-acesso-externo-pen4--660ccfe29fd2.md](../../cassettes/pen4/processo-acesso-externo-pen4--660ccfe29fd2.md)

## go

Estado registrado no Todos: **demonstrada**.

- `processo-acesso-externo-go#7` → [processo-acesso-externo-go--34dedaf86d52.md](../../cassettes/go/processo-acesso-externo-go--34dedaf86d52.md)

## Limite da evidência

As respostas HTTP de ambientes públicos não comprovam, sozinhas,
corpos integrais de POST, paridade da CLI ou efeito de uma escrita.
Exija conferência pós-operação antes de declarar sucesso.

Consulte [cassettes](../../docs/cassettes.md),
[SEI empiricamente](../../docs/como-o-sei-funciona/README.md)
e [contrato provisório](../cli.md).
