---
type: OperationEvidence
title: documento.liberar_assinatura_externa
operation: documento.liberar_assinatura_externa
source: todos-demonstration-matrix
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
writes: true
ies_implemented: false
upstream_steps: 1
unresolved_steps: 0
---

# documento.liberar_assinatura_externa

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

Estado registrado no Todos: **na**.

Motivo registrado: a instalação não tem usuário externo cadastrado: medido em 2026-10-20 na unidade TESTE do PEN1, a busca do usuário externo da prévia de liberar não casou com nenhum dos 15 termos (as cinco vogais, 'teste', 'externo', 'usuario', 'ext', '@', '1', '.com', 'silva', 'maria' e 'orgao'), num Despacho descartável; o cadastro de usuário externo é do administrador, que as contas de treinamento não são

Nenhum passo de prova publicado nesta célula.


## pen2

Estado registrado no Todos: **na**.

Motivo registrado: a instalação não tem usuário externo cadastrado: medido em 2026-10-20 na unidade TESTE do PEN2, a busca do usuário externo da prévia de liberar não casou com nenhum dos 15 termos (as cinco vogais, 'teste', 'externo', 'usuario', 'ext', '@', '1', '.com', 'silva', 'maria' e 'orgao'), num Despacho descartável; o cadastro de usuário externo é do administrador, que as contas de treinamento não são

Nenhum passo de prova publicado nesta célula.


## ufrj

Estado registrado no Todos: **demonstrada**.

- `documento-assinatura-externa-ufrj#5` → [documento-assinatura-externa-ufrj--7e09a1815b35.md](../../cassettes/ufrj/documento-assinatura-externa-ufrj--7e09a1815b35.md)

## pen3

Estado registrado no Todos: **na**.

Motivo registrado: a instalação não tem usuário externo cadastrado: medido em 2026-10-20 na unidade TESTE do PEN3, a busca do usuário externo da prévia de liberar não casou com nenhum dos 15 termos (as cinco vogais, 'teste', 'externo', 'usuario', 'ext', '@', '1', '.com', 'silva', 'maria' e 'orgao'), num Despacho descartável; o cadastro de usuário externo é do administrador, que as contas de treinamento não são

Nenhum passo de prova publicado nesta célula.


## pen4

Estado registrado no Todos: **na**.

Motivo registrado: a instalação não tem usuário externo cadastrado: medido em 2026-10-20 na unidade TESTE do PEN4, a busca do usuário externo da prévia de liberar não casou com nenhum dos 15 termos (as cinco vogais, 'teste', 'externo', 'usuario', 'ext', '@', '1', '.com', 'silva', 'maria' e 'orgao'), num Despacho descartável; o cadastro de usuário externo é do administrador, que as contas de treinamento não são

Nenhum passo de prova publicado nesta célula.


## go

Estado registrado no Todos: **na**.

Motivo registrado: o único usuário externo que o GO oferece é o endereço de um terceiro real (achado com 'ext' nas unidades CGE/GETEC-18153 e SEAD/CDE-09516 em 2026-10-21; a busca por 15 termos (as cinco vogais, 'teste', 'externo', 'usuario', 'ext', '@', '1', '.com', 'silva', 'maria' e 'orgao') não trouxe outro), e liberar a assinatura externa faz o SEI enviar e-mail a ele: não se libera. Trocar esse e-mail ou cadastrar um usuário externo descartável é função de administrador, que as contas de treinamento não são; se um administrador cadastrar um usuário externo com e-mail descartável, a célula volta a pendente

Nenhum passo de prova publicado nesta célula.


## Limite da evidência

As respostas HTTP de ambientes públicos não comprovam, sozinhas,
corpos integrais de POST, paridade da CLI ou efeito de uma escrita.
Exija conferência pós-operação antes de declarar sucesso.

Consulte [cassettes](../../docs/cassettes.md),
[SEI empiricamente](../../docs/como-o-sei-funciona/README.md)
e [contrato provisório](../cli.md).
