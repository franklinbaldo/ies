---
type: ImplementationDecision
title: Ler os marcadores do processo pela tela de gerenciamento da barra e corrigir a célula do Controle
operation: processo.obter
implementation: partial
status: adopted-for-scope
scope: apêndice da decisão de processo.obter (Refs #59); ies processo NUP lê os marcadores na tela andamento_marcador_gerenciar publicada na barra do nó do processo, por um GET, sem tocar na escrita da tela; a coluna Marcadores da visão detalhada do Controle passa a ser lista de objetos, um por a.ancMarcador
selected_strategy: web-marcadores-da-barra
alternatives: web-marcadores-da-barra,web-marcadores-da-celula-do-controle,web-marcadores-url-montada,web-marcadores-texto-unico
evidence_level: capturas-privadas-estrutura-unb-e-testes-sinteticos
live_validation: not-run
---

# Marcadores do processo

Apêndice da [decisão de `processo.obter`](../processo-obter.md), que
passaria de 20.000 bytes. Contrato:
[marcadores de `ies processo NUP`](../../../contracts/cli/processo-obter-marcadores.md)
e [visão detalhada](../../../contracts/cli/unidade-listar-processos-visao-detalhada.md);
estrutura em
[marcadores do processo](../../como-o-sei-funciona/marcadores-do-processo.md).

**Um processo pode ter mais de um marcador** (decisão do proprietário,
04/11/2026). Toda saída é lista; nomes nunca são concatenados.

## Alternativas consideradas

- **web-marcadores-da-barra** (escolhida): um GET do
  `andamento_marcador_gerenciar` assinado que a barra do nó `PROCESSO`
  publica, lendo `table#tblMarcadores` (uma linha por marcador: id, nome,
  cor, texto, sigla do usuário, data/hora).
- **web-marcadores-da-celula-do-controle**: reaproveitar a célula
  Marcadores da linha do processo na visão detalhada. Rejeitada para
  `ies processo NUP`: só existe no caminho 4a, depende da coluna marcada
  no Nível de detalhe e não traz usuário nem data. Mantida, corrigida,
  em `ies unidade listar`.
- **web-marcadores-url-montada**: montar a URL da tela. Rejeitada: o
  `infra_hash` é assinado pelo servidor
  ([infra_hash](../../como-o-sei-funciona/infra-hash.md)).
- **web-marcadores-texto-unico**: manter `marcadores` como texto da
  célula. Rejeitada: com dois marcadores a célula publicava os nomes
  colados (medido ao vivo na UnB 4.0.9).

## Comparação

| Critério | Barra → tela | Célula do Controle | URL montada | Texto único |
| --- | --- | --- | --- | --- |
| Requisições | +1 GET | 0 (só no 4a) | +1 | 0 |
| Um objeto por marcador | sim | sim (corrigida) | sim | **não** |
| Usuário e data | sim | não | sim | não |
| Assinatura servida | sim | sim | **não** | sim |
| Escrita na página | sim, nunca tocada | não | sim | não |

## Decisão

`ies processo NUP` lê a tela pela barra (caminho 1: o link guardado
primeiro, se a barra recém-lida oferece a tela; velho é esquecido).
Barra sem o link: `marcadores: null` com motivo, sem requisição. Tela
sem tabela: `null` com motivo, nunca `[]` sem observação. Legenda maior
que as linhas: `marcadores_total_declarado` e `marcadores_faixa_observada`,
sem alegar completude; menor: recusa. `--sem-marcadores` não pede nada.

A célula do Controle vira lista `{nome, cor, texto, id}` lida pela forma
exata da âncora: `href` com `acao=andamento_marcador_gerenciar` e o
`id_procedimento` da linha, `#ID-<n>`, ícone `svg/marcador_<cor>.svg`,
tooltip `return infraTooltipMostrar('<literal>');` lido como literal JS
(nenhum JavaScript executado). Mudança incompatível documentada no
contrato da visão detalhada.

**Orçamento.** +1 GET. Repetição quente no caminho 1: **4** (árvore,
histórico, propriedades e marcadores guardados), **3** com
`--sem-marcadores`, **2** com `--sem-andamentos --sem-propriedades`,
**1** com os três; guardado velho, **5**. Caminho 4a: **6**.

**Efeito.** GET de tela de gerenciamento, declarado leitura. A tela tem
escrita (`btnAdicionar`, `btnRemover` → `andamento_marcador_remover`,
`andamento_marcador_alterar`, `acaoRemover`); o IES só confere o
`action` do formulário como identidade e nunca envia, clica ou segue.

## Evidências verificáveis

- Capturas privadas `0600` da UnB 4.0.9 (04/11/2026), só estrutura: a
  visão detalhada do Controle com oito âncoras em sete processos, um
  deles com dois marcadores, e a tela de gerenciamento desse processo.
  Testes locais temporários (apagados) deram **dois** marcadores nos dois
  leitores, com os mesmos ids (`#ID-<n>` da célula = `value` do
  checkbox da tela); o link da barra foi lido nas três árvores completas
  capturadas.
- Testes sintéticos: `caixa_detalhada::tests_marcadores`,
  `marcadores_processo::tests_tela`,
  `arvore_processo::tests_barra_marcadores`,
  `processo_obter::tests_marcadores` (sequências exatas) e
  `processo_obter_cli::tests_marcadores`; golden
  `tests/fixtures/caixa-detalhada-v2.json`; contrato da CLI em
  [cli_contract](../../../tests/cli_contract.rs).
- [Evidência da operação](../../../contracts/evidencias/processo-obter.md).

## Limitações e revisão

Validação live **NOT_RUN** (sem execução do comando nesta fatia; o agente
pai valida na UnB). Não observados: tela sem marcador, paginação da
tabela, marcadores de outras unidades e a célula em 5.0.x. Revisar se a
tela publicar paginação, se a célula do Controle trouxer outra forma, ou
se a barra publicar a ação noutra forma.
