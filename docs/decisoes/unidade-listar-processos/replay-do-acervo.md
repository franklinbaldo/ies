---
type: ImplementationDecision
title: Replay de conformidade do acervo público pelos leitores do Controle de Processos
operation: unidade.listar_processos
implementation: partial
status: adopted-for-scope
scope: apêndice da decisão de unidade.listar_processos (Refs #57); um teste ignorado do próprio crate passa cada resposta gravada de procedimento_controlar (GET e POST) das 495 cassettes públicas pelos leitores de produção de ies unidade listar-processos, recebidos e gerados, offline, classifica lido, recusado ou não aplicável e confere a continuidade entre páginas gravadas; sem superfície nova na CLI e sem rede no CI
selected_strategy: teste-ignorado-no-crate-com-base-compartilhada
alternatives: teste-ignorado-no-crate-com-base-compartilhada,segundo-harness-independente,replay-pelo-fluxo-com-envio-gravado,corpos-gravados-como-fixture-do-ci
evidence_level: replay-offline-do-acervo-publico-e-testes-sinteticos
live_validation: not-run-replay-offline-nao-substitui-ensaio-live
---

# Replay do acervo — Controle de Processos

Apêndice da [decisão de `unidade.listar_processos`](../unidade-listar-processos.md).
Contratos: [visão detalhada](../../../contracts/cli/unidade-listar-processos-visao-detalhada.md),
[páginas](../../../contracts/cli/unidade-listar-processos-paginas.md) e
[Recebidos/Gerados](../../../contracts/cli/unidade-recebidos-gerados.md).
Resultado no [relatório do replay](../../validacao-replay/2026-11-04-unidade-listar-processos-acervo.md).

O critério aberto do #57 pede cassettes reais verificadas, replays e
testes de múltiplas páginas e de casos negativos. Os leitores do
Controle foram escritos a partir de capturas privadas e de poucas
cassettes; o acervo tem 757 respostas de `procedimento_controlar` nas
sete instalações. A pergunta é se os **mesmos leitores** as leem, onde
recusam e o que o acervo prova sobre páginas.

## Alternativas consideradas

- **teste-ignorado-no-crate-com-base-compartilhada** (escolhida): módulo
  `cfg(test)` `visao_do_controle::acervo`, filho do módulo da troca de
  visão, com acesso aos passos e ao plano da D. Percorrer os cards,
  conferir a cassette, a classe da leitura e a tabela leitor × instalação
  saíram do replay de processo NUP (#59) para `cassette::acervo`, usado
  pelos dois; cada replay tem sua tabela de leitores e seu relatório.
- **segundo-harness-independente**: copiar o de #59. Rejeitada: duas
  cópias da conferência e da iteração divergiriam.
- **replay-pelo-fluxo-com-envio-gravado**: rodar `listar_na_visao_detalhada`
  com um `EnvioDoControle` que devolve o POST seguinte gravado.
  Rejeitada: as cassettes não guardam o corpo dos POST nem a ordem de
  requisições do IES; o par GET→POST gravado não é o do fluxo. Os
  leitores de cada tela, e o plano da página seguinte, são o que a
  gravação permite conferir.
- **corpos-gravados-como-fixture-do-ci**: rejeitada como em #59
  (`infra_hash`, URLs assinadas, dezenas de MB).

## Comparação

| Critério | Base compartilhada | Segundo harness | Fluxo com envio gravado | Fixture bruta |
| --- | --- | --- | --- | --- |
| Mesmos leitores de produção | sim | sim | sim | sim |
| Código de conferência duplicado | não | **sim** | não | não |
| Depende do corpo do POST | não | não | **sim** | não |
| Credencial gravada no Git | não | não | não | **sim** |
| Rede no CI | não | não | não | não |

## Decisão

Por resposta, na ordem da CLI: `decodificar_resposta` (status, MIME,
limite, charset e tela de login; no POST, a tela de login vira
`tela_de_login_recebida_apos_envio`), `contexto_da_pagina`
(`hdnTipoVisualizacao`), `unidade_da_tela`; na R, `paginas::juntar`,
`unidade_categoria::selecionar` por categoria e
`pagina_do_controle::planejar` da página exibida e da seguinte; na D,
`ler_detalhada` (colunas, `colunas_ausentes`, célula de marcadores),
`posicao` e `planejar` da D; e as linhas que a colheita do índice lê
(`linhas_com_link`). Leitor da outra visão é `nao_aplicavel`: o comando
trocaria de visão por POST. Tela servida com `tipo_filtro` na URL é
`nao_aplicavel` quando recusada: o comando lê a URL do Controle guardada.

Identidade só da gravação: o `--unidade` de cada tela é o rótulo da tela
do Controle anterior na mesma cassette (a mesma unidade tem de ser lida;
a outra, depois de "Detectada troca de unidade", recusada). As telas
`sip/login.php` gravadas passam pelo transporte do Controle. A
continuidade compara cada página com a anterior da mesma lista, unidade,
critério (`hdn{T}HashCriterios`, só comparado) e total.

O relatório guarda contagens, formas e `cassette#passo`; nenhuma URL,
cookie, hash de critério ou `infra_hash`.

## Evidências verificáveis

- Testes sintéticos: `visao_do_controle::acervo::tests` e
  `paginas::tests_filtro` (correções desta fatia).
- [Relatório do replay](../../validacao-replay/2026-11-04-unidade-listar-processos-acervo.md)
  e [evidência da operação](../../../contracts/evidencias/unidade-listar_processos.md).

## Limitações e revisão

O acervo não tem página além da primeira (nenhuma faixa começa depois
de 1, nenhum POST de paginação): a continuidade entre páginas reais fica
com o [ensaio live](../../validacao-live/); o replay prova o plano da
página 2 de cada lista paginada e o verificador de continuidade, com
testes sintéticos. Rever quando o acervo ganhar cassettes de página N,
ou se um leitor do Controle mudar de forma aceita.
