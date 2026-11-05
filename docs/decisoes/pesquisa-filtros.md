---
type: ImplementationDecision
title: Critérios de tipo e período na pesquisa de protocolos
operation: processo.buscar
implementation: partial
status: adopted-for-scope
scope: filtros condicionados à tela de pesquisa avançada
selected_strategy: web-post-observed-controls
alternatives: web-post-observed-controls,extra-autocomplete-get,unverified-field-injection
evidence_level: source-code-oracle-and-structural-fixtures
live_validation: not-run
---

# Decisão — filtros na pesquisa (#247)

## Alternativas e evidências

A tela `frmPesquisaProtocolo` publicada nas fixtures já usa
`selTipoProcedimentoPesquisa`. O Todos descreve `selSeriePesquisa`,
`txtDataInicio`, `txtDataFim` e `txtAssunto` com `hdnIdAssunto`.
O Todos é oráculo de comportamento, não infraestrutura a importar.

- **Escolhida:** usar `option` com texto exato para tipo e campos de data
  publicados; recusar campo ausente. Conserva três requisições (quatro
  com paginação), sem alteração do transporte.
- **Autocompletar adicional:** custaria requisição e precisaria de
  confirmação do ID na tela; fora do orçamento atual, adiado.
- **Campo por suposição:** poderia produzir falso positivo silencioso;
  recusado.

## Decisão e trade-offs

Tipos têm valores opacos; só a opção da tela fornece o identificador.
As datas exigem formato DD/MM/AAAA e ordem cronológica antes da rede.
`--assunto` é reconhecido pela CLI, mas recusa tipadamente enquanto
não se observa a resolução do identificador. Em paginação, o filtro
pedido é rederivado da página de resultado; se o campo desaparecer,
não envia um POST não filtrado. Sem tentativa de restaurar a tela.

## Validação e revisão

RED: NOT_RUN; testes sintéticos adicionados sem contagem RED observada.
Replay: NOT_RUN (preservado relatório de 05/11). Live UnB/PEN3: NOT_RUN.
CI é a primeira execução disponível no ambiente conectado. Revisar
com cassettes por instalação e ensaio live de leitura antes de fechar #247.
