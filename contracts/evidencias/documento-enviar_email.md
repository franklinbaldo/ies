---
type: OperationEvidence
title: documento.enviar_email
operation: documento.enviar_email
source: todos-demonstration-matrix
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
writes: true
ies_implemented: false
upstream_steps: 5
unresolved_steps: 0
---

# documento.enviar_email

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

- `documento-enviar-email-pen1#6` → [documento-enviar-email-pen1--9d67c3f5a210.md](../../cassettes/pen1/documento-enviar-email-pen1--9d67c3f5a210.md)

## pen2

Estado registrado no Todos: **demonstrada**.

- `documento-enviar-email-pen2#6` → [documento-enviar-email-pen2--31c391c5a7ba.md](../../cassettes/pen2/documento-enviar-email-pen2--31c391c5a7ba.md), [documento-enviar-email-pen2--4ad79478048f.md](../../cassettes/pen2/documento-enviar-email-pen2--4ad79478048f.md) (revisões não desambiguadas)

## ufrj

Estado registrado no Todos: **demonstrada**.

- `documento-enviar-email-ufrj#6` → [documento-enviar-email-ufrj--32650350f953.md](../../cassettes/ufrj/documento-enviar-email-ufrj--32650350f953.md), [documento-enviar-email-ufrj--dba24f43d7be.md](../../cassettes/ufrj/documento-enviar-email-ufrj--dba24f43d7be.md) (revisões não desambiguadas)

## pen3

Estado registrado no Todos: **na**.

Motivo registrado: o servidor de e-mails da instalação não conecta: medido em 2026-10-20 no PEN3, o envio de documento recusado pelo SEI na confirmação com "Falha na conexão com o servidor de e-mails" (processo 98390.000093/2026-78, documento 0001050), a mesma falha da liberação de acesso externo (quatro tentativas); a barra oferece o envio e o Todos chega ao formulário, quem falha é o SMTP; se o servidor voltar, a célula volta a pendente

Nenhum passo de prova publicado nesta célula.


## pen4

Estado registrado no Todos: **demonstrada**.

- `documento-enviar-email-pen4#6` → [documento-enviar-email-pen4--234fc8731b5c.md](../../cassettes/pen4/documento-enviar-email-pen4--234fc8731b5c.md), [documento-enviar-email-pen4--b327005d18d8.md](../../cassettes/pen4/documento-enviar-email-pen4--b327005d18d8.md) (revisões não desambiguadas)

## go

Estado registrado no Todos: **demonstrada**.

- `documento-enviar-email-go#6` → [documento-enviar-email-go--a09d2abff90b.md](../../cassettes/go/documento-enviar-email-go--a09d2abff90b.md), [documento-enviar-email-go--b493d15b0b86.md](../../cassettes/go/documento-enviar-email-go--b493d15b0b86.md) (revisões não desambiguadas)

## Limite da evidência

As respostas HTTP de ambientes públicos não comprovam, sozinhas,
corpos integrais de POST, paridade da CLI ou efeito de uma escrita.
Exija conferência pós-operação antes de declarar sucesso.

Consulte [cassettes](../../docs/cassettes.md),
[SEI empiricamente](../../docs/como-o-sei-funciona/README.md)
e [contrato provisório](../cli.md).
