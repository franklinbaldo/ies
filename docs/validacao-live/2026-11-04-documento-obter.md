---
type: SEIObservation
title: Ensaio live de documento DOC pela Web na UnB
description: PASS/FAIL/NOT_RUN da leitura de documentos internos pela pesquisa rápida, árvore e link assinado do conteúdo, e das recusas tipadas, UnB 4.0.9, 04/11/2026
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
