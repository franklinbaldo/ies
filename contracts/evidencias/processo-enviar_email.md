---
type: OperationEvidence
title: processo.enviar_email
operation: processo.enviar_email
source: todos-demonstration-matrix
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
writes: true
ies_implemented: false
upstream_steps: 5
unresolved_steps: 0
---

# processo.enviar_email

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

- `processo-enviar-email-pen1#3` → [processo-enviar-email-pen1--e496d96004b2.md](../../cassettes/pen1/processo-enviar-email-pen1--e496d96004b2.md)

## pen2

Estado registrado no Todos: **demonstrada**.

- `processo-enviar-email-pen2#3` → [processo-enviar-email-pen2--6b870cdb480b.md](../../cassettes/pen2/processo-enviar-email-pen2--6b870cdb480b.md), [processo-enviar-email-pen2--e858a3161fa2.md](../../cassettes/pen2/processo-enviar-email-pen2--e858a3161fa2.md) (revisões não desambiguadas)

## ufrj

Estado registrado no Todos: **demonstrada**.

- `processo-enviar-email-ufrj#3` → [processo-enviar-email-ufrj--4d55d5d9d28e.md](../../cassettes/ufrj/processo-enviar-email-ufrj--4d55d5d9d28e.md), [processo-enviar-email-ufrj--a3b592b881b8.md](../../cassettes/ufrj/processo-enviar-email-ufrj--a3b592b881b8.md) (revisões não desambiguadas)

## pen3

Estado registrado no Todos: **na**.

Motivo registrado: o servidor de e-mails da instalação não conecta: medido em 2026-10-20 no PEN3, o envio de processo recusado pelo SEI na confirmação com "Falha na conexão com o servidor de e-mails" (processo 98390.000092/2026-23), a mesma falha da liberação de acesso externo (quatro tentativas); a barra oferece o envio e o Todos chega ao formulário, quem falha é o SMTP; se o servidor voltar, a célula volta a pendente

Nenhum passo de prova publicado nesta célula.


## pen4

Estado registrado no Todos: **demonstrada**.

- `processo-enviar-email-pen4#3` → [processo-enviar-email-pen4--375f734dbc8f.md](../../cassettes/pen4/processo-enviar-email-pen4--375f734dbc8f.md), [processo-enviar-email-pen4--65cc17f2142f.md](../../cassettes/pen4/processo-enviar-email-pen4--65cc17f2142f.md) (revisões não desambiguadas)

## go

Estado registrado no Todos: **demonstrada**.

- `processo-enviar-email-go#3` → [processo-enviar-email-go--196c07d1d0f1.md](../../cassettes/go/processo-enviar-email-go--196c07d1d0f1.md), [processo-enviar-email-go--b24c1ffccbde.md](../../cassettes/go/processo-enviar-email-go--b24c1ffccbde.md) (revisões não desambiguadas)

## Limite da evidência

As respostas HTTP de ambientes públicos não comprovam, sozinhas,
corpos integrais de POST, paridade da CLI ou efeito de uma escrita.
Exija conferência pós-operação antes de declarar sucesso.

Consulte [cassettes](../../docs/cassettes.md),
[SEI empiricamente](../../docs/como-o-sei-funciona/README.md)
e [contrato provisório](../cli.md).
