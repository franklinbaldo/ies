---
type: Guide
title: Matriz de derivação das fatias WIP da CLI
description: Notas de handoff das fatias parciais de processo, separadas da matriz principal
---

# Matriz — fatias WIP de processo

Complementa a [matriz principal](matriz-derivacao-cli.md). Cada seção
registra o estado de uma fatia de handoff, sem promover NOT_RUN a PASS.

## Pendente: comentários do processo (#257)

`ies processo comentarios NUP` e `ies processo comentar NUP --texto T [--confirmar]` estão em **PENDENTE PR**, sem despacho no `main`. A PR Draft #268 oferece leitor estrutural offline SEI 4.x/5.x e conferência por id/texto; faltam fluxo Web, teste de orçamento de requisições, replay e validação live. [Contrato](../contracts/cli/processo-comentarios.md).

## Handoff #247 — filtros da pesquisa (PR draft)

`processo buscar` e `documento buscar` recebem flags opcionais `--tipo`,
`--assunto`, `--de` e `--ate`. Tipo e período derivam do formulário Web
servido; ausência do campo ou opção exata recusa antes do POST. O assunto
permanece **parcial**: falta observar e resolver o ID do autocompletar
sem romper orçamento. Não se altera o termo, a paginação ou os 3/4 pedidos.
RED, replay e live UnB/PEN3: NOT_RUN nesta execução; CI pendente.
[Contrato de processo](../contracts/cli/processo-buscar.md) ·
[contrato de documento](../contracts/cli/documento-buscar.md) ·
[decisão](decisoes/processo-buscar.md).

## Handoff #259 — registrar andamento (PR draft)

O comando de escrita novo, autorizado apenas em treinamento, lê a barra do processo e o histórico, valida frmAtividadeListar e deriva um POST único. A verificação exige linha nova no topo e total anterior +1; resposta HTTP não é prova. Parcial: faltam testes de transporte/contagem, replay e live (NOT_RUN). [Contrato](../contracts/cli/processo-andamento.md) e [decisão](decisoes/processo-andamento.md).
## Handoff #269 — sobrestar e retomar (PRs #279 e #284)

A PR #279 entregou prévia com três GET e sem escrita;
a PR #284 implementa `--confirmar` para sobrestar e retomar:
uma única escrita derivada do formulário ou função do SEI,
seguida de releitura obrigatória do estado; saída 3 quando incerto.
Orçamento em teste sintético frio: seis requisições para cada
operação, sendo um POST no sobrestar e um GET mutável no retomar.
RED da #284: quatro falhas novas antes do código.
**Ainda pendentes**: replay do acervo e escrita autenticada em
treinamento. Live NOT_RUN, zero requisições.
[Contrato](../contracts/cli/processo-sobrestar.md) ·
[decisão](decisoes/processo-sobrestar.md).

## Handoff #258 — relacionamento de processos (PR Draft)

Novo caminho de CLI proposto pela issue #258:
'ies processo relacionar NUP --com NUP [--remover] [--confirmar]'.
Estado: **PARCIAL WEB** na branch (não na main). A prévia lê o link da
barra do processo e a tabela; adicionar consulta o complementar publicado
pela tela (um POST de leitura); remover só lê a lista. A idempotência
é decidida por identidades observadas, sem procurar números no HTML solto.
'--confirmar' está **bloqueado antes de abrir a sessão** até implementar
a escrita única e a verificação independente. RED local, replay, live:
NOT_RUN. Sem residuo no SEI.
[Contrato](../contracts/cli/processo-relacionar.md) ·
[decisão](decisoes/processo-relacionar.md).

## Fatia de ciência #271 (PR #280)

`ies processo ciente NUP [--confirmar]`, integrado pela
[PR #280](https://github.com/franklinbaldo/ies/pull/280) como **PARCIAL**:
prévia apenas pela abertura do processo; confirmação com um único GET
mutador de `procedimento_ciencia` derivado da tela servida; nova leitura da
árvore e conferência da `id_atividade` no checkbox da Lista de Ciências.
Testes do fluxo contam o orçamento exato (3 GET na prévia a frio e 7 GET
na escrita seguida de verificação no caso sintético). A CLI de
`documento ciente`, a conferência do usuário na lista, o replay e o ensaio
live ainda não foram entregues.
[Contrato](../contracts/cli/processo-ciente.md) ·
[decisão](decisoes/processo-ciente.md).
