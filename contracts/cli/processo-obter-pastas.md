---
type: Contract
title: Pastas da árvore do processo
description: ies processo NUP lê processos com pastas, abre as fechadas com um GET de Abrir todas as Pastas e confere cada pasta pela identidade dos protocolos declarados ou, para os documentos sem identidade na barra, pela contagem
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

## Integridade: identidade por documento, o resto pela contagem

Cada `Pastas[k]['protocolos'] = '<p1>,<p2>,…';` (decimais, sem repetição,
não vazia) é a lista declarada da pasta `k`; pasta sem declaração,
declaração sem pasta e declaração repetida são recusa.

**Por documento.** A identidade é o único `id_rel_protocolo_protocolo`
dos links da barra (`Nos[i].acoes = '…';`); valores diferentes são
recusa. Barra vazia, ou com links sem esse parâmetro (ao vivo:
`acompanhamento_gerenciar`, `documento_consultar`,
`documento_imprimir_web`, `documento_receber`, `procedimento_gerar_pdf`,
`protocolo_modelo_gerenciar`, `documento_consultar_recebido`, sem
`comentario_listar`), deixa o documento **sem identidade**. Documento sem
linha `.acoes` nunca foi observado e é
`documento_da_pasta_sem_identidade_na_barra`.

**Por pasta.** Toda identidade achada está na lista declarada e aparece
**exatamente uma vez** (`documento_da_pasta_nao_declarado`,
`protocolo_repetido_na_pasta`), e os declarados sem documento
identificado são **tantos quantos** os documentos sem identidade da pasta.
Se todos têm identidade, faltar um é `protocolo_declarado_ausente_da_pasta`;
senão, a diferença é `contagem_da_pasta_diferente_da_declarada`. Os ids
de documento (a1) são únicos na árvore inteira e, depois de abrir, não
resta `AGUARDE`. A `verificacao` da pasta é `por_identidade` (todos com
identidade), `mista` (alguns) ou `por_contagem` (nenhum).

A regra é a barra observada, nunca o texto "aberto somente na unidade" do
`.html` do processo: mensagem é apresentação.

A pasta fechada da primeira leitura não é conferida (os documentos dela
não estão lá); uma pasta com `AGUARDE` **e** documentos é
`pasta_carregada_pela_metade`. Nunca se publica lista parcial como
completa: qualquer divergência encerra o comando.

## Saída

Cada item de `documentos` ganha `pasta`: o id do nó (`"PASTA1"`) ou `null`
na raiz. Ao lado, `pastas` lista as pastas na ordem da árvore:

```json
{"documentos": [{"id_documento": "D", "rotulo": "…", "numero_sei": "…", "pasta": "PASTA1"}],
 "pastas": [{"id": "PASTA1", "rotulo": "I", "total": 20, "verificacao": "mista",
             "conferidos_por_identidade": 19, "conferidos_por_contagem": 1}],
 "verificacao_das_pastas": "mista",
 "verificacao_das_pastas_motivo": "documentos_sem_identidade_na_barra"}
```

`total` é a quantidade declarada em `Pastas[k]['protocolos']`, já
conferida; `verificacao` e as duas quantidades dizem como. No topo,
`verificacao_das_pastas` é a mais fraca presente (`por_contagem` <
`mista` < `por_identidade`) ou `null` sem pastas; se alguma pasta não é
toda por identidade, `verificacao_das_pastas_motivo:
"documentos_sem_identidade_na_barra"`. Identidade só é alegada onde foi
conferida. `pastas` sai `[]` sem pastas, e as três
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
`protocolo_declarado_ausente_da_pasta`,
`contagem_da_pasta_diferente_da_declarada`, `pasta_carregada_pela_metade`,
`pastas_nao_carregadas_sem_acao_de_abrir`,
`acao_de_abrir_pastas_fora_da_forma_observada`,
`acoes_de_abrir_pastas_repetidas`,
`endereco_de_abrir_pastas_nao_reconhecido`,
`pastas_nao_carregadas_apos_abrir_pastas`. Login depois de abrir é
`sessao_encerrada_rode_ies_login`; outro status, o erro de transporte.

## Limitações

O documento sem identidade só prova a quantidade e a unicidade, não
que é o protocolo declarado; por isso a saída diz `mista` ou
`por_contagem`. Ao vivo na UnB (03/11/2026, medição do agente pai) a
identidade variou **por documento**: num processo de 8 pastas, seis com
todas as barras vazias, uma com 18 vazias e 2 com links sem identidade,
outra com 12 e 5; noutro, 19 com identidade e 1 vazia numa pasta. Pastas noutras instalações: não
observadas. Validação live do comando: **NOT_RUN** até o agente pai.
