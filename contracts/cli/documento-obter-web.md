---
type: Contract
title: Ler documento pela Web
description: ies documento DOC lê pela sessão Web o texto e os metadados de documentos internos, pela pesquisa rápida do número SEI, a árvore e o link assinado do conteúdo, com recusa tipada para os demais casos
---

# `ies documento DOC` pela Web (Refs #60, fatia 1)

Complementa o [contrato de `documento.obter`](documento-obter.md).
`ies documento DOC [DOC ...] [--saida DIR] [--json] [--unidade SIGLA]` usa
a sessão de `ies login` e lê, na ordem pedida, o conteúdo de cada
documento **interno** (editor do SEI) pelo número SEI. Opções ainda sem
implementação (`--processo`, `--versao`, `--sem-copia`,
`--com-propriedades`, `--ocultar-inacessiveis`) continuam no stub
(`nao_implementado`, saída 1). `--html-privado` segue o recorte local.

## Caminho e requisições

Por documento, sem retry e sem seguir redirect:

1. pesquisa rápida do número SEI exatamente como pedido: POST ao
   `action` guardado no [índice local](../../docs/arquitetura/0003-indice-local.md)
   ou, sem ele, GET do Controle e POST pelo `form#frmProtocoloPesquisaRapida`
   servido (o Controle lido é colhido no índice);
2. 302 para `procedimento_trabalhar` com `id_protocolo` (o id do
   documento), conferido como no `processo NUP`;
3. GET da tela; o `src` do `iframe#ifrArvore` tem de trazer
   `id_documento` igual a esse id e um `id_procedimento`;
4. GET da árvore, conferida pelo NUP do próprio nó `PROCESSO` e pelo id
   do processo; o `DOCUMENTO` de mesmo id tem de ter o número SEI (a15)
   **igual** ao pedido (opaco, zeros à esquerda inclusos);
5. GET do `Nos[i].src` assinado que a árvore publica
   (`documento_visualizar` com o mesmo `id_documento`); o título
   `SEI/… - DOC - tipo` tem de ser do número pedido.

Custo: **4** requisições com o `action` guardado, **5** sem ele. O IES
nunca monta URL de documento: as sem assinatura derrubam a sessão de
todos (ADR 0003). O índice aprende o par NUP ↔ id, o link da árvore e o
`action` da pesquisa rápida da tela.

## Saída

JSON `{"documentos": [...], "requisicoes": N}`; cada documento traz
`numero`, `id_documento`, `rotulo` (a5), `tipo` (do título), `icone`,
`cancelado`, `pasta`, `processo` (`nup`, `id`, `tipo`), `unidade` (da
tela), `resolvido_por: "pesquisa_rapida"`, `requisicoes`,
`fonte: "documento_visualizar"`, `versao_atual: null` e `texto` (o texto
visível do corpo, com as entidades HTML decodificadas). Sem `--json`, uma
linha por documento e o texto. Nunca URL, `infra_hash`, cookie ou HTML.

Com `--saida DIR` (diretório existente), cada texto vai para `DOC.txt`
0600, sem sobrescrever nem seguir symlink, **depois** de todos lidos; o
JSON troca `texto` por `arquivo` e `bytes`.

## Falhas tipadas

O primeiro documento que falha encerra o comando (saída 1, nada
publicado), com `documento` e `documentos_lidos_antes` no JSON.

| Subcódigo | Fato |
| --- | --- |
| `numero_documento_repetido` (saída 2) | o mesmo número duas vezes no pedido |
| `pesquisa_rapida_nao_abriu_o_documento` | a pesquisa não respondeu o 302 (visto com número inexistente; o resultado não é lido) |
| `endereco_da_arvore_de_outro_documento` | a tela não seleciona o documento pesquisado (o número era de um processo) |
| `numero_sei_da_arvore_diferente_do_pedido` | o nó tem outro a15 (por exemplo `11505` pedido, `0011505` na árvore) |
| `numero_sei_repetido_na_arvore` | dois `DOCUMENTO` com o número pedido |
| `documento_ausente_da_arvore` / `documento_em_pasta_nao_carregada` | o id não está na árvore (com pasta fechada, o segundo) |
| `conteudo_nao_publicado_na_arvore` | `.src` vazio: a árvore não publica o conteúdo à sessão (visto com documento não assinado de outra unidade) |
| `documento_cancelado_sem_conteudo_na_arvore` | `.src` vazio num nó cancelado |
| `documento_externo_por_download_de_anexo` | `.src` para `documento_download_anexo` (PDF, TXT, HTML recebido): fatia futura |
| `link_do_conteudo_*`, `endereco_do_conteudo_*` | `.src` ausente, repetido, fora da forma ou de outro documento |
| `titulo_do_conteudo_de_outro_documento` | o HTML servido é de outro número SEI |

Sessão, Controle, formulário, 302, tela e árvore usam os subcódigos de
[`processo NUP`](processo-obter-arvore.md), inclusive
`sessao_encerrada_rode_ies_login`.

## Efeitos

Leitura. A pesquisa rápida pode ficar lembrada na sessão; o GET da
árvore pode marcar o processo como visualizado (medido em GO no #59).
Se o GET do conteúdo marca o documento como visitado não foi medido. O
IES não restaura a tela.
