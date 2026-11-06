---
type: CommandContract
title: Ciência do usuário no documento pelo SEI Web
operation: documento.ciente
command: ies documento ciente DOC [--confirmar]
effect: write-document-state
implementation: partial
---

# `ies documento ciente DOC [--confirmar]` — #271

A prévia do documento usa a árvore conferida pela identidade do número SEI e
processo, com `--processo NUP` quando conhecido. O botão
`cienciaDocumento();` precisa estar na **barra do nó desse documento**.
`--confirmar` obtém a tela `arvore_visualizar` já publicada, identifica o
link assinado de `documento_ciencia` no corpo da função (SEI 4.0) ou na
variável `linkCienciaDocumento` (4.1/5.0), e envia **um único GET mutador**,
sem seguir redirecionamento, sem retry e sem executar JavaScript.

O aceite é vinculado ao documento por `id_procedimento`, `id_documento`,
`documento_visualizar_ciencias=1` e `id_atividade`; nunca basta status 200.
Uma **nova leitura** encontra a árvore e a Lista de Ciências com ação
`CIENCIAS`, código `CD<id_documento>` e URLs conferidas para **ambos**
os IDs. Somente a atividade exata no checkbox dessa lista comprova a escrita.
A Lista de Ciências do processo ou de outro documento não serve. O SEI
registra sempre em nome do usuário da sessão; não inferir outros usuários.

Uma recusa, 302, timeout, árvore ou lista ilegível após GET mutador implicam
`efeito_incerto_na_ciencia_do_documento` e saída 3; jamais repetir escrita
automaticamente. A prévia nunca consulta `documento_ciencia`.

## Orçamento sintético (índice frio, processo informado)

| Fluxo | Requisições | GETs de escrita |
| --- | ---: | ---: |
| Número inválido | 0 | 0 |
| Prévia, barra oferece ciência | 3 | 0 |
| Barra não oferece ciência | 3 | 0 |
| GET mutador, árvore relida e lista conferida | 7 | 1 |
| GET mutador sem atividade ou falha, árvore relida | 6 | 1 |
| Atividade não encontrada na lista relida | 7 | 1 |

Índice quente pode reduzir a abertura. Não se afirma custo fixo para
`documento DOC` sem `--processo`.

## Subcódigos próprios

- `barra_da_ciencia_do_documento_fora_da_forma` — atributos da barra inválidos
- `botoes_de_ciencia_do_documento_repetidos` — botão duplicado
- `ciencia_nao_oferecida_no_documento` — barra sem botão
- `tela_da_ciencia_do_documento_nao_publicada` — nó sem tela vinculada
- `tela_da_ciencia_do_documento_redirecionada` — prévia redirecionada
- `efeito_incerto_na_ciencia_do_documento` — escrita sem prova posterior

Erros HTML de origem, de duplicidade e atributos têm causa
`ErroHtml` tipada; erros de transporte conservam a causa original.
A saída JSON publica documento, processo, unidade se observada,
`confirmado`, `verificado`, `id_atividade` (`null` quando não observada)
e número de requisições. Nunca imprime cookies, senhas ou URLs assinadas.

## Evidência e pendências

Oráculo: Todos `tests/test_documento_ciente_010.py` @323eb01
(UnB 4.0.9, segundo GET recusado pela validação da Infra).
[Decisão](../../docs/decisoes/documento-ciente.md) e
[relatório live](../../docs/validacao-live/2026-11-06-documento-ciente.md).
Replay integral e ensaio autenticado em UnB e segunda instalação
estão pendentes; não declarar a capacidade concluída sem eles.
