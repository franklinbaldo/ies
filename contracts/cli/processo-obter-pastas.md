---
type: Contract
title: Pastas da árvore do processo
description: ies processo NUP lê processos com pastas, abre as fechadas com um GET de Abrir todas as Pastas e confere cada pasta pela identidade dos protocolos declarados ou, com todas as barras vazias, pela contagem
---

# Pastas de `ies processo NUP` (Refs #59, fatia 3)

Complementa o [contrato da árvore](processo-obter-arvore.md). Na UnB,
processos com mais de 20 documentos chegam com a árvore dividida em pastas
de até 20; só a mais recente vem carregada. Estrutura em
[árvore do processo](../../docs/como-o-sei-funciona/arvore-do-processo.md#pastas);
decisão em [processo.obter](../../docs/decisoes/processo-obter.md).

## Nós aceitos

- `PASTA` (15 argumentos): a1 `PASTA<k>`, a2 o id do processo, a3
  exatamente `javascript:abrirFecharPasta('PASTA<k>');`, a4 `null`, a5 o
  rótulo (`I`, `II`…). Pasta repetida, de outro pai ou fora dessa forma é
  recusa.
- `AGUARDE` (15 argumentos): a1 `AGUARDE<k>`, a2 `PASTA<k>` de uma pasta
  declarada, a3 `null`. Marca a pasta como **não carregada**.
- `DOCUMENTO`: o pai (a2) é o processo ou uma `PASTA<k>` declarada.
- Qualquer outro tipo continua `tipo_de_no_da_arvore_nao_suportado`.

A primeira pasta pode vir na mesma linha do `.html` do nó anterior
(`Nos[i-1].html = '…';Nos[i] = new infraArvoreNo(…);`), como observado;
só essa forma é aceita.

## Abrir as pastas fechadas

Se a árvore lida tem algum `AGUARDE`, o comando lê a ação
`NosAcoes[j] = new infraArvoreAcao("ABRIR_PASTAS","AP<id>","<id>","<URL>",null,"…","…",true);`
(uma só; duas são `acoes_de_abrir_pastas_repetidas`) e confere a URL como
os outros links assinados: `controlador.php?…` relativo, uma
`acao=procedimento_visualizar`, `id_procedimento` igual ao aberto, um
`abrir_pastas=1`, nenhum `fechar_pastas`, um `infra_hash` de 64 hex, na
mesma instalação. Faz **um** GET dela e lê de novo a árvore inteira, com a
mesma conferência de identidade. Sem a ação:
`pastas_nao_carregadas_sem_acao_de_abrir`, sem outra requisição. Depois
do GET, qualquer `AGUARDE` restante é
`pastas_nao_carregadas_apos_abrir_pastas`. Sem `AGUARDE`, nada a mais.

O IES **não** abre pasta por pasta (`procedimento_paginar` por POST) nem
chama `FECHAR_PASTAS`, e não restaura o estado das pastas.

## Integridade: por identidade ou por contagem

Cada `Pastas[k]['protocolos'] = '<p1>,<p2>,…';` (decimais, sem repetição,
não vazia) é a lista declarada da pasta `k`; pasta sem declaração,
declaração sem pasta e declaração repetida são recusa. A identidade de
cada documento de pasta é o `id_rel_protocolo_protocolo` dos links da sua
barra (`Nos[i].acoes = '…';`); links com valores diferentes são recusa.
Cada pasta carregada usa **uma** regra, escolhida pelas barras dela:

- **Por identidade** (`por_identidade`): toda barra da pasta tem
  identidade. Cada protocolo declarado aparece **exatamente uma vez** entre
  os documentos da pasta, e nenhum documento fica fora da lista.
- **Por contagem** (`por_contagem`): toda barra da pasta é exatamente
  `Nos[i].acoes = '';`. A quantidade de documentos da pasta tem de ser a
  declarada (`contagem_da_pasta_diferente_da_declarada`); os ids de
  documento (a1) continuam únicos na árvore inteira e, depois de abrir,
  não pode restar `AGUARDE`.
- **Mista** (barras vazias ao lado de barras com identidade):
  `identidade_parcial_na_pasta`. As regras nunca se misturam.
- Barra com links e sem identidade, ou documento sem linha `.acoes`:
  `documento_da_pasta_sem_identidade_na_barra`.

A regra é o fato observado (as barras vazias), nunca o texto "aberto
somente na unidade" do `.html` do processo: mensagem é apresentação.

A pasta fechada da primeira leitura não é conferida (os documentos dela
não estão lá); uma pasta com `AGUARDE` **e** documentos é
`pasta_carregada_pela_metade`. Nunca se publica lista parcial como
completa: qualquer divergência encerra o comando.

## Saída

Cada item de `documentos` ganha `pasta`: o id do nó (`"PASTA1"`) ou `null`
na raiz. Ao lado, `pastas` lista as pastas na ordem da árvore:

```json
{"documentos": [{"id_documento": "D", "rotulo": "…", "numero_sei": "…", "pasta": "PASTA1"}],
 "pastas": [{"id": "PASTA1", "rotulo": "I", "total": 20, "verificacao": "por_contagem"}],
 "verificacao_das_pastas": "por_contagem",
 "verificacao_das_pastas_motivo": "barras_vazias"}
```

`total` é a quantidade declarada em `Pastas[k]['protocolos']`, já
conferida, e `verificacao` a regra da pasta. No topo,
`verificacao_das_pastas` é `"por_identidade"`, `"por_contagem"` (se ao
menos uma pasta usou a contagem, a mais fraca) ou `null` sem pastas; com
contagem, `verificacao_das_pastas_motivo: "barras_vazias"`. Identidade só
é alegada onde foi conferida. `pastas` sai `[]` sem pastas, e as três
chaves somem com `--sem-documentos`.
A ordem dos documentos é a da árvore servida.

## Índice e orçamento

A URL conferida de "Abrir todas as Pastas" é guardada com a chave
`acao = procedimento_visualizar` e `objeto = "<id>:abrir_pastas"`
(distinta do link da árvore, objeto `<id>`). No caminho 1 ela é tentada
**antes** do link da árvore; login, redirect ou árvore que não é a do
processo a apagam e o link da árvore segue.

| Situação | Requisições até a árvore |
| --- | --- |
| frio, sem `AGUARDE` | as do caminho ([contrato](processo-obter-arvore.md)) |
| frio, com `AGUARDE` | caminho + 1 GET |
| quente, link de abrir pastas guardado | 1 GET |
| quente, só o link da árvore, com `AGUARDE` | 2 GET |

**Efeito declarado.** O GET de `ABRIR_PASTAS` é declarado
`PaginaOuFiltroLembrado` (a ação irmã `FECHAR_PASTAS` sugere estado
lembrado). Nas cassettes da UnB, a árvore simples lida depois dele voltou
fechada; não medido pelo IES.

**Tamanho.** Uma árvore aberta de 157 documentos tem cerca de 1 MB; o
limite do transporte é 8 MB e a leitura é linear, linha a linha.

## Falhas (saída 1)

`pasta_fora_da_forma_observada`, `pasta_repetida_na_arvore`,
`pasta_com_pai_diferente_do_processo`, `aguarde_fora_da_forma_observada`,
`aguarde_com_pai_diferente_de_pasta`,
`protocolos_da_pasta_fora_da_forma_observada`,
`protocolos_da_pasta_declarados_mais_de_uma_vez`,
`protocolos_declarados_para_pasta_ausente`,
`pasta_sem_protocolos_declarados`, `barra_de_no_fora_da_forma_observada`,
`identidades_divergentes_na_barra_do_documento`,
`documento_da_pasta_sem_identidade_na_barra`,
`documento_da_pasta_nao_declarado`, `protocolo_repetido_na_pasta`,
`protocolo_declarado_ausente_da_pasta`, `identidade_parcial_na_pasta`,
`contagem_da_pasta_diferente_da_declarada`, `pasta_carregada_pela_metade`,
`pastas_nao_carregadas_sem_acao_de_abrir`,
`acao_de_abrir_pastas_fora_da_forma_observada`,
`acoes_de_abrir_pastas_repetidas`,
`endereco_de_abrir_pastas_nao_reconhecido`,
`pastas_nao_carregadas_apos_abrir_pastas`. Login depois de abrir é
`sessao_encerrada_rode_ies_login`; outro status, o erro de transporte.

## Limitações

Por contagem, a pasta prova a quantidade e a unicidade dos documentos,
não que cada um é o protocolo declarado; por isso a saída diz
`por_contagem`. Ao vivo na UnB (03/11/2026, medição do agente pai), dois
processos abertos só em outra unidade vieram com 150 barras vazias e
contagens iguais às declaradas. Pastas noutras instalações: não
observadas. Validação live do comando: **NOT_RUN** até o agente pai.
