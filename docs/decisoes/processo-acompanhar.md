---
type: ImplementationDecision
title: Acompanhamento Especial pela barra e cadastro da interface Web
description: Estratégia Web de prévia, POST único e releitura da inclusão, mantendo retirada bloqueada
operation: processo.acompanhar
implementation: partial
status: adopted-for-scope
scope: ies processo acompanhar NUP com inclusão confirmada por um POST e uma releitura, além de prévia de retirada; confirmação da retirada ainda bloqueada
selected_strategy: web-barra-cadastro-tabela
alternatives: web-barra-cadastro-tabela,rest-acompanhamento,simular-estado
evidence_level: todos-unb-e-indices-de-cassettes
live_validation: not-run
---

# Acompanhamento Especial (Refs #254)

## Contexto

O Todos `src/todos/acompanhamento.py` e o teste
`tests/test_processo_acompanhar_010.py` @323eb01 registram a UnB
em 15–16/10/2026: a barra oferece `acompanhamento_gerenciar`,
que redireciona ao cadastro na ausência de acompanhamento, ou traz
a tabela `Tabela de Acompanhamentos.`. O `btnAdicionar` abre
`frmAcompanhamentoCadastro` com `selGrupoAcompanhamento`,
`txaObservacao` e `hdnIdProtocolo`. Os grupos dependem da unidade.

## Alternativas consideradas

1. **Web barra/cadastro/tabela (escolhida):** usa somente URLs assinadas
   servidas, valida origem, ação e processo, trata 302 como dado,
   não executa JS e recusa HTML truncado.
2. **REST:** precisaria de contrato e medição de backend habilitado;
   não foi escolhido para esta interface.
3. **Simular o estado:** inferir vazios da ausência de tabela poderia
   afirmar ausência em HTML cortado; rejeitado.

## Comparação

| Critério | Web | REST | Simulação |
| --- | --- | --- | --- |
| Caminho medido na UnB | sim | não | não |
| Opções por unidade observadas | sim | a verificar | não |
| GET adicional ao cadastro | até 1 | indefinido | 0 |
| Risco de inventar ausência | baixo, falha fechado | a verificar | alto |

## Decisão

A abertura reutiliza `processo_obter`; a barra do nó raiz só fornece
uma URL se tiver exatamente um link válido, conferido pelo leitor
`arvore_processo::acompanhar`. O gerenciador fornece a tabela ou o
302 para cadastro. A prévia valida a identidade do formulário, escolhe
nome exato ou branco observado, e exige identidade única na remoção.
A contagem inclui todos os GETs. Para incluir, a prévia também
confere se o primitivo `formulario_web` pode derivar o POST exato da
tela; somente `--confirmar` consome a submissão uma vez. O formulário
é escolhido pelo identificador `frmAcompanhamentoCadastro` e a
submissão reenvia ocultos, grupo, observação e botão conforme servidos.
Nenhum 302 é seguido e nenhuma falha de rede dispara retry.
O gerente é relido uma vez após o POST; só uma linha nova com
grupo/observação correspondentes e linhas anteriores preservadas
comprova sucesso. Sem prova, a saída é 3. A confirmação de retirada
continua bloqueada antes da sessão.

A suíte injetada mede 5 GETs para prévia e 3 GETs de abertura +
GET de gerenciar + GET de cadastro + POST + GET de releitura (7
requisições) para inclusão. Uma falha na resposta do POST não elimina
a necessidade de verificação. Testes sintéticos não substituem a
validação nas instalações de treinamento.

## Evidências verificáveis

- [Contrato da operação](../../contracts/cli/processo-acompanhar.md).
- [Matriz de evidências](../../contracts/evidencias/processo-acompanhar.md).
- [Testes de interface](../../tests/cli_processo_acompanhar.rs).
- [Testes de parsing e reconciliação](../../src/processo_acompanhar_tests.rs).
- [Testes de fluxo e orçamento](../../src/processo_acompanhar/tests_fluxo.rs).
- [Índice da gravação da UnB](../../cassettes/unb/processo-acompanhar-unb--4e19a6146bc4.md):
  45 interações, das quais 39 GET e 6 POST; o índice não prova os corpos.
- [Relatório live](../validacao-live/2026-11-05-processo-acompanhar.md).

## Bancada de prévia de retirada com estado explícito (#348)

A migração SQLite v35 registra a fotografia de acompanhamentos por
instalação, cenário, processo e unidade. Uma linha de observação com zero
itens é lista comprovada vazia; ausência dessa linha continua sendo dado
desconhecido e não autoriza publicar link ou inferir ausência de grupo.

O anti-IES publica `acompanhamento_gerenciar` na árvore somente quando
existe observação da unidade aberta. Seu GET oferece tabela estrutural
com seis colunas e IDs de linha preservados. A CLI Rust percorre a
mesma navegação e identifica G sem selecionar H, ou informa no-op
quando só H está presente. Não se fabrica formulário de exclusão nem
se oferece `--deixar --confirmar` na bancada.

A inspeção das cassettes de acompanhamento UnB orienta os nomes de ações
e a estrutura, mas não demonstra a transição G/H. Essa transição é
hipótese sintética testada em TLS loopback com o mesmo SQLite, não replay
semântico nem validação live. A exclusão confirmada BDD-348-02 permanece
pendente de derivação de URL e campos da página publicada.

## Limitações e revisão

RED local NOT_RUN (sem toolchain e DNS no sandbox); CI só comprova
o head ao qual estiver vinculado. Replay das cassettes e validação
live na UnB e numa segunda instalação: NOT_RUN, 0 requisições ao SEI.
Nenhuma escrita e nenhum resíduo foram produzidos.

Ainda faltam replay integral do acervo e teste ao vivo em
treinamento, com prévia anterior ao `--confirmar`, envio único,
releitura e registro de resíduo. A retirada exige derivar a
`acompanhamento_excluir` da linha e conferir sua identidade, sem
copiar a arquitetura do Todos. A versão parcial permanece DRAFT.
