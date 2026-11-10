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
`ErroCienciaDocumento::NaoConfirmado` e saída 1; jamais repetir escrita
automaticamente. A prévia nunca consulta `documento_ciencia`. Na apresentação humana, identifica documento, rótulo e processo, acrescentando a unidade apenas quando observada na abertura; separa as frases com espaçamento normal e exibe `--confirmar` e a contagem de requisições. A saída JSON mantém a unidade opcional e não inventa dado ausente. A causa tipada
do único GET mutador permanece em `falha_do_aceite`, no erro:
transporte/status HTTP (`Transporte(Status(..))`), validação da Infra
(`ValidacaoDaInfra(mensagem)`, numa resposta 200), ou aceite sem atividade
reconhecível, no `Debug` do erro. Não confundir ausência de prova com ausência de efeito.

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

| Erro (stderr) | Condição observada |
| --- | --- |
| `ErroBarraDoDocumento::BarraDaCienciaDoDocumentoForaDaForma` | Atributos da barra inválidos |
| `ErroBarraDoDocumento::BotoesDeCienciaDoDocumentoRepetidos` | Botão duplicado |

- `barra_da_ciencia_do_documento_fora_da_forma` — atributos da barra inválidos
- `botoes_de_ciencia_do_documento_repetidos` — botão duplicado
- `ciencia_nao_oferecida_no_documento` — barra sem botão
- `tela_da_ciencia_do_documento_nao_publicada` — nó sem tela vinculada
- `tela_da_ciencia_do_documento_redirecionada` — prévia redirecionada
- `ErroCienciaDocumento::NaoConfirmado` — escrita sem prova posterior

Erros HTML de origem, de duplicidade e atributos são do leitor que os
detecta (`ErroBarraDoDocumento`, `ErroCienciaNaArvore`); erros de
transporte conservam a causa original.
A saída JSON publica documento, processo, unidade se observada,
`confirmado`, `verificado`, `id_atividade` (`null` quando não observada),
`usuario` (nome da coluna explícita «Usuário» da mesma linha da atividade na
Lista de Ciências relida, ou `null` se não publicado) e número de requisições.
A ausência de cabeçalho não autoriza inferir o autor de outra coluna,
revisão ou documento. A atividade continua sendo a prova da escrita;
a autoria exibida não prova identidade autenticada fora da sessão. Nunca imprime cookies, senhas ou URLs assinadas.

## Evidência e pendências

Oráculo: Todos `tests/test_documento_ciente_010.py` @323eb01
(UnB 4.0.9, segundo GET recusado pela validação da Infra).
[Decisão](../../docs/decisoes/documento-ciente.md) e
[relatório live](../../docs/validacao-live/2026-11-06-documento-ciente.md).
Replay integral e ensaio autenticado em UnB e segunda instalação
estão pendentes; não declarar a capacidade concluída sem eles.

Também no aceite documental, `atualizarArvore` precisa constar em
código reconhecido da resposta: texto em comentário, literal ou
`<template>` não comprova `id_atividade`. A ausência de prova
depois do GET potencialmente mutador é escrita não confirmada, saída 1,
sem repetição da escrita.


### Aceite com argumento adicional (#271)

A prova estática exige atualizarArvore('url') com um único argumento
literal. Um argumento adicional pode lançar uma exceção antes da
recarga e não comprova uma ciência documental. O leitor compartilhado
recusa a forma não observada; após o GET mutador, verifica por leitura
e devolve escrita não confirmada, sem retry ou sucesso por ciência anterior.


### Lista de Ciências em contêiner inerte (#271)

O parser HTML pode encontrar `table#tblAndamentos` dentro de um
`<template>` ancestral. Essa tabela não integra a interface exibida e
não comprova ciência, ainda que seu checkbox contenha o `id_atividade`
do aceite. A releitura a recusa como forma não observada, retorna
**escrita não confirmada (3)** e não repete o GET de escrita. Também se recusa
`<template>` descendente da tabela, sem aceitar IDs antigos ou ocultos.

## Segunda ciência e HTML inerte (#271, 06/11/2026)

O Todos `tests/test_documento_ciente_010.py` @323eb01 registrou que a
**segunda ciência no mesmo documento**, sem limite de dia, é recusada com
`txaInfraValidacao`: «Usuário já deu ciência neste documento.».
A recusa HTTP 200 preserva a causa tipada `ValidacaoDaInfra`, não comprova
nova escrita e não autoriza repetição. Teste sintético: 6 GET totais,
um GET potencialmente mutador, uma releitura e saída 1.

