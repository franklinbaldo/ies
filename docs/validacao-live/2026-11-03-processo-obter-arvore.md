---
type: SEIObservation
title: Ensaio live de processo NUP pela árvore em três instalações de treinamento
description: PASS/FAIL/NOT_RUN dos quatro caminhos de abertura de ies processo NUP (índice, entrada por id, pesquisa rápida e Controle), 03/11/2026
---

# Ensaio live de `ies processo NUP` — 03/11/2026

Primeiro ensaio do [recorte 1 da #59](../decisoes/processo-obter.md),
binário em release do ramo do PR. Cada instalação usou um
[índice local](../arquitetura/0003-indice-local.md) novo em pasta `0700`
temporária. Nenhum NUP, id, cookie, URL assinada ou HTML foi versionado;
abaixo só formas e contagens.

## Condições

- **Sessão:** `ies login` com as contas públicas do
  [roteiro de login](roteiro-login.md); `ies login --forcar` para gerar
  uma sessão nova antes do caminho 2.
- **Processos:** o primeiro e o último de `ies unidade recebidos`, mais o
  primeiro de `unidade gerados` na UnB (fora da página 1 do Controle).
- **Efeito:** o GET da árvore marca o processo como visualizado na
  unidade (medido antes em GO); sem restauração. Os processos abertos já
  estavam visualizados, salvo onde dito.

## Resultado

| Caso | UnB 4.0.9 | GO 5.0.3 | PEN1 4.0.12 |
| --- | --- | --- | --- |
| 4a, linha do Controle | **PASS**: `caixa`, 3 req. | **PASS** | **PASS** |
| 1, link da árvore no índice (mesma sessão) | **PASS**: `indice_link`, 1 req., `unidade: null` | **PASS** | **PASS** |
| 3, pesquisa rápida guardada | **PASS**: `pesquisa_rapida`, 3 req. | **PASS** | **PASS** |
| 4b, pesquisa rápida do Controle (índice novo) | **PASS**: 4 req. | NOT_RUN | NOT_RUN |
| 2, entrada por id (sessão nova) | **PASS**: `indice_entrada`, 3 req.; índice marca aceita | **PASS** | **PASS** |
| NUP inexistente | **PASS (recusa)**: `pesquisa_rapida_sem_url_de_trabalho` | NOT_RUN | NOT_RUN |
| `--sem-documentos` | **PASS**: sem `documentos`, 1 req. | NOT_RUN | NOT_RUN |
| `--unidade` errada | **PASS (recusa)**: `unidade_da_tela_diferente_da_solicitada` | NOT_RUN | NOT_RUN |

Em todos os PASS o NUP publicado é igual ao pedido, com id e tipo
presentes; as listas de documentos tiveram de 0 a 10 itens. O índice
ficou `0600` e registrou processos, links de `procedimento_trabalhar`,
`procedimento_visualizar` e `protocolo_pesquisa_rapida`.

## Observações

- **PEN1 aceita a entrada por id.** Medição nova: o Todos não tinha
  medido a PEN1 (503 na época). Com UnB e GO, são três instalações
  aceitas pelo próprio IES.
- **Recusa da entrada por id:** NOT_RUN ao vivo, porque nenhuma das três
  recusou; o caminho (registro da recusa e erro
  `entrada_por_id_recusada_rode_ies_login`) está coberto só pelos testes
  offline.
- UFRJ, PEN2, PEN3 e PEN4: NOT_RUN nesta rodada.
