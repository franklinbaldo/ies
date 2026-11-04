---
type: SEIObservation
title: Ensaio live de documento DOC pela Web na UnB
description: PASS/FAIL/NOT_RUN da leitura de documentos internos pela pesquisa rápida, pelo índice (link da árvore, link de trabalho, entrada por id com documento) e por --processo, árvore e link assinado do conteúdo, e das recusas tipadas, UnB 4.0.9, 04/11/2026
---

# Ensaio live de `documento DOC` — 04/11/2026

[Leitura Web](../../contracts/cli/documento-obter-web.md), binário em
release do ramo `slice-documento`, UnB 4.0.9 (`aluno1`, Unidade1), sessão
existente validada por `ies login` (1 requisição), índice da sessão
(com o `action` da pesquisa rápida já colhido) e, no último caso, índice
novo. Orçamento declarado: até 20 comandos, ≥ 20 s entre eles, só GET e
o POST da pesquisa rápida, nenhuma repetição. Usados: 9 comandos, 36
requisições pela contagem do caminho (a recusa não publica a contagem). Só formas, números SEI e contagens abaixo.

## Resultado (UnB 4.0.9)

| Caso | Pedido | Resultado |
| --- | --- | --- |
| Conteúdo, `action` guardado | `0011505` | **PASS**: 4 req., `Despacho 0011505` do processo `23106.001559/2026-11`, ícone `documento_interno` |
| Lote | `0011506 0011527` | **PASS**: 8 req. (4 + 4), dois documentos do mesmo processo |
| Lote com documento sem conteúdo | `0011506 0011525` | **PASS** (recusa tipada): `conteudo_nao_publicado_na_arvore` em `0011525`, `documentos_lidos_antes: 1`, nada publicado |
| Externo (PDF) | `0011537` | **PASS** (recusa tipada): `documento_externo_por_download_de_anexo`, 3 req. (POST, tela, árvore), sem GET do anexo |
| Número sem os zeros | `11505` | **PASS** (recusa tipada): a pesquisa abriu o documento, a árvore traz `0011505`; `numero_sei_da_arvore_diferente_do_pedido`, 3 req. (POST, tela, árvore) |
| Número inexistente | `9999999` | **PASS** (recusa tipada): `pesquisa_rapida_nao_abriu_o_documento`, 1 req. (a pesquisa respondeu 200) |
| `--saida DIR`, texto humano | `0011505` | **PASS**: `0011505.txt` 0600, 925 bytes, sem entidade HTML restante |
| Conteúdo pelo Controle (índice novo) | `0011527` | **PASS**: 5 req. (Controle, POST, tela, árvore, conteúdo) |
| Documento em pasta fechada, cancelado, duplicado na árvore | — | NOT_RUN: sem caso conhecido na UnB; cobertos por fixture sintética e pelo [replay](../validacao-replay/2026-11-04-documento-obter-acervo.md) |

Nenhuma saída trouxe `infra_hash`, URL ou cookie (conferido por busca
nas saídas JSON).

## Observações

- O primeiro ensaio mostrou o texto com entidades HTML cruas
  (`Coordena&ccedil;&atilde;o`, `&nbsp;`, `&ordm;`): o leitor do corpo
  passou a decodificá-las; o caso `--saida` e o do Controle já rodaram
  com a correção.
- `0011525` tinha `.src` de `documento_visualizar` numa captura privada
  de 03/11 feita na Unidade2; hoje, na Unidade1, o `.src` veio vazio. A
  árvore daquela captura não publica ação `ASSINATURA` para o documento:
  coerente com minuta visível só à unidade geradora. Causa não medida.
- A pesquisa rápida do SEI aceita o número sem zeros e abre o mesmo
  documento; o IES recusa porque o número é opaco. Pedir com os zeros.
- Efeito: a pesquisa pode ficar lembrada na sessão; se o GET do
  conteúdo marca o documento como visitado não foi medido.

## Fatia 2: índice e `--processo` (UnB 4.0.9)

Binário em release do ramo `slice-doc2`, sessão existente da UnB
(`aluno1`, Unidade1), **cópia** do índice da sessão (migrada da versão 1
para a 2; o original ficou intacto para os outros agentes, cujo binário
desligaria o índice diante da versão 2). Linhas de link removidas da
cópia entre casos para forçar cada passo. Orçamento declarado: até 20
comandos, ≥ 20 s entre eles, só GET e o POST da pesquisa rápida, nenhuma
repetição. Usados: 12 comandos, **38 requisições**, contadas pelos túneis
CONNECT de um relay local (iguais às publicadas em todos os casos).
Processo A: `23106.001559/2026-11`; B: `23106.001767/2026-11` (22
documentos em 2 pastas, PASTA1 fechada na árvore simples).

| Caso | Pedido | Resultado |
| --- | --- | --- |
| Sem o número no índice, `action` guardado | `0011505` | **PASS**: `pesquisa_rapida`, 4 req.; o índice aprendeu os 7 documentos de A |
| Índice + link da árvore | `0011506` | **PASS**: `indice_link`, **2** req. (antes 4), `unidade` `null` |
| `--processo` com link da árvore | `0011527 --processo A` | **PASS**: `indice_link`, 2 req. |
| Entrada por id com documento (sem links de A) | `0011505` | **PASS**: `indice_entrada_documento`, 4 req. (entrada, tela, árvore, conteúdo); 302 aceito e conferido; aceite gravado só para esta forma |
| `--processo` frio, documento em pasta fechada | `0011917 --processo B` | **PASS**: `pesquisa_rapida` do NUP, 5 req. (POST, tela, árvore, abrir pastas, conteúdo); `PASTA1`; 22 documentos aprendidos |
| Documento visível em pasta aberta | `0011935 --processo B` | **PASS**: 2 req., `PASTA2`, sem abrir pastas |
| Índice, documento em pasta fechada | `0011918` | **PASS**: `indice_link`, 3 req. (árvore, abrir pastas, conteúdo) |
| Índice desmentido (número apontado para A) | `0011919` | **PASS**: árvore de A sem o número, palpite esquecido, uma queda na pesquisa: 5 req.; reaprendido em B |
| `--processo` sem o número | `0011505 --processo B` | **PASS** (recusa tipada): `numero_sei_ausente_da_arvore_do_processo`, 1 req. (link de abrir pastas) |
| Lote pelo índice | `0011506 0011527` | **PASS**: 4 req. (2 + 2; antes 8) |
| Entrada com documento em pasta fechada | `0011920` | **PASS**: 4 req., `PASTA1` sem abrir pastas |
| Texto humano | `0011936` | **PASS**: cabeçalho sem unidade (link da árvore), 2 req. |
| Recusa da entrada com documento, link de trabalho guardado | — | NOT_RUN: a UnB aceita; nenhum link de trabalho de A ou B na geração; cobertos por fixture sintética |

Nenhuma saída trouxe `infra_hash`, `controlador.php` ou cookie (busca nas
saídas; o texto do documento traz o endereço público de verificação).
A sessão seguiu válida depois da entrada com documento.

### Observações da fatia 2

- A tela aberta com `id_documento` (pela entrada com documento ou pela
  pesquisa rápida do número) serve a árvore com a pasta do documento
  carregada: `0011919` e `0011920` vieram de `PASTA1` sem "Abrir todas as
  Pastas". A árvore aberta só pelo processo (link guardado, NUP) veio com
  `PASTA1` fechada.
- O índice da UnB não tinha linha em `instalacao`: a forma só com o
  processo nunca tinha sido registrada por um comando; a com documento
  ficou `aceita` depois do caso 4.
