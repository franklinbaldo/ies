---
type: SEIObservation
title: Ensaio live dos marcadores na visão detalhada e em processo NUP na UnB
description: PASS/FAIL/NOT_RUN dos marcadores como lista no Controle e na tela do processo, com o processo sem marcador, 04/11/2026
---

# Ensaio live dos marcadores — 04/11/2026

Marcadores ([contrato](../../contracts/cli/processo-obter-marcadores.md)),
binário em release do ramo do PR, índice novo, sessão nova, ritmo
espaçado, UnB 4.0.9 de treinamento com todas as colunas do Nível de
detalhe ligadas (`ies unidade colunas --definir todas --confirmar`).

## Resultado

| Caso | Resultado |
| --- | --- |
| `unidade listar-processos`: célula com 2 marcadores (processo 23106.001747/2026-40) | **PASS**: lista de 2 — `Aguardando` (amarelo, texto "Documento pronto, aguardando liberação.", id 279) e `em análise` (azul, sem texto, id 237); antes saía `"Aguardandoem análise"` |
| `processo NUP` frio (link colhido no login) | **PASS**: 5 req., 2 marcadores, total 2, faixa 1–2, usuário e data presentes |
| Quente (índice) | **PASS**: 4 req. |
| `--sem-andamentos --sem-propriedades` | **PASS**: 2 req. |
| `--sem-marcadores` | **PASS**: 3 req., motivo `omitidos_por_sem_marcadores` |
| Processo sem marcador | **PASS** (depois da correção): 3 req., `marcadores: []`, total 0, faixa `null` |
| Texto humano | **PASS**: `Marcadores: 2 de 2 declarados:` e uma linha por marcador |
| `rotulo` `<id>#<nome>` (`--sem-andamentos --sem-propriedades`, outro processo) | **PASS**: 3 req., 1 marcador, `"rotulo": "208#Diretor Assinar"` ao lado de `id` e `nome` |

## Observações

- **Id do marcador:** é o do marcador da unidade, não o da atribuição. Na
  célula do Controle, `#ID-237` aparece em dois processos e `#ID-279` em
  outros dois; na tela, é o `value` do checkbox e o `id_marcador` do link
  de alterar.

- **Processo sem marcador:** a tela de marcadores responde **302** para
  `andamento_marcador_cadastrar` ("SEI - Adicionar Marcador") do mesmo
  processo. A primeira versão recusava (`redirecionamento_nao_reconhecido`);
  agora o 302 conferido vira lista vazia, sem ser seguido.
- **PEN1:** NOT_RUN. O `--definir todas --confirmar` saiu com efeito
  incerto e a leitura mostrou que não aplicou; não foi repetido.
- GO: NOT_RUN (WAF; ritmo).
