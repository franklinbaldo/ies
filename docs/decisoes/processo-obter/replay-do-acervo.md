---
type: ImplementationDecision
title: Replay de conformidade do acervo público pelos leitores de processo NUP
operation: processo.obter
implementation: partial
status: adopted-for-scope
scope: apêndice da decisão de processo.obter (Refs #59); um teste ignorado do próprio crate passa cada resposta gravada de tela do processo, árvore, histórico, propriedades e marcadores das 495 cassettes públicas pelos leitores de produção de ies processo NUP, offline, e classifica lido, recusado ou não aplicável; sem superfície nova na CLI e sem rede no CI
selected_strategy: teste-ignorado-no-crate
alternatives: teste-ignorado-no-crate,subcomando-oculto-de-replay,teste-de-integracao-pelo-binario,corpos-gravados-como-fixture-do-ci
evidence_level: replay-offline-do-acervo-publico-e-testes-sinteticos
live_validation: not-run-replay-offline-nao-substitui-ensaio-live
---

# Replay de conformidade do acervo

Apêndice da [decisão de `processo.obter`](../processo-obter.md). Contrato
da [árvore](../../../contracts/cli/processo-obter-arvore.md); resultado no
[relatório do replay](../../validacao-replay/2026-11-04-processo-obter-acervo.md).

Os leitores de `ies processo NUP` foram escritos a partir de poucas
capturas por instalação. O acervo público
([cassettes](../../cassettes.md)) tem milhares de respostas das mesmas
telas nas sete instalações. A pergunta é se os **mesmos leitores** leem
todas, e onde recusam.

## Alternativas consideradas

- **teste-ignorado-no-crate** (escolhida): módulo `cfg(test)` em
  `src/processo_obter/acervo/`, com acesso aos leitores `pub(crate)`. O
  teste `replay_do_acervo_inteiro` é `#[ignore]`: lê os ZIPs do cache
  local (`IES_ACERVO_CACHE`, padrão `~/.cache/ies`), confere cada card
  (SHA-1 do ZIP, SHA-256 do membro, schema, host) pela mesma conferência
  do `ies replay` e escreve o relatório em `IES_ACERVO_SAIDA`. O CI roda só
  os testes sintéticos da classificação.
- **subcomando-oculto-de-replay**: `ies replay acervo`. Rejeitada:
  superfície de CLI nova (mesmo oculta, entra no contrato e no esquema)
  para uma ferramenta de conformidade, não de uso.
- **teste-de-integracao-pelo-binario** (`tests/`): o crate é só binário;
  um teste de integração enxerga apenas o executável, e cada leitor
  exigiria uma porta de entrada na CLI. Rejeitada pelo mesmo motivo.
- **corpos-gravados-como-fixture-do-ci**: copiar respostas para
  `tests/fixtures`. Rejeitada: os corpos trazem `infra_hash` e URLs
  assinadas, e o CI ficaria preso a dezenas de MB. As formas novas viram
  fixtures **sintéticas** no teste do leitor (como a concatenação vazia).

## Comparação

| Critério | Teste ignorado | Subcomando | Integração | Fixture bruta |
| --- | --- | --- | --- | --- |
| Superfície pública nova | não | sim | sim | não |
| Mesmos leitores de produção | sim | sim | só via CLI | sim |
| Rede no CI | não | não | não | não |
| Credencial gravada no Git | não | não | não | **sim** |
| Roda o acervo inteiro | sob demanda | sob demanda | sob demanda | parcial |

## Decisão

A identidade pedida sai da gravação, nunca do nome do cenário: o id é o
`id_procedimento` único da URL servida (ou `id_protocolo` quando a tela
veio da pesquisa rápida, a chave que `conferir_trabalho` lê); o NUP é o
declarado pelo nó `PROCESSO` de uma árvore do mesmo id na mesma
cassette. Sem um dos dois, a leitura é `nao_aplicavel` com o motivo. Na
árvore, o NUP vem dela mesma: a conferência que vale ali é a do id.

A árvore passa por `ler_arvore_da_pagina` (a leitura pura que
`ler_arvore_aberta` agora delega: árvore, vínculo, movidos e os três
links), ou por `aberta_inteira_da_pagina` quando a URL servida traz
`abrir_pastas=1`; pasta fechada passa por `ler_acao_de_abrir_pastas`.
Histórico, propriedades, marcadores e tela usam os leitores do fluxo; o
302 de zero marcadores, a mesma conferência de `andamento_marcador_cadastrar`.
Situação e natureza nunca falham o comando: ali "recusado" é o campo que
sairia `null`.

O relatório guarda só contagens, formas e `cassette#passo`; nenhuma URL,
cookie ou `infra_hash`. A conferência da cassette foi extraída para
`cassette::conferencia` (um SHA-1 por ZIP, reaproveitado entre cards);
o teto do JSON do `ies replay` segue 30 MB, e o replay do acervo usa
120 MB para ler as cassettes grandes de pastas.

## Evidências verificáveis

- Testes sintéticos: `processo_obter::acervo::tests` e
  `arvore_processo::tests_pastas_concatenacao`.
- [Relatório do replay](../../validacao-replay/2026-11-04-processo-obter-acervo.md)
  e [evidência da operação](../../../contracts/evidencias/processo-obter.md).
- [Matriz de derivação](../../matriz-derivacao-cli.md).

## Limitações e revisão

Replay offline não certifica fluxo HTTP live (as cassettes não guardam
todos os corpos de POST nem a ordem de requisições do IES). Ele prova
que cada tela gravada é lida ou recusada com subcódigo. Rever se o
acervo ganhar cassettes novas, se um leitor mudar de forma aceita ou se
o tempo do teste ignorado passar de alguns minutos.
