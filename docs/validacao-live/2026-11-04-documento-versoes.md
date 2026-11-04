---
type: SEIObservation
title: Ensaio live de documento DOC --versao N e cópia de trabalho na PEN4
description: PASS/FAIL/NOT_RUN de --versao N, versao_atual pelo rodapé e pela lista, cópia DOC.vN.html e recusas tipadas, PEN4 5.0.4, 04/11/2026
---

# Ensaio live de versões de documento — 04/11/2026

[Contrato de versões](../../contracts/cli/documento-obter-versoes.md),
binário em release do ramo `slice-versoes` (casos `a`–`h` com o GREEN
inicial, ainda sem o leitor do rodapé e sem commit próprio; `i`–`p` no
commit `5cdf01b`), PEN4 5.0.4
(unidade TESTE), sessão e índice existentes (com o `action` da pesquisa
rápida guardado). Orçamento declarado: até 20 comandos, ≥ 22 s entre
eles, só GET e o POST da pesquisa rápida, nenhuma repetição, nenhuma
escrita (nenhum documento editado para criar versão). Usados: 16
comandos, 68 requisições contadas por um relay CONNECT local (um túnel
por requisição). Só formas, números SEI e contagens abaixo; nenhuma URL
assinada, `infra_hash` ou cookie foi impresso (o texto de `0000867`
cita a URL pública de conferência do SEI, sem hash).

Documentos com mais de uma versão: `0000867` (assinado) e `0000878`
(não assinado), ambos com 2 versões, criados pelos testes do Todos em
20/10/2026 em processos abertos na TESTE.

## Resultado (PEN4 5.0.4)

| Caso | Pedido | Req. | Resultado |
| --- | --- | --- | --- |
| a | `0000871` (sem leitor do rodapé) | 4 | **PASS**: `versao_atual: null`, `nao_pedida` |
| i | `0000871` | 4 | **PASS**: `versao: 2`, `versao_atual: 2` do rodapé, 0 GET a mais |
| b | `0000871 --versao 1` (processo concluído) | 3 | **PASS** (recusa tipada): `lista_de_versoes_nao_publicada_na_arvore`, nenhum GET depois da árvore |
| e | `0000867 --versao 1` | 5 | **PASS**: texto da versão 1 (rodapé `versão 1`), `versao_atual: 2` da lista, conteúdo atual não pedido |
| f | `0000867 --versao 2` | 5 | **PASS**: texto da versão 2, diferente da 1 (assinatura e rodapé `versão 2`) |
| g | `0000867 --saida` (assinado) | 4 | **PASS**: `0000867.txt`; sem cópia, `edicao_nao_publicada_na_arvore` (a barra do assinado não publica `editarConteudo` na 5.0.4); lista não pedida |
| h | `0000878 --saida` (sem leitor do rodapé) | 5 | **PASS**: cópia `0000878.v2.html` (35 520 bytes), base e manifesto 0600, versão pela lista |
| j | `0000878 --saida` | 4 | **PASS**: mesma cópia, versão 2 pelo rodapé, sem a lista; byte a byte igual à de `h` |
| — | `documento salvar 0000878` em `h` (local) | 0 | **PASS**: prévia `versao_base: 2`, `alterado: false` |
| k | `0000878 --versao 1` | 1 | **NOT_RUN**: 403 do Cloudflare no POST da pesquisa rápida (saiu como `pesquisa_rapida_nao_abriu_o_documento` com `status_http` 403; não repetido) |
| l | `0000878 --versao 9` (6 min depois) | 4 | **PASS** (recusa tipada): `versao_nao_observada`; a versão não é pedida |
| m | `0000878 --versao 2 --saida` | 5 | **PASS**: cópia v2 pela linha 2 da lista, igual à de `j` |
| n | `0000878 --versao 1 --saida` | 5 | **PASS**: texto da v1 (rodapé `versão 1`), `versao_atual: 2`, sem cópia, `versao_historica` |
| o | `0000566 --versao 1` (ZIP de processo de outra unidade) | 3 | **PASS** (recusa tipada): `conteudo_nao_publicado_na_arvore`, antes da barra |
| p | `0000878 --saida --sem-copia` | 4 | **PASS**: só `0000878.txt`, `copia_motivo: nao_pedida` |
| c, d | `processo NUP` (achar documentos) | 5 e 7 | apoio: árvores dos dois processos |

Orçamentos medidos iguais aos dos testes (`G` = 3 com o `action`
guardado): leitura G + 1; `--versao N` G + 2; cópia G + 1 com o rodapé.

## Não executado

- Documento externo por download de anexo com `--versao`: sem PDF
  conhecido na TESTE; coberto por fixture (recusa antes da barra).
- `editarConteudo('S')` ao vivo: a 5.0.4 não publicou a edição no
  assinado; a forma `'S'` está no acervo (todas as instalações).
- Cópia já existente (`copia_versao_ja_existe`): só em teste local.
- UnB e PEN1 (pedidos pelo #88): fora deste ensaio (só PEN4).

## Observação lateral

O 403 do Cloudflare no POST da pesquisa rápida não é classificado como
`bloqueio_de_seguranca_da_instalacao` nesse caminho (vale para
`documento DOC` desde o #60): a tela de bloqueio virou
`pesquisa_rapida_nao_abriu_o_documento`. Corrigir no transporte da
pesquisa, fora deste recorte.
