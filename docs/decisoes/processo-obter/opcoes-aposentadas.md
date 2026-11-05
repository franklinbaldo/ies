---
type: ImplementationDecision
title: Aposentadoria de processo NUP --resumo e das opções de captura privada
operation: processo.obter
implementation: partial
status: adopted-for-scope
scope: apêndice da decisão de processo.obter; remoção de --resumo, --saida, --caixa-local, --observado-na-pagina, --pagina-arquivo, --historico-privado e --resultado-pesquisa-file de ies processo NUP, por decisão do proprietário de 04/11/2026
selected_strategy: aposentar-resumo-e-capturas
alternatives: aposentar-resumo-e-capturas,alinhar-resumo-a-pagina-observada,manter-resumo-e-capturas
evidence_level: auditoria-do-roadmap-de-04-11-2026-e-testes-da-superficie
live_validation: nao-se-aplica-remocao-de-opcoes
---

# Aposentadoria de `--resumo` e das opções de captura privada

Apêndice da [decisão de processo.obter](../processo-obter.md). Decisão do
proprietário de 04/11/2026, sobre a proposta da auditoria do roadmap
(dívida de superfície de #59).

## Alternativas

- **manter-resumo-e-capturas:** deixar `--resumo` (um GET da caixa, sem
  abrir o processo) e as fontes offline como estavam.
- **alinhar-resumo-a-pagina-observada:** reescrever `--resumo` para a
  página observada com `total_declarado` e `faixa_observada`.
- **aposentar-resumo-e-capturas** (escolhida): retirar as opções, sem
  alias.

## Por quê

`--resumo` exigia cobertura de todas as páginas da caixa
(`paginas_do_controle_nao_obtidas`), contra a regra da
[página observada](../../../contracts/cli.md); `--observado-na-pagina` e
`--pagina-arquivo` eram remendos dessa exigência, e `--saida` e
`--caixa-local` só exportavam e reliam esse resumo. A leitura pela árvore
com o [índice local](../../arquitetura/0003-indice-local.md) e a colheita
do Controle já respondem "onde está este processo" com 1 a 4 requisições
e a identidade confirmada pelo nó raiz. `--historico-privado` e
`--resultado-pesquisa-file` liam HTML capturado à mão, papel que
`ies replay` cumpria sobre cassettes verificadas (hoje, o replay do acervo). Alinhar o resumo
duplicaria a leitura da caixa de `unidade listar-processos`.

## Efeito

As sete opções são erro de uso do Clap (saída 2, nada em stdout, sem ler
a sessão): [teste](../../../tests/cli_contract.rs),
`opcoes_aposentadas_de_processo_sao_erro_de_uso`. Saíram os módulos
`processo_resumo_cli`, `resumo_exportacao`, `resumo_pagina`,
`paginas_capturadas`, `caixa_local`, `processo_historico_privado_cli` e
`processo_pesquisa_privada_cli` e seus testes. Ficam: `historico.rs` e
`pesquisa_web` (usados pela árvore, por `processo buscar` e por
`ies replay`), `resumo_processo` (só `ies replay --numero-processo`; saiu
com o `ies replay` em 05/11/2026, [replay fora da CLI](../acervo/replay-fora-da-cli.md)) e a
exportação `--saida` de `unidade listar-processos|recebidos|gerados`.
`documento --html-privado` não entrou nesta decisão.
