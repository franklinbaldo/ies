---
type: OperationEvidence
title: documento.mover
operation: documento.mover
source: todos-demonstration-matrix
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
writes: true
ies_implemented: false
unresolved_steps: 0
UNRESOLVED_PLACEHOLDER
---

# documento.mover

Origem: matriz de demonstrações do Todos. IES Rust: NÃO IMPLEMENTADO.
Os estados abaixo pertencem ao Todos, não são vereditos do IES.

As referências #N são passos de roteiro; o link leva à revisão com
o mesmo cenário, mas ainda não certifica igualdade de revisão.
Confronte os hashes com o manifesto atual antes de alegar paridade.

## unb

Estado registrado no Todos: **na**.

Motivo registrado: o perfil das contas de treinamento não tem a ação: medido em 2026-10-20 com aluno1 e aluno2, na Unidade1 e na Unidade2, num Despacho e num externo, recém-criados, assinados e depois de tramitar, nenhuma barra a oferece, e nenhuma das cassettes da UnB a tem; o PEN1, também 4.0, tem (documento-mover-cancelar-unb)

- `documento-mover-cancelar-unb#10,11` → [documento-mover-cancelar-unb--0c37c22872c9.md](../../cassettes/unb/documento-mover-cancelar-unb--0c37c22872c9.md)

## pen1

Estado registrado no Todos: **demonstrada**.

- `documento-mover-cancelar-pen1#11,12` → [documento-mover-cancelar-pen1--0c59af5df1ba.md](../../cassettes/pen1/documento-mover-cancelar-pen1--0c59af5df1ba.md)

## pen2

Estado registrado no Todos: **demonstrada**.

- `documento-mover-cancelar-pen2#11,12` → [documento-mover-cancelar-pen2--2375a0354d05.md](../../cassettes/pen2/documento-mover-cancelar-pen2--2375a0354d05.md)

## ufrj

Estado registrado no Todos: **na**.

Motivo registrado: a barra do documento nunca oferece 'Mover Documento para outro Processo' para a conta na UFRJ: medido em 2026-10-21 num processo novo com quatro documentos externos (tipos Anexo, Despacho, Ofício e Comprovante, enviados pelo juntar) e um Despacho interno, antes e depois de assinar, nenhum dos cinco traz o link documento_mover na barra (só documento_alterar_recebido), e as gravações do documento-mover-cancelar dão MoverNaoOferecido; no PEN órgão 2, no mesmo teste, os quatro externos oferecem. É o padrão de uma permissão do perfil da conta, que o administrador concede; se o perfil ganhar a permissão, a célula volta a pendente

- `documento-mover-cancelar-ufrj#9,10` → [documento-mover-cancelar-ufrj--de1301eb5634.md](../../cassettes/ufrj/documento-mover-cancelar-ufrj--de1301eb5634.md)

## pen3

Estado registrado no Todos: **demonstrada**.

- `documento-mover-cancelar-pen3#10,11` → [documento-mover-cancelar-pen3--60eb468849de.md](../../cassettes/pen3/documento-mover-cancelar-pen3--60eb468849de.md)

## pen4

Estado registrado no Todos: **na**.

Motivo registrado: a barra do documento nunca oferece 'Mover Documento para outro Processo' para a conta no PEN órgão 4: medido em 2026-10-21 num processo novo com quatro documentos externos (tipos Anexo, Despacho, Ofício e Comprovante, enviados pelo juntar) e um Despacho interno, antes e depois de assinar, nenhum dos cinco traz o link documento_mover na barra (só documento_alterar_recebido), e as gravações do documento-mover-cancelar dão MoverNaoOferecido; no PEN órgão 2, no mesmo teste, os quatro externos oferecem. É o padrão de uma permissão do perfil da conta, que o administrador concede; se o perfil ganhar a permissão, a célula volta a pendente

- `documento-mover-cancelar-pen4#10,11` → [documento-mover-cancelar-pen4--55cfedad7880.md](../../cassettes/pen4/documento-mover-cancelar-pen4--55cfedad7880.md)

## go

Estado registrado no Todos: **demonstrada**.

- `documento-mover-cancelar-go#9,10` → [documento-mover-cancelar-go--a152c0306b45.md](../../cassettes/go/documento-mover-cancelar-go--a152c0306b45.md)

## Limite da evidência

As respostas HTTP de ambientes públicos não comprovam, sozinhas,
corpos integrais de POST, paridade da CLI ou efeito de uma escrita.
Exija conferência pós-operação antes de declarar sucesso.

Consulte [cassettes](../../docs/cassettes.md),
[SEI empiricamente](../../docs/como-o-sei-funciona/README.md)
e [contrato provisório](../cli.md).
