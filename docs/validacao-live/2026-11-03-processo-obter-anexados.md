---
type: SEIObservation
title: Ensaio live dos processos anexados em processo NUP na UnB
description: PASS/FAIL/NOT_RUN da leitura dos anexados pelo pai e do pai pelo anexado, com o índice, 03/11/2026
---

# Ensaio live dos anexados de `ies processo NUP` — 03/11/2026

Processos anexados ([contrato](../../contracts/cli/processo-obter-arvore.md)),
binário em release do ramo do PR, índice novo, sessão nova, ritmo espaçado.
Um processo pai com um anexado, na UnB 4.0.9 de treinamento. Só formas e
contagens abaixo.

## Resultado

| Caso | Resultado |
| --- | --- |
| Anexado aberto a frio (pesquisa rápida) | **PASS**: 3 req., `anexado: true`, `anexado_a` com o NUP do pai |
| Pai logo depois (link do `trabalhar` aprendido no anexado) | **PASS**: `indice_trabalhar`, 2 req., anexado listado, 7 documentos |
| Pai de novo | **PASS**: `indice_link`, 1 req. |

## Observações

- **Primeira tentativa falhou ao vivo:** aberta a partir do anexado (o
  `ifrArvore` traz `id_procedimento_anexado`), a árvore do pai marca o nó
  do anexado com a12 `"noVisitado"` em vez de `null`. A regra passou a
  aceitar esse único valor de estado visual; os demais continuam recusa.
- A pesquisa rápida com o NUP do anexado abre a árvore **do anexado**
  (só os documentos dele), não a do pai, nesta instalação; o vínculo ao
  pai é o link assinado em `Nos[0].html`.
- Outras instalações: NOT_RUN (nenhum anexado conhecido).
