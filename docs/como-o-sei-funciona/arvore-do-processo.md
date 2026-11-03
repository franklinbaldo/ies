---
type: SEIObservation
title: Tela de trabalho e árvore do processo
description: Estrutura observada de procedimento_trabalhar, ifrArvore e das declarações infraArvoreNo
---

# Tela de trabalho e árvore do processo

Observado na UnB 4.0.9 e em Goiás (treinamento, 03/11/2026), em
capturas privadas `0600` lidas só pela estrutura. Nenhum valor real
aparece aqui ou nas fixtures.

## Como se chega à árvore

1. `controlador.php?acao=procedimento_trabalhar&…&id_procedimento=N&…&infra_hash=<64 hex>`:
   o `Location` do 302 da [pesquisa](pesquisa-de-protocolo.md) com NUP
   exato, ou o `href` do link do processo na linha `tr#P{id}` do
   Controle (visões R e D).
2. A resposta 200 (`<title>SEI - Processo`) traz exatamente um
   `<iframe id="ifrArvore" name="ifrArvore" … src="controlador.php?acao=procedimento_visualizar&acao_origem=procedimento_trabalhar&acao_retorno=…&id_procedimento=N&id_documento=&infra_sistema=…&infra_unidade_atual=…&infra_hash=…">`,
   com `&` cru nas duas instalações, e um `iframe#ifrVisualizacao` com
   `src="about:blank"`. O `acao_retorno` variou (`principal` na UnB,
   `procedimento_controlar` em GO).
3. O GET do `src` de `ifrArvore` devolve a árvore: um `script` inline com
   as declarações dos nós e, no corpo, `div#divArvore` vazio (a árvore
   é montada no navegador), `form#frmArvore` com `hdnArvore`,
   `hdnPastaAtual` e `hdnProtocolos` vazios e `iframe#ifrPasta` oculto.

## Efeito medido

Em GO, num processo recebido ainda não visualizado
(`processoNaoVisualizado`), o GET de `procedimento_trabalhar` sozinho
**não** mudou a caixa; o GET seguinte de `procedimento_visualizar` (a
árvore) o marcou como visualizado em `ies unidade recebidos`. Nenhuma
outra linha mudou. Se um andamento de recebimento também é lançado:
**NOT_RUN**.

## Declaração de um nó

Cada nó é uma linha inteira, sem indentação:

`Nos[i] = new infraArvoreNo(a0,…,a15);`

com 16 argumentos literais (aspas duplas, `null`, `true`, `false`),
índices `i` consecutivos a partir de 0. Seguem-se linhas
`Nos[i].acoes = '…'`, `Nos[i].src = '…'` e `Nos[i].html = '…'`; o
`Nos[0].processar = function(){…}` vem depois, dentro de `if (Nos.length)`.
As ações da barra (`NosAcoes[k] = new infraArvoreAcao(…)`) são outro
construtor.

| Arg. | PROCESSO | DOCUMENTO |
| --- | --- | --- |
| a0 | `"PROCESSO"` | `"DOCUMENTO"` |
| a1 | id do procedimento | id do documento |
| a2 | `null` | id do pai (o procedimento) |
| a3 | `arvore_visualizar` assinado | idem, ou outra forma (não lido) |
| a4 | `"ifrVisualizacao"` | `ifrVisualizacao` (UnB) ou `ifrConteudoVisualizacao` (GO) |
| a5 | NUP | rótulo: tipo do documento e número SEI |
| a6 | tipo do processo (dica) | dica; igual a a5 ou texto livre (em e-mail, cabeçalhos com `\n`) |
| a7–a9 | ícone, três vezes igual | idem (`documento_interno`, `documento_email`, `documento_cancelado`…) |
| a10, a11 | `true`, `true` | `true`, `true` |
| a12–a14 | `"noVisitado"`, `null`, `"noVisitado"` | `null`, `null`, `"noVisitado"` ou `null` |
| a15 | NUP outra vez | número SEI do documento |

O significado de a10–a14 não foi demonstrado (o Todos também não o
publica). Nos documentos, a15 aparece dentro de a5. A dica de e-mail traz
remetente e destinatários: dado pessoal que o IES **não** publica.

## A palavra PASTA

Na UnB, `PASTA` aparece 11 vezes e `AGUARDE` 8 vezes sem nenhuma pasta:
todas dentro de funções inline (`objArvore.getNoAnterior`, navegação por
teclado, `processarPasta`) que comparam `this.nodes[i].tipo == 'PASTA'`
ou trocam `PASTA` por `AGUARDE`, e nos ids `ifrPasta`/`hdnPastaAtual`.
Em GO, `PASTA` aparece uma vez e `AGUARDE` nenhuma. Pasta só existe como
nó se houver `new infraArvoreNo("PASTA",…)`; o texto solto não é nó.
Árvores grandes com pastas ficam para a fatia 2 de #59.

Veja também [rotas e árvore](rotas-e-arvore.md) e
[efeitos e verificação](efeitos-e-verificacao.md).
