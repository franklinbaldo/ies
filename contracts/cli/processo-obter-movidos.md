---
type: Contract
title: Documentos movidos da árvore do processo
description: ies processo NUP publica cada DOCUMENTO_MOVIDO fora dos documentos, com o destino lido do link assinado do nó e se o mesmo documento está entre os documentos do processo
---

# Documentos movidos de `ies processo NUP` (Refs #59)

Complementa o [contrato da árvore](processo-obter-arvore.md), de onde
esta seção saiu para mantê-lo abaixo de 20.000 bytes. Estrutura em
[árvore do processo](../../docs/como-o-sei-funciona/arvore-do-processo.md#documentos-movidos);
decisões em [documentos movidos](../../docs/decisoes/processo-obter/documentos-movidos.md)
e [movido presente](../../docs/decisoes/processo-obter/movido-presente.md).

## Nó aceito

Forma observada na PEN1 5.0.4 (04/11/2026) e no acervo público (GO,
PEN1, PEN2, PEN3), a única aceita: `DOCUMENTO_MOVIDO` com 16 argumentos,
a1 composto `<decimal>-<decimal>` (opaco; na captura, a primeira parte é
o `id_documento` do link do destino e a segunda o
`id_rel_protocolo_protocolo` da barra), a2 igual ao id do processo (pasta
ou outro pai é recusa), a3 começando por `controlador.php?` com uma
`acao=arvore_visualizar`, um `id_procedimento` igual a a2 e nenhum
`id_documento`, a5 rótulo não vazio e a15 número SEI (sem SEI, a chave sai). O movido
**não** é documento do processo: não entra em `documentos`, nas pastas
nem nas conferências delas. Composto repetido ou primeira parte repetida
entre movidos são recusa. A dica (a6, frase com o NUP e o motivo
digitado) e a barra (`.acoes`) não são lidas.

**Presente entre os documentos.** Quando a primeira parte de a1 é o id de
um `DOCUMENTO` da mesma árvore (de pasta ou não), o movido é aceito e
publica `presente_entre_os_documentos: true`; o `DOCUMENTO` sai em
`documentos` como qualquer outro. Sem esse `DOCUMENTO`, `false`. A
comparação é só de ids (opaca): rótulo, número SEI, ordem dos nós e dica
não entram. O fluxo só publica árvore sem pasta fechada (as fechadas são
abertas antes, [pastas](processo-obter-pastas.md)), então `false` vale
para a árvore inteira. O IES não afirma "devolvido": o nó diz que o
documento saiu deste processo, e o `DOCUMENTO` diz que ele está nele
agora; o caminho entre os dois não é lido.

## Destino

O destino vem do `.html` do nó (mesmo leitor de literais e de atributos
do vínculo de anexação): uma tag `<a>` com `href` é o link do destino,
conferido como os outros links assinados (uma
`acao=procedimento_trabalhar`, um `infra_hash` de 64 hex, um
`id_procedimento` decimal diferente do aberto), com um `id_documento`
igual à primeira parte de a1, sem `id_procedimento_anexado`, e texto não
vazio, sem espaço nas pontas, entidade ou tag, diferente do NUP aberto: o
NUP do destino. Nenhuma tag `<a>` com `href` (ou nenhum `.html`):
sem `destino` (chave ausente), não observado. Com destino conferido, o índice aprende o
par e o link `procedimento_trabalhar` do destino (objeto = id do
destino), com ou sem `presente_entre_os_documentos`.

## Saída

```json
"documentos_movidos": [{"id": "a1 (n-m)", "rotulo": "a5", "numero_sei": "a15",
                        "destino": {"numero": "NUP", "id_procedimento": "N"},
                        "presente_entre_os_documentos": false}]
```

`documentos_movidos` segue a ordem dos nós (`[]` sem movido; `destino`
sai sem link); o texto humano lista cada um com
`movido para <NUP>, ies processo <NUP>` ou `destino não publicado` e,
com `presente_entre_os_documentos: true`, acrescenta
`(o documento está de novo neste processo)`. `--sem-documentos` tira a
chave, como tira `documentos`.

## Falhas (saída 1)

`documento_movido_fora_da_forma_observada`,
`DocumentoMovidoComPaiDiferenteDoProcesso`,
`documento_movido_repetido_na_arvore`,
`destino_do_documento_movido_fora_da_forma_observada`,
`destinos_do_documento_movido_repetidos` (duas tags `<a>` com `href` ou
dois `.html` do nó). `documento_movido_presente_entre_os_documentos` não
existe mais: virou `presente_entre_os_documentos: true` (decisão em #59).

## Validação

Documentos movidos: **PASS** na PEN1
([relatório](../../docs/validacao-live/2026-11-04-processo-obter-movidos.md)).
Movido presente entre os documentos: **NOT_RUN** ao vivo (mover documento
é escrita; não foi feito); a evidência é o replay das cassettes públicas
`documento-mover-cancelar-*`, 32 árvores de GO, PEN1, PEN2 e PEN3
([relatório](../../docs/validacao-replay/2026-11-04-processo-obter-acervo.md)).