O HTML da Lista de Ciências sob `<noscript>`, ou contendo checkbox dentro
dele, não comprova ciência em contexto com JavaScript habilitado. O
reconhecedor rejeita `<noscript>` ou `<template>` ancestral/descendente
da tabela de prova; a confirmação sem prova fica em **escrita não confirmada (3)**.
Teste documental: 7 GET, exatamente 1 mutador, sem retry. Nenhum efeito
adicional da prévia. Falta validar as formas nas instalações de treinamento.

## Script de aceite em `noscript` (regressão #271)

Em `<noscript>`, um `<script>` não executado não comprova a recarga
da árvore nem o `id_atividade` do documento quando a sessão Web usa JS.
O leitor compartilhado recusa a forma inerte, mesmo que o parser HTML
a exponha, e a ausência de prova após o GET único resulta em saída 1,
sem repetir escrita. RED do leitor e do fluxo de processo:
[CI 42302428489](https://github.com/franklinbaldo/ies/actions/runs/42302428489),
2 falhas novas / 1.775 PASS / 22 ignored preexistentes.

O replay offline dedicado de `processo-ciente` e `documento-ciente`
usa os mesmos leitores e hashes do acervo. Não converte resposta 200/302
em confirmação, nem dispensa leitura independente live.

## Replay específico do acervo (#271)

O [relatório de replay](../../docs/validacao-replay/2026-11-06-ciencia.md)
registra 14 cards, 42 respostas e integridade verificada em sete
instalações; porém o leitor recusou os 28 aceites como sem atividade
reconhecida e não pôde verificar nenhuma das 14 listas. O teste de
execução passou, **não a prova semântica**. Confrontar as respostas
reais e separar tela preparatória de GET mutador antes de alegar
compatibilidade. Os testes sintéticos ainda não substituem live.

## Script ativo e contêiner inerte na mesma resposta (#271)

A presença de um `<template>` ou `<noscript>` com script inerte não
invalida um **outro script ativo** que publique a atividade do documento.
O leitor ignora por nó DOM somente o script inerte; não confunde texto
`<template><script>` dentro de literal JavaScript com marcação HTML.
Identidade documental, releitura e política de GET único não mudam.

## Tipo MIME do script do aceite (#271)

O leitor da resposta aceita a chamada literal apenas em script clássico:
`type` ausente ou vazio, `text/javascript` ou `application/javascript`,
com comparação MIME sem distinção de maiúsculas e minúsculas. O conteúdo
continua inerte com `src`, tipo de dados ou parâmetro MIME no atributo
(`text/javascript; charset=utf-8`); ele não pode provar a atividade.
Identidade do alvo, GET mutador único e releitura não mudam.

## Recusa Infra HTTP 200 na apresentação (#271)

Em resposta `txaInfraValidacao`, o SEI pode recusar uma segunda ciência
com HTTP **200**. A falha sai com 1 e
`Error: NaoConfirmado { requisicoes, falha_do_aceite: Some(ValidacaoDaInfra(mensagem)), .. }`
(`ErroCienciaDocumento`), com as `requisicoes` reais observadas pelo fluxo. HTTP 200 não significa sucesso nem autoriza retry:
a ciência continua sem prova de escrita e exige conferência no SEI.
A contagem de requisições observadas acompanha a escrita não confirmada; antes de outra confirmação, confira a Lista de Ciências e **não repita a escrita**. Nova prévia continua permitida: ela é somente leitura.
Testes públicos de apresentação preservam a contagem sintética de
**6 GET, um potencialmente mutador, nenhum retry**; live NOT_RUN.
 
## Escrita não confirmada (#271, 07/11/2026; forma tipada desde o #695)

Com efeito incerto, a falha é `ErroCienciaDocumento::NaoConfirmado`,
saída 1, com `requisicoes` observadas, a verificação e a causa original
do aceite (incluindo o HTTP), sem GET extra. Antes de uma **nova
confirmação**, consulte a **Lista de Ciências do documento**; nova prévia
de leitura continua permitida.
RED da regressão JSON:
[CI 42367722667](https://github.com/franklinbaldo/ies/actions/runs/42367722667),
**1 FAIL novo, 1.872 PASS, 24 ignored**, antes da correção.

## Instruções vazias no aceite (WIP #271)

Separadores `;` sem instrução antes da recarga não tornam a chamada
inalcançável. O leitor conserva a identidade do documento e a prova
posterior na Lista de Ciências; instruções anteriores que possam
interromper a recarga continuam não reconhecidas.
