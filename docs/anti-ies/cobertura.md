---
type: Guide
title: Cobertura anti-IES por capacidade observável
description: Estado separado da CLI IES, com evidência e marcos de integração
---

# Matriz de cobertura anti-IES

Esta matriz acompanha o **servidor experimental**; não substitui
a [matriz da CLI](../matriz-derivacao-cli.md). PASS de um teste de
roteamento não autoriza declarar compatibilidade SEI, nem a cobertura
do cliente implica cobertura do servidor.

| Operação observável | Estado do IES | Estado anti-IES | Próxima evidência |
| --- | --- | --- | --- |
| **unidade.atual** | Leitura Web real e testes em várias instalações (ver [contrato](../../contracts/cli/unidade-atual.md)) | **NÃO COBERTA**: bootstrap responde 501 ao controlador; sem Controle compatível | [P1 #353](https://github.com/franklinbaldo/ies/issues/353) → [P2 #354](https://github.com/franklinbaldo/ies/issues/354) → [P3 #355](https://github.com/franklinbaldo/ies/issues/355) |
| unidade.listar_processos | Leitura Web existente | Não coberta | Depois de unidade.atual: listas, D, paginação, contagem |
| processo.obter (NUP) | Leitura Web existente | Não coberta | Árvore, documentos, histórico, ausências |
| documento.obter | Leitura Web em evolução | Não coberta | Conteúdo interno/externo, versão, MIME |
| processo.enviar | Escrita Web com cenários live | Não coberta | Estado antes/depois, recusa, idempotência, efeito incerto |
| documento.incluir PDF / documento nativo | Issues prioritárias em andamento | Não coberta | Fluxo de documentos após primeiras leituras |

## Estados admitidos

- **NÃO COBERTA:** nenhum produtor validado para a operação; HTTP 501
  continua sendo recusa honesta.
- **ORÁCULO:** cassettes e expectativas independentes testadas, ainda sem
  produtor equivalente.
- **SERVIDOR:** resposta própria do anti-IES passa os cenários
  observados, mas o fluxo real do IES ainda não foi exercitado.
- **INTEGRADA:** fluxo real IES → anti-IES, oráculo independente e CI
  passam nos casos declarados; cobertura fica **restrita às variantes
  e instalações comprovadas**.
- **DIVERGENTE:** caso observado contradiz o servidor; registrar causa
  e teste RED antes de reclassificar.

Para atualizar, registrar a issue/PR, SHA do CI, instalação e forma do
Controle quando pertinente, positivos/negativos, número de GET/POST e
NOT_RUN. Não acrescentar check obrigatório à fila principal antes de
existir a primeira integração real. Veja o [plano](roadmap.md).
